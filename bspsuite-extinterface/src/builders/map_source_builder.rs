use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::ops::{Deref, DerefMut};

use crate::types::{DPlane3, DVec2, DVec3};
use bspffi::types::{XCOption, XCStr};
use log::trace;
use thin_trait_object::thin_trait_object;

#[derive(Debug, PartialEq)]
pub enum OperationError
{
	/// A required operation had not been started.
	OperationNotStarted,

	/// A previous operation had not been completed before starting a new one.
	OperationNotFinished,

	/// A previous operation failed, prohibiting others from taking place.
	OperationFailed,
}

pub enum BuilderResult
{
	Entities(Vec<Entity>),
	Failure(BuilderError),
}

impl Default for BuilderResult
{
	fn default() -> Self
	{
		return Self::Entities(Vec::new());
	}
}

impl From<BuilderResult> for Result<Vec<Entity>, BuilderError>
{
	fn from(value: BuilderResult) -> Self
	{
		return match value
		{
			BuilderResult::Entities(entities) => Ok(entities),
			BuilderResult::Failure(error) => Err(error),
		};
	}
}

#[derive(Debug, Clone)]
pub struct BuilderError
{
	pub line: usize,
	pub column: usize,
	pub description: String,
}

impl fmt::Display for BuilderError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		write!(
			f,
			"Line {}, column {}: {}",
			self.line, self.column, self.description
		)
	}
}

impl Error for BuilderError
{
}

/// Interface for functions used to build a map source file from geometry
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
/// produced by the map source builder is valid.
#[thin_trait_object(drop_abi = "C")]
pub trait MapSourceBuilderApi
{
	/// Instructs the builder that the build process has finished. If this is
	/// not called, the result of the build process is considered to be
	/// OperationNotFinished. If desired, this can be called from the Drop trait
	/// of the implementer struct.
	fn finish(&mut self);

	/// Sets a failure state on the builder, including the line and column in
	/// the input where the failure occurred.
	///
	/// Regardless of which components have been added by other function calls,
	/// the build will be considered to have failed if this function is called.
	fn set_failure(&mut self, description: XCStr);

	/// Sets a failure state on the builder, including the line and column in
	/// the input where the failure occurred.
	///
	/// Regardless of which components have been added by other function calls,
	/// the build will be considered to have failed if this function is called.
	fn set_failure_with_location(&mut self, line: usize, column: usize, description: XCStr);

	/// Begins construction of an entity. Must be paired with [end_entity].
	///
	/// If there is already a current entity, brush or face, returns
	/// [OperationError::OperationNotFinished].
	fn begin_entity(&mut self) -> Result<(), OperationError>;

	/// Ends construction of an entity previously begun with [begin_entity].
	///
	/// If [begin_entity] has not previously been called, returns
	/// [OperationError::OperationNotStarted]. If there is any current
	/// unfinished brush or face, returns
	/// [OperationError::OperationNotFinished].
	fn end_entity(&mut self) -> Result<(), OperationError>;

	/// Adds a key-value pair to the current entity.
	///
	/// If [begin_entity] has not previously been called, returns
	/// [OperationError::OperationNotStarted]. If there is any current
	/// unfinished brush or face, returns
	/// [OperationError::OperationNotFinished].
	fn add_entity_keyvalue(&mut self, key: XCStr, value: XCStr) -> Result<(), OperationError>;

	/// Begins a brush within the current entity. Must be paired with
	/// [end_brush].
	///
	/// If there is already a current brush or face, returns
	/// [OperationError::OperationNotFinished]. If there is no current entity,
	/// returns [OperationError::OperationNotStarted].
	fn begin_brush(&mut self) -> Result<(), OperationError>;

	/// Ends construction of a brush previously begun with [begin_brush].
	///
	/// If [begin_brush] has not previously been called, returns
	/// [OperationError::OperationNotStarted]. If there is any current
	/// unfinished face, returns [OperationError::OperationNotFinished].
	fn end_brush(&mut self) -> Result<(), OperationError>;

	/// Begins a face within the current brush. Must be paired with
	/// [end_brush_face].
	///
	/// If there is already a current face, returns
	/// [OperationError::OperationNotFinished]. If there is no current entity or
	/// brush, returns [OperationError::OperationNotStarted].
	fn begin_brush_face(&mut self) -> Result<(), OperationError>;

