use crate::types::{DPlane, DVec2, DVec3};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum BuilderError
{
	/// A required operation had not been started.
	OperationNotStarted,

	/// A previous operation had not been completed before starting a new one.
	OperationNotFinished,
}

/// Interface for functions used to build a map blueprint from geometry
/// primitives.
///
/// This interface does not validate properties of the elements (eg. whether all
/// entities have classnames, whether face planes and texture axes are valid,
/// etc). It only checks that the order of the construction operations is OK:
/// entities contain brushes, which contain faces, and each of these elements
/// must finish construction before a new one can be started.
///
/// If functions to set properties on each of these elements are not called, the
/// properties will remain at their defaults. It is the responsibility of the
/// CSG phase of map compliation to check whether the resulting geometry
/// produced by the map blueprint builder is valid.
pub trait IMapBlueprintBuilder
{
	/// Begins construction of an entity. Must be paired with [end_entity].
	///
	/// If there is already a current entity, brush or face, returns
	/// [BuilderError::OperationNotFinished].
	fn begin_entity(&mut self) -> Result<(), BuilderError>;

	/// Ends construction of an entity previously begun with [begin_entity].
	///
	/// If [begin_entity] has not previously been called, returns
	/// [BuilderError::OperationNotStarted]. If there is any current unfinished
	/// brush or face, returns [BuilderError::OperationNotFinished].
	fn end_entity(&mut self) -> Result<(), BuilderError>;

	/// Adds a key-value pair to the current entity.
	///
	/// If [begin_entity] has not previously been called, returns
	/// [BuilderError::OperationNotStarted]. If there is any current unfinished
	/// brush or face, returns [BuilderError::OperationNotFinished].
	fn add_entity_keyvalue(&mut self, key: &str, value: &str) -> Result<(), BuilderError>;

	/// Begins a brush within the current entity. Must be paired with
	/// [end_brush].
	///
	/// If there is already a current brush or face, returns
	/// [BuilderError::OperationNotFinished]. If there is no current entity,
	/// returns [BuilderError::OperationNotStarted].
	fn begin_brush(&mut self) -> Result<(), BuilderError>;

	/// Ends construction of a brush previously begun with [begin_brush].
	///
	/// If [begin_brush] has not previously been called, returns
	/// [BuilderError::OperationNotStarted]. If there is any current unfinished
	/// face, returns [BuilderError::OperationNotFinished].
	fn end_brush(&mut self) -> Result<(), BuilderError>;

	/// Begins a face within the current brush. Must be paired with
	/// [end_brush_face].
	///
	/// If there is already a current face, returns
	/// [BuilderError::OperationNotFinished]. If there is no current entity or
	/// brush, returns [BuilderError::OperationNotStarted].
	fn begin_brush_face(&mut self) -> Result<(), BuilderError>;

	/// Ends construction of a face previously begun with [begin_brush_face].
	///
	/// If [begin_brush_face] jas not previously been called, returns
	/// [BuilderError::OperationNotStarted].
	fn end_brush_face(&mut self) -> Result<(), BuilderError>;

	/// Sets the plane of the current brush face.
	///
	/// If there is no current face, returns
	/// [BuilderError::OperationNotStarted].
	fn set_brush_face_plane(&mut self, plane: DPlane) -> Result<(), BuilderError>;

	/// Sets the material for the current brush face.
	///
	/// If there is no current face, returns
	/// [BuilderError::OperationNotStarted].
	fn set_brush_face_material(&mut self, material_name: String) -> Result<(), BuilderError>;

	/// Sets the material axes for the current brush face.
	///
	/// If there is no current face, returns
	/// [BuilderError::OperationNotStarted].
	fn set_brush_face_material_axes(
		&mut self,
		u_unit_axis: DVec3,
		v_unit_axis: DVec3,
	) -> Result<(), BuilderError>;

	/// Sets the material translation for the current brush face.
	///
	/// If there is no current face, returns
	/// [BuilderError::OperationNotStarted].
	fn set_brush_face_material_translation(
		&mut self,
		translation: DVec2,
	) -> Result<(), BuilderError>;

	/// Sets the material scale for the current brush face.
	///
	/// If there is no current face, returns
	/// [BuilderError::OperationNotStarted].
	fn set_brush_face_material_scale(&mut self, scale: DVec2) -> Result<(), BuilderError>;

	/// Gets the index of the current entity, or None if there is no current
	/// entity.
	fn current_entity_index(&self) -> Option<usize>;

	/// Gets the index of the current brush in the current entity, or None if
	/// there is no current brush or entity.
	fn current_brush_index(&self) -> Option<usize>;

