use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct CompileTuningParametersConfig
{
	/// Value for deciding whether a point lies on a plane. If the perpendicular
	/// distance between the plane surface and the point is less than this
	/// value, the point is considered on the plane.
	///
	/// This value is used by operations such as construction of brushes and
	/// binary space partitioning, where objects are split or sorted with
	/// reference to a plane. Making the value too small may cause floating
	/// point rounding errors to fragment or corrupt geometry; making it too
	/// large may cause very precice geometry to be unnecessarily coalesced.
	pub on_plane_epsilon: Option<f64>,

	/// Value for deciding whether two points in space are considered equal. If
	/// the distance between the two points is less than this value, they are
	/// considered equal.
	///
	/// This value is used by operations such as construction of brushes, where
	/// independent intersection points are generated and compared. Making the
	/// value too small may cause erroneous duplication of vertices; making it
	/// too large may cause vertices very close together to be merged.
	pub equal_point_radius_epsilon: Option<f64>,

	/// Value for deciding whether two vectors are component-wise equal. The
	/// vectors are considered equal if, for each pair of their components, the
	/// absolute difference between the components is less than this value.
	///
	/// This value is different to [equal_point_radius_epsilon] because it is
	/// intended to mitigate against rounding errors when comparing individual
	/// floating point values, and so is less forgiving than
	/// [equal_point_radius_epsilon]. It is primarily used for computing the
	/// equality of directional vectors such as normals, and as such does not
	/// have a straightforward relation to physical space like
	/// [equal_point_radius_epsilon].
	pub equal_vector_component_epsilon: Option<f64>,

	/// Value for deciding if a number should be considered zero. If the
	/// magnitude of the number is less than this value, is is considered zero.
	///
	/// This value is commonly used when comparing small vector lengths for
	/// equality, by deciding whether the difference between the lengths of the
	/// two vectors is small enough to be negligible. This method is used in
	/// preference to [equal_vector_component_epsilon] when it is the length of
	/// the vectors that is salient, rather than their particular components.
	pub zero_epsilon: Option<f64>,
}

impl CompileTuningParametersConfig
{
	pub fn load(path: &Path) -> Result<Self>
	{
		todo!();
	}

	pub fn merge(existing_cfg: Self, override_cfg: Self) -> Self
	{
		return Self {
			on_plane_epsilon: decide_property(
				existing_cfg.on_plane_epsilon,
				override_cfg.on_plane_epsilon,
			),
			equal_point_radius_epsilon: decide_property(
				existing_cfg.equal_point_radius_epsilon,
				override_cfg.equal_point_radius_epsilon,
			),
			equal_vector_component_epsilon: decide_property(
				existing_cfg.equal_vector_component_epsilon,
				override_cfg.equal_vector_component_epsilon,
			),
			zero_epsilon: decide_property(existing_cfg.zero_epsilon, override_cfg.zero_epsilon),
		};
	}
}

fn decide_property<T>(existing_prop: Option<T>, override_prop: Option<T>) -> Option<T>
{
	return match override_prop
	{
		Some(prop) => Some(prop),
		None => existing_prop,
	};
}
