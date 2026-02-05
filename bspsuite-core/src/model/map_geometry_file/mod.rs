mod brush_builder;
mod edge_collection;
mod point_collection;

use std::collections::HashMap;

use crate::math::{CompileTuningParameters, DPlane3};
use crate::model::MapSourceFile;
use anyhow::Result;
use brush_builder::BrushBuilder;
use glam::{DVec2, DVec3};

pub struct MapGeomBrushFaceVertex
{
	pub index_in_brush: usize,
	pub tex_coord: DVec2,
}

pub struct MapGeomBrushFace
{
	pub global_face_index: usize,
	pub plane: DPlane3,
	pub vertices: Vec<MapGeomBrushFaceVertex>,
	pub material: String, // TODO: Rc to an object?
}

pub struct MapGeomBrush
{
	pub global_brush_index: usize,
	pub vertices: Vec<DVec3>,
	pub faces: Vec<MapGeomBrushFace>,
}

pub struct MapGeomEntity
{
	pub global_entity_index: usize,
	pub brushes: Vec<MapGeomBrush>,
	pub keyvalues: HashMap<String, String>,
}

pub struct MapGeomFile
{
	pub entities: Vec<MapGeomEntity>,
}

impl MapGeomFile
{
	pub fn construct(source: MapSourceFile, params: &CompileTuningParameters)
	-> Result<MapGeomFile>
	{
		let mut geom_entities: Vec<MapGeomEntity> = Vec::with_capacity(source.entities.len());

		for source_ent in source.entities.into_iter()
		{
			let mut ent: MapGeomEntity = MapGeomEntity {
				global_entity_index: source_ent.global_entity_index,
				brushes: Vec::with_capacity(source_ent.brushes.len()),
				keyvalues: source_ent.keyvalues,
			};

			for source_brush in source_ent.brushes.into_iter()
			{
				let brush: MapGeomBrush = BrushBuilder::build(&source_brush, params)?;
				ent.brushes.push(brush);
			}

			geom_entities.push(ent);
		}

		return Ok(MapGeomFile {
			entities: geom_entities,
		});
	}
}
