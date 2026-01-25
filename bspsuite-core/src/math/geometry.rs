use crate::math::comparison::{length_is_zero, value_is_zero, values_are_equal};
use crate::math::{DLine3, DPlane3};
use glam::DVec3;

pub enum LinePlaneIntersection
{
	None,
	Point(DVec3),
	InPlane,
}

impl LinePlaneIntersection
{
	pub fn is_intersection(&self) -> bool
	{
		return match self
		{
			LinePlaneIntersection::None => false,
			_ => true,
		};
	}

	pub fn is_single_intersection(&self) -> bool
	{
		return match self
		{
			LinePlaneIntersection::Point(_) => true,
			_ => false,
		};
	}

	pub fn is_planar_intersection(&self) -> bool
	{
		return match self
		{
			LinePlaneIntersection::InPlane => true,
			_ => false,
		};
	}

	pub fn is_no_intersection(&self) -> bool
	{
		return match self
		{
			LinePlaneIntersection::None => true,
			_ => false,
		};
	}

	pub fn intersection_point(&self) -> Option<DVec3>
	{
		return match self
		{
			LinePlaneIntersection::Point(p) => Some(*p),
			_ => None,
		};
	}
}

// Given a vector, returns (normalised, true) if the vector's length was not
// zero, or (zero, false) if it was zero.
pub fn vector_to_unit_or_null(vec: DVec3, zero_epsilon: f64) -> (DVec3, bool)
{
	let length_sq: f64 = vec.length_squared();
	let eps_sq: f64 = zero_epsilon * zero_epsilon;

	if value_is_zero(length_sq, eps_sq)
	{
		return (DVec3::ZERO, false);
	}
	else if values_are_equal(length_sq, 1.0, eps_sq)
	{
		return (vec, true);
	}
	else
	{
		return (vec.normalize(), true);
	}
}

// Built on https://en.wikipedia.org/wiki/Plane%E2%80%93plane_intersection#Formulation
// The direction of the intersection line is right-handed with respect to the
// plane normals.
pub fn intersect_planes(a: &DPlane3, b: &DPlane3, zero_epsilon: f64) -> Option<DLine3>
{
	assert!(!a.is_null() && !b.is_null(), "Expected non-null planes");

	let intersection_dir: DVec3 = a.normal().cross(b.normal());

	if length_is_zero(intersection_dir, zero_epsilon)
	{
		return None;
	}

	let dot_normals: f64 = a.normal().dot(b.normal());

	debug_assert!(
		!values_are_equal(dot_normals, 1.0, zero_epsilon),
		"Expected dot product of plane normals to be non-zero"
	);

	let h1: f64 = a.distance();
	let h2: f64 = b.distance();
	let one_minus_dot_normals_sq: f64 = 1.0 - (dot_normals * dot_normals);
	let c1: f64 = (h1 - (h2 * dot_normals)) / one_minus_dot_normals_sq;
	let c2: f64 = (h2 - (h1 * dot_normals)) / one_minus_dot_normals_sq;
	let line_origin: DVec3 = (c1 * a.normal()) + (c2 * b.normal());

	return Some(DLine3::from_point_and_direction(
		line_origin,
		intersection_dir,
		zero_epsilon,
	));
}

pub fn intersect_line_and_plane(
	line: &DLine3,
	plane: &DPlane3,
	zero_epsilon: f64,
	contact_epsilon: f64,
) -> LinePlaneIntersection
{
	assert!(!line.is_null(), "Expected non-null line");
	assert!(!plane.is_null(), "Expected non-null plane");

	let plane_normal: DVec3 = plane.normal();
	let line_dot_plane_normal: f64 = line.direction().dot(plane_normal);

	if value_is_zero(line_dot_plane_normal, zero_epsilon)
	{
		return if point_lies_on_plane(line.origin(), plane, contact_epsilon)
		{
			LinePlaneIntersection::InPlane
		}
		else
		{
			return LinePlaneIntersection::None;
		};
	}

	let line_to_plane: DVec3 = plane.origin() - line.origin();
	let multiples_of_normal: f64 = line_to_plane.dot(plane_normal);
	let line_parameter: f64 = multiples_of_normal / line_dot_plane_normal;

	return LinePlaneIntersection::Point(line.parametric_point(line_parameter));
}

pub fn point_distance_from_plane(point: DVec3, plane: &DPlane3) -> f64
{
	let projected_point: DVec3 = project_point_onto_plane(point, plane);
	return (point - projected_point).length();
}

