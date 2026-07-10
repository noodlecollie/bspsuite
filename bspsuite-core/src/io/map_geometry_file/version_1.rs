use std::collections::HashMap;

use crate::io::helpers::{IOFmtSignature, IOFormat};
use crate::model::{MapGeomBrush, MapGeomBrushFaceVertex, MapGeomEntity, MapGeomFile};
use crate::{math::DPlane3, model::MapGeomBrushFace};
use glam::{DVec2, DVec3};
use serde::{Deserialize, Serialize};

pub(super) const VERSION: u64 = 1;

#[derive(Serialize, Deserialize, Debug)]
pub struct V1File
{
	signature: IOFmtSignature<V1File>,
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
	pub vertices: Vec<DVec3>,
	pub faces: Vec<V1BrushFace>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct V1BrushFace
{
	pub plane: DPlane3,
	pub vertices: Vec<V1BrushFaceVertex>,
	pub material: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct V1BrushFaceVertex
{
	pub index_in_brush: usize,
	pub tex_coord: DVec2,
}

impl IOFormat for V1File
{
	fn format_name() -> &'static str
	{
		return "mapgeometry";
	}

	fn format_version() -> u64
	{
		return VERSION;
	}
}

////////////////////////////////////////////////////////
// MapGeomFile -> V1File
////////////////////////////////////////////////////////

impl From<&MapGeomFile> for V1File
{
	fn from(value: &MapGeomFile) -> Self
	{
		return Self {
			signature: IOFmtSignature::new(),
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
			index_in_brush: value.index_in_brush,
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
			index_in_brush: value.index_in_brush,
			tex_coord: value.tex_coord,
		};
	}
}

#[cfg(test)]
mod tests
{
	use super::*;
	use crate::math::DEFAULT_ZERO_EPSILON;
	use serde_json;

	#[test]
	fn serialize_and_deserialize_simple_object()
	{
		let mut kv: HashMap<String, String> = HashMap::new();
		kv.insert("classname".to_owned(), "worldspawn".to_owned());
		kv.insert("description".to_owned(), "This is a test entity".to_owned());

		let map_geom_file: MapGeomFile = MapGeomFile {
			entities: vec![MapGeomEntity {
				global_entity_index: 0,
				keyvalues: kv,
				brushes: vec![MapGeomBrush {
					global_brush_index: 0,
					vertices: vec![
						DVec3::new(0.0, 2.0, 3.0),
						DVec3::new(0.0, 1.0, 3.0),
						DVec3::new(1.0, 1.0, 3.0),
						DVec3::new(1.0, 2.0, 3.0),
					],
					faces: vec![MapGeomBrushFace {
						global_face_index: 0,
						plane: DPlane3::new(DVec3::Z, 3.0, DEFAULT_ZERO_EPSILON),
						vertices: vec![
							MapGeomBrushFaceVertex {
								index_in_brush: 0,
								tex_coord: DVec2::new(0.0, 1.0),
							},
							MapGeomBrushFaceVertex {
								index_in_brush: 1,
								tex_coord: DVec2::new(0.0, 0.0),
							},
							MapGeomBrushFaceVertex {
								index_in_brush: 2,
								tex_coord: DVec2::new(1.0, 0.0),
							},
							MapGeomBrushFaceVertex {
								index_in_brush: 3,
								tex_coord: DVec2::new(1.0, 1.0),
							},
						],
						material: "some_material".to_owned(),
					}],
				}],
			}],
		};

		let v1_file_out: V1File = V1File::from(&map_geom_file);
		let json_string: String = serde_json::to_string(&v1_file_out).unwrap();
		let v1_file_in: V1File = serde_json::from_str::<V1File>(&json_string).unwrap();
		let recovered_geom_file: MapGeomFile = v1_file_in.into();

		assert_eq!(map_geom_file, recovered_geom_file);
	}
}
