// These defaults are set to match those in sources like
// https://github.com/ericwa/ericw-tools/tree/main/include/common/mathlib.hh and
// https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/public/mathlib/vplane.h
pub const DEFAULT_CONTACT_EPSILON: f64 = 0.1;
pub const DEFAULT_EQUAL_POINT_RADIUS_EPSILON: f64 = 0.05;
pub const DEFAULT_EQUAL_VECTOR_COMPONENT_EPSILON: f64 = 0.000001;
pub const DEFAULT_ZERO_EPSILON: f64 = 0.0001;

// For descriptions of these values, see documentation for
// CompileTuningParametersConfig.
pub struct CompileTuningParameters
{
	pub contact_epsilon: f64,
	pub equal_point_radius_epsilon: f64,
	pub equal_vector_component_epsilon: f64,
	pub zero_epsilon: f64,
}

impl Default for CompileTuningParameters
{
	fn default() -> Self
	{
		return Self {
			contact_epsilon: DEFAULT_CONTACT_EPSILON,
			equal_point_radius_epsilon: DEFAULT_EQUAL_POINT_RADIUS_EPSILON,
			equal_vector_component_epsilon: DEFAULT_EQUAL_VECTOR_COMPONENT_EPSILON,
			zero_epsilon: DEFAULT_ZERO_EPSILON,
		};
	}
}