pub fn project_point_onto_plane(point: DVec3, plane: &DPlane3) -> DVec3
{
	assert!(!plane.is_null(), "Expected non-null plane");

	let origin_to_point: DVec3 = point - plane.origin();
	let normal: DVec3 = plane.normal();
	let multiples_of_normal: f64 = normal.dot(origin_to_point);

	return point - (multiples_of_normal * normal);
}

pub fn point_lies_on_plane(point: DVec3, plane: &DPlane3, contact_epsilon: f64) -> bool
{
	return point_distance_from_plane(point, plane).abs() < contact_epsilon;
}

pub fn point_distance_from_line(point: DVec3, line: &DLine3) -> f64
{
	let projected_point: DVec3 = project_point_onto_line(point, line);
	return (point - projected_point).length();
}

pub fn project_point_onto_line(point: DVec3, line: &DLine3) -> DVec3
{
	assert!(!line.is_null(), "Expected non-null line");

	let line_org_to_point: DVec3 = point - line.origin();
	let multiples_of_direction = line.direction().dot(line_org_to_point);

	return line.origin() + (multiples_of_direction * line.direction());
}

pub fn point_lies_on_line(point: DVec3, line: &DLine3, contact_epsilon: f64) -> bool
{
	return point_distance_from_line(point, line).abs() < contact_epsilon;
}

#[cfg(test)]
mod tests
{
	use super::*;
	use crate::math::comparison::vectors_are_equal;
	use crate::math::{
		DEFAULT_CONTACT_EPSILON, DEFAULT_EQUAL_VECTOR_COMPONENT_EPSILON, DEFAULT_ZERO_EPSILON,
	};

	#[test]
	fn plane_intersections()
	{
		{
			let plane1: DPlane3 =
				DPlane3::new(DVec3::new(1.0, 0.0, 0.0), 5.0, DEFAULT_ZERO_EPSILON);
			let plane2: DPlane3 =
				DPlane3::new(DVec3::new(1.0, 0.0, 0.0), 10.0, DEFAULT_ZERO_EPSILON);
			let result = intersect_planes(&plane1, &plane2, DEFAULT_ZERO_EPSILON);
			assert!(result.is_none());
		}

		{
			let plane1: DPlane3 =
				DPlane3::new(DVec3::new(1.0, 0.0, 0.0), 5.0, DEFAULT_ZERO_EPSILON);
			let plane2: DPlane3 =
				DPlane3::new(DVec3::new(0.0, 0.0, 1.0), 10.0, DEFAULT_ZERO_EPSILON);
			let result = intersect_planes(&plane1, &plane2, DEFAULT_ZERO_EPSILON);
			let line: DLine3 = result.expect("Expected intersection result to be a line");

			assert!(vectors_are_equal(
				line.direction(),
				-DVec3::Y,
				DEFAULT_EQUAL_VECTOR_COMPONENT_EPSILON
			));

			let point_on_line: DVec3 = DVec3::new(5.0, 0.0, 10.0);

			assert!(
				point_lies_on_line(point_on_line, &line, DEFAULT_CONTACT_EPSILON),
				"Expected point {:?} to be on line {:?}",
				point_on_line,
				line
			)
		}

		{
			let plane1: DPlane3 =
				DPlane3::new(DVec3::new(1.0, 0.0, 0.0), 5.0, DEFAULT_ZERO_EPSILON);
			let plane2: DPlane3 =
				DPlane3::new(DVec3::new(0.0, 1.0, 0.0), 10.0, DEFAULT_ZERO_EPSILON);
			let result = intersect_planes(&plane1, &plane2, DEFAULT_ZERO_EPSILON);
			let line: DLine3 = result.expect("Expected intersection result to be a line");

			assert!(vectors_are_equal(
				line.direction(),
				DVec3::Z,
				DEFAULT_EQUAL_VECTOR_COMPONENT_EPSILON
			));

			let point_on_line: DVec3 = DVec3::new(5.0, 10.0, 20.333);

			assert!(
				point_lies_on_line(point_on_line, &line, DEFAULT_CONTACT_EPSILON),
				"Expected point {:?} to be on line {:?}",
				point_on_line,
				line
			)
		}
	}

