use crate::math::DEFAULT_ZERO_EPSILON;
use crate::math::comparison::{length_is_equal, length_is_zero};
use crate::math::geometry::vector_to_unit_or_null;
use glam::DVec3;
use serde::{Deserialize, Serialize};

// Plane in the form ax + by + cz + d = 0
// Normal = (a, b, c)
// Distance = d
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct DPlane3
{
	#[serde(rename = "normal")]
	norm: DVec3,

	#[serde(rename = "distance")]
	dist: f64,
}

impl DPlane3
{
	pub const NULL: Self = Self {
		norm: DVec3::new(0.0, 0.0, 0.0),
		dist: 0.0,
	};

	#[inline]
	#[must_use = "Constructed plane was not used"]
	pub fn new(normal: DVec3, distance: f64, zero_epsilon: f64) -> Self
	{
		let (normal, not_null) = vector_to_unit_or_null(normal, zero_epsilon);

		return if not_null
		{
			Self {
				norm: normal,
				dist: distance,
			}
		}
		else
		{
			DPlane3::NULL
		};
	}

	// In release mode, no checks are performed to make sure that the normal is
	// valid.
	#[inline]
	#[must_use = "Constructed plane was not used"]
	pub const fn new_unchecked(normal: DVec3, distance: f64) -> Self
	{
		debug_assert!(
			length_is_zero(normal, DEFAULT_ZERO_EPSILON)
				|| length_is_equal(normal, 1.0, DEFAULT_ZERO_EPSILON)
		);

		return Self {
			norm: normal,
			dist: distance,
		};
	}

	#[inline]
	pub fn is_null(&self) -> bool
	{
		return self == &DPlane3::NULL;
	}

	#[inline]
	pub fn normal(&self) -> DVec3
	{
		return self.norm;
	}

	#[inline]
	pub fn distance(&self) -> f64
	{
		return self.dist;
	}

	#[inline]
	pub fn origin(&self) -> DVec3
	{
		return self.norm * self.dist;
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
			norm: DVec3::new(0.0, 0.0, 0.0),
			dist: 0.0,
		};

		assert!(null1.is_null());

		let null2: DPlane3 = DPlane3::NULL.clone();
		assert!(null2.is_null());

		assert!(DPlane3::NULL.is_null());
		assert_eq!(&null1, &DPlane3::NULL);
		assert_eq!(&null2, &DPlane3::NULL);
	}
}