	/// Ends construction of a face previously begun with [begin_brush_face].
	///
	/// If [begin_brush_face] jas not previously been called, returns
	/// [OperationError::OperationNotStarted].
	fn end_brush_face(&mut self) -> Result<(), OperationError>;

	/// Sets the plane of the current brush face.
	///
	/// If there is no current face, returns
	/// [OperationError::OperationNotStarted].
	fn set_brush_face_plane(&mut self, plane: DPlane3) -> Result<(), OperationError>;

	/// Sets the material for the current brush face.
	///
	/// If there is no current face, returns
	/// [OperationError::OperationNotStarted].
	fn set_brush_face_material(&mut self, material_name: XCStr) -> Result<(), OperationError>;

	/// Sets the material axes for the current brush face.
	///
	/// If there is no current face, returns
	/// [OperationError::OperationNotStarted].
	fn set_brush_face_material_axes(
		&mut self,
		u_unit_axis: DVec3,
		v_unit_axis: DVec3,
	) -> Result<(), OperationError>;

	/// Sets the material translation for the current brush face.
	///
	/// If there is no current face, returns
	/// [OperationError::OperationNotStarted].
	fn set_brush_face_material_translation(
		&mut self,
		translation: DVec2,
	) -> Result<(), OperationError>;

	/// Sets the material scale for the current brush face.
	///
	/// If there is no current face, returns
	/// [OperationError::OperationNotStarted].
	fn set_brush_face_material_scale(&mut self, scale: DVec2) -> Result<(), OperationError>;

	/// Gets the index of the current entity, or None if there is no current
	/// entity.
	fn current_entity_index(&self) -> XCOption<usize>;

	/// Gets the index of the current brush in the current entity, or None if
	/// there is no current brush or entity.
	fn current_brush_index(&self) -> XCOption<usize>;

	/// Gets the index of the current face in the current brush and entity, or
	/// None if there is no current face, brush or entity.
	fn current_brush_face_index(&self) -> XCOption<usize>;

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
	pub plane: DPlane3,
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
			plane: DPlane3::NULL,
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

// This struct is implemented in this crate so that extensions can use it to
// build their own tests for parsing the map formats they support. This
// implementation is used in the core compiler crate too.
pub struct MapSourceBuilder<'l>
{
	result: RefMut<'l, BuilderResult>,
	current_entity: Option<Entity>,
	current_brush: Option<Brush>,
	current_face: Option<BrushFace>,
}

impl<'l> MapSourceBuilder<'l>
{
	pub fn new(mut result_ref: RefMut<'l, BuilderResult>) -> Self
	{
		*result_ref = BuilderResult::default();

		return Self {
			result: result_ref,
			current_entity: None,
			current_brush: None,
			current_face: None,
		};
	}

	pub fn run<Closure>(closure: Closure) -> Result<Vec<Entity>, BuilderError>
	where
		Closure: FnOnce(BoxedMapSourceBuilderApi),
	{
		let result: RefCell<BuilderResult> = RefCell::new(BuilderResult::default());

		closure(BoxedMapSourceBuilderApi::new(MapSourceBuilder::new(
			result.borrow_mut(),
		)));

		return result.into_inner().into();
	}
}

impl<'l> Drop for MapSourceBuilder<'l>
{
	fn drop(&mut self)
	{
		self.finish();
	}
}

impl<'l> MapSourceBuilderApi for MapSourceBuilder<'l>
{
	fn finish(&mut self)
	{
		match self.result.deref()
		{
			BuilderResult::Failure(_) => return,
			BuilderResult::Entities(_) =>
			{
				let name: Option<&str> = if self.current_face.is_some()
				{
					Some("Face")
				}
				else if self.current_brush.is_some()
				{
					Some("Brush")
				}
				else if self.current_entity.is_some()
				{
					Some("Entity")
				}
				else
				{
					None
				};

				if let Some(name) = name
				{
					*self.result = BuilderResult::Failure(BuilderError {
						line: 1,
						column: 0,
						description: format!("{name} creation operation was not terminated"),
					});
				}
			}
		};
	}

	fn set_failure(&mut self, description: XCStr)
	{
		self.set_failure_with_location(1, 0, description);
	}

