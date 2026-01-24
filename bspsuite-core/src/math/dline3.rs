use crate::math::fuzzy_comparison::length_is_equal;
use glam::DVec3;

#[derive(Clone, Copy)]
pub struct DLine3
{
	pub origin: DVec3,
	pub direction: DVec3,
}

impl DLine3
{
	pub const NULL: Self = Self::new(DVec3::ZERO, DVec3::ZERO);

	// Does NOT enforce that the direction is a unit vector!
	#[inline]
	#[must_use = "Constructed line was not used"]
	pub const fn new(origin: DVec3, direction: DVec3) -> Self
	{
		return DLine3 { origin, direction };
	}

	// Converts direction to a unit vector, or a zero vector
	// if normalisation is not possible.
	#[inline]
	#[must_use = "Constructed line was not used"]
	pub fn unit_from_point_and_direction(origin: DVec3, direction: DVec3) -> Self
	{
		return DLine3::new(origin, direction.normalize_or_zero());
	}

	// Converts direction to a unit vector, or a zero vector
	// if normalisation is not possible.
	#[inline]
	#[must_use = "Constructed line was not used"]
	pub fn unit_from_points(p0: DVec3, p1: DVec3) -> Self
	{
		return DLine3::unit_from_point_and_direction(p0, p1 - p0);
	}

	#[inline]
	pub fn direction_is_null(&self) -> bool
	{
		return self.direction == DVec3::ZERO;
	}

	#[inline]
	#[must_use = "Returned point was not used"]
	pub fn parametric_point(&self, t: f64) -> DVec3
	{
		return self.origin + (t * self.direction);
	}

	#[inline]
	pub fn direction_is_unit_vector(&self, epsilon: f64) -> bool
	{
		return length_is_equal(self.direction, 1.0, epsilon);
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn construct_null_line()
	{
		let null1: DLine3 = DLine3 {
			origin: DVec3::ZERO,
			direction: DVec3::ZERO,
		};

		assert!(null1.direction_is_null());

		let null2: DLine3 = DLine3::new(DVec3::ZERO, DVec3::ZERO);
		assert!(null2.direction_is_null());

		let null3: DLine3 = DLine3::new(DVec3::new(1.0, 2.0, 3.0), DVec3::ZERO);
		assert!(null3.direction_is_null());

		let null4: DLine3 =
			DLine3::unit_from_point_and_direction(DVec3::new(1.0, 2.0, 3.0), DVec3::ZERO);
		assert!(null4.direction_is_null());

		let null5: DLine3 = DLine3::unit_from_point_and_direction(
			DVec3::new(1.0, 2.0, 3.0),
			DVec3::new(1e-6, 0.0, 0.0),
		);
		assert!(null5.direction_is_null());

		let null6: DLine3 =
			DLine3::unit_from_points(DVec3::new(1.0, 2.0, 3.0), DVec3::new(1.0, 2.0, 3.0));
		assert!(null6.direction_is_null());

		assert!(DLine3::NULL.direction_is_null());
		assert_eq!(DLine3::NULL.origin, DVec3::ZERO);
		assert_eq!(DLine3::NULL.direction, DVec3::ZERO);
	}
}
