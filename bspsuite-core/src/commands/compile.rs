use std::ffi::OsStr;
use std::path::PathBuf;

use super::types::{BaseArgs, ResultCode};
use super::utils::wrap_residual_errors;
use crate::compiler_error::{CompilerError, CompilerErrorCode};
use crate::extensions::{ExtensionList, ExtensionRef, extension_routines};
use crate::game_configs::GameConfig;
use crate::toolchain::Toolchain;
use anyhow::{Result, anyhow, bail, ensure};
use bspextifc::types::{PortableOption, StringRef};
use log::{debug, info};

#[repr(C)]
pub struct CompileArgs<'l>
{
	pub base: BaseArgs<'l>,
	pub input_file: StringRef<'l>,
	pub game: StringRef<'l>,
	pub map_format_override: PortableOption<StringRef<'l>>,
}

#[unsafe(no_mangle)]
pub extern "C" fn bspcore_run_compile(args: &CompileArgs) -> ResultCode
{
	return wrap_residual_errors(|| run_compile(args));
}

fn run_compile(args: &CompileArgs) -> Result<()>
{
	let toolchain: Toolchain = Toolchain::new(&args.base.toolchain_root_path());

	let game_config: GameConfig =
		GameConfig::load_for_game(toolchain.root_path(), args.game.as_str())
			.map_err(|err| CompilerError::new_anyhow(CompilerErrorCode::ConfigError, err))?;

	let extensions: ExtensionList = toolchain.find_extensions();
	extension_routines::register_map_formats(&extensions);

	let input_path: PathBuf = PathBuf::from(args.input_file.as_str());
	let map_format_override: Option<String> = if args.map_format_override.is_some()
	{
		Some(args.map_format_override.unwrap_ref().to_string())
	}
	else
	{
		None
	};

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
			&map_format_override.as_ref().map(|val| val.as_str()),
		);

	let map_format: extension_routines::ExtensionForParsingMapFormat = map_format_result
		.map_err(|err| CompilerError::new_anyhow(CompilerErrorCode::ArgumentError, err))?;

	debug!(
		"Input map format: {} ({})",
		map_format.map_format_name,
		map_format_override
			.map(|_| "provided as argument")
			.unwrap_or("inferred from file extension")
	);

	info!("Compile complete");

	return Ok(());
}
