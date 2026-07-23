use std::io::{Read, Write};

use crate::configs::{CompileTuningParametersConfig, GameConfig};
use crate::io::helpers::{IOFmtSignature, IOFormat, VersionedIOFormat};
use crate::io::{TrimmedString, validate_items_non_empty};
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use serde_valid::toml::{FromTomlReader, ToTomlWriter};

pub(super) const VERSION: u64 = 1;

pub struct GameConfigIOFormatV1;

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct V1File
{
	#[serde(flatten)]
	signature: IOFmtSignature<V1File>,

	#[validate(
		pattern = r"^\w+$",
		message = "Game ID must be alphanumeric and non-empty"
	)]
	pub game_id: String,

	#[validate(min_length = 1, message = "Game name cannot be empty")]
	pub game_name: TrimmedString,

	#[validate(min_items = 1, message = "Expected at least one map format")]
	#[validate(unique_items, message = "Duplicate map formats are not allowed")]
	#[validate(custom = validate_items_non_empty)]
	pub map_formats: Vec<TrimmedString>,

	// This is essentially a set, but ordered.
	#[validate(min_items = 1, message = "Expected at least one VFS format")]
	#[validate(unique_items, message = "Duplicate VFS formats are not allowed")]
	#[validate(custom = validate_items_non_empty)]
	pub vfs_formats: Vec<TrimmedString>,

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
			game_name: TrimmedString::from(value.game_name.as_str()),
			map_formats: value
				.map_formats
				.iter()
				.map(|s| s.as_str().into())
				.collect(),
			vfs_formats: value
				.vfs_formats
				.iter()
				.map(|s| s.as_str().into())
				.collect(),
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
			game_name: value.game_name.into(),
			map_formats: value.map_formats.into_iter().map(|s| s.into()).collect(),
			vfs_formats: value.vfs_formats.into_iter().map(|s| s.into()).collect(),
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

////////////////////////////////////////////////////////
// VersionedIOFormat
////////////////////////////////////////////////////////

impl VersionedIOFormat for GameConfigIOFormatV1
{
	type InnerFormat = GameConfig;
	type SerializableFormat = V1File;

	fn serialize_impl<Writer>(writer: Writer, data: &Self::SerializableFormat) -> Result<()>
	where
		Writer: Write,
		Self::SerializableFormat: Serialize,
	{
		return Ok(data.to_toml_writer(writer)?);
	}

	fn deserialize_impl<Reader>(reader: Reader) -> Result<Self::SerializableFormat>
	where
		Reader: Read,
		Self::SerializableFormat: DeserializeOwned,
	{
		return Ok(V1File::from_toml_reader(reader)?);
	}
}

#[cfg(test)]
mod tests
{
	use serde_json::json;
	use serde_valid::toml::{FromTomlStr, ToTomlString};

	use super::*;

	#[test]
	fn serialize_and_deserialize_simple_object()
	{
		let config: GameConfig = GameConfig {
			game_id: "my_game".to_owned(),
			game_name: "My Game".to_owned(),
			map_formats: vec!["mapone".into(), "maptwo".into()],
			vfs_formats: vec!["directory".to_owned()],
			default_compile_tuning_parameters: Some(CompileTuningParametersConfig {
				contact_epsilon: Some(1e-3),
				equal_point_radius_epsilon: Some(1e-4),
				equal_vector_component_epsilon: Some(1e-2),
				zero_epsilon: Some(1e-3),
			}),
		};

		let v1_file_out: V1File = V1File::from(&config);
		let toml_string: String = v1_file_out.to_toml_string().unwrap();
		let v1_file_in: V1File = V1File::from_toml_str(toml_string.as_str()).unwrap();
		let recovered_config: GameConfig = v1_file_in.into();

		assert_eq!(config, recovered_config);
	}

