use crate::math::{Classification, DPlane3};
use glam::DVec3;

pub fn classify_point_against_plane(point: DVec3, plane: DPlane3, tolerance: f64)
-> Classification
{
	let dist: f64 = plane.normal.dot(point) - plane.distance;

	if dist < -tolerance
	{
		return Classification::Behind;
	}
	else if dist > tolerance
	{
		return Classification::InFront;
	}
	else
	{
		return Classification::On;
	}
}
