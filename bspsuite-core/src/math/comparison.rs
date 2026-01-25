use crate::math::DPlane3;
use crate::math::const_fns::{dvec3_length_sq, dvec3_subtract};
use glam::DVec3;

#[inline]
pub const fn points_are_equal_radial_sq(a: DVec3, b: DVec3, zero_epsilon_squared: f64) -> bool
{
	return value_is_zero(dvec3_length_sq(dvec3_subtract(b, a)), zero_epsilon_squared);
}

#[inline]
pub const fn points_are_equal_radial(a: DVec3, b: DVec3, zero_epsilon: f64) -> bool
{
	return points_are_equal_radial_sq(a, b, zero_epsilon * zero_epsilon);
}

#[inline]
pub const fn length_is_equal_sq(vec: DVec3, val: f64, zero_epsilon_squared: f64) -> bool
{
	return values_are_equal(dvec3_length_sq(vec), val, zero_epsilon_squared);
}

#[inline]
pub const fn length_is_equal(vec: DVec3, val: f64, zero_epsilon: f64) -> bool
{
	return length_is_equal_sq(vec, val * val, zero_epsilon * zero_epsilon);
}

#[inline]
pub const fn length_is_zero(vec: DVec3, zero_epsilon: f64) -> bool
{
	return value_is_zero(dvec3_length_sq(vec), zero_epsilon * zero_epsilon);
}

#[inline]
pub const fn values_are_equal(a: f64, b: f64, zero_epsilon: f64) -> bool
{
	return value_is_zero(b - a, zero_epsilon);
}

#[inline]
pub const fn value_is_zero(val: f64, zero_epsilon: f64) -> bool
{
	return val.abs() < zero_epsilon;
}

#[inline]
pub const fn vectors_are_equal(a: DVec3, b: DVec3, component_epsilon: f64) -> bool
{
	return (b.x - a.x).abs() < component_epsilon
		&& (b.y - a.y).abs() < component_epsilon
		&& (b.z - a.z).abs() < component_epsilon;
}
