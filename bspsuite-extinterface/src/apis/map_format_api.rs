use super::api_info::ApiInfo;
use crate::builders::map_blueprint_builder::{BuilderError, IMapBlueprintBuilder};
use crate::types::{DPlane, DVec2, DVec3, PortableOption, StringRef};
use std::ffi::c_void;

pub const API_INFO: ApiInfo = ApiInfo::new("MapFormatApi", 1);
pub type RegisterMapFormatsFn = extern "C" fn(&mut Api);
pub type MapParseFn = extern "C" fn(&StringRef, &mut MapBlueprintBuilder);

#[repr(C)]
pub struct Api<'l>
{
	fns: internal::CoreFns<'l>,
}

#[repr(C)]
pub struct Callbacks
{
	pub register_map_formats: RegisterMapFormatsFn,
}

impl<'l> Api<'l>
{
	pub fn register_map_format(&mut self, format_name: StringRef, parse_fn: MapParseFn)
	{
		unsafe { (self.fns.register_map_format_fn)(*self.fns.context, &format_name, parse_fn) };
	}
}

#[repr(C)]
pub struct MapBlueprintBuilder<'l>
{
	fns: internal::CoreBuilderFns<'l>,
}

// SAFETY: Core library responsible for ensuring that self.fns.context
// is valid for this struct's lifetime, and that the function being
// called knows what type to convert the context into.
impl<'l> IMapBlueprintBuilder for MapBlueprintBuilder<'l>
{
	fn begin_entity(&mut self) -> Result<(), BuilderError>
	{
		return unsafe { (self.fns.begin_entity)(*self.fns.context).to_result() };
	}

	fn end_entity(&mut self) -> Result<(), BuilderError>
	{
		return unsafe { (self.fns.end_entity)(*self.fns.context).to_result() };
	}

	fn add_entity_keyvalue(&mut self, key: &str, value: &str) -> Result<(), BuilderError>
	{
		return unsafe {
			(self.fns.add_entity_keyvalue)(
				*self.fns.context,
				&StringRef::new(key),
				&StringRef::new(value),
			)
			.to_result()
		};
	}

	fn begin_brush(&mut self) -> Result<(), BuilderError>
	{
		return unsafe { (self.fns.begin_brush)(*self.fns.context).to_result() };
	}

	fn end_brush(&mut self) -> Result<(), BuilderError>
	{
		return unsafe { (self.fns.end_brush)(*self.fns.context).to_result() };
	}

	fn begin_brush_face(&mut self) -> Result<(), BuilderError>
	{
		return unsafe { (self.fns.begin_brush_face)(*self.fns.context).to_result() };
	}

	fn end_brush_face(&mut self) -> Result<(), BuilderError>
	{
		return unsafe { (self.fns.end_brush_face)(*self.fns.context).to_result() };
	}

	fn set_brush_face_plane(&mut self, plane: DPlane) -> Result<(), BuilderError>
	{
		return unsafe { (self.fns.set_brush_face_plane)(*self.fns.context, plane).to_result() };
	}

	fn set_brush_face_material(&mut self, material_name: String) -> Result<(), BuilderError>
	{
		return unsafe {
			(self.fns.set_brush_face_material)(*self.fns.context, &StringRef::new(&material_name))
				.to_result()
		};
	}

	fn set_brush_face_material_axes(
		&mut self,
		u_unit_axis: DVec3,
		v_unit_axis: DVec3,
	) -> Result<(), BuilderError>
	{
		return unsafe {
			(self.fns.set_brush_face_material_axes)(*self.fns.context, u_unit_axis, v_unit_axis)
				.to_result()
		};
	}

	fn set_brush_face_material_translation(
		&mut self,
		translation: DVec2,
	) -> Result<(), BuilderError>
	{
		return unsafe {
			(self.fns.set_brush_face_material_translation)(*self.fns.context, translation)
				.to_result()
		};
	}

	fn set_brush_face_material_scale(&mut self, scale: DVec2) -> Result<(), BuilderError>
	{
		return unsafe {
			(self.fns.set_brush_face_material_scale)(*self.fns.context, scale).to_result()
		};
	}

	fn current_entity_index(&self) -> Option<usize>
	{
		unsafe { return (self.fns.current_entity_index)(*self.fns.context).into() };
	}

