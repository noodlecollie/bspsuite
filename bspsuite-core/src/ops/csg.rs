use crate::model::{MapCsgBrush, MapCsgBrushFace, MapSourceBrush, MapSourceBrushFace};
use anyhow::{Result, bail};
use maths_rs::Vec3d;

pub fn construct_brush(source: MapSourceBrush) -> Result<MapCsgBrush>
{
	let faces: Vec<MapCsgBrushFace> = create_faces_from_planes(&source.faces);
	todo!();
}

// As per the Stefan Hajnoczi paper (see the notes directory in this repo),
// we will need to perform the following:
// - Compute all intersection points between all triplets of planes.
// - Discard extraneous points which are in front of any plane.
// - Order the points clockwise on each face.
// At this stage, we will also need to verify whether the convex volume
// is closed (ie. not missing any faces). This can be done using Euler's
// formula:
//   V - E + F = 2
// where V = vertex count, E = edge count and F = face count.
// If this formula is not satisfied, the volume is not valid.
fn create_faces_from_planes(brush_faces: &Vec<MapSourceBrushFace>) -> Vec<MapCsgBrushFace>
{
	todo!();
}

fn compute_all_planar_intersection_points(
	brush_faces: &Vec<MapSourceBrushFace>,
) -> Result<Vec<MapCsgBrushFace>>
{
	if brush_faces.len() < 4
	{
		bail!("Not enough planes to form a valid solid")
	}

	for plane1_index in 0..brush_faces.len() - 2
	{
		for plane2_index in 0..brush_faces.len() - 1
		{
			for plane3_index in 0..brush_faces.len()
			{
				if plane1_index == plane2_index && plane2_index == plane3_index
				{
					continue;
				}

				let planes = (
					&brush_faces[plane1_index].plane,
					&brush_faces[plane2_index].plane,
					&brush_faces[plane3_index].plane,
				);

				todo!();
			}
		}
	}

	todo!();
}

fn discard_intersection_points_outside_minimum_convex_hull(
	csg_faces: Vec<MapCsgBrushFace>,
) -> Vec<MapCsgBrushFace>
{
	todo!();
}

fn order_vertices_clockwise_on_all_faces(csg_faces: Vec<MapCsgBrushFace>) -> Vec<MapCsgBrushFace>
{
	return csg_faces
		.into_iter()
		.map(|face| order_vertices_clockwise(face))
		.collect();
}

fn order_vertices_clockwise(face: MapCsgBrushFace) -> MapCsgBrushFace
{
	todo!();
}

fn is_point_in_front_of_any_face(point: Vec3d, brush_faces: &Vec<MapSourceBrushFace>) -> bool
{
	todo!();
}