	#[test]
	fn points_and_lines()
	{
		let line: DLine3 = DLine3::from_points(
			DVec3::new(1.0, 1.0, 1.0),
			DVec3::new(5.0, 1.0, 1.0),
			DEFAULT_ZERO_EPSILON,
		);

		let point: DVec3 = DVec3::new(3.0, 1.0, 2.0);
		let projected_point: DVec3 = project_point_onto_line(point, &line);

		assert!(
			vectors_are_equal(
				projected_point,
				DVec3::new(3.0, 1.0, 1.0),
				DEFAULT_EQUAL_VECTOR_COMPONENT_EPSILON
			),
			"Expected projected point {:?} to equal {:?}",
			projected_point,
			point
		);

		let point_dist_from_line: f64 = point_distance_from_line(point, &line);
		let expected_distance: f64 = 1.0;

		assert!(
			values_are_equal(
				point_dist_from_line,
				expected_distance,
				DEFAULT_ZERO_EPSILON
			),
			"Expected projected point distance {point_dist_from_line} to be {expected_distance}"
		);

		let point_on_line: DVec3 = DVec3::new(3.0, 1.0, 1.0);

		assert!(
			point_lies_on_line(point_on_line, &line, DEFAULT_CONTACT_EPSILON),
			"Expected point {:?} to be considered on line",
			point_on_line
		);

		let fuzzy_point_on_line: DVec3 =
			DVec3::new(3.0, 1.0, 1.0 + (DEFAULT_CONTACT_EPSILON / 2.0));

		assert!(
			point_lies_on_line(fuzzy_point_on_line, &line, DEFAULT_CONTACT_EPSILON),
			"Expected point {:?} to be considered on line",
			fuzzy_point_on_line
		);
	}

	#[test]
	fn lines_and_planes()
	{
		let plane: DPlane3 = DPlane3::new(DVec3::new(0.0, 0.0, 1.0), 5.0, DEFAULT_ZERO_EPSILON);

		let perpendicular_line: DLine3 = DLine3::from_points(
			DVec3::new(1.0, 1.0, -10.0),
			DVec3::new(1.0, 1.0, 10.0),
			DEFAULT_ZERO_EPSILON,
		);

		let intersection: LinePlaneIntersection = intersect_line_and_plane(
			&perpendicular_line,
			&plane,
			DEFAULT_ZERO_EPSILON,
			DEFAULT_CONTACT_EPSILON,
		);

		let intersection_point: DVec3 = intersection
			.intersection_point()
			.expect("Expected a single intersection");

		let expected_point: DVec3 = DVec3::new(1.0, 1.0, 5.0);

		assert!(
			vectors_are_equal(
				intersection_point,
				expected_point,
				DEFAULT_EQUAL_VECTOR_COMPONENT_EPSILON
			),
			"Expected intersection point {:?} for perpendicular line to equal {:?}",
			intersection_point,
			expected_point
		);

		let skew_line: DLine3 = DLine3::from_points(
			DVec3::new(5.5, 3.4, 6.0),
			DVec3::new(1.0, 1.0, 4.0),
			DEFAULT_ZERO_EPSILON,
		);

		let intersection: LinePlaneIntersection = intersect_line_and_plane(
			&skew_line,
			&plane,
			DEFAULT_ZERO_EPSILON,
			DEFAULT_CONTACT_EPSILON,
		);

		let intersection_point: DVec3 = intersection
			.intersection_point()
			.expect("Expected a single intersection");

		let expected_point: DVec3 = DVec3::new(3.25, 2.2, 5.0);

		assert!(
			vectors_are_equal(
				intersection_point,
				expected_point,
				DEFAULT_EQUAL_VECTOR_COMPONENT_EPSILON
			),
			"Expected intersection point {:?} for skew line to equal {:?}",
			intersection_point,
			expected_point
		);

		let parallel_line: DLine3 = DLine3::from_points(
			DVec3::new(1.0, 1.0, 10.0),
			DVec3::new(2.0, 2.0, 10.0),
			DEFAULT_ZERO_EPSILON,
		);

		let intersection: LinePlaneIntersection = intersect_line_and_plane(
			&parallel_line,
			&plane,
			DEFAULT_ZERO_EPSILON,
			DEFAULT_CONTACT_EPSILON,
		);

		assert!(intersection.is_no_intersection());
		assert!(!intersection.is_intersection());
		assert!(!intersection.is_planar_intersection());
		assert!(!intersection.is_single_intersection());

		let planar_line: DLine3 = DLine3::from_points(
			DVec3::new(1.0, 1.0, 5.0),
			DVec3::new(2.0, 2.0, 5.0),
			DEFAULT_ZERO_EPSILON,
		);

		let intersection: LinePlaneIntersection = intersect_line_and_plane(
			&planar_line,
			&plane,
			DEFAULT_ZERO_EPSILON,
			DEFAULT_CONTACT_EPSILON,
		);

		assert!(!intersection.is_no_intersection());
		assert!(intersection.is_intersection());
		assert!(intersection.is_planar_intersection());
		assert!(!intersection.is_single_intersection());
	}

