use crate::math::DEFAULT_ZERO_EPSILON;
use crate::math::comparison::{length_is_equal, length_is_zero};
use crate::math::geometry::vector_to_unit_or_null;
use glam::DVec3;

// Plane in the form ax + by + cz + d = 0
// Normal = (a, b, c)
// Distance = d
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DPlane3
{
	_normal: DVec3,
	_distance: f64,
}

impl DPlane3
{
	pub const NULL: Self = Self {
		_normal: DVec3::new(0.0, 0.0, 0.0),
		_distance: 0.0,
	};

	#[inline]
	#[must_use = "Constructed plane was not used"]
	pub fn new(normal: DVec3, distance: f64, zero_epsilon: f64) -> Self
	{
		let (normal, not_null) = vector_to_unit_or_null(normal, zero_epsilon);

		return if not_null
		{
			Self {
				_normal: normal.normalize(),
				_distance: distance,
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
			_normal: normal,
			_distance: distance,
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
		return self._normal;
	}

	#[inline]
	pub fn distance(&self) -> f64
	{
		return self._distance;
	}

	#[inline]
	pub fn origin(&self) -> DVec3
	{
		return self._normal * self._distance;
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
			_normal: DVec3::new(0.0, 0.0, 0.0),
			_distance: 0.0,
		};

		assert!(null1.is_null());

		let null2: DPlane3 = DPlane3::NULL.clone();
		assert!(null2.is_null());

		assert!(DPlane3::NULL.is_null());
		assert_eq!(&null1, &DPlane3::NULL);
		assert_eq!(&null2, &DPlane3::NULL);
	}
}
