// These defaults are set to match those in sources like
// https://github.com/ericwa/ericw-tools/tree/main/include/common/mathlib.hh and
// https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/public/mathlib/vplane.h
const DEFAULT_ON_PLANE_EPSILON: f64 = 0.1;
const DEFAULT_EQUAL_POINT_RADIUS_EPSILON: f64 = 0.05;

pub struct CompileTuningParameters
{
	pub on_plane_epsilon: f64,
	pub equal_point_radius_epsilon: f64,
}

impl Default for CompileTuningParameters
{
	fn default() -> Self
	{
		return Self {
			on_plane_epsilon: DEFAULT_ON_PLANE_EPSILON,
			equal_point_radius_epsilon: DEFAULT_EQUAL_POINT_RADIUS_EPSILON,
		};
	}
}
