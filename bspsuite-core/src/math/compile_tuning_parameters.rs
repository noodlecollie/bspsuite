// These defaults are set to match those in sources like
//  https://github.com/ericwa/ericw-tools/tree/main/include/common/mathlib.hh and
// https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/public/mathlib/vplane.h
pub const DEFAULT_ON_PLANE_EPSILON: f64 = 0.1;

pub struct CompileTuningParameters
{
	pub on_plane_epsilon: f64,
}

impl Default for CompileTuningParameters
{
	fn default() -> Self
	{
		return Self {
			on_plane_epsilon: DEFAULT_ON_PLANE_EPSILON,
		};
	}
}