	/// Gets the index of the current face in the current brush and entity, or
	/// None if there is no current face, brush or entity.
	fn current_brush_face_index(&self) -> Option<usize>;

	/// Gets the total number of entities, including any currently unfinished
	/// ones.
	fn num_entities(&self) -> usize;

	/// Gets the total number of brushes in the current entity, including any
	/// unfinished ones, or 0 if there is no current entity.
	fn num_current_brushes(&self) -> usize;

	/// Gets the total number of faces in the current brush, including any
	/// unfinished ones, or 0 is there is no current brush.
	fn num_current_brush_faces(&self) -> usize;
}

#[derive(Debug)]
pub struct BrushFace
{
	pub plane: DPlane,
	pub material_name: String,
	pub material_axes: (DVec3, DVec3),
	pub material_offset: DVec2,
	pub material_scale: DVec2,
}

impl BrushFace
{
	pub fn new() -> Self
	{
		return Self {
			plane: DPlane::NULL,
			material_name: String::new(),
			material_axes: (DVec3::NULL, DVec3::NULL),
			material_offset: (DVec2::NULL),
			material_scale: DVec2::NULL,
		};
	}
}

#[derive(Debug)]
pub struct Brush
{
	pub faces: Vec<BrushFace>,
}

impl Brush
{
	pub fn new() -> Self
	{
		return Self { faces: Vec::new() };
	}
}

#[derive(Debug)]
pub struct Entity
{
	pub keyvalues: HashMap<String, String>,
	pub brushes: Vec<Brush>,
}

impl Entity
{
	pub fn new() -> Self
	{
		return Self {
			keyvalues: HashMap::new(),
			brushes: Vec::new(),
		};
	}
}

pub struct MapBlueprintBuilder
{
	entities: Vec<Entity>,
	current_entity: Option<Entity>,
	current_brush: Option<Brush>,
	current_face: Option<BrushFace>,
}

impl MapBlueprintBuilder
{
	pub fn new() -> Self
	{
		return Self {
			entities: Vec::new(),
			current_entity: None,
			current_brush: None,
			current_face: None,
		};
	}

	pub fn collect(self) -> Result<Vec<Entity>, BuilderError>
	{
		if self.current_entity.is_some()
			|| self.current_brush.is_some()
			|| self.current_face.is_some()
		{
			return Err(BuilderError::OperationNotFinished);
		}

		return Ok(self.entities);
	}
}

impl IMapBlueprintBuilder for MapBlueprintBuilder
{
	fn begin_entity(&mut self) -> Result<(), BuilderError>
	{
		if self.current_entity.is_some()
			|| self.current_brush.is_some()
			|| self.current_face.is_some()
		{
			return Err(BuilderError::OperationNotFinished);
		}

		self.current_entity = Some(Entity::new());
		return Ok(());
	}

	fn end_entity(&mut self) -> Result<(), BuilderError>
	{
		if self.current_entity.is_none()
		{
			return Err(BuilderError::OperationNotStarted);
		}

		if self.current_brush.is_some() || self.current_face.is_some()
		{
			return Err(BuilderError::OperationNotFinished);
		}

		self.entities.push(self.current_entity.take().unwrap());
		return Ok(());
	}

	fn add_entity_keyvalue(&mut self, key: &str, value: &str) -> Result<(), BuilderError>
	{
		if self.current_entity.is_none()
		{
			return Err(BuilderError::OperationNotStarted);
		}

		if self.current_brush.is_some() || self.current_face.is_some()
		{
			return Err(BuilderError::OperationNotFinished);
		}

		self.current_entity
			.as_mut()
			.unwrap()
			.keyvalues
			.insert(String::from(key), String::from(value));

		return Ok(());
	}

	fn begin_brush(&mut self) -> Result<(), BuilderError>
	{
		if self.current_entity.is_none()
		{
			return Err(BuilderError::OperationNotStarted);
		}

		if self.current_brush.is_some() || self.current_face.is_some()
		{
			return Err(BuilderError::OperationNotFinished);
		}

		self.current_brush = Some(Brush::new());
		return Ok(());
	}

	fn end_brush(&mut self) -> Result<(), BuilderError>
	{
		if self.current_face.is_some()
		{
			return Err(BuilderError::OperationNotFinished);
		}

		if self.current_entity.is_none() || self.current_brush.is_none()
		{
			return Err(BuilderError::OperationNotStarted);
		}

		self.current_entity
			.as_mut()
			.unwrap()
			.brushes
			.push(self.current_brush.take().unwrap());

		return Ok(());
	}

