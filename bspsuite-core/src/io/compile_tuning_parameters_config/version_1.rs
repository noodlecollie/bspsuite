use crate::configs::CompileTuningParametersConfig;
use crate::io::helpers::{IOFmtSignature, IOFormat};
use serde::{Deserialize, Serialize};

pub(super) const VERSION: u64 = 1;

#[derive(Serialize, Deserialize, Debug)]
pub struct V1File
{
	signature: IOFmtSignature<V1File>,
	pub contact_epsilon: Option<f64>,
	pub equal_point_radius_epsilon: Option<f64>,
	pub equal_vector_component_epsilon: Option<f64>,
	pub zero_epsilon: Option<f64>,
}

impl IOFormat for V1File
{
	fn format_name() -> &'static str
	{
		return "compiletuningparamsconfig";
	}

	fn format_version() -> u64
	{
		return VERSION;
	}

	fn type_desc() -> &'static str
	{
		return "compile tuning parameters config";
	}
}

////////////////////////////////////////////////////////
// CompileTuningParametersConfig -> V1File
////////////////////////////////////////////////////////

impl From<&CompileTuningParametersConfig> for V1File
{
	fn from(value: &CompileTuningParametersConfig) -> Self
	{
		return Self {
			signature: IOFmtSignature::new(),
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

impl From<V1File> for CompileTuningParametersConfig
{
	fn from(value: V1File) -> Self
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
		let config: CompileTuningParametersConfig = CompileTuningParametersConfig {
			contact_epsilon: Some(1e-3),
			equal_point_radius_epsilon: Some(1e-4),
			equal_vector_component_epsilon: Some(1e-2),
			zero_epsilon: Some(1e-3),
		};

		let v1_file_out: V1File = V1File::from(&config);
		let json_string: String = serde_json::to_string(&v1_file_out).unwrap();
		let v1_file_in: V1File = serde_json::from_str::<V1File>(&json_string).unwrap();
		let recovered_config: CompileTuningParametersConfig = v1_file_in.into();

		assert_eq!(config, recovered_config);
	}
}
