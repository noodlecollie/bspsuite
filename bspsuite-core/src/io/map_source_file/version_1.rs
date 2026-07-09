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

#[cfg(test)]
mod tests
{
	use super::*;
	use crate::math::{DEFAULT_ZERO_EPSILON, DPlane3};
	use serde_json;

	#[test]
	fn serialize_and_deserialize_simple_object()
	{
		let mut kv: HashMap<String, String> = HashMap::new();
		kv.insert("classname".to_owned(), "worldspawn".to_owned());
		kv.insert("description".to_owned(), "This is a test entity".to_owned());

		let map_source_file: MapSourceFile = MapSourceFile {
			entities: vec![MapSourceEntity {
				global_entity_index: 0,
				keyvalues: kv,
				brushes: vec![MapSourceBrush {
					global_brush_index: 0,
					faces: vec![MapSourceBrushFace {
						global_face_index: 0,
						plane: DPlane3::new(DVec3::Z, 5.0, DEFAULT_ZERO_EPSILON),
						material_name: "face_material".to_owned(),
						material_axes: (DVec3::X, DVec3::Y),
						material_offset: DVec2::new(10.0, 20.0),
						material_scale: DVec2::new(0.25, 0.75),
					}],
				}],
			}],
		};

		let v1_file_out: V1File = V1File::from(&map_source_file);
		let json_string: String = serde_json::to_string(&v1_file_out).unwrap();
		let v1_file_in: V1File = serde_json::from_str::<V1File>(&json_string).unwrap();
		let recovered_source_file: MapSourceFile = v1_file_in.into();

		assert!(map_source_file == recovered_source_file);
	}
}
