use std::collections::HashMap;
use std::io::{Read, Write};

use anyhow::{Context, Result, bail};
use glam::{DVec2, DVec3};
use serde_json::Value;
use serde_json::de::from_reader;
use serde_json::ser::to_writer_pretty;

use crate::io::json_utils;
use crate::io::json_utils::{JsonArray, JsonObject};
use crate::math::DPlane3;
use crate::model::{
	MapGeomBrush, MapGeomBrushFace, MapGeomBrushFaceVertex, MapGeomEntity, MapGeomFile,
};

pub(super) const VERSION: u64 = 1;

pub(super) fn serialize<Writer>(writer: Writer, map: &MapGeomFile) -> Result<()>
where
	Writer: Write,
{
	let entities: JsonArray = json_utils::to_json_array(&map.entities, |ent| serialize_entity(ent))
		.with_context(|| "Entities")?;

	let mut document: JsonObject = JsonObject::new();

	document.insert("version".to_string(), VERSION.into());
	document.insert("entities".to_string(), entities.into());

	to_writer_pretty(writer, &Value::Object(document))?;
	return Ok(());
}

pub(super) fn deserialize<Reader>(reader: Reader) -> Result<MapGeomFile>
where
	Reader: Read,
{
	let document: JsonObject = from_reader(reader)?;
	json_utils::check_version(&document, VERSION)?;

	let entities: Vec<MapGeomEntity> = deserialize_entities(&document)?;

	let mut map_file = MapGeomFile { entities };
	map_file.assign_global_indices();

	return Ok(map_file);
}

fn deserialize_entities(document: &JsonObject) -> Result<Vec<MapGeomEntity>>
{
	let entities: &Vec<Value> = json_utils::get_array(document, "entities")?;

	return entities
		.iter()
		.enumerate()
		.map(|(index, value)| deserialize_entity(value).with_context(|| format!("Entity {index}")))
		.collect();
}

fn serialize_entity(entity: &MapGeomEntity) -> Result<JsonObject>
{
	let brushes: JsonArray =
		json_utils::to_json_array(&entity.brushes, |brush| serialize_brush(brush))
			.with_context(|| "Brushes")?;

	let keyvalues: JsonObject = json_utils::to_json_object(&entity.keyvalues);

	let mut obj: JsonObject = JsonObject::new();
	obj.insert("brushes".to_string(), brushes.into());
	obj.insert("keyvalues".to_string(), keyvalues.into());
	return Ok(obj);
}

fn deserialize_entity(value: &Value) -> Result<MapGeomEntity>
{
	let entity: &JsonObject = value
		.as_object()
		.with_context(|| "Entity was not an object")?;

	let brushes: Vec<MapGeomBrush> = json_utils::get_array(entity, "brushes")?
		.iter()
		.enumerate()
		.map(|(index, value)| -> Result<MapGeomBrush> {
			deserialize_brush(value).with_context(|| format!("Brush {index}"))
		})
		.collect::<Result<Vec<MapGeomBrush>>>()?;

	let keyvalues: HashMap<String, String> = json_utils::get_object(entity, "properties")?
		.iter()
		.map(|(k, v)| -> Result<(String, String)> {
			let value_str: String = v
				.as_str()
				.with_context(|| format!("Entity property '{k}': value was not a string"))?
				.to_owned();

			Ok((k.clone(), value_str))
		})
		.collect::<Result<HashMap<String, String>>>()?;

	return Ok(MapGeomEntity {
		global_entity_index: 0, // Assigned later
		brushes,
		keyvalues,
	});
}

fn serialize_brush(brush: &MapGeomBrush) -> Result<JsonObject>
{
	let vertices: JsonArray =
		json_utils::to_json_array(&brush.vertices, |vert| json_utils::serialize_vec3(vert))
			.with_context(|| "Vertices")?;

	let faces: JsonArray = json_utils::to_json_array(&brush.faces, |face| serialize_face(face))
		.with_context(|| "Faces")?;

	let mut obj: JsonObject = JsonObject::new();
	obj.insert("vertices".to_string(), vertices.into());
	obj.insert("faces".to_string(), faces.into());
	return Ok(obj);
}

