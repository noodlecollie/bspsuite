use glam::DVec3;

use crate::math::compile_tuning_parameters::DEFAULT_ZERO_EPSILON;
use crate::math::fuzzy_comparison::{length_is_equal, length_is_zero};

// Plane in the form ax + by + cz + d = 0
// Normal = (a, b, c)
// Distance = d
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DPlane3
{
	pub normal: DVec3,
	pub distance: f64,
}

impl DPlane3
{
	pub const NULL: Self = Self::new_xyzd(0.0, 0.0, 0.0, 0.0);

	#[inline]
	#[must_use = "Constructed plane was not used"]
	pub const fn new(normal: DVec3, distance: f64) -> Self
	{
		debug_assert!(
			length_is_zero(normal, DEFAULT_ZERO_EPSILON)
				|| length_is_equal(normal, 1.0, DEFAULT_ZERO_EPSILON),
			"Expected plane normal to be null or a unit vector"
		);

		return DPlane3 { normal, distance };
	}

	#[inline]
	#[must_use = "Constructed plane was not used"]
	pub const fn new_xyzd(x: f64, y: f64, z: f64, d: f64) -> Self
	{
		return Self::new(DVec3::new(x, y, z), d);
	}

	#[inline]
	pub fn is_null(&self, zero_epsilon: f64) -> bool
	{
		return length_is_zero(self.normal, zero_epsilon);
	}

	#[inline]
	pub fn origin(&self) -> DVec3
	{
		return self.normal * self.distance;
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn construct_null_plane()
	{
		let null1: DPlane3 = DPlane3 {
			normal: DVec3::new(0.0, 0.0, 0.0),
			distance: 0.0,
		};

		assert!(null1.is_null(DEFAULT_ZERO_EPSILON));

		let null2: DPlane3 = DPlane3::new(DVec3::new(0.0, 0.0, 0.0), 0.0);
		assert!(null2.is_null(DEFAULT_ZERO_EPSILON));

		let null3: DPlane3 = DPlane3::new_xyzd(0.0, 0.0, 0.0, 0.0);
		assert!(null3.is_null(DEFAULT_ZERO_EPSILON));

		let null4: DPlane3 = DPlane3::NULL.clone();
		assert!(null4.is_null(DEFAULT_ZERO_EPSILON));

		assert!(DPlane3::NULL.is_null(DEFAULT_ZERO_EPSILON));
		assert_eq!(&null1, &DPlane3::NULL);
		assert_eq!(&null2, &DPlane3::NULL);
		assert_eq!(&null3, &DPlane3::NULL);
		assert_eq!(&null4, &DPlane3::NULL);
	}
}