	fn set_failure_with_location(&mut self, line: usize, column: usize, description: XCStr)
	{
		*self.result = BuilderResult::Failure(BuilderError {
			line: line,
			column: column,
			description: description.into(),
		});
	}

	fn begin_entity(&mut self) -> Result<(), OperationError>
	{
		if self.current_entity.is_some()
			|| self.current_brush.is_some()
			|| self.current_face.is_some()
		{
			return Err(OperationError::OperationNotFinished);
		}

		self.current_entity = Some(Entity::new());
		trace!(
			"Begin entity {}",
			self.current_entity_index().into_option().unwrap()
		);

		return Ok(());
	}

	fn end_entity(&mut self) -> Result<(), OperationError>
	{
		if let BuilderResult::Failure(_) = self.result.deref()
		{
			return Err(OperationError::OperationFailed);
		}

		if self.current_entity.is_none()
		{
			return Err(OperationError::OperationNotStarted);
		}

		if self.current_brush.is_some() || self.current_face.is_some()
		{
			return Err(OperationError::OperationNotFinished);
		}

		trace!(
			"End entity {}",
			self.current_entity_index().into_option().unwrap()
		);

		match self.result.deref_mut()
		{
			BuilderResult::Entities(entities) =>
			{
				entities.push(self.current_entity.take().unwrap());
				return Ok(());
			}
			BuilderResult::Failure(_) => unreachable!(),
		}
	}

	fn add_entity_keyvalue(&mut self, key: XCStr, value: XCStr) -> Result<(), OperationError>
	{
		if self.current_entity.is_none()
		{
			return Err(OperationError::OperationNotStarted);
		}

		if self.current_brush.is_some() || self.current_face.is_some()
		{
			return Err(OperationError::OperationNotFinished);
		}

		trace!(
			"Add entity {} keyvalue: \"{}\" = \"{}\"",
			self.current_entity_index().into_option().unwrap(),
			key.as_str(),
			value.as_str()
		);

		self.current_entity
			.as_mut()
			.unwrap()
			.keyvalues
			.insert(key.into(), value.into());

		return Ok(());
	}

	fn begin_brush(&mut self) -> Result<(), OperationError>
	{
		if self.current_entity.is_none()
		{
			return Err(OperationError::OperationNotStarted);
		}

		if self.current_brush.is_some() || self.current_face.is_some()
		{
			return Err(OperationError::OperationNotFinished);
		}

		self.current_brush = Some(Brush::new());
		trace!(
			"Begin entity brush {}",
			self.current_brush_index().into_option().unwrap()
		);

		return Ok(());
	}

	fn end_brush(&mut self) -> Result<(), OperationError>
	{
		if self.current_face.is_some()
		{
			return Err(OperationError::OperationNotFinished);
		}

		if self.current_entity.is_none() || self.current_brush.is_none()
		{
			return Err(OperationError::OperationNotStarted);
		}

		trace!(
			"End entity brush {}",
			self.current_brush_index().into_option().unwrap()
		);

		self.current_entity
			.as_mut()
			.unwrap()
			.brushes
			.push(self.current_brush.take().unwrap());

		return Ok(());
	}

	fn begin_brush_face(&mut self) -> Result<(), OperationError>
	{
		if self.current_face.is_some()
		{
			return Err(OperationError::OperationNotFinished);
		}

		if self.current_entity.is_none() || self.current_brush.is_none()
		{
			return Err(OperationError::OperationNotStarted);
		}

		self.current_face = Some(BrushFace::new());
		trace!(
			"Begin entity brush face {}",
			self.current_brush_face_index().into_option().unwrap()
		);

		return Ok(());
	}

	fn end_brush_face(&mut self) -> Result<(), OperationError>
	{
		if self.current_entity.is_none()
			|| self.current_brush.is_none()
			|| self.current_face.is_none()
		{
			return Err(OperationError::OperationNotStarted);
		}

		trace!(
			"End entity brush face {}",
			self.current_brush_face_index().into_option().unwrap()
		);

		self.current_brush
			.as_mut()
			.unwrap()
			.faces
			.push(self.current_face.take().unwrap());

		return Ok(());
	}

