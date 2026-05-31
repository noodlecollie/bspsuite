mod version_1
{
	use std::io::Write;

	use anyhow::Result;
	use serde_json::ser::to_writer_pretty;
	use serde_json::{Map, Value};

	use crate::io::json_utils;
	use crate::io::json_utils::{JsonArray, JsonObject};
	use crate::model::{MapGeomEntity, MapGeomFile};

	pub(super) const VERSION: u64 = 1;

	pub(super) fn serialize<Writer>(writer: Writer, map: &MapGeomFile) -> Result<()>
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

	fn serialize_entity(entity: &MapGeomEntity) -> Result<JsonObject>
	{
		todo!();
	}
}