	#[test]
	fn serialize_and_deserialize_without_tuning_params()
	{
		let config: GameConfig = GameConfig {
			game_id: "my_game".to_owned(),
			game_name: "My Game".to_owned(),
			map_formats: vec!["mapone".to_owned(), "maptwo".to_owned()],
			vfs_formats: vec!["directory".to_owned()],
			default_compile_tuning_parameters: None,
		};

		let v1_file_out: V1File = V1File::from(&config);
		let toml_string: String = v1_file_out.to_toml_string().unwrap();
		let v1_file_in: V1File = V1File::from_toml_str(toml_string.as_str()).unwrap();
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

		let deserialized_data = V1File::from_toml_str(toml_string);
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

		let deserialized_data = V1File::from_toml_str(toml_string);
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

		let deserialized_data = V1File::from_toml_str(toml_string);
		let error = deserialized_data.expect_err("Expected deserialization to fail");
		let error_string: String = error.to_string();
		let suffix: &str = "missing field `version`\n";

		assert!(
			error_string.ends_with(suffix),
			"Error string:\n  \"{error_string}\"\nshould end with suffix\n  \"{suffix}\""
		);
	}

	#[test]
	fn deserialize_with_invalid_game_id()
	{
		{
			let toml_string: &str = r##"
				format = "gameconfig"
				version = 1
				game_id = "contains spaces"
				game_name = "My Game"
				map_formats = ["mapone", "maptwo"]
				vfs_formats = ["directory"]"##;

			let deserialized_data = V1File::from_toml_str(toml_string);
			let error = deserialized_data.expect_err("Expected deserialization to fail");
			let error_string: String = error.to_string();

			assert_eq!(
				error_string,
				json!({
					"errors": [],
					"properties": {
						"game_id": {
							"errors": [
								"Game ID must be alphanumeric and non-empty"
							]
						}
					}
				})
				.to_string()
			);
		}

		{
			let toml_string: &str = r##"
				format = "gameconfig"
				version = 1
				game_id = " startswithspace"
				game_name = "My Game"
				map_formats = ["mapone", "maptwo"]
				vfs_formats = ["directory"]"##;

			let deserialized_data = V1File::from_toml_str(toml_string);
			let error = deserialized_data.expect_err("Expected deserialization to fail");
			let error_string: String = error.to_string();

			assert_eq!(
				error_string,
				json!({
					"errors": [],
					"properties": {
						"game_id": {
							"errors": [
								"Game ID must be alphanumeric and non-empty"
							]
						}
					}
				})
				.to_string()
			);
		}

		{
			let toml_string: &str = r##"
				format = "gameconfig"
				version = 1
				game_id = ""
				game_name = "My Game"
				map_formats = ["mapone", "maptwo"]
				vfs_formats = ["directory"]"##;

			let deserialized_data = V1File::from_toml_str(toml_string);
			let error = deserialized_data.expect_err("Expected deserialization to fail");
			let error_string: String = error.to_string();

			assert_eq!(
				error_string,
				json!({
					"errors": [],
					"properties": {
						"game_id": {
							"errors": [
								"Game ID must be alphanumeric and non-empty"
							]
						}
					}
				})
				.to_string()
			);
		}
	}

	#[test]
	fn deserialize_with_invalid_game_name()
	{
		let toml_string: &str = r##"
			format = "gameconfig"
			version = 1
			game_id = "my_game"
			game_name = ""
			map_formats = ["mapone", "maptwo"]
			vfs_formats = ["directory"]"##;

		let deserialized_data = V1File::from_toml_str(toml_string);
		let error = deserialized_data.expect_err("Expected deserialization to fail");
		let error_string: String = error.to_string();

		assert_eq!(
			error_string,
			json!({
				"errors": [],
				"properties": {
					"game_name": {
						"errors": [
							"Game name cannot be empty"
						]
					}
				}
			})
			.to_string()
		);
	}

