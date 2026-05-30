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
	use std::io::{Read, Write};

	use crate::math::DPlane3;
	use crate::model::{MapSourceBrush, MapSourceBrushFace, MapSourceEntity, MapSourceFile};
	use anyhow::{Context, Result, bail};
	use glam::{DVec2, DVec3, DVec4, Vec4Swizzles};
	use serde_json::de::from_reader;
	use serde_json::ser::to_writer_pretty;
	use serde_json::{Map, Number, Value};

	type JsonObject = Map<String, Value>;
	type JsonArray = Vec<Value>;

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
		let entities: JsonArray = to_json_array(&map.entities, |ent| serialize_entity(ent))?;

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
		check_version(&document)?;

		let entities: Vec<MapSourceEntity> = deserialize_entities(&document)?;

		let mut map_file = MapSourceFile { entities };
		map_file.assign_global_indices();

		return Ok(map_file);
	}

	fn deserialize_entities(document: &JsonObject) -> Result<Vec<MapSourceEntity>>
	{
		let entities: &Vec<Value> = get_array(document, "entities")?;

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
		let brushes: JsonArray = to_json_array(&entity.brushes, |brush| serialize_brush(brush))?;
		let properties: JsonObject = to_json_object(&entity.keyvalues);

		let mut obj: JsonObject = JsonObject::new();
		obj.insert(
			"global_entity_index".to_string(),
			usize_to_number(entity.global_entity_index),
		);
		obj.insert("properties".to_string(), properties.into());
		obj.insert("brushes".to_string(), brushes.into());

		return Ok(obj);
	}

	fn deserialize_entity(value: &Value) -> Result<MapSourceEntity>
	{
		let entity: &JsonObject = value
			.as_object()
			.with_context(|| "Entity was not an object")?;

		let brushes: Vec<MapSourceBrush> = get_array(entity, "brushes")?
			.iter()
			.enumerate()
			.map(|(index, value)| -> Result<MapSourceBrush> {
				deserialize_brush(value).with_context(|| format!("Brush {index}"))
			})
			.collect::<Result<Vec<MapSourceBrush>>>()?;

		let keyvalues: HashMap<String, String> = get_object(entity, "properties")?
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
		let faces: JsonArray = to_json_array(&brush.faces, |face| serialize_face(face))?;

		let mut obj: JsonObject = JsonObject::new();
		obj.insert(
			"global_brush_index".to_string(),
			usize_to_number(brush.global_brush_index),
		);
		obj.insert("faces".to_string(), faces.into());

		return Ok(obj);
	}

	fn deserialize_brush(value: &Value) -> Result<MapSourceBrush>
	{
		let brush: &JsonObject = value
			.as_object()
			.with_context(|| "Brush was not an object")?;

		let faces: Vec<MapSourceBrushFace> = get_array(brush, "faces")?
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
		let mat_axis_u: JsonArray = serialize_vec3(&face.material_axes.0)?;
		let mat_axis_v: JsonArray = serialize_vec3(&face.material_axes.1)?;
		let mat_offset: JsonArray = serialize_vec2(&face.material_offset)?;
		let mat_scale: JsonArray = serialize_vec2(&face.material_scale)?;
		let plane: JsonArray = serialize_dplane3(face.plane)?;
		let material_axes = [Value::Array(mat_axis_u), Value::Array(mat_axis_v)];

		let mut obj: JsonObject = JsonObject::new();

		obj.insert(
			"global_face_index".to_string(),
			Value::Number(face.global_face_index.into()),
		);
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

		let material_axes: &JsonArray = get_array_of_length(face, "material_axes", 2)?;

		let mat_axis_u: DVec3 = json_array_to_dvec(
			material_axes[0]
				.as_array()
				.with_context(|| format!("material_axes[0]"))?,
		)?;

		let mat_axis_v: DVec3 = json_array_to_dvec(
			material_axes[1]
				.as_array()
				.with_context(|| format!("material_axes[1]"))?,
		)?;

		let material_offset: DVec2 = get_dvec(face, "material_offset")?;
		let material_scale: DVec2 = get_dvec(face, "material_scale")?;
		let plane: DPlane3 = get_dplane(face, "plane")?;
		let material_name: &str = get_string(face, "material_name")?;

		return Ok(MapSourceBrushFace {
			global_face_index: 0, // Assigned later
			plane,
			material_name: material_name.to_owned(),
			material_axes: (mat_axis_u, mat_axis_v),
			material_offset,
			material_scale,
		});
	}

	fn serialize_dplane3(plane: DPlane3) -> Result<JsonArray>
	{
		let normal: DVec3 = plane.normal();
		let values: [f64; 4] = [normal.x, normal.y, normal.z, plane.distance()];
		return dvec_slice_to_json_array(values);
	}

	fn serialize_vec3(vec: &DVec3) -> Result<JsonArray>
	{
		return dvec_slice_to_json_array(vec.to_array());
	}

	fn serialize_vec2(vec: &DVec2) -> Result<JsonArray>
	{
		return dvec_slice_to_json_array(vec.to_array());
	}

	fn dvec_slice_to_json_array<const LEN: usize>(contents: [f64; LEN]) -> Result<JsonArray>
	{
		return to_json_array(&contents, |num| {
			Number::from_f64(*num)
				.with_context(|| "Encountered floating point value that was Inf or NaN")
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

	fn check_version(root: &JsonObject) -> Result<()>
	{
		let version: &Value = root
			.get("version")
			.with_context(|| "No 'version' found in document")?;

		let version: u64 = version
			.as_u64()
			.with_context(|| "Could not parse 'version' as u64")?;

		if version != VERSION
		{
			bail!("Expected map version {VERSION} but got {version}");
		}

		return Ok(());
	}

	fn get_array<'l>(object: &'l JsonObject, key: &str) -> Result<&'l JsonArray>
	{
		let value: &Value = get_property(object, key)?;

		return value
			.as_array()
			.with_context(|| format!("Item '{key}' was not an array"));
	}

	fn get_array_of_length<'l>(
		object: &'l JsonObject,
		key: &str,
		length: usize,
	) -> Result<&'l JsonArray>
	{
		let array: &JsonArray = get_array(object, key)?;

		if array.len() != length
		{
			bail!(
				"'{key}': Expected array of length {length}, but got length {}",
				array.len()
			);
		}

		return Ok(array);
	}

	fn get_object<'l>(object: &'l JsonObject, key: &str) -> Result<&'l JsonObject>
	{
		let value: &Value = get_property(object, key)?;

		return value
			.as_object()
			.with_context(|| format!("Item '{key}' was not an object"));
	}

	fn get_number<'l>(object: &'l JsonObject, key: &str) -> Result<&'l Number>
	{
		let value: &Value = get_property(object, key)?;

		return value
			.as_number()
			.with_context(|| format!("Item '{key}' was not a number"));
	}

	fn get_string<'l>(object: &'l JsonObject, key: &str) -> Result<&'l str>
	{
		let value: &Value = get_property(object, key)?;

		return value
			.as_str()
			.with_context(|| format!("Item '{key}' was not a string"));
	}

	fn get_usize(object: &JsonObject, key: &str) -> Result<usize>
	{
		return Ok(get_number(object, key)?
			.as_u64()
			.with_context(|| "Could not convert to u64")?
			.try_into()
			.with_context(|| "Could not convert u64 to usize")?);
	}

	fn get_dvec<VecType, const LEN: usize>(object: &JsonObject, key: &str) -> Result<VecType>
	where
		VecType: From<[f64; LEN]>,
	{
		let array: &JsonArray = get_array(object, key)?;
		return json_array_to_dvec::<VecType, LEN>(array).with_context(|| format!("'{key}':"));
	}

	fn get_dplane(object: &JsonObject, key: &str) -> Result<DPlane3>
	{
		let vec: DVec4 = get_dvec(object, key)?;
		return Ok(DPlane3::new_unchecked(vec.xyz(), vec.w));
	}

	fn json_array_to_dvec<VecType, const LEN: usize>(array: &JsonArray) -> Result<VecType>
	where
		VecType: From<[f64; LEN]>,
	{
		if array.len() != LEN
		{
			bail!(
				"Expected array of {LEN} items, but got {} items",
				array.len()
			);
		}

		let mut values: [f64; LEN] = [0.0; LEN];

		array
			.iter()
			.enumerate()
			.try_for_each(|(index, val)| -> Result<()> {
				values[index] = val.as_f64().with_context(|| format!("Index {index}"))?;
				Ok(())
			})?;

		return Ok(values.into());
	}

	fn get_property<'l>(object: &'l JsonObject, key: &str) -> Result<&'l Value>
	{
		return object
			.get(key)
			.with_context(|| format!("No '{key}' property found"));
	}
}
