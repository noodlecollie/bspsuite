use std::collections::HashSet;

use crate::configs::{CompileTuningParametersConfig, GameConfig};
use crate::io::helpers::{IOFmtSignature, IOFormat};
use serde::{Deserialize, Serialize};

pub(super) const VERSION: u64 = 1;

#[derive(Serialize, Deserialize, Debug)]
pub struct V1File
{
	#[serde(flatten)]
	signature: IOFmtSignature<V1File>,
	pub game_id: String,
	pub game_name: String,
	pub map_formats: HashSet<String>,
	pub vfs_formats: Vec<String>,
	pub default_compile_tuning_parameters: Option<V1TuningParams>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct V1TuningParams
{
	pub contact_epsilon: Option<f64>,
	pub equal_point_radius_epsilon: Option<f64>,
	pub equal_vector_component_epsilon: Option<f64>,
	pub zero_epsilon: Option<f64>,
}

impl IOFormat for V1File
{
	fn format_name() -> &'static str
	{
		return "gameconfig";
	}

	fn format_version() -> u64
	{
		return VERSION;
	}

	fn type_desc() -> &'static str
	{
		return "game config";
	}
}

////////////////////////////////////////////////////////
// CompileTuningParametersConfig -> V1File
////////////////////////////////////////////////////////

impl From<&GameConfig> for V1File
{
	fn from(value: &GameConfig) -> Self
	{
		return Self {
			signature: IOFmtSignature::new(),
			game_id: value.game_id.clone(),
			game_name: value.game_name.clone(),
			map_formats: value.map_formats.clone(),
			vfs_formats: value.vfs_formats.clone(),
			default_compile_tuning_parameters: value
				.default_compile_tuning_parameters
				.as_ref()
				.map(|v| v.into()),
		};
	}
}

impl From<&CompileTuningParametersConfig> for V1TuningParams
{
	fn from(value: &CompileTuningParametersConfig) -> Self
	{
		return Self {
			contact_epsilon: value.contact_epsilon,
			equal_point_radius_epsilon: value.equal_point_radius_epsilon,
			equal_vector_component_epsilon: value.equal_vector_component_epsilon,
			zero_epsilon: value.zero_epsilon,
		};
	}
}

////////////////////////////////////////////////////////
// V1File -> CompileTuningParametersConfig
////////////////////////////////////////////////////////

impl From<V1File> for GameConfig
{
	fn from(value: V1File) -> Self
	{
		return Self {
			game_id: value.game_id,
			game_name: value.game_name,
			map_formats: value.map_formats,
			vfs_formats: value.vfs_formats,
			default_compile_tuning_parameters: value
				.default_compile_tuning_parameters
				.map(|v| v.into()),
		};
	}
}

impl From<V1TuningParams> for CompileTuningParametersConfig
{
	fn from(value: V1TuningParams) -> Self
	{
		return Self {
			contact_epsilon: value.contact_epsilon,
			equal_point_radius_epsilon: value.equal_point_radius_epsilon,
			equal_vector_component_epsilon: value.equal_vector_component_epsilon,
			zero_epsilon: value.zero_epsilon,
		};
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn serialize_and_deserialize_simple_object()
	{
		let config: GameConfig = GameConfig {
			game_id: "my_game".to_owned(),
			game_name: "My Game".to_owned(),
			map_formats: HashSet::from(["mapone".to_owned(), "maptwo".to_owned()]),
			vfs_formats: vec!["directory".to_owned()],
			default_compile_tuning_parameters: Some(CompileTuningParametersConfig {
				contact_epsilon: Some(1e-3),
				equal_point_radius_epsilon: Some(1e-4),
				equal_vector_component_epsilon: Some(1e-2),
				zero_epsilon: Some(1e-3),
			}),
		};

		let v1_file_out: V1File = V1File::from(&config);
		let toml_string: String = toml::to_string(&v1_file_out).unwrap();
		let v1_file_in: V1File = toml::from_str::<V1File>(&toml_string).unwrap();
		let recovered_config: GameConfig = v1_file_in.into();

		assert_eq!(config, recovered_config);
	}

	#[test]
	fn serialize_and_deserialize_without_tuning_params()
	{
		let config: GameConfig = GameConfig {
			game_id: "my_game".to_owned(),
			game_name: "My Game".to_owned(),
			map_formats: HashSet::from(["mapone".to_owned(), "maptwo".to_owned()]),
			vfs_formats: vec!["directory".to_owned()],
			default_compile_tuning_parameters: None,
		};

		let v1_file_out: V1File = V1File::from(&config);
		let toml_string: String = toml::to_string(&v1_file_out).unwrap();
		let v1_file_in: V1File = toml::from_str::<V1File>(&toml_string).unwrap();
		let recovered_config: GameConfig = v1_file_in.into();

		assert_eq!(config, recovered_config);
	}

	#[test]
	fn deserialize_with_missing_format_and_version()
	{
		let toml_string: &str = r##"
			game_id = "my_game"
			game_name = "My Game"
			map_formats = ["mapone", "maptwo"]
			vfs_formats = ["directory"]"##;

		let deserialized_data = toml::from_str::<V1File>(&toml_string);
		let error = deserialized_data.expect_err("Expected deserialization to fail");
		let error_string: String = error.to_string();
		let suffix: &str = "missing field `format`\n";

		assert!(
			error_string.ends_with(suffix),
			"Error string:\n  \"{error_string}\"\nshould end with suffix\n  \"{suffix}\""
		);
	}

	#[test]
	fn deserialize_with_missing_format()
	{
		let toml_string: &str = r##"
			version = 1
			game_id = "my_game"
			game_name = "My Game"
			map_formats = ["mapone", "maptwo"]
			vfs_formats = ["directory"]"##;

		let deserialized_data = toml::from_str::<V1File>(&toml_string);
		let error = deserialized_data.expect_err("Expected deserialization to fail");
		let error_string: String = error.to_string();
		let suffix: &str = "missing field `format`\n";

		assert!(
			error_string.ends_with(suffix),
			"Error string:\n  \"{error_string}\"\nshould end with suffix\n  \"{suffix}\""
		);
	}

	#[test]
	fn deserialize_with_missing_version()
	{
		let toml_string: &str = r##"
			format = "gameconfig"
			game_id = "my_game"
			game_name = "My Game"
			map_formats = ["mapone", "maptwo"]
			vfs_formats = ["directory"]"##;

		let deserialized_data = toml::from_str::<V1File>(&toml_string);
		let error = deserialized_data.expect_err("Expected deserialization to fail");
		let error_string: String = error.to_string();
		let suffix: &str = "missing field `version`\n";

		assert!(
			error_string.ends_with(suffix),
			"Error string:\n  \"{error_string}\"\nshould end with suffix\n  \"{suffix}\""
		);
	}
}
