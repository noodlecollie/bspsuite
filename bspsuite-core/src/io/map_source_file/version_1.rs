use std::collections::HashMap;

use crate::io::helpers::DeserializeVersionHelper;
use crate::math::DPlane3;
use crate::model::{MapSourceBrush, MapSourceBrushFace, MapSourceEntity, MapSourceFile};
use glam::{DVec2, DVec3};
use serde::{Deserialize, Serialize};

pub(super) const VERSION: u64 = 1;

#[derive(Serialize, Deserialize, Debug)]
pub struct V1File
{
	#[serde(deserialize_with = "DeserializeVersionHelper::<VERSION>::deserialize_version")]
	version: u64,

	pub entities: Vec<V1Entity>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct V1Entity
{
	pub brushes: Vec<V1Brush>,
	pub keyvalues: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct V1Brush
{
	pub faces: Vec<V1Face>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct V1Face
{
	pub plane: DPlane3,
	pub material_name: String,
	pub material_axes: (DVec3, DVec3),
	pub material_offset: DVec2,
	pub material_scale: DVec2,
}

////////////////////////////////////////////////////////
// MapSourceFile -> V1File
////////////////////////////////////////////////////////

impl From<&MapSourceFile> for V1File
{
	fn from(value: &MapSourceFile) -> Self
	{
		return Self {
			version: VERSION,
			entities: value.entities.iter().map(|ent| ent.into()).collect(),
		};
	}
}

impl From<&MapSourceEntity> for V1Entity
{
	fn from(value: &MapSourceEntity) -> Self
	{
		return Self {
			brushes: value.brushes.iter().map(|brush| brush.into()).collect(),
			keyvalues: value.keyvalues.clone(),
		};
	}
}

impl From<&MapSourceBrush> for V1Brush
{
	fn from(value: &MapSourceBrush) -> Self
	{
		return Self {
			faces: value.faces.iter().map(|face| face.into()).collect(),
		};
	}
}

impl From<&MapSourceBrushFace> for V1Face
{
	fn from(value: &MapSourceBrushFace) -> Self
	{
		return Self {
			plane: value.plane.clone(),
			material_name: value.material_name.clone(),
			material_axes: value.material_axes,
			material_offset: value.material_offset,
			material_scale: value.material_scale,
		};
	}
}

////////////////////////////////////////////////////////
// V1File -> MapSourceFile
////////////////////////////////////////////////////////

impl From<V1File> for MapSourceFile
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

impl From<V1Entity> for MapSourceEntity
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

impl From<V1Brush> for MapSourceBrush
{
	fn from(value: V1Brush) -> Self
	{
		return Self {
			global_brush_index: 0, // Assigned later
			faces: value.faces.into_iter().map(|face| face.into()).collect(),
		};
	}
}

impl From<V1Face> for MapSourceBrushFace
{
	fn from(value: V1Face) -> Self
	{
		return Self {
			global_face_index: 0, // Assigned later
			plane: value.plane,
			material_name: value.material_name,
			material_axes: value.material_axes,
			material_offset: value.material_offset,
			material_scale: value.material_scale,
		};
	}
}
