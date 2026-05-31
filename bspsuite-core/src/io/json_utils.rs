use std::collections::HashMap;

use anyhow::{Context, Result, bail};
use glam::{DVec2, DVec3, DVec4, Vec4Swizzles};
use serde_json::{Map, Number, Value};

use crate::math::DPlane3;

pub type JsonObject = Map<String, Value>;
pub type JsonArray = Vec<Value>;

pub fn serialize_dplane3(plane: DPlane3) -> Result<JsonArray>
{
	let normal: DVec3 = plane.normal();
	let values: [f64; 4] = [normal.x, normal.y, normal.z, plane.distance()];
	return dvec_slice_to_json_array(values);
}

pub fn serialize_vec3(vec: &DVec3) -> Result<JsonArray>
{
	return dvec_slice_to_json_array(vec.to_array());
}

pub fn serialize_vec2(vec: &DVec2) -> Result<JsonArray>
{
	return dvec_slice_to_json_array(vec.to_array());
}

pub fn dvec_slice_to_json_array<const LEN: usize>(contents: [f64; LEN]) -> Result<JsonArray>
{
	return to_json_array(&contents, |num| {
		Number::from_f64(*num)
			.with_context(|| "Encountered floating point value that was Inf or NaN")
	});
}

pub fn to_json_array<InType, OutType, F>(list: &[InType], callback: F) -> Result<JsonArray>
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

pub fn to_json_object<T>(map: &HashMap<String, T>) -> JsonObject
where
	T: ToString + Into<Value>,
{
	return map
		.iter()
		.map(|(key, value)| (key.clone(), Value::String(value.to_string())))
		.collect();
}

pub fn usize_to_number(val: usize) -> Value
{
	return Value::Number(
		Number::try_from(val).expect("Unexpected failure to represent usize as a JSON number type"),
	);
}

pub fn check_version(root: &JsonObject, expected_version: u64) -> Result<()>
{
	let version: &Value = root
		.get("version")
		.with_context(|| "No 'version' found in document")?;

	let version: u64 = version
		.as_u64()
		.with_context(|| "Could not parse 'version' as u64")?;

	if version != expected_version
	{
		bail!("Expected map version {expected_version} but got {version}");
	}

	return Ok(());
}

pub fn get_array<'l>(object: &'l JsonObject, key: &str) -> Result<&'l JsonArray>
{
	let value: &Value = get_property(object, key)?;

	return value
		.as_array()
		.with_context(|| format!("Item '{key}' was not an array"));
}

pub fn get_array_of_length<'l>(
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

pub fn get_object<'l>(object: &'l JsonObject, key: &str) -> Result<&'l JsonObject>
{
	let value: &Value = get_property(object, key)?;

	return value
		.as_object()
		.with_context(|| format!("Item '{key}' was not an object"));
}

pub fn get_number<'l>(object: &'l JsonObject, key: &str) -> Result<&'l Number>
{
	let value: &Value = get_property(object, key)?;

	return value
		.as_number()
		.with_context(|| format!("Item '{key}' was not a number"));
}

pub fn get_string<'l>(object: &'l JsonObject, key: &str) -> Result<&'l str>
{
	let value: &Value = get_property(object, key)?;

	return value
		.as_str()
		.with_context(|| format!("Item '{key}' was not a string"));
}

pub fn get_usize(object: &JsonObject, key: &str) -> Result<usize>
{
	return Ok(get_number(object, key)?
		.as_u64()
		.with_context(|| "Could not convert to u64")?
		.try_into()
		.with_context(|| "Could not convert u64 to usize")?);
}

pub fn get_dvec<VecType, const LEN: usize>(object: &JsonObject, key: &str) -> Result<VecType>
where
	VecType: From<[f64; LEN]>,
{
	let array: &JsonArray = get_array(object, key)?;
	return json_array_to_dvec::<VecType, LEN>(array).with_context(|| format!("'{key}':"));
}

pub fn get_dplane(object: &JsonObject, key: &str) -> Result<DPlane3>
{
	let vec: DVec4 = get_dvec(object, key)?;
	return Ok(DPlane3::new_unchecked(vec.xyz(), vec.w));
}

pub fn json_array_to_dvec<VecType, const LEN: usize>(array: &JsonArray) -> Result<VecType>
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

pub fn get_property<'l>(object: &'l JsonObject, key: &str) -> Result<&'l Value>
{
	return object
		.get(key)
		.with_context(|| format!("No '{key}' property found"));
}
