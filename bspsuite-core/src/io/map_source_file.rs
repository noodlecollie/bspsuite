use crate::model::MapSourceFile;
use anyhow::{Context, Result};
use log::info;
use std::fs::File;
use std::io::Write;
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

pub fn write(path: &Path, map: &MapSourceFile) -> Result<()>
{
	info!("Dumping parsed map source to {}", path.display());

	let out_file: File = File::create(path)
		.with_context(|| format!("Failed to open file {} for writing", path.display()))?;

	return serialize(out_file, map);
}

mod version_1
{
	use std::collections::HashMap;
	use std::io::Write;

	use crate::model::{
		DPlane3, MapSourceBrush, MapSourceBrushFace, MapSourceEntity, MapSourceFile,
	};
	use anyhow::{Context, Result, anyhow};
	use glam::{DVec2, DVec3};
	use serde_json::ser::to_writer_pretty;
	use serde_json::{Map, Number, Value};

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
		let entities: JsonArray = to_json_array(&map.entities, |ent| process_entity(ent))?;

		let mut document: JsonObject = Map::new();
		document.insert("version".to_string(), VERSION.into());
		document.insert("entities".to_string(), entities.into());

		to_writer_pretty(writer, &Value::Object(document))?;
		return Ok(());
	}

	fn process_entity(entity: &MapSourceEntity) -> Result<JsonObject>
	{
		let brushes: JsonArray = to_json_array(&entity.brushes, |brush| process_brush(brush))?;
		let properties: JsonObject = to_json_object(&entity.keyvalues);

		let mut obj: JsonObject = JsonObject::new();
		obj.insert(
			"entity_index".to_string(),
			usize_to_number(entity.entity_index),
		);
		obj.insert("properties".to_string(), properties.into());
		obj.insert("brushes".to_string(), brushes.into());

		return Ok(obj);
	}

	fn process_brush(brush: &MapSourceBrush) -> Result<JsonObject>
	{
		let faces: JsonArray = to_json_array(&brush.faces, |face| process_face(face))?;

		let mut obj: JsonObject = JsonObject::new();
		obj.insert(
			"brush_index".to_string(),
			usize_to_number(brush.brush_index),
		);
		obj.insert("faces".to_string(), faces.into());

		return Ok(obj);
	}

	fn process_face(face: &MapSourceBrushFace) -> Result<JsonObject>
	{
		let mat_axis_u: JsonArray = process_vec3(&face.material_axes.0)?;
		let mat_axis_v: JsonArray = process_vec3(&face.material_axes.1)?;
		let mat_offset: JsonArray = process_vec2(&face.material_offset)?;
		let mat_scale: JsonArray = process_vec2(&face.material_scale)?;
		let plane: JsonArray = process_dplane3(face.plane)?;
		let material_axes = [Value::Array(mat_axis_u), Value::Array(mat_axis_v)];

		let mut obj: JsonObject = JsonObject::new();

		obj.insert("face_index".to_string(), usize_to_number(face.face_index));
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

	fn process_dplane3(plane: DPlane3) -> Result<JsonArray>
	{
		let values: [f64; 4] = [
			plane.normal.x,
			plane.normal.y,
			plane.normal.z,
			plane.distance,
		];

		return dvec_slice_to_array(values);
	}

	fn process_vec3(vec: &DVec3) -> Result<JsonArray>
	{
		return dvec_slice_to_array(vec.to_array());
	}

	fn process_vec2(vec: &DVec2) -> Result<JsonArray>
	{
		return dvec_slice_to_array(vec.to_array());
	}

	fn dvec_slice_to_array<const LEN: usize>(contents: [f64; LEN]) -> Result<JsonArray>
	{
		return to_json_array(&contents, |num| {
			Number::from_f64(*num)
				.ok_or_else(|| anyhow!("Encountered floating point value that was Inf or NaN"))
		});
	}

	fn to_json_array<InType, OutType, F>(list: &[InType], callback: F) -> Result<JsonArray>
	where
		F: Fn(&InType) -> Result<OutType>,
		OutType: Into<Value>,
	{
		let mut out: JsonArray = JsonArray::with_capacity(list.len());

		for (index, item) in list.iter().enumerate()
		{
			let new_item: OutType =
				callback(item).with_context(|| format!("Failed to serialise item {index}"))?;

			out.push(new_item.into());
		}

		return Ok(out);
	}

	fn to_json_object<T>(map: &HashMap<String, T>) -> JsonObject
	where
		T: ToString + Into<Value>,
	{
		return map
			.iter()
			.map(|(key, value)| (key.clone(), Value::String(value.to_string())))
			.collect();
	}

	fn usize_to_number(val: usize) -> Value
	{
		return Value::Number(
			Number::try_from(val)
				.expect("Unexpected failure to represent usize as a JSON number type"),
		);
	}
}
