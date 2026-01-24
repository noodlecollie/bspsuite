use crate::math::fuzzy_comparison::{length_is_equal, length_is_zero, values_are_equal};
use crate::math::{DLine3, DPlane3};
use glam::DVec3;

// Built on https://en.wikipedia.org/wiki/Plane%E2%80%93plane_intersection#Formulation
pub fn intersect_planes(a: &DPlane3, b: &DPlane3, zero_epsilon: f64) -> Option<DLine3>
{
	if a.is_null(zero_epsilon) || b.is_null(zero_epsilon)
	{
		return None;
	}

	debug_assert!(
		length_is_equal(a.normal, 1.0, zero_epsilon),
		"Expected first plane to have unit normal"
	);

	debug_assert!(
		length_is_equal(b.normal, 1.0, zero_epsilon),
		"Expected second plane to have unit normal"
	);

	let intersection_dir: DVec3 = a.normal.cross(b.normal);

	if length_is_zero(intersection_dir, zero_epsilon)
	{
		return None;
	}

	let dot_normals: f64 = a.normal.dot(b.normal);

	debug_assert!(
		!values_are_equal(dot_normals, 1.0, zero_epsilon),
		"Expected dot product of plane normals to be non-zero"
	);

	let h1: f64 = -a.distance;
	let h2: f64 = -b.distance;
	let one_minus_dot_normals_sq: f64 = 1.0 - (dot_normals * dot_normals);
	let c1: f64 = (h1 - (h2 * dot_normals)) / one_minus_dot_normals_sq;
	let c2: f64 = (h2 - (h1 * dot_normals)) / one_minus_dot_normals_sq;
	let line_origin: DVec3 = (c1 * a.normal) + (c2 * b.normal);

	return Some(DLine3::unit_from_point_and_direction(
		line_origin,
		intersection_dir,
	));
}
