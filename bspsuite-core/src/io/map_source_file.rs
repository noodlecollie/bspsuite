use crate::model::MapSourceFile;
use anyhow::{Context, Result};
use log::info;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub fn serialize<Writer>(writer: Writer, map: &MapSourceFile) -> Result<()>
where
	Writer: Write,
{
	use version_1 as current_version;

	return current_version::serialize(writer, map).with_context(|| {
		format!(
			"Failed to serialise version {} map source file",
			current_version::VERSION
		)
	});
}

pub fn deserialize<Reader>(reader: Reader) -> Result<MapSourceFile>
where
	Reader: Read,
{
	// If we support more than one version, we'll need to change this.
	use version_1 as current_version;

	return current_version::deserialize(reader).with_context(|| {
		format!(
			"Failed to deserialise version {} map source file",
			current_version::VERSION
		)
	});
}

pub fn write(path: &Path, map: &MapSourceFile) -> Result<()>
{
	info!("Dumping parsed map source to {}", path.display());

	let out_file: File = File::create(path)
		.with_context(|| format!("Failed to open file {} for writing", path.display()))?;

	return serialize(out_file, map);
}

pub fn read(path: &Path) -> Result<MapSourceFile>
{
	info!("Reading map source file from {}", path.display());

	let in_file: File = File::open(path)
		.with_context(|| format!("Failed to open file {} for reading", path.display()))?;

	return deserialize(in_file);
}

mod version_1
{
	use std::collections::HashMap;
	use std::io::{Read, Write};

	use anyhow::{Context, Result};
	use glam::{DVec2, DVec3};
	use serde_json::de::from_reader;
	use serde_json::ser::to_writer_pretty;
	use serde_json::{Map, Value};

	use crate::io::json_utils;
	use crate::io::json_utils::{JsonArray, JsonObject};
	use crate::math::DPlane3;
	use crate::model::{MapSourceBrush, MapSourceBrushFace, MapSourceEntity, MapSourceFile};

	pub(super) const VERSION: u64 = 1;

	// We do not blanket serialise the map source file struct
	// using a derive macro, because we want the format of
	// the serialised file to be independent of the structure
	// of the struct. This gives us freedom to change the
	// struct as we see fit, and not affect the serialised
	// file.
	pub(super) fn serialize<Writer>(writer: Writer, map: &MapSourceFile) -> Result<()>
	where
		Writer: Write,
	{
		let entities: JsonArray =
			json_utils::to_json_array(&map.entities, |ent| serialize_entity(ent))?;

		let mut document: JsonObject = Map::new();
		document.insert("version".to_string(), VERSION.into());
		document.insert("entities".to_string(), entities.into());

		to_writer_pretty(writer, &Value::Object(document))?;
		return Ok(());
	}

	pub(super) fn deserialize<Reader>(reader: Reader) -> Result<MapSourceFile>
	where
		Reader: Read,
	{
		let document: JsonObject = from_reader(reader)?;
		json_utils::check_version(&document, VERSION)?;

		let entities: Vec<MapSourceEntity> = deserialize_entities(&document)?;

		let mut map_file = MapSourceFile { entities };
		map_file.assign_global_indices();

		return Ok(map_file);
	}

	fn deserialize_entities(document: &JsonObject) -> Result<Vec<MapSourceEntity>>
	{
		let entities: &Vec<Value> = json_utils::get_array(document, "entities")?;

		return entities
			.iter()
			.enumerate()
			.map(|(index, value)| {
				deserialize_entity(value).with_context(|| format!("Entity {index}"))
			})
			.collect();
	}

	fn serialize_entity(entity: &MapSourceEntity) -> Result<JsonObject>
	{
		let brushes: JsonArray =
			json_utils::to_json_array(&entity.brushes, |brush| serialize_brush(brush))?;
		let properties: JsonObject = json_utils::to_json_object(&entity.keyvalues);

		let mut obj: JsonObject = JsonObject::new();
		obj.insert("properties".to_string(), properties.into());
		obj.insert("brushes".to_string(), brushes.into());

		return Ok(obj);
	}

