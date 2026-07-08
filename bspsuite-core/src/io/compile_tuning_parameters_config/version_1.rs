use crate::configs::CompileTuningParametersConfig;
use crate::io::helpers::DeserializeVersionHelper;
use serde::{Deserialize, Serialize};

pub(super) const VERSION: u64 = 1;

#[derive(Serialize, Deserialize, Debug)]
pub struct V1File
{
	#[serde(deserialize_with = "DeserializeVersionHelper::<VERSION>::deserialize_version")]
	version: u64,

	pub contact_epsilon: Option<f64>,
	pub equal_point_radius_epsilon: Option<f64>,
	pub equal_vector_component_epsilon: Option<f64>,
	pub zero_epsilon: Option<f64>,
}

////////////////////////////////////////////////////////
// CompileTuningParametersConfig -> V1File
////////////////////////////////////////////////////////

impl From<&CompileTuningParametersConfig> for V1File
{
	fn from(value: &CompileTuningParametersConfig) -> Self
	{
		return Self {
			version: VERSION,
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