	fn set_brush_face_plane(&mut self, plane: DPlane3) -> Result<(), OperationError>
	{
		if self.current_entity.is_none()
			|| self.current_brush.is_none()
			|| self.current_face.is_none()
		{
			return Err(OperationError::OperationNotStarted);
		}

		trace!(
			"Set face {} plane: {:?}",
			self.current_brush_face_index().into_option().unwrap(),
			plane
		);

		self.current_face.as_mut().unwrap().plane = plane;
		return Ok(());
	}

	fn set_brush_face_material(&mut self, material_name: XCStr) -> Result<(), OperationError>
	{
		if self.current_entity.is_none()
			|| self.current_brush.is_none()
			|| self.current_face.is_none()
		{
			return Err(OperationError::OperationNotStarted);
		}

		trace!(
			"Set face {} material: {}",
			self.current_brush_face_index().into_option().unwrap(),
			material_name.as_str()
		);

		self.current_face.as_mut().unwrap().material_name = material_name.into();
		return Ok(());
	}

	fn set_brush_face_material_axes(
		&mut self,
		u_unit_axis: DVec3,
		v_unit_axis: DVec3,
	) -> Result<(), OperationError>
	{
		if self.current_entity.is_none()
			|| self.current_brush.is_none()
			|| self.current_face.is_none()
		{
			return Err(OperationError::OperationNotStarted);
		}

		trace!(
			"Set face {} material axes: ({:?}, {:?})",
			self.current_brush_face_index().into_option().unwrap(),
			u_unit_axis,
			v_unit_axis
		);

		let face: &mut BrushFace = self.current_face.as_mut().unwrap();
		face.material_axes = (u_unit_axis, v_unit_axis);

		return Ok(());
	}

	fn set_brush_face_material_translation(
		&mut self,
		translation: DVec2,
	) -> Result<(), OperationError>
	{
		if self.current_entity.is_none()
			|| self.current_brush.is_none()
			|| self.current_face.is_none()
		{
			return Err(OperationError::OperationNotStarted);
		}

		trace!(
			"Set face {} material translation: {:?}",
			self.current_brush_face_index().into_option().unwrap(),
			translation
		);

		let face: &mut BrushFace = self.current_face.as_mut().unwrap();
		face.material_offset = translation;

		return Ok(());
	}

	fn set_brush_face_material_scale(&mut self, scale: DVec2) -> Result<(), OperationError>
	{
		if self.current_entity.is_none()
			|| self.current_brush.is_none()
			|| self.current_face.is_none()
		{
			return Err(OperationError::OperationNotStarted);
		}

		trace!(
			"Set face {} material scale: {:?}",
			self.current_brush_face_index().into_option().unwrap(),
			scale
		);

		let face: &mut BrushFace = self.current_face.as_mut().unwrap();
		face.material_scale = scale;

		return Ok(());
	}

	fn current_entity_index(&self) -> XCOption<usize>
	{
		return if self.current_entity.is_some()
			&& let BuilderResult::Entities(entities) = self.result.deref()
		{
			entities.len().into()
		}
		else
		{
			XCOption::None
		};
	}

	fn current_brush_index(&self) -> XCOption<usize>
	{
		return self
			.current_brush
			.as_ref()
			.map(|_| self.current_entity.as_ref().unwrap().brushes.len())
			.into();
	}

	fn current_brush_face_index(&self) -> XCOption<usize>
	{
		return self
			.current_face
			.as_ref()
			.map(|_| self.current_brush.as_ref().unwrap().faces.len())
			.into();
	}