	fn begin_brush_face(&mut self) -> Result<(), BuilderError>
	{
		if self.current_face.is_some()
		{
			return Err(BuilderError::OperationNotFinished);
		}

		if self.current_entity.is_none() || self.current_brush.is_none()
		{
			return Err(BuilderError::OperationNotStarted);
		}

		self.current_face = Some(BrushFace::new());
		return Ok(());
	}

	fn end_brush_face(&mut self) -> Result<(), BuilderError>
	{
		if self.current_entity.is_none()
			|| self.current_brush.is_none()
			|| self.current_face.is_none()
		{
			return Err(BuilderError::OperationNotStarted);
		}

		self.current_brush
			.as_mut()
			.unwrap()
			.faces
			.push(self.current_face.take().unwrap());

		return Ok(());
	}

	fn set_brush_face_plane(&mut self, plane: DPlane) -> Result<(), BuilderError>
	{
		if self.current_entity.is_none()
			|| self.current_brush.is_none()
			|| self.current_face.is_none()
		{
			return Err(BuilderError::OperationNotStarted);
		}

		self.current_face.as_mut().unwrap().plane = plane;
		return Ok(());
	}

	fn set_brush_face_material(&mut self, material_name: String) -> Result<(), BuilderError>
	{
		if self.current_entity.is_none()
			|| self.current_brush.is_none()
			|| self.current_face.is_none()
		{
			return Err(BuilderError::OperationNotStarted);
		}

		self.current_face.as_mut().unwrap().material_name = material_name;
		return Ok(());
	}

	fn set_brush_face_material_axes(
		&mut self,
		u_unit_axis: DVec3,
		v_unit_axis: DVec3,
	) -> Result<(), BuilderError>
	{
		if self.current_entity.is_none()
			|| self.current_brush.is_none()
			|| self.current_face.is_none()
		{
			return Err(BuilderError::OperationNotStarted);
		}

		let face: &mut BrushFace = self.current_face.as_mut().unwrap();
		face.material_axes = (u_unit_axis, v_unit_axis);

		return Ok(());
	}

	fn set_brush_face_material_translation(
		&mut self,
		translation: DVec2,
	) -> Result<(), BuilderError>
	{
		if self.current_entity.is_none()
			|| self.current_brush.is_none()
			|| self.current_face.is_none()
		{
			return Err(BuilderError::OperationNotStarted);
		}

		let face: &mut BrushFace = self.current_face.as_mut().unwrap();
		face.material_offset = translation;

		return Ok(());
	}

	fn set_brush_face_material_scale(&mut self, scale: DVec2) -> Result<(), BuilderError>
	{
		if self.current_entity.is_none()
			|| self.current_brush.is_none()
			|| self.current_face.is_none()
		{
			return Err(BuilderError::OperationNotStarted);
		}

		let face: &mut BrushFace = self.current_face.as_mut().unwrap();
		face.material_scale = scale;

		return Ok(());
	}

	fn current_entity_index(&self) -> Option<usize>
	{
		return self.current_entity.as_ref().map(|_| self.entities.len());
	}

	fn current_brush_index(&self) -> Option<usize>
	{
		return self
			.current_brush
			.as_ref()
			.map(|_| self.current_entity.as_ref().unwrap().brushes.len());
	}

	fn current_brush_face_index(&self) -> Option<usize>
	{
		return self
			.current_face
			.as_ref()
			.map(|_| self.current_brush.as_ref().unwrap().faces.len());
	}

	fn num_entities(&self) -> usize
	{
		return self.entities.len() + self.current_entity.as_ref().map_or(0, |_| 1);
	}

	fn num_current_brushes(&self) -> usize
	{
		return self.current_entity.as_ref().map_or(0, |ent| {
			ent.brushes.len() + self.current_brush.as_ref().map_or(0, |_| 1)
		});
	}