	fn deserialize_entity(value: &Value) -> Result<MapSourceEntity>
	{
		let entity: &JsonObject = value
			.as_object()
			.with_context(|| "Entity was not an object")?;

		let brushes: Vec<MapSourceBrush> = json_utils::get_array(entity, "brushes")?
			.iter()
			.enumerate()
			.map(|(index, value)| -> Result<MapSourceBrush> {
				deserialize_brush(value).with_context(|| format!("Brush {index}"))
			})
			.collect::<Result<Vec<MapSourceBrush>>>()?;

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

		return Ok(MapSourceEntity {
			global_entity_index: 0, // Assigned later
			brushes,
			keyvalues,
		});
	}

	fn serialize_brush(brush: &MapSourceBrush) -> Result<JsonObject>
	{
		let faces: JsonArray =
			json_utils::to_json_array(&brush.faces, |face| serialize_face(face))?;

		let mut obj: JsonObject = JsonObject::new();
		obj.insert("faces".to_string(), faces.into());

		return Ok(obj);
	}

	fn deserialize_brush(value: &Value) -> Result<MapSourceBrush>
	{
		let brush: &JsonObject = value
			.as_object()
			.with_context(|| "Brush was not an object")?;

		let faces: Vec<MapSourceBrushFace> = json_utils::get_array(brush, "faces")?
			.iter()
			.enumerate()
			.map(|(index, value)| -> Result<MapSourceBrushFace> {
				deserialize_face(value).with_context(|| format!("Face {index}"))
			})
			.collect::<Result<Vec<MapSourceBrushFace>>>()?;

		return Ok(MapSourceBrush {
			global_brush_index: 0, // Assigned later
			faces: faces,
		});
	}

	fn serialize_face(face: &MapSourceBrushFace) -> Result<JsonObject>
	{
		let mat_axis_u: JsonArray = json_utils::serialize_vec3(&face.material_axes.0)?;
		let mat_axis_v: JsonArray = json_utils::serialize_vec3(&face.material_axes.1)?;
		let mat_offset: JsonArray = json_utils::serialize_vec2(&face.material_offset)?;
		let mat_scale: JsonArray = json_utils::serialize_vec2(&face.material_scale)?;
		let plane: JsonArray = json_utils::serialize_dplane3(face.plane)?;
		let material_axes = [Value::Array(mat_axis_u), Value::Array(mat_axis_v)];

		let mut obj: JsonObject = JsonObject::new();

		obj.insert("plane".to_string(), plane.into());
		obj.insert(
			"material_name".to_string(),
			face.material_name.clone().into(),
		);
		obj.insert("material_axes".to_string(), material_axes.to_vec().into());
		obj.insert("material_offset".to_string(), mat_offset.into());
		obj.insert("material_scale".to_string(), mat_scale.into());

		return Ok(obj);
	}

	fn deserialize_face(value: &Value) -> Result<MapSourceBrushFace>
	{
		let face: &JsonObject = value
			.as_object()
			.with_context(|| "Brush face was not an object")?;

		let material_axes: &JsonArray = json_utils::get_array_of_length(face, "material_axes", 2)?;

		let mat_axis_u: DVec3 = json_utils::json_array_to_dvec(
			material_axes[0]
				.as_array()
				.with_context(|| format!("material_axes[0]"))?,
		)?;

		let mat_axis_v: DVec3 = json_utils::json_array_to_dvec(
			material_axes[1]
				.as_array()
				.with_context(|| format!("material_axes[1]"))?,
		)?;

		let material_offset: DVec2 = json_utils::get_dvec(face, "material_offset")?;
		let material_scale: DVec2 = json_utils::get_dvec(face, "material_scale")?;
		let plane: DPlane3 = json_utils::get_dplane(face, "plane")?;
		let material_name: &str = json_utils::get_string(face, "material_name")?;

		return Ok(MapSourceBrushFace {
			global_face_index: 0, // Assigned later
			plane,
			material_name: material_name.to_owned(),
			material_axes: (mat_axis_u, mat_axis_v),
			material_offset,
			material_scale,
		});
	}
}
