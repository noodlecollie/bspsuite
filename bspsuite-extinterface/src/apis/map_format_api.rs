use crate::ApiInfo;
use crate::builders::map_source_builder::{IMapSourceBuilder, OperationError};
use crate::types::{DPlane3, DVec2, DVec3};
use bspffi::types::{XCOption, XCSlice, XCStr};

pub const API_INFO: ApiInfo = ApiInfo::new("MapFormatApi", 1);
pub type RegisterMapFormatsFn = extern "C" fn(&mut MapFormatApi);
pub type MapParseFn = extern "C" fn(&XCStr, &mut MapSourceBuilderApi);

#[repr(C)]
pub struct MapFormatApi<'l>
{
	ffi_table: internal::ApiFfiTable<'l>,
}

#[repr(C)]
pub struct MapFormatApiCallbacks
{
	pub register_map_formats: RegisterMapFormatsFn,
}

impl<'l> MapFormatApi<'l>
{
	pub fn register_map_format(
		&mut self,
		format_name: &XCStr,
		file_extensions: &XCSlice<XCStr>,
		parse_fn: MapParseFn,
	)
	{
		unsafe {
			(self.ffi_table.register_map_format_fn)(
				&mut self.ffi_table.context,
				format_name,
				file_extensions,
				parse_fn,
			)
		};
	}
}

#[repr(C)]
pub struct MapSourceBuilderApi<'l>
{
	ffi_table: internal::BuilderFfiTable<'l>,
}

// SAFETY: Core library responsible for ensuring that self.ffi_table.context
// is valid for this struct's lifetime, and that the function being
// called knows what type to convert the context into.
impl<'l> IMapSourceBuilder for MapSourceBuilderApi<'l>
{
	fn set_failure(&mut self, description: String)
	{
		unsafe {
			(self.ffi_table.set_failure)(&mut self.ffi_table.context, &XCStr::new(&description))
		};
	}

	fn set_failure_with_location(&mut self, line: usize, column: usize, description: String)
	{
		unsafe {
			(self.ffi_table.set_failure_with_location)(
				&mut self.ffi_table.context,
				line,
				column,
				&XCStr::new(&description),
			)
		};
	}

	fn begin_entity(&mut self) -> Result<(), OperationError>
	{
		return unsafe { (self.ffi_table.begin_entity)(&mut self.ffi_table.context).into() };
	}

	fn end_entity(&mut self) -> Result<(), OperationError>
	{
		return unsafe { (self.ffi_table.end_entity)(&mut self.ffi_table.context).into() };
	}

	fn add_entity_keyvalue(&mut self, key: String, value: String) -> Result<(), OperationError>
	{
		return unsafe {
			(self.ffi_table.add_entity_keyvalue)(
				&mut self.ffi_table.context,
				&XCStr::new(&key),
				&XCStr::new(&value),
			)
			.into()
		};
	}

	fn begin_brush(&mut self) -> Result<(), OperationError>
	{
		return unsafe { (self.ffi_table.begin_brush)(&mut self.ffi_table.context).into() };
	}

	fn end_brush(&mut self) -> Result<(), OperationError>
	{
		return unsafe { (self.ffi_table.end_brush)(&mut self.ffi_table.context).into() };
	}

	fn begin_brush_face(&mut self) -> Result<(), OperationError>
	{
		return unsafe { (self.ffi_table.begin_brush_face)(&mut self.ffi_table.context).into() };
	}

	fn end_brush_face(&mut self) -> Result<(), OperationError>
	{
		return unsafe { (self.ffi_table.end_brush_face)(&mut self.ffi_table.context).into() };
	}

	fn set_brush_face_plane(&mut self, plane: DPlane3) -> Result<(), OperationError>
	{
		return unsafe {
			(self.ffi_table.set_brush_face_plane)(&mut self.ffi_table.context, plane).into()
		};
	}