	#[test]
	fn points_and_planes()
	{
		let plane: DPlane3 = DPlane3::new(DVec3::new(0.0, 0.0, 1.0), 5.0, DEFAULT_ZERO_EPSILON);
		let point: DVec3 = DVec3::new(10.0, 20.0, 0.0);
		let projected_point: DVec3 = project_point_onto_plane(point, &plane);
		let expected_point: DVec3 = DVec3::new(10.0, 20.0, 5.0);

		assert!(
			vectors_are_equal(
				projected_point,
				expected_point,
				DEFAULT_EQUAL_VECTOR_COMPONENT_EPSILON
			),
			"Expected projected point {:?} to equal {:?}",
			projected_point,
			expected_point
		);

		let point_distance: f64 = point_distance_from_plane(point, &plane);
		let expected_distance: f64 = 5.0;

		assert!(
			values_are_equal(point_distance, expected_distance, DEFAULT_ZERO_EPSILON),
			"Expected projected point distance {point_distance} to be {expected_distance}"
		);

		assert!(!point_lies_on_plane(point, &plane, DEFAULT_CONTACT_EPSILON));

		let point_on_plane: DVec3 = DVec3::new(10.0, 20.0, 5.0);

		assert!(point_lies_on_plane(
			point_on_plane,
			&plane,
			DEFAULT_CONTACT_EPSILON
		));

		assert!(point_lies_on_plane(
			point_on_plane + DVec3::new(0.0, 0.0, DEFAULT_CONTACT_EPSILON / 2.0),
			&plane,
			DEFAULT_CONTACT_EPSILON
		));
	}

	#[test]
	#[should_panic]
	fn intersect_one_null_plane_a()
	{
		intersect_planes(
			&DPlane3::NULL,
			&DPlane3::new(DVec3::new(1.0, 0.0, 0.0), 5.0, DEFAULT_ZERO_EPSILON),
			DEFAULT_ZERO_EPSILON,
		);
	}

	#[test]
	#[should_panic]
	fn intersect_one_null_plane_b()
	{
		intersect_planes(
			&DPlane3::new(DVec3::new(1.0, 0.0, 0.0), 5.0, DEFAULT_ZERO_EPSILON),
			&DPlane3::NULL,
			DEFAULT_ZERO_EPSILON,
		);
	}

	#[test]
	#[should_panic]
	fn intersect_two_null_planes()
	{
		intersect_planes(&DPlane3::NULL, &DPlane3::NULL, DEFAULT_ZERO_EPSILON);
	}

	#[test]
	#[should_panic]
	fn intersect_line_with_null_plane()
	{
		intersect_line_and_plane(
			&DLine3::from_points(
				DVec3::new(0.0, 0.0, 0.0),
				DVec3::new(1.0, 0.0, 0.0),
				DEFAULT_ZERO_EPSILON,
			),
			&DPlane3::NULL,
			DEFAULT_ZERO_EPSILON,
			DEFAULT_CONTACT_EPSILON,
		);
	}

	#[test]
	#[should_panic]
	fn intersect_null_line_with_plane()
	{
		intersect_line_and_plane(
			&DLine3::NULL,
			&DPlane3::new(DVec3::new(1.0, 0.0, 0.0), 5.0, DEFAULT_ZERO_EPSILON),
			DEFAULT_ZERO_EPSILON,
			DEFAULT_CONTACT_EPSILON,
		);
	}

	#[test]
	#[should_panic]
	fn intersect_null_line_with_null_plane()
	{
		intersect_line_and_plane(
			&DLine3::NULL,
			&DPlane3::NULL,
			DEFAULT_ZERO_EPSILON,
			DEFAULT_CONTACT_EPSILON,
		);
	}

	#[test]
	#[should_panic]
	fn project_point_onto_null_plane()
	{
		project_point_onto_plane(DVec3::new(1.0, 0.0, 0.0), &DPlane3::NULL);
	}

	#[test]
	#[should_panic]
	fn project_point_onto_null_line()
	{
		project_point_onto_line(DVec3::new(1.0, 0.0, 0.0), &DLine3::NULL);
	}
}
