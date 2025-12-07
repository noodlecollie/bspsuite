use crate::types::{DVec3, DVec4, StringRef};

pub enum BuilderError
{
	/// A previous operation had not been completed before starting a new one.
	UnfinishedOperation,

	/// The result of finishing an operation produced an invalid object.
	InvalidObject,

	/// Invalid arguments were provided to the function.
	InvalidArguments,
}

pub trait MapBlueprintBuilder
{
	/// Begins construction of an entity. Must be paired with [end_entity].
	///
	/// A map blueprint is basically a list of entities. Solid brushes that are
	/// not linked to any specific game entity fall under the "world" entity,
	/// with the "classname" being "world".
	fn begin_entity(&mut self) -> Result<(), BuilderError>;

	/// Ends construction of an entity previously begun with [begin_entity].
	fn end_entity(&mut self) -> Result<(), BuilderError>;

	/// Adds a key-value pair to the current entity.
	fn add_entity_keyvalue(&mut self, key: &str, value: &str) -> Result<(), BuilderError>;

	/// Begins a brush within the current entity. Must be paired with
	/// [end_brush].
	fn begin_brush(&mut self) -> Result<(), BuilderError>;

	/// Ends construction of a brush previously begun with [begin_brush].
	fn end_brush(&mut self) -> Result<(), BuilderError>;

	/// Begins a face within the current brush. Must be paired with
	/// [end_brush_face].
	fn begin_brush_face(&mut self) -> Result<(), BuilderError>;

	/// Ends construction of a face previously begun with [begin_brush_face].
	fn end_brush_face(&mut self) -> Result<(), BuilderError>;

	/// Sets the plane of the current brush face, based on three unique points.
	fn set_brush_face_points(
		&mut self,
		p0: DVec3,
		p1: DVec3,
		p2: DVec3,
	) -> Result<(), BuilderError>;

	/// Sets the material for the current brush face.
	fn set_brush_face_material(&mut self, material_name: StringRef) -> Result<(), BuilderError>;

	/// Sets the material axes for the current brush face.
	fn set_brush_face_material_axes(
		&mut self,
		u_axis: DVec4,
		uscale: f64,
		v_axis: DVec4,
		v_scale: f64,
	) -> Result<(), BuilderError>;

	/// Gets the index of the current entity, or None if there is no current
	/// entity.
	fn current_entity_index(&self) -> Option<u32>;

	/// Gets the index of the current brush in the current entity, or None if
	/// there is no current brush or entity.
	fn current_brush_index(&self) -> Option<u32>;

	/// Gets the index of the current face in the current brush and entity, or
	/// None if there is no current face, brush or entity.
	fn current_face_index(&self) -> Option<u32>;

	/// Gets the total number of entities, including any currently unfinished
	/// ones.
	fn num_entities(&self) -> u32;

	/// Gets the total number of brushes in the current entity, including any
	/// unfinished ones, or 0 if there is no current entity.
	fn num_brushes(&self) -> u32;

	/// Gets the total number of faces in the current brush, including any
	/// unfinished ones, or 0 is there is no current brush.
	fn num_faces(&self) -> u32;
}