	fn set_brush_face_material(&mut self, material_name: String) -> Result<(), OperationError>
	{
		return unsafe {
			(self.ffi_table.set_brush_face_material)(
				&mut self.ffi_table.context,
				&XCStr::new(&material_name),
			)
			.into()
		};
	}

	fn set_brush_face_material_axes(
		&mut self,
		u_unit_axis: DVec3,
		v_unit_axis: DVec3,
	) -> Result<(), OperationError>
	{
		return unsafe {
			(self.ffi_table.set_brush_face_material_axes)(
				&mut self.ffi_table.context,
				u_unit_axis,
				v_unit_axis,
			)
			.into()
		};
	}

	fn set_brush_face_material_translation(
		&mut self,
		translation: DVec2,
	) -> Result<(), OperationError>
	{
		return unsafe {
			(self.ffi_table.set_brush_face_material_translation)(
				&mut self.ffi_table.context,
				translation,
			)
			.into()
		};
	}

	fn set_brush_face_material_scale(&mut self, scale: DVec2) -> Result<(), OperationError>
	{
		return unsafe {
			(self.ffi_table.set_brush_face_material_scale)(&mut self.ffi_table.context, scale)
				.into()
		};
	}

	fn current_entity_index(&self) -> Option<usize>
	{
		unsafe { return (self.ffi_table.current_entity_index)(&self.ffi_table.context).into() };
	}

	fn current_brush_index(&self) -> Option<usize>
	{
		unsafe { return (self.ffi_table.current_brush_index)(&self.ffi_table.context).into() };
	}

	fn current_brush_face_index(&self) -> Option<usize>
	{
		unsafe {
			return (self.ffi_table.current_brush_face_index)(&self.ffi_table.context).into();
		};
	}

	fn num_entities(&self) -> usize
	{
		return unsafe { (self.ffi_table.num_entities)(&self.ffi_table.context) };
	}

	fn num_current_brushes(&self) -> usize
	{
		return unsafe { (self.ffi_table.num_current_brushes)(&self.ffi_table.context) };
	}

	fn num_current_brush_faces(&self) -> usize
	{
		return unsafe { (self.ffi_table.num_current_brush_faces)(&self.ffi_table.context) };
	}
}

pub mod internal
{
	use super::*;
	use bspffi::types::XCSlice;
	use bspffi::types::internal::ContextPtr;
	use core::marker::{PhantomData, PhantomPinned};

	pub type ApiCtx<'l> = ContextPtr<'l, ApiOpaqueContext>;
	pub type BuilderCtx<'l> = ContextPtr<'l, BuilderOpaqueContext>;

	#[repr(C)]
	pub struct ApiOpaqueContext
	{
		_data: (),
		_marker: PhantomData<(*mut u8, PhantomPinned)>,
	}

	#[repr(C)]
	pub struct BuilderOpaqueContext
	{
		_data: (),
		_marker: PhantomData<(*mut u8, PhantomPinned)>,
	}

	#[repr(C)]
	pub enum BuilderErrorCode
	{
		Ok,
		OperationNotStarted,
		OperationNotFinished,
	}

	impl From<Result<(), OperationError>> for BuilderErrorCode
	{
		fn from(value: Result<(), OperationError>) -> Self
		{
			return value
				.map(|_| BuilderErrorCode::Ok)
				.unwrap_or_else(|err| match err
				{
					OperationError::OperationNotStarted => BuilderErrorCode::OperationNotStarted,
					OperationError::OperationNotFinished => BuilderErrorCode::OperationNotFinished,
				});
		}
	}

	impl Into<Result<(), OperationError>> for BuilderErrorCode
	{
		fn into(self) -> Result<(), OperationError>
		{
			return match self
			{
				BuilderErrorCode::Ok => Ok(()),
				BuilderErrorCode::OperationNotStarted => Err(OperationError::OperationNotStarted),
				BuilderErrorCode::OperationNotFinished => Err(OperationError::OperationNotFinished),
			};
		}
	}