	fn num_current_brush_faces(&self) -> usize
	{
		return self.current_brush.as_ref().map_or(0, |brush| {
			brush.faces.len() + self.current_face.as_ref().map_or(0, |_| 1)
		});
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn error_on_operations_not_started()
	{
		// No current entity
		{
			let mut builder = MapBlueprintBuilder::new();
			assert_eq!(builder.end_entity(), Err(BuilderError::OperationNotStarted));
			assert_eq!(
				builder.begin_brush(),
				Err(BuilderError::OperationNotStarted)
			);
			assert_eq!(builder.end_brush(), Err(BuilderError::OperationNotStarted));
			assert_eq!(
				builder.begin_brush_face(),
				Err(BuilderError::OperationNotStarted)
			);
			assert_eq!(
				builder.end_brush_face(),
				Err(BuilderError::OperationNotStarted)
			);

			let entities = builder.collect();
			assert!(entities.is_ok());
			assert_eq!(entities.as_ref().unwrap().len(), 0);
		}

		// No current brush
		{
			let mut builder = MapBlueprintBuilder::new();
			assert_eq!(builder.begin_entity(), Ok(()));

			assert_eq!(builder.end_brush(), Err(BuilderError::OperationNotStarted));
			assert_eq!(
				builder.begin_brush_face(),
				Err(BuilderError::OperationNotStarted)
			);
			assert_eq!(
				builder.end_brush_face(),
				Err(BuilderError::OperationNotStarted)
			);

			let entities = builder.collect();
			assert!(entities.is_err());
			assert_eq!(entities.unwrap_err(), BuilderError::OperationNotFinished);
		}

		// No current face
		{
			let mut builder = MapBlueprintBuilder::new();
			assert_eq!(builder.begin_entity(), Ok(()));
			assert_eq!(builder.begin_brush(), Ok(()));

			assert_eq!(
				builder.end_brush_face(),
				Err(BuilderError::OperationNotStarted)
			);

			let entities = builder.collect();
			assert!(entities.is_err());
			assert_eq!(entities.unwrap_err(), BuilderError::OperationNotFinished);
		}
	}

	#[test]
	fn error_on_operations_not_finished()
	{
		// Begin new entity without finishing previous entity
		{
			let mut builder = MapBlueprintBuilder::new();
			assert_eq!(builder.begin_entity(), Ok(()));

			assert_eq!(
				builder.begin_entity(),
				Err(BuilderError::OperationNotFinished)
			);

			let entities = builder.collect();
			assert!(entities.is_err());
			assert_eq!(entities.unwrap_err(), BuilderError::OperationNotFinished);
		}

		// Begin new brush without finishing previous brush
		{
			let mut builder = MapBlueprintBuilder::new();
			assert_eq!(builder.begin_entity(), Ok(()));
			assert_eq!(builder.begin_brush(), Ok(()));

			assert_eq!(
				builder.begin_brush(),
				Err(BuilderError::OperationNotFinished)
			);

			let entities = builder.collect();
			assert!(entities.is_err());
			assert_eq!(entities.unwrap_err(), BuilderError::OperationNotFinished);
		}

		// Begin new face without finishing previous face
		{
			let mut builder = MapBlueprintBuilder::new();
			assert_eq!(builder.begin_entity(), Ok(()));
			assert_eq!(builder.begin_brush(), Ok(()));
			assert_eq!(builder.begin_brush_face(), Ok(()));

			assert_eq!(
				builder.begin_brush_face(),
				Err(BuilderError::OperationNotFinished)
			);

			let entities = builder.collect();
			assert!(entities.is_err());
			assert_eq!(entities.unwrap_err(), BuilderError::OperationNotFinished);
		}

		// Begin new entity without finishing brush
		{
			let mut builder = MapBlueprintBuilder::new();
			assert_eq!(builder.begin_entity(), Ok(()));
			assert_eq!(builder.begin_brush(), Ok(()));

			assert_eq!(
				builder.begin_entity(),
				Err(BuilderError::OperationNotFinished)
			);

			let entities = builder.collect();
			assert!(entities.is_err());
			assert_eq!(entities.unwrap_err(), BuilderError::OperationNotFinished);
		}

		// Begin new entity or brush without finishing face
		{
			let mut builder = MapBlueprintBuilder::new();
			assert_eq!(builder.begin_entity(), Ok(()));
			assert_eq!(builder.begin_brush(), Ok(()));
			assert_eq!(builder.begin_brush_face(), Ok(()));

			assert_eq!(
				builder.begin_entity(),
				Err(BuilderError::OperationNotFinished)
			);

			assert_eq!(
				builder.begin_brush(),
				Err(BuilderError::OperationNotFinished)
			);

			let entities = builder.collect();
			assert!(entities.is_err());
			assert_eq!(entities.unwrap_err(), BuilderError::OperationNotFinished);
		}
	}

	#[test]
	fn construct_empty()
	{
		let builder = MapBlueprintBuilder::new();
		let entities = builder.collect();
		assert!(entities.is_ok());
		assert_eq!(entities.as_ref().unwrap().len(), 0);
	}

	#[test]
	fn construct_single_empty_entity()
	{
		let mut builder = MapBlueprintBuilder::new();
		assert_eq!(builder.begin_entity(), Ok(()));
		assert_eq!(builder.end_entity(), Ok(()));

		let entities = builder.collect();
		assert!(entities.is_ok());

		let entities = entities.unwrap();
		assert_eq!(entities.len(), 1);

		let ent: &Entity = &entities[0];
		assert_eq!(ent.keyvalues.len(), 0);
		assert_eq!(ent.brushes.len(), 0);
	}

	#[test]
	fn construct_single_entity_and_empty_brush()
	{
		let mut builder = MapBlueprintBuilder::new();
		assert_eq!(builder.begin_entity(), Ok(()));
		assert_eq!(builder.begin_brush(), Ok(()));
		assert_eq!(builder.end_brush(), Ok(()));
		assert_eq!(builder.end_entity(), Ok(()));

		let entities = builder.collect();
		assert!(entities.is_ok());

		let entities = entities.unwrap();
		assert_eq!(entities.len(), 1);

		let ent: &Entity = &entities[0];
		assert_eq!(ent.keyvalues.len(), 0);
		assert_eq!(ent.brushes.len(), 1);

		let brush: &Brush = &ent.brushes[0];
		assert_eq!(brush.faces.len(), 0);
	}

	#[test]
	fn construct_single_entity_and_brush_with_single_face()
	{
		let mut builder = MapBlueprintBuilder::new();
		assert_eq!(builder.begin_entity(), Ok(()));
		assert_eq!(builder.begin_brush(), Ok(()));
		assert_eq!(builder.begin_brush_face(), Ok(()));
		assert_eq!(builder.end_brush_face(), Ok(()));
		assert_eq!(builder.end_brush(), Ok(()));
		assert_eq!(builder.end_entity(), Ok(()));

		let entities = builder.collect();
		assert!(entities.is_ok());

		let entities = entities.unwrap();
		assert_eq!(entities.len(), 1);

		let ent: &Entity = &entities[0];
		assert_eq!(ent.keyvalues.len(), 0);
		assert_eq!(ent.brushes.len(), 1);

		let brush: &Brush = &ent.brushes[0];
		assert_eq!(brush.faces.len(), 1);

		let face: &BrushFace = &brush.faces[0];
		assert_eq!(face.plane, DPlane::NULL);
		assert_eq!(face.material_name, "");
		assert_eq!(face.material_axes, (DVec3::NULL, DVec3::NULL));
		assert_eq!(face.material_offset, DVec2::NULL);
		assert_eq!(face.material_scale, DVec2::NULL);
	}

	#[test]
	fn construct_with_example_properties()
	{
		let face_plane = DPlane::new(DVec3::new(1.0, 0.0, 0.0), 10.0);
		let face_axis_u = DVec3::new(-1.0, 0.0, 0.0);
		let face_axis_v = DVec3::new(0.0, 0.0, 1.0);
		let face_translation = DVec2::new(10.0, 20.0);
		let face_scale = DVec2::new(1.0, 1.5);
		let face_material = String::from("example_material");

		let mut builder = MapBlueprintBuilder::new();
		assert_eq!(builder.begin_entity(), Ok(()));
		assert_eq!(
			builder.add_entity_keyvalue("classname", "worldspawn"),
			Ok(())
		);
		assert_eq!(builder.begin_brush(), Ok(()));
		assert_eq!(builder.begin_brush_face(), Ok(()));
		assert_eq!(
			builder.set_brush_face_material(face_material.clone()),
			Ok(())
		);
		assert_eq!(builder.set_brush_face_plane(face_plane), Ok(()));
		assert_eq!(
			builder.set_brush_face_material_axes(face_axis_u, face_axis_v),
			Ok(())
		);
		assert_eq!(
			builder.set_brush_face_material_translation(face_translation),
			Ok(())
		);
		assert_eq!(builder.set_brush_face_material_scale(face_scale), Ok(()));
		assert_eq!(builder.end_brush_face(), Ok(()));
		assert_eq!(builder.end_brush(), Ok(()));
		assert_eq!(builder.end_entity(), Ok(()));

		let entities = builder.collect();
		assert!(entities.is_ok());

		let entities = entities.unwrap();
		assert_eq!(entities.len(), 1);

		let ent: &Entity = &entities[0];
		assert_eq!(ent.keyvalues.len(), 1);
		assert_eq!(
			ent.keyvalues.get("classname"),
			Some(String::from("worldspawn")).as_ref()
		);
		assert_eq!(ent.brushes.len(), 1);

		let brush: &Brush = &ent.brushes[0];
		assert_eq!(brush.faces.len(), 1);

		let face: &BrushFace = &brush.faces[0];
		assert_eq!(face.plane, face_plane);
		assert_eq!(face.material_name, face_material);
		assert_eq!(face.material_axes, (face_axis_u, face_axis_v));
		assert_eq!(face.material_offset, face_translation);
		assert_eq!(face.material_scale, face_scale);
	}
}
