use std::collections::HashMap;

use crate::math::{CompileTuningParameters, DPlane3};
use crate::model::MapSourceFile;
use crate::ops::csg::construct_brush;
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
	pub fn construct(source: MapSourceFile, params: &CompileTuningParameters)
	-> Result<MapCsgFile>
	{
		let mut csg_entities: Vec<MapCsgEntity> = Vec::with_capacity(source.entities.len());

		for source_ent in source.entities.into_iter()
		{
			let mut ent: MapCsgEntity = MapCsgEntity {
				global_entity_index: source_ent.global_entity_index,
				brushes: Vec::with_capacity(source_ent.brushes.len()),
				keyvalues: source_ent.keyvalues,
			};

			for source_brush in source_ent.brushes.into_iter()
			{
				let brush: MapCsgBrush = construct_brush(source_brush, params)?;
				ent.brushes.push(brush);
			}

			csg_entities.push(ent);
		}

		return Ok(MapCsgFile {
			entities: csg_entities,
		});
	}
}
