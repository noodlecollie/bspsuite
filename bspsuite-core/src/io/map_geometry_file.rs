mod version_1
{
	use std::io::Write;

	use anyhow::{Context, Result};
	use serde_json::Value;
	use serde_json::ser::to_writer_pretty;

	use crate::io::json_utils;
	use crate::io::json_utils::{JsonArray, JsonObject};
	use crate::model::{
		MapGeomBrush, MapGeomBrushFace, MapGeomBrushFaceVertex, MapGeomEntity, MapGeomFile,
	};

	pub(super) const VERSION: u64 = 1;

	pub(super) fn serialize<Writer>(writer: Writer, map: &MapGeomFile) -> Result<()>
	where
		Writer: Write,
	{
		let entities: JsonArray =
			json_utils::to_json_array(&map.entities, |ent| serialize_entity(ent))
				.with_context(|| "Entities")?;

		let mut document: JsonObject = JsonObject::new();

		document.insert("version".to_string(), VERSION.into());
		document.insert("entities".to_string(), entities.into());

		to_writer_pretty(writer, &Value::Object(document))?;
		return Ok(());
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

	fn serialize_face(face: &MapGeomBrushFace) -> Result<JsonObject>
	{
		let plane: JsonArray =
			json_utils::serialize_dplane3(face.plane).with_context(|| "Plane")?;

		let face_vertices: JsonArray =
			json_utils::to_json_array(&face.vertices, |vertex| serialize_vertex(vertex))
				.with_context(|| "Vertex")?;

		let mut obj: JsonObject = JsonObject::new();
		obj.insert("plane".to_string(), plane.into());
		obj.insert("material".to_string(), face.material.clone().into());
		obj.insert("face_vertices".to_string(), face_vertices.into());
		return Ok(obj);
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
}