fn deserialize_brush(value: &Value) -> Result<MapGeomBrush>
{
	let brush: &JsonObject = value
		.as_object()
		.with_context(|| "Brush was not an object")?;

	let vertices: Vec<DVec3> = json_utils::get_array(brush, "vertices")?
		.iter()
		.enumerate()
		.map(|(index, value)| -> Result<DVec3> {
			json_utils::json_array_to_dvec(
				value
					.as_array()
					.with_context(|| format!("Brush vertex {index}"))?,
			)
			.with_context(|| format!("Brush vertex {index}"))
		})
		.collect::<Result<Vec<DVec3>>>()?;

	let num_vertices: usize = vertices.len();

	let faces: Vec<MapGeomBrushFace> = json_utils::get_array(brush, "faces")?
		.iter()
		.enumerate()
		.map(|(index, value)| -> Result<MapGeomBrushFace> {
			deserialize_face(value, num_vertices).with_context(|| format!("Face {index}"))
		})
		.collect::<Result<Vec<MapGeomBrushFace>>>()?;

	return Ok(MapGeomBrush {
		global_brush_index: 0, // Assigned later
		vertices,
		faces,
	});
}

fn serialize_face(face: &MapGeomBrushFace) -> Result<JsonObject>
{
	let plane: JsonArray = json_utils::serialize_dplane3(face.plane).with_context(|| "Plane")?;

	let face_vertices: JsonArray =
		json_utils::to_json_array(&face.vertices, |vertex| serialize_vertex(vertex))
			.with_context(|| "Vertex")?;

	let mut obj: JsonObject = JsonObject::new();
	obj.insert("plane".to_string(), plane.into());
	obj.insert("material".to_string(), face.material.clone().into());
	obj.insert("face_vertices".to_string(), face_vertices.into());
	return Ok(obj);
}

fn deserialize_face(value: &Value, num_vertices: usize) -> Result<MapGeomBrushFace>
{
	let face: &JsonObject = value
		.as_object()
		.with_context(|| "Face was not an object")?;

	let plane: DPlane3 = json_utils::get_dplane(face, "plane")?;
	let material: String = json_utils::get_string(face, "material")?.to_string();

	let vertices: Vec<MapGeomBrushFaceVertex> = json_utils::get_array(face, "face_vertices")?
		.iter()
		.enumerate()
		.map(|(index, value)| -> Result<MapGeomBrushFaceVertex> {
			deserialize_vertex(value, num_vertices).with_context(|| format!("Face vertex {index}"))
		})
		.collect::<Result<Vec<MapGeomBrushFaceVertex>>>()?;

	return Ok(MapGeomBrushFace {
		global_face_index: 0, // Assigned later
		plane,
		vertices,
		material,
	});
}

fn serialize_vertex(vertex: &MapGeomBrushFaceVertex) -> Result<JsonObject>
{
	let tex_coord: JsonArray =
		json_utils::serialize_vec2(&vertex.tex_coord).with_context(|| "Texture co-ords")?;

	let mut obj: JsonObject = JsonObject::new();
	obj.insert(
		"index_in_brush".to_string(),
		json_utils::usize_to_number(vertex.index_in_brush),
	);
	obj.insert("tex_coord".to_string(), tex_coord.into());
	return Ok(obj);
}

fn deserialize_vertex(value: &Value, num_vertices: usize) -> Result<MapGeomBrushFaceVertex>
{
	let vertex: &JsonObject = value
		.as_object()
		.with_context(|| "Vertex was not an object")?;

	let tex_coord: DVec2 = json_utils::get_dvec(vertex, "tex_coord")?;
	let index_in_brush: usize = json_utils::get_usize(vertex, "index_in_brush")?;

	if index_in_brush > num_vertices
	{
		bail!(
			"Out of range index {} in brush (brush contains {} vertices)",
			index_in_brush,
			num_vertices
		);
	}

	return Ok(MapGeomBrushFaceVertex {
		index_in_brush,
		tex_coord,
	});
}
