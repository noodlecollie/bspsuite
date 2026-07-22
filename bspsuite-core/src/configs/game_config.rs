use crate::configs::CompileTuningParametersConfig;
use crate::io::{GameConfigIO, VersionedIOFormat};
use anyhow::{Context, Result, ensure};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct GameConfig
{
	pub game_id: String,
	pub game_name: String,
	pub map_formats: HashSet<String>,
	pub vfs_formats: Vec<String>,

	pub default_compile_tuning_parameters: Option<CompileTuningParametersConfig>,
}

impl GameConfig
{
	pub fn load_for_game(toolchain_root: &PathBuf, game: &str) -> Result<Self>
	{
		let root_dir: PathBuf = GameConfig::game_config_root_directory(toolchain_root);
		let game_config_path: PathBuf = root_dir.join(game).join(format!("config.toml"));

		ensure!(
			game_config_path.exists(),
			"Config file {} for game \"{game}\" not found on disk",
			game_config_path.display()
		);

		return GameConfigIO::read(game_config_path.as_path()).with_context(|| {
			format!(
				"Failed to parse game config file {}",
				game_config_path.display()
			)
		});
	}

	fn game_config_root_directory(toolchain_root: &PathBuf) -> PathBuf
	{
		return toolchain_root.join("games");
	}
}
