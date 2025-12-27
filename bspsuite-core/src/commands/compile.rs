use std::ffi::OsStr;
use std::path::PathBuf;

use super::types::{BaseArgs, ResultCode};
use super::utils::wrap_panics;
use crate::extensions::{ExtensionList, ExtensionRef, extension_routines};
use crate::game_configs::GameConfig;
use crate::toolchain::Toolchain;
use anyhow::{Error, Result, anyhow, bail};
use bspextifc::types::{PortableOption, StringRef};
use log::{debug, error, info};

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
	return wrap_panics(|| {
		let toolchain: Toolchain = Toolchain::new(&args.base.toolchain_root_path());

		let game_config: Result<GameConfig, Error> =
			GameConfig::load_for_game(toolchain.root_path(), args.game.as_str());

		if let Err(err) = game_config
		{
			error!("{err}");
			return ResultCode::ConfigError;
		}

		let game_config: GameConfig = game_config.unwrap();

		let extensions: ExtensionList = toolchain.find_extensions();
		extension_routines::register_map_formats(&extensions);

		let input_path: PathBuf = PathBuf::from(args.input_file.as_str());

		let map_format: Result<String> = if args.map_format_override.is_some()
		{
			Ok(args.map_format_override.unwrap_ref().to_string())
		}
		else
		{
			infer_map_format_from_input_file_extension(&extensions, &game_config, &input_path)
		};

		if let Err(err) = map_format
		{
			error!("{err}");
			return ResultCode::ArgumentError;
		}

		let map_format: String = map_format.unwrap();

		debug!("Input map format: {map_format}");

		info!("Compile complete");
		return ResultCode::Ok;
	});
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

	let allowed_formats: Vec<&str> = game_config
		.supported_map_formats
		.iter()
		.map(|item| item.as_str())
		.collect();

	let supported_exts: Vec<(&ExtensionRef, String)> =
		extension_routines::find_extensions_supporting_source_file_extension(
			extensions,
			input_ext,
			&allowed_formats,
		);

	if supported_exts.is_empty()
	{
		bail!(format!(
			"No compiler extensions recognised input map file with extension .{input_ext}"
		));
	}

	// TODO: Support better disambiguation in this case.
	if supported_exts.len() > 1
	{
		let matches_str: String = supported_exts
			.iter()
			.map(|(ext, fmt)| format!("{} (map format {fmt})", ext.get_name()))
			.collect::<Vec<String>>()
			.join(", ");

		bail!(format!(
			"Input map file extension .{input_ext} matched more than compiler extension: {matches_str}"
		));
	}

	return Ok(supported_exts[0].1.clone());
}
