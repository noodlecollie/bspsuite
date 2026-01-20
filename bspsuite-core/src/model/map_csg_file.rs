use std::collections::HashMap;

use crate::model::{DPlane3, MapSourceBrush, MapSourceBrushFace, MapSourceFile};
use anyhow::Result;
use maths_rs::{Vec2d, Vec3d};

pub struct MapCsgVertex
{
	pub pos: Vec3d,
	pub tex_coord: Vec2d,
}

pub struct MapCsgBrushFace
{
	pub face_index: usize,
	pub plane: DPlane3,
	pub vertices: Vec<MapCsgVertex>,
}

pub struct MapCsgBrush
{
	pub brush_index: usize,
}

pub struct MapCsgEntity
{
	pub entity_index: usize,
	pub brushes: Vec<MapCsgBrush>,
	pub keyvalues: HashMap<String, String>,
}

pub struct MapCsgFile
{
	pub entities: Vec<MapCsgEntity>,
}

impl MapCsgFile
{
	pub fn construct(source: MapSourceFile) -> Result<MapCsgFile>
	{
		todo!();
	}
}

fn construct_brush(source: MapSourceBrush) -> Result<MapCsgBrush>
{
	let faces: Vec<MapCsgBrushFace> = compute_faces_from_planes(&source.faces);
	todo!();
}

fn compute_faces_from_planes(brush_faces: &Vec<MapSourceBrushFace>) -> Vec<MapCsgBrushFace>
{
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
	todo!();
}

fn compute_all_planar_intersection_points(
	brush_faces: &Vec<MapSourceBrushFace>,
) -> Vec<MapCsgBrushFace>
{
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