	fn current_brush_index(&self) -> Option<usize>
	{
		unsafe { return (self.fns.current_brush_index)(*self.fns.context).into() };
	}

	fn current_brush_face_index(&self) -> Option<usize>
	{
		unsafe {
			return (self.fns.current_brush_face_index)(*self.fns.context).into();
		};
	}

	fn num_entities(&self) -> usize
	{
		return unsafe { (self.fns.num_entities)(*self.fns.context) };
	}

	fn num_current_brushes(&self) -> usize
	{
		return unsafe { (self.fns.num_current_brushes)(*self.fns.context) };
	}

	fn num_current_brush_faces(&self) -> usize
	{
		return unsafe { (self.fns.num_current_brush_faces)(*self.fns.context) };
	}
}

pub mod internal
{
	use super::*;

	#[repr(C)]
	pub struct CoreFns<'l>
	{
		pub context: &'l *mut c_void,

		pub register_map_format_fn: unsafe extern "C" fn(*mut c_void, &StringRef, MapParseFn),
	}

	#[repr(C)]
	pub enum CoreBuilderResultCode
	{
		Ok,
		OperationNotStarted,
		OperationNotFinished,
	}

	impl CoreBuilderResultCode
	{
		pub fn to_result(self) -> Result<(), BuilderError>
		{
			return match self
			{
				CoreBuilderResultCode::Ok => Ok(()),
				CoreBuilderResultCode::OperationNotStarted =>
				{
					Err(BuilderError::OperationNotStarted)
				}
				CoreBuilderResultCode::OperationNotFinished =>
				{
					Err(BuilderError::OperationNotFinished)
				}
			};
		}
	}

	#[repr(C)]
	pub struct CoreBuilderFns<'l>
	{
		pub context: &'l *mut c_void,

		pub begin_entity: unsafe extern "C" fn(*mut c_void) -> CoreBuilderResultCode,
		pub end_entity: unsafe extern "C" fn(*mut c_void) -> CoreBuilderResultCode,
		pub add_entity_keyvalue: unsafe extern "C" fn(
			*mut c_void,
			key: &StringRef,
			value: &StringRef,
		) -> CoreBuilderResultCode,
		pub begin_brush: unsafe extern "C" fn(*mut c_void) -> CoreBuilderResultCode,
		pub end_brush: unsafe extern "C" fn(*mut c_void) -> CoreBuilderResultCode,
		pub begin_brush_face: unsafe extern "C" fn(*mut c_void) -> CoreBuilderResultCode,
		pub end_brush_face: unsafe extern "C" fn(*mut c_void) -> CoreBuilderResultCode,
		pub set_brush_face_plane:
			unsafe extern "C" fn(*mut c_void, plane: DPlane) -> CoreBuilderResultCode,
		pub set_brush_face_material:
			unsafe extern "C" fn(*mut c_void, material_name: &StringRef) -> CoreBuilderResultCode,
		pub set_brush_face_material_axes: unsafe extern "C" fn(
			*mut c_void,
			u_unit_axis: DVec3,
			v_unit_axis: DVec3,
		) -> CoreBuilderResultCode,
		pub set_brush_face_material_translation:
			unsafe extern "C" fn(*mut c_void, translation: DVec2) -> CoreBuilderResultCode,
		pub set_brush_face_material_scale:
			unsafe extern "C" fn(*mut c_void, scale: DVec2) -> CoreBuilderResultCode,
		pub current_entity_index: unsafe extern "C" fn(*const c_void) -> PortableOption<usize>,
		pub current_brush_index: unsafe extern "C" fn(*const c_void) -> PortableOption<usize>,
		pub current_brush_face_index: unsafe extern "C" fn(*const c_void) -> PortableOption<usize>,
		pub num_entities: unsafe extern "C" fn(*const c_void) -> usize,
		pub num_current_brushes: unsafe extern "C" fn(*const c_void) -> usize,
		pub num_current_brush_faces: unsafe extern "C" fn(*const c_void) -> usize,
	}

	pub fn create_map_format_api<'l>(fns: internal::CoreFns<'l>) -> Api<'l>
	{
		return Api { fns: fns };
	}

	pub fn create_map_blueprint_builder<'l>(
		fns: internal::CoreBuilderFns<'l>,
	) -> MapBlueprintBuilder<'l>
	{
		return MapBlueprintBuilder { fns: fns };
	}
}
