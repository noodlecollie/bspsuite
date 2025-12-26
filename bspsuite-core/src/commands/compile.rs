use super::types::{BaseArgs, ResultCode};
use super::utils::wrap_panics;
use crate::extensions::{ExtensionList, extension_routines};
use crate::game_configs::GameConfig;
use crate::toolchain::Toolchain;
use anyhow::Error;
use bspextifc::types::StringRef;
use log::{error, info};

#[repr(C)]
pub struct CompileArgs<'l>
{
	pub base: BaseArgs<'l>,
	pub input_file: StringRef<'l>,
	pub game: StringRef<'l>,
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

		info!("Compile complete");
		return ResultCode::Ok;
	});
}