	#[test]
	fn deserialize_with_invalid_map_formats()
	{
		{
			let toml_string: &str = r##"
				format = "gameconfig"
				version = 1
				game_id = "my_game"
				game_name = "My Game"
				map_formats = []
				vfs_formats = ["directory"]"##;

			let deserialized_data = V1File::from_toml_str(toml_string);
			let error = deserialized_data.expect_err("Expected deserialization to fail");
			let error_string: String = error.to_string();

			assert_eq!(
				error_string,
				json!({
					"errors": [],
					"properties": {
						"map_formats": {
							"errors": [
								"Expected at least one map format"
							]
						}
					}
				})
				.to_string()
			);
		}

		{
			let toml_string: &str = r##"
				format = "gameconfig"
				version = 1
				game_id = "my_game"
				game_name = "My Game"
				map_formats = ["one", "one"]
				vfs_formats = ["directory"]"##;

			let deserialized_data = V1File::from_toml_str(toml_string);
			let error = deserialized_data.expect_err("Expected deserialization to fail");
			let error_string: String = error.to_string();

			assert_eq!(
				error_string,
				json!({
					"errors": [],
					"properties": {
						"map_formats": {
							"errors": [
								"Duplicate map formats are not allowed"
							]
						}
					}
				})
				.to_string()
			);
		}

		{
			let toml_string: &str = r##"
				format = "gameconfig"
				version = 1
				game_id = "my_game"
				game_name = "My Game"
				map_formats = ["one", "   "]
				vfs_formats = ["directory"]"##;

			let deserialized_data = V1File::from_toml_str(toml_string);
			let error = deserialized_data.expect_err("Expected deserialization to fail");
			let error_string: String = error.to_string();

			assert_eq!(
				error_string,
				json!({
					"errors": [],
					"properties": {
						"map_formats": {
							"errors": [
								"Index 1: The length of the value must be `>= 1`."
							]
						}
					}
				})
				.to_string()
			);
		}
	}

	#[test]
	fn deserialize_with_invalid_vfs_formats()
	{
		{
			let toml_string: &str = r##"
				format = "gameconfig"
				version = 1
				game_id = "my_game"
				game_name = "My Game"
				map_formats = ["map"]
				vfs_formats = []"##;

			let deserialized_data = V1File::from_toml_str(toml_string);
			let error = deserialized_data.expect_err("Expected deserialization to fail");
			let error_string: String = error.to_string();

			assert_eq!(
				error_string,
				json!({
					"errors": [],
					"properties": {
						"vfs_formats": {
							"errors": [
								"Expected at least one VFS format"
							]
						}
					}
				})
				.to_string()
			);
		}

		{
			let toml_string: &str = r##"
				format = "gameconfig"
				version = 1
				game_id = "my_game"
				game_name = "My Game"
				map_formats = ["map"]
				vfs_formats = ["directory", "directory"]"##;

			let deserialized_data = V1File::from_toml_str(toml_string);
			let error = deserialized_data.expect_err("Expected deserialization to fail");
			let error_string: String = error.to_string();

			assert_eq!(
				error_string,
				json!({
					"errors": [],
					"properties": {
						"vfs_formats": {
							"errors": [
								"Duplicate VFS formats are not allowed"
							]
						}
					}
				})
				.to_string()
			);
		}

		{
			let toml_string: &str = r##"
				format = "gameconfig"
				version = 1
				game_id = "my_game"
				game_name = "My Game"
				map_formats = ["map"]
				vfs_formats = ["   ", "directory"]"##;

			let deserialized_data = V1File::from_toml_str(toml_string);
			let error = deserialized_data.expect_err("Expected deserialization to fail");
			let error_string: String = error.to_string();

			assert_eq!(
				error_string,
				json!({
					"errors": [],
					"properties": {
						"vfs_formats": {
							"errors": [
								"Index 0: The length of the value must be `>= 1`."
							]
						}
					}
				})
				.to_string()
			);
		}
	}
}