	fn num_entities(&self) -> usize
	{
		return match self.result.deref()
		{
			BuilderResult::Entities(entities) =>
			{
				entities.len() + self.current_entity.as_ref().map_or(0, |_| 1)
			}
			BuilderResult::Failure(_) => 0,
		};
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
	fn drop_trait_calls_finish()
	{
		// First check when we call finish ourselves
		let result: Result<Vec<Entity>, BuilderError> = MapSourceBuilder::run(|mut builder| {
			assert_eq!(builder.begin_entity(), Ok(()));
			assert_eq!(builder.end_entity(), Ok(()));
			builder.finish();
		});

		assert_eq!(result.expect("Expected no failure").len(), 1);

		// Then check when we let the Drop trait do it
		let result: Result<Vec<Entity>, BuilderError> = MapSourceBuilder::run(|mut builder| {
			assert_eq!(builder.begin_entity(), Ok(()));
			assert_eq!(builder.end_entity(), Ok(()));
		});

		assert_eq!(result.expect("Expected no failure").len(), 1);
	}

	#[test]
	fn error_on_operations_not_started()
	{
		// No current entity
		{
			let result: Result<Vec<Entity>, BuilderError> = MapSourceBuilder::run(|mut builder| {
				assert_eq!(
					builder.end_entity(),
					Err(OperationError::OperationNotStarted)
				);
				assert_eq!(
					builder.begin_brush(),
					Err(OperationError::OperationNotStarted)
				);
				assert_eq!(
					builder.end_brush(),
					Err(OperationError::OperationNotStarted)
				);
				assert_eq!(
					builder.begin_brush_face(),
					Err(OperationError::OperationNotStarted)
				);
				assert_eq!(
					builder.end_brush_face(),
					Err(OperationError::OperationNotStarted)
				);
			});

			assert_eq!(result.expect("Expected no failure").len(), 0);
		}

		// No current brush
		{
			let result: Result<Vec<Entity>, BuilderError> = MapSourceBuilder::run(|mut builder| {
				assert_eq!(builder.begin_entity(), Ok(()));

				assert_eq!(
					builder.end_brush(),
					Err(OperationError::OperationNotStarted)
				);
				assert_eq!(
					builder.begin_brush_face(),
					Err(OperationError::OperationNotStarted)
				);
				assert_eq!(
					builder.end_brush_face(),
					Err(OperationError::OperationNotStarted)
				);
			});

			assert_eq!(
				result.expect_err("Expected a failure").description,
				"Entity creation operation was not terminated"
			);
		}

		// No current face
		{
			let result: Result<Vec<Entity>, BuilderError> = MapSourceBuilder::run(|mut builder| {
				assert_eq!(builder.begin_entity(), Ok(()));
				assert_eq!(builder.begin_brush(), Ok(()));

				assert_eq!(
					builder.end_brush_face(),
					Err(OperationError::OperationNotStarted)
				);
			});

			assert_eq!(
				result.expect_err("Expected a failure").description,
				"Brush creation operation was not terminated"
			);
		}
	}

	#[test]
	fn error_on_operations_not_finished()
	{
		// Begin new entity without finishing previous entity
		{
			let result: Result<Vec<Entity>, BuilderError> = MapSourceBuilder::run(|mut builder| {
				assert_eq!(builder.begin_entity(), Ok(()));

				assert_eq!(
					builder.begin_entity(),
					Err(OperationError::OperationNotFinished)
				);
			});

			assert_eq!(
				result.expect_err("Expected a failure").description,
				"Entity creation operation was not terminated"
			);
		}

		// Begin new brush without finishing previous brush
		{
			let result: Result<Vec<Entity>, BuilderError> = MapSourceBuilder::run(|mut builder| {
				assert_eq!(builder.begin_entity(), Ok(()));
				assert_eq!(builder.begin_brush(), Ok(()));

				assert_eq!(
					builder.begin_brush(),
					Err(OperationError::OperationNotFinished)
				);
			});

			assert_eq!(
				result.expect_err("Expected a failure").description,
				"Brush creation operation was not terminated"
			);
		}

		// Begin new face without finishing previous face
		{
			let result: Result<Vec<Entity>, BuilderError> = MapSourceBuilder::run(|mut builder| {
				assert_eq!(builder.begin_entity(), Ok(()));
				assert_eq!(builder.begin_brush(), Ok(()));
				assert_eq!(builder.begin_brush_face(), Ok(()));

				assert_eq!(
					builder.begin_brush_face(),
					Err(OperationError::OperationNotFinished)
				);
			});

			assert_eq!(
				result.expect_err("Expected a failure").description,
				"Face creation operation was not terminated"
			);
		}

		// Begin new entity without finishing brush
		{
			let result: Result<Vec<Entity>, BuilderError> = MapSourceBuilder::run(|mut builder| {
				assert_eq!(builder.begin_entity(), Ok(()));
				assert_eq!(builder.begin_brush(), Ok(()));

				assert_eq!(
					builder.begin_entity(),
					Err(OperationError::OperationNotFinished)
				);
			});

			assert_eq!(
				result.expect_err("Expected a failure").description,
				"Brush creation operation was not terminated"
			);
		}

		// Begin new entity or brush without finishing face
		{
			let result: Result<Vec<Entity>, BuilderError> = MapSourceBuilder::run(|mut builder| {
				assert_eq!(builder.begin_entity(), Ok(()));
				assert_eq!(builder.begin_brush(), Ok(()));
				assert_eq!(builder.begin_brush_face(), Ok(()));

				assert_eq!(
					builder.begin_entity(),
					Err(OperationError::OperationNotFinished)
				);

				assert_eq!(
					builder.begin_brush(),
					Err(OperationError::OperationNotFinished)
				);
			});
			assert_eq!(
				result.expect_err("Expected a failure").description,
				"Face creation operation was not terminated"
			);
		}
	}

	#[test]
	fn construct_empty()
	{
		let result: Result<Vec<Entity>, BuilderError> = MapSourceBuilder::run(|_| {});
		assert_eq!(result.expect("Expected no failure").len(), 0);
	}

	#[test]
	fn construct_single_empty_entity()
	{
		let result: Result<Vec<Entity>, BuilderError> = MapSourceBuilder::run(|mut builder| {
			assert_eq!(builder.begin_entity(), Ok(()));
			assert_eq!(builder.end_entity(), Ok(()));
		});

		let entities: Vec<Entity> = result.expect("Expected no failure");
		assert_eq!(entities.len(), 1);

		let ent: &Entity = &entities[0];
		assert_eq!(ent.keyvalues.len(), 0);
		assert_eq!(ent.brushes.len(), 0);
	}

	#[test]
	fn construct_single_entity_and_empty_brush()
	{
		let result: Result<Vec<Entity>, BuilderError> = MapSourceBuilder::run(|mut builder| {
			assert_eq!(builder.begin_entity(), Ok(()));
			assert_eq!(builder.begin_brush(), Ok(()));
			assert_eq!(builder.end_brush(), Ok(()));
			assert_eq!(builder.end_entity(), Ok(()));
		});

		let entities: Vec<Entity> = result.expect("Expected no failure");
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
		let result: Result<Vec<Entity>, BuilderError> = MapSourceBuilder::run(|mut builder| {
			assert_eq!(builder.begin_entity(), Ok(()));
			assert_eq!(builder.begin_brush(), Ok(()));
			assert_eq!(builder.begin_brush_face(), Ok(()));
			assert_eq!(builder.end_brush_face(), Ok(()));
			assert_eq!(builder.end_brush(), Ok(()));
			assert_eq!(builder.end_entity(), Ok(()));
		});

		let entities: Vec<Entity> = result.expect("Expected no failure");
		assert_eq!(entities.len(), 1);

		let ent: &Entity = &entities[0];
		assert_eq!(ent.keyvalues.len(), 0);
		assert_eq!(ent.brushes.len(), 1);

		let brush: &Brush = &ent.brushes[0];
		assert_eq!(brush.faces.len(), 1);

		let face: &BrushFace = &brush.faces[0];
		assert_eq!(face.plane, DPlane3::NULL);
		assert_eq!(face.material_name, "");
		assert_eq!(face.material_axes, (DVec3::NULL, DVec3::NULL));
		assert_eq!(face.material_offset, DVec2::NULL);
		assert_eq!(face.material_scale, DVec2::NULL);
	}

	#[test]
	fn construct_with_example_properties()
	{
		let face_plane = DPlane3::new(DVec3::new(1.0, 0.0, 0.0), 10.0);
		let face_axis_u = DVec3::new(-1.0, 0.0, 0.0);
		let face_axis_v = DVec3::new(0.0, 0.0, 1.0);
		let face_translation = DVec2::new(10.0, 20.0);
		let face_scale = DVec2::new(1.0, 1.5);
		let face_material = String::from("example_material");

		let result: Result<Vec<Entity>, BuilderError> = MapSourceBuilder::run(|mut builder| {
			assert_eq!(builder.begin_entity(), Ok(()));
			assert_eq!(
				builder.add_entity_keyvalue("classname".into(), "worldspawn".into()),
				Ok(())
			);
			assert_eq!(builder.begin_brush(), Ok(()));
			assert_eq!(builder.begin_brush_face(), Ok(()));
			assert_eq!(
				builder.set_brush_face_material(face_material.as_str().into()),
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
		});

		let entities: Vec<Entity> = result.expect("Expected no failure");
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
