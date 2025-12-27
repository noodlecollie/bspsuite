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

	let map_format_result: Result<String> = if args.map_format_override.is_some()
	{
		Ok(args.map_format_override.unwrap_ref().to_string())
	}
	else
	{
		infer_map_format_from_input_file_extension(&extensions, &game_config, &input_path)
	};

	let map_format: String = map_format_result
		.map_err(|err| CompilerError::new_anyhow(CompilerErrorCode::ArgumentError, err))?;

	debug!("Input map format: {map_format}");

	info!("Compile complete");

	return Ok(());
}

fn infer_map_format_from_input_file_extension(
	extensions: &ExtensionList,
	game_config: &GameConfig,
	input_path: &PathBuf,
) -> Result<String>
{
	let input_ext: &OsStr = input_path.extension().ok_or_else(|| {
		anyhow!("Input file has no extension, cannot infer map format from file types")
	})?;

	let input_ext: &str = input_ext
		.to_str()
		.ok_or_else(|| anyhow!("Could not parse input file extension string"))?;

	ensure!(
		!input_ext.is_empty(),
		"Input file {} had no file extension, cannot infer map format",
		input_path.to_str().unwrap_or("<unknown>")
	);

	let allowed_formats: Vec<&str> = game_config
		.map_formats
		.iter()
		.map(|item| item.as_str())
		.collect();

	debug!(
		"Inferring map format from input file extension .{input_ext}. Formats allowed for game {}: {}",
		game_config.game_id,
		allowed_formats.join(", ")
	);

	let supported_exts: Vec<(&ExtensionRef, String)> =
		extension_routines::find_extensions_supporting_source_file_extension(
			extensions,
			input_ext,
			&allowed_formats,
		);

	if supported_exts.len() != 1
	{
		let matches_str: String = supported_exts
			.iter()
			.map(|(ext, fmt)| format!("{} (map format {fmt})", ext.get_name()))
			.collect::<Vec<String>>()
			.join(", ");

		// TODO: Support better disambiguation in this case.
		if supported_exts.len() > 1
		{
			bail!(
				"Input map file extension .{input_ext} supported by more than compiler extension: \
				{matches_str}. Unable to deduce which one to use."
			);
		}
		else
		{
			let indent: &'static str = "    ";

			bail!(
				"No compiler extensions recognised input map file with extension .{input_ext}\n\
				{indent}Formats allowed for game {}: {}\n\
				{indent}Formats supported by compiler: {}",
				game_config.game_id,
				allowed_formats.join(", "),
				extension_routines::supported_map_formats_and_file_extensions(extensions)
					.join("; ")
			);
		}
	}

	return Ok(supported_exts[0].1.clone());
}
