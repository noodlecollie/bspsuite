use crate::math::DEFAULT_ZERO_EPSILON;
use crate::math::comparison::{length_is_equal, length_is_zero};
use crate::math::geometry::vector_to_unit_or_null;
use glam::DVec3;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DLine3
{
	_origin: DVec3,
	_direction: DVec3,
}

impl DLine3
{
	pub const NULL: Self = Self {
		_origin: DVec3::ZERO,
		_direction: DVec3::ZERO,
	};

	// Converts direction to a unit vector, or a zero vector
	// if normalisation is not possible.
	#[inline]
	#[must_use = "Constructed line was not used"]
	pub fn from_point_and_direction(origin: DVec3, direction: DVec3, zero_epsilon: f64) -> Self
	{
		let (direction, not_null) = vector_to_unit_or_null(direction, zero_epsilon);

		return if not_null
		{
			DLine3::from_point_and_direction_unchecked(origin, direction)
		}
		else
		{
			DLine3::NULL
		};
	}

	// Converts direction to a unit vector, or a zero vector
	// if normalisation is not possible.
	#[inline]
	#[must_use = "Constructed line was not used"]
	pub fn from_points(p0: DVec3, p1: DVec3, zero_epsilon: f64) -> Self
	{
		let (direction, not_null) = vector_to_unit_or_null(p1 - p0, zero_epsilon);

		return if not_null
		{
			DLine3::from_point_and_direction_unchecked(p0, direction)
		}
		else
		{
			DLine3::NULL
		};
	}

	// In release mode, no checks are performed to make sure that the direction is
	// valid.
	#[inline]
	#[must_use = "Constructed line was not used"]
	pub fn from_point_and_direction_unchecked(origin: DVec3, direction: DVec3) -> Self
	{
		debug_assert!(
			length_is_zero(direction, DEFAULT_ZERO_EPSILON)
				|| length_is_equal(direction, 1.0, DEFAULT_ZERO_EPSILON)
		);

		return Self {
			_origin: origin,
			_direction: direction,
		};
	}

	#[inline]
	pub fn is_null(&self) -> bool
	{
		return self == &DLine3::NULL;
	}

	#[inline]
	pub fn origin(&self) -> DVec3
	{
		return self._origin;
	}

	#[inline]
	pub fn direction(&self) -> DVec3
	{
		return self._direction;
	}

	#[inline]
	#[must_use = "Returned point was not used"]
	pub fn parametric_point(&self, t: f64) -> DVec3
	{
		return self._origin + (t * self._direction);
	}
}

#[cfg(test)]
mod tests
{
	use super::*;
	use crate::math::DEFAULT_ZERO_EPSILON;

	#[test]
	fn construct_null_line()
	{
		let null1: DLine3 = DLine3 {
			_origin: DVec3::ZERO,
			_direction: DVec3::ZERO,
		};

		assert!(null1.is_null());

		let null2: DLine3 = DLine3::from_point_and_direction(
			DVec3::new(1.0, 2.0, 3.0),
			DVec3::ZERO,
			DEFAULT_ZERO_EPSILON,
		);
		assert!(null2.is_null());

		let null3: DLine3 = DLine3::from_point_and_direction(
			DVec3::new(1.0, 2.0, 3.0),
			DVec3::new(1e-6, 0.0, 0.0),
			DEFAULT_ZERO_EPSILON,
		);
		assert!(null3.is_null());

		let null4: DLine3 = DLine3::from_points(
			DVec3::new(1.0, 2.0, 3.0),
			DVec3::new(1.0, 2.0, 3.0),
			DEFAULT_ZERO_EPSILON,
		);
		assert!(null4.is_null());

		assert!(DLine3::NULL.is_null());
		assert_eq!(DLine3::NULL._origin, DVec3::ZERO);
		assert_eq!(DLine3::NULL._direction, DVec3::ZERO);
	}
}
