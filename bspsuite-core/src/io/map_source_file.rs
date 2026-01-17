use crate::model::MapSourceFile;
use anyhow::{Context, Result};
use std::io::Write;

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

mod version_1
{
	use crate::model::{
		DPlane3, MapSourceBrush, MapSourceBrushFace, MapSourceEntity, MapSourceFile,
	};
	use anyhow::Result;
	use glam::{DVec2, DVec3};
	use serde_json::ser::to_writer;
	use serde_json::{Map, Number, Value};
	use std::io::Write;

	type JsonObject = Map<String, Value>;
	type JsonArray = Vec<Value>;

	pub(super) const VERSION: usize = 1;

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
		let entities: JsonArray = map
			.entities
			.iter()
			.map(|ent| Value::Object(process_entity(ent)))
			.collect();

		let mut document: JsonObject = Map::new();
		document.insert("version".to_string(), VERSION.into());
		document.insert("entities".to_string(), entities.into());

		to_writer(writer, &Value::Object(document))?;
		return Ok(());
	}

	fn process_entity(entity: &MapSourceEntity) -> JsonObject
	{
		let mut properties: JsonObject = JsonObject::new();

		for (key, value) in entity.keyvalues.iter()
		{
			properties.insert(key.clone(), value.clone().into());
		}

		let brushes: JsonArray = entity
			.brushes
			.iter()
			.map(|brush| Value::Object(process_brush(brush)))
			.collect();

		let mut obj: JsonObject = JsonObject::new();
		obj.insert("properties".to_string(), properties.into());
		obj.insert("brushes".to_string(), brushes.into());

		return obj;
	}

	fn process_brush(brush: &MapSourceBrush) -> JsonObject
	{
		let faces: JsonArray = brush
			.faces
			.iter()
			.map(|face| Value::Object(process_face(face)))
			.collect();

		let mut obj: JsonObject = JsonObject::new();
		obj.insert("faces".to_string(), faces.into());

		return obj;
	}

	fn process_face(face: &MapSourceBrushFace) -> JsonObject
	{
		let material_axes = [
			Value::Array(process_dvec3(&face.material_axes.0)),
			Value::Array(process_dvec3(&face.material_axes.1)),
		];

		let mut obj: JsonObject = JsonObject::new();

		obj.insert("plane".to_string(), process_dplane3(face.plane).into());
		obj.insert(
			"material_name".to_string(),
			face.material_name.clone().into(),
		);
		obj.insert("material_axes".to_string(), material_axes.to_vec().into());
		obj.insert(
			"material_offset".to_string(),
			process_dvec2(&face.material_offset).into(),
		);
		obj.insert(
			"material_scale".to_string(),
			process_dvec2(&face.material_scale).into(),
		);

		return obj;
	}

	fn process_dplane3(plane: DPlane3) -> JsonArray
	{
		let values: [f64; 4] = [
			plane.normal.x,
			plane.normal.y,
			plane.normal.z,
			plane.distance,
		];

		return dvec_slice_to_array(values);
	}

	fn process_dvec3(vec: &DVec3) -> JsonArray
	{
		return dvec_slice_to_array(vec.to_array());
	}

	fn process_dvec2(vec: &DVec2) -> JsonArray
	{
		return dvec_slice_to_array(vec.to_array());
	}

	// TODO: Handle errors
	fn dvec_slice_to_array<const LEN: usize>(contents: [f64; LEN]) -> JsonArray
	{
		return contents
			.into_iter()
			.map(|num| Value::Number(Number::from_f64(num).unwrap()))
			.collect();
	}
}
