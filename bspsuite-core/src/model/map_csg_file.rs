use std::collections::HashMap;

use crate::math::DPlane3;
use crate::model::MapSourceFile;
use anyhow::Result;
use glam::{DVec2, DVec3};

pub struct MapCsgBrushFaceVertex
{
	pub index_in_brush: usize,
	pub tex_coord: DVec2,
}

pub struct MapCsgBrushFace
{
	pub global_face_index: usize,
	pub plane: DPlane3,
	pub vertices: Vec<MapCsgBrushFaceVertex>,
	pub material: String, // TODO: Rc to an object?
}

pub struct MapCsgBrush
{
	pub global_brush_index: usize,
	pub vertices: Vec<DVec3>,
	pub faces: Vec<MapCsgBrushFace>,
}

pub struct MapCsgEntity
{
	pub global_entity_index: usize,
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
		// TODO: Once all brushes are generated, perform CSG union on all of them.
		// This may need some BBox tree, otherwise we'll be doing O(n^2) operations.
		// Check to see how existing compilers do it.
		todo!();
	}
}

impl MapCsgBrushFace
{
	pub fn new(global_index: usize, plane: DPlane3, material: String) -> Self
	{
		return Self {
			global_face_index: global_index,
			plane: plane,
			vertices: Vec::new(),
			material: material,
		};
	}
}

impl MapCsgBrushFaceVertex
{
	pub fn new(index_in_brush: usize) -> Self
	{
		return Self {
			index_in_brush: index_in_brush,
			tex_coord: DVec2::new(0.0, 0.0),
		};
	}
}