	// SAFETY:
	// The creator of this struct must guarantee:
	// - The data pointed to by the context pointer lives for at least as long as
	//   this struct lives. This is implied by the reference lifetime.
	// - Functions stored in the struct convert the context pointer to the correct
	//   type before they use it.
	#[repr(C)]
	pub struct ApiFfiTable<'l>
	{
		pub context: ApiCtx<'l>,

		pub register_map_format_fn:
			unsafe extern "C" fn(&mut ApiCtx, &XCStr, &XCSlice<XCStr>, MapParseFn),
	}

	// SAFETY:
	// The creator of this struct must guarantee:
	// - The data pointed to by the context pointer lives for at least as long as
	//   this struct lives. This is implied by the reference lifetime.
	// - Functions stored in the struct convert the context pointer to the correct
	//   type before they use it.
	#[repr(C)]
	pub struct BuilderFfiTable<'l>
	{
		pub context: BuilderCtx<'l>,

		pub set_failure: unsafe extern "C" fn(&mut BuilderCtx, &XCStr),
		pub set_failure_with_location: unsafe extern "C" fn(&mut BuilderCtx, usize, usize, &XCStr),
		pub begin_entity: unsafe extern "C" fn(&mut BuilderCtx) -> BuilderErrorCode,
		pub end_entity: unsafe extern "C" fn(&mut BuilderCtx) -> BuilderErrorCode,
		pub add_entity_keyvalue:
			unsafe extern "C" fn(&mut BuilderCtx, &XCStr, &XCStr) -> BuilderErrorCode,
		pub begin_brush: unsafe extern "C" fn(&mut BuilderCtx) -> BuilderErrorCode,
		pub end_brush: unsafe extern "C" fn(&mut BuilderCtx) -> BuilderErrorCode,
		pub begin_brush_face: unsafe extern "C" fn(&mut BuilderCtx) -> BuilderErrorCode,
		pub end_brush_face: unsafe extern "C" fn(&mut BuilderCtx) -> BuilderErrorCode,
		pub set_brush_face_plane:
			unsafe extern "C" fn(&mut BuilderCtx, DPlane3) -> BuilderErrorCode,
		pub set_brush_face_material:
			unsafe extern "C" fn(&mut BuilderCtx, &XCStr) -> BuilderErrorCode,
		pub set_brush_face_material_axes:
			unsafe extern "C" fn(&mut BuilderCtx, DVec3, DVec3) -> BuilderErrorCode,
		pub set_brush_face_material_translation:
			unsafe extern "C" fn(&mut BuilderCtx, DVec2) -> BuilderErrorCode,
		pub set_brush_face_material_scale:
			unsafe extern "C" fn(&mut BuilderCtx, DVec2) -> BuilderErrorCode,
		pub current_entity_index: unsafe extern "C" fn(&BuilderCtx) -> XCOption<usize>,
		pub current_brush_index: unsafe extern "C" fn(&BuilderCtx) -> XCOption<usize>,
		pub current_brush_face_index: unsafe extern "C" fn(&BuilderCtx) -> XCOption<usize>,
		pub num_entities: unsafe extern "C" fn(&BuilderCtx) -> usize,
		pub num_current_brushes: unsafe extern "C" fn(&BuilderCtx) -> usize,
		pub num_current_brush_faces: unsafe extern "C" fn(&BuilderCtx) -> usize,
	}

	pub fn create_map_format_api<'l>(ffi_table: ApiFfiTable<'l>) -> MapFormatApi<'l>
	{
		return MapFormatApi {
			ffi_table: ffi_table,
		};
	}

	pub fn create_map_source_builder_api<'l>(
		ffi_table: BuilderFfiTable<'l>,
	) -> MapSourceBuilderApi<'l>
	{
		return MapSourceBuilderApi {
			ffi_table: ffi_table,
		};
	}
}
