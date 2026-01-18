use std::path::PathBuf;

use super::types::{BaseArgs, InputPathMetadata, ResultCode};
use super::utils::wrap_residual_errors;
use crate::compiler_error::{CompilerError, CompilerErrorCode};
use crate::extensions::{ExtensionList, extension_routines};
use crate::game_configs::GameConfig;
use crate::io::map_source_file;
use crate::model::MapSourceFile;
use crate::toolchain::Toolchain;
use anyhow::{Context, Result, anyhow};
use bspextifc::builders::map_source_builder::{Entity, MapSourceBuilder};
use bspffi::types::{XCOption, XCStr};
use log::{debug, info};

#[repr(C)]
pub struct CompileArgs<'l>
{
	pub base: BaseArgs<'l>,
	pub input_file: XCStr<'l>,
	pub game: XCStr<'l>,
	pub map_format_override: XCOption<XCStr<'l>>,
	pub dump_source_file: bool,
}

#[unsafe(no_mangle)]
pub extern "C" fn bspcore_run_compile(args: &CompileArgs) -> ResultCode
{
	return wrap_residual_errors(|| run_compile(args));
}

fn run_compile(args: &CompileArgs) -> Result<(), CompilerError>
{
	let toolchain: Toolchain = Toolchain::new(&args.base.toolchain_root_path());

	let game_config: GameConfig =
		GameConfig::load_for_game(toolchain.root_path(), args.game.as_str())
			.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::ConfigError, err))?;

	let extensions: ExtensionList = toolchain.find_extensions();
	extension_routines::register_map_formats(&extensions);

	let input_path_md: InputPathMetadata =
		InputPathMetadata::new(&PathBuf::from(args.input_file.as_str()).as_path())
			.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::ArgumentError, err))?;
	let input_path: &PathBuf = &input_path_md.full_path;

	let map_format_override: Option<&str> = args.map_format_override.unmarshal_as_ref_option();

	debug!(
		"Allowed map formats for game {}: {}",
		game_config.game_id,
		game_config.map_formats.join(", ")
	);

	let map_format_result: Result<extension_routines::ExtensionForParsingMapFormat> =
		extension_routines::choose_extension_to_parse_map(
			&extensions,
			input_path.as_path(),
			&game_config
				.map_formats
				.iter()
				.map(|fmt| fmt.as_str())
				.collect(),
			&map_format_override,
		);

	let map_format: extension_routines::ExtensionForParsingMapFormat = map_format_result
		.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::ArgumentError, err))?;

	debug!(
		"Input map format: {} ({})",
		map_format.map_format_name,
		map_format_override
			.map(|_| "provided as argument")
			.unwrap_or("inferred from file extension")
	);

	info!("Loading {}", input_path.display());

	let input_file: String = std::fs::read_to_string(&input_path)
		.with_context(|| format!("Failed to read {}", input_path.display()))
		.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::IoError, err))?;

	info!("Parsing {}", input_path.display());

	let builder: MapSourceBuilder =
		extension_routines::parse_map(&extensions, &input_file, &map_format)
			.with_context(|| "Failed to initiate map parsing")
			.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::InternalError, err))?;

	let parsed_entities: Vec<Entity> = builder.collect().map_err(|err| {
		CompilerError::from_anyhow(
			CompilerErrorCode::IoError,
			anyhow!("Failed to parse map {}. {err}", input_path.display()),
		)
	})?;

	let map_source: MapSourceFile = parsed_entities.into();

	debug!(
		"Input map parsed successfully, contains {} entities",
		map_source.entities.len()
	);

	map_source_file::write(
		input_path_md
			.directory_path
			.join(format!("{}.source.json", input_path_md.file_name))
			.as_path(),
		&map_source,
	)
	.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::IoError, err))?;

	info!("Compile complete");

	return Ok(());
}
