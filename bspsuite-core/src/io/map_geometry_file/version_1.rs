use std::collections::HashMap;

use crate::io::helpers::DeserializeVersionHelper;
use crate::model::{MapGeomBrush, MapGeomBrushFaceVertex, MapGeomEntity, MapGeomFile};
use crate::{math::DPlane3, model::MapGeomBrushFace};
use glam::{DVec2, DVec3};
use serde::{Deserialize, Serialize};

pub(super) const VERSION: u64 = 1;

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct V1File
{
	#[serde(deserialize_with = "DeserializeVersionHelper::<VERSION>::deserialize_version")]
	version: u64,

	pub entities: Vec<V1Entity>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct V1Entity
{
	pub brushes: Vec<V1Brush>,
	pub keyvalues: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct V1Brush
{
	pub vertices: Vec<DVec3>,
	pub faces: Vec<V1BrushFace>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct V1BrushFace
{
	pub plane: DPlane3,
	pub vertices: Vec<V1BrushFaceVertex>,
	pub material: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct V1BrushFaceVertex
{
	pub tex_coord: DVec2,
}

////////////////////////////////////////////////////////
// MapGeomFile -> V1File
////////////////////////////////////////////////////////

impl From<&MapGeomFile> for V1File
{
	fn from(value: &MapGeomFile) -> Self
	{
		return Self {
			version: VERSION,
			entities: value.entities.iter().map(|ent| ent.into()).collect(),
		};
	}
}

impl From<&MapGeomEntity> for V1Entity
{
	fn from(value: &MapGeomEntity) -> Self
	{
		return Self {
			brushes: value.brushes.iter().map(|brush| brush.into()).collect(),
			keyvalues: value.keyvalues.clone(),
		};
	}
}

impl From<&MapGeomBrush> for V1Brush
{
	fn from(value: &MapGeomBrush) -> Self
	{
		return Self {
			vertices: value.vertices.clone(),
			faces: value.faces.iter().map(|face| face.into()).collect(),
		};
	}
}

impl From<&MapGeomBrushFace> for V1BrushFace
{
	fn from(value: &MapGeomBrushFace) -> Self
	{
		return Self {
			plane: value.plane.clone(),
			vertices: value.vertices.iter().map(|vert| vert.into()).collect(),
			material: value.material.clone(),
		};
	}
}

impl From<&MapGeomBrushFaceVertex> for V1BrushFaceVertex
{
	fn from(value: &MapGeomBrushFaceVertex) -> Self
	{
		return Self {
			tex_coord: value.tex_coord,
		};
	}
}

////////////////////////////////////////////////////////
// V1File -> MapGeomFile
////////////////////////////////////////////////////////

impl From<V1File> for MapGeomFile
{
	fn from(value: V1File) -> Self
	{
		let mut file: Self = Self {
			entities: value.entities.into_iter().map(|ent| ent.into()).collect(),
		};

		file.assign_global_indices();
		return file;
	}
}

impl From<V1Entity> for MapGeomEntity
{
	fn from(value: V1Entity) -> Self
	{
		return Self {
			global_entity_index: 0, // Assigned later
			brushes: value
				.brushes
				.into_iter()
				.map(|brush| brush.into())
				.collect(),
			keyvalues: value.keyvalues,
		};
	}
}

impl From<V1Brush> for MapGeomBrush
{
	fn from(value: V1Brush) -> Self
	{
		return Self {
			global_brush_index: 0, // Assigned later
			vertices: value.vertices.into_iter().map(|vert| vert.into()).collect(),
			faces: value.faces.into_iter().map(|face| face.into()).collect(),
		};
	}
}

impl From<V1BrushFace> for MapGeomBrushFace
{
	fn from(value: V1BrushFace) -> Self
	{
		return Self {
			global_face_index: 0, // Assigned later
			plane: value.plane,
			vertices: value.vertices.into_iter().map(|vert| vert.into()).collect(),
			material: value.material.clone(),
		};
	}
}

impl From<V1BrushFaceVertex> for MapGeomBrushFaceVertex
{
	fn from(value: V1BrushFaceVertex) -> Self
	{
		return Self {
			index_in_brush: 0, // Assigned later
			tex_coord: value.tex_coord,
		};
	}
}
