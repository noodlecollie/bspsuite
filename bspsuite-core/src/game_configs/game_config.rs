use anyhow::{Context, Error, ensure};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use toml;

#[derive(Deserialize, Serialize)]
pub struct GameConfig
{
	pub game_id: String,
	pub game_name: String,
	pub map_formats: Vec<String>,
}

impl GameConfig
{
	pub fn load_for_game(toolchain_root: &PathBuf, game: &str) -> Result<Self, Error>
	{
		let root_dir: PathBuf = GameConfig::game_config_root_directory(toolchain_root);
		let game_config_path: PathBuf = root_dir.join(game).join(format!("{game}.cfg"));

		ensure!(
			game_config_path.exists(),
			"Config file {} for game \"{game}\" not found on disk",
			game_config_path.to_str().unwrap_or("<unknown>")
		);

		let file_contents: String = fs::read_to_string(&game_config_path).with_context(|| {
			format!(
				"Failed to read game config file {}",
				game_config_path.to_str().unwrap_or("<unknown>")
			)
		})?;

		return GameConfig::load(&file_contents).with_context(|| {
			format!(
				"Failed to parse game config file {}",
				game_config_path.to_str().unwrap_or("<unknown>")
			)
		});
	}

	pub fn load(data: &str) -> Result<Self, toml::de::Error>
	{
		return toml::from_str(data);
	}

	pub fn to_string(&self) -> Result<String, toml::ser::Error>
	{
		return toml::to_string_pretty(self);
	}

	fn game_config_root_directory(toolchain_root: &PathBuf) -> PathBuf
	{
		return toolchain_root.join("games");
	}
}
