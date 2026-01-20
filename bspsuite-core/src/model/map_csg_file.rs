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
