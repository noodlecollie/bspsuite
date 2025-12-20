use super::opaque_ptr::OpaqueMutPtr;
use bspextifc::map_format_api;
use bspextifc::types::StringRef;
use log::{debug, warn};
use std::collections::HashMap;
use std::ffi::c_void;

pub struct MapParseCallback
{
	// This callback must be encapsulated, since it's
	// copyable/cloneable and depends on the extension
	// library, but we have no way to codify this dependency!
	// Instead, we treat the callback as being owned
	// by the endpoint, which in turn is owned by the
	// extension.
	parse_fn: map_format_api::MapParseFn,
}

pub struct Endpoint
{
	inner: map_format_api::Callbacks,
	map_formats: HashMap<String, MapParseCallback>,
}

impl Endpoint
{
	pub fn new(callbacks: map_format_api::Callbacks) -> Self
	{
		return Self {
			inner: callbacks,
			map_formats: HashMap::new(),
		};
	}

	pub fn register_map_formats(&mut self, extension_name: &str)
	{
		let mut api_impl: ApiImpl = ApiImpl::new(extension_name);
		let mut context: OpaqueMutPtr<ApiImpl> = OpaqueMutPtr::new(&mut api_impl);

		let core_fns: map_format_api::internal::CoreFns = map_format_api::internal::CoreFns {
			context: context.as_mut_void_ref(),
			register_map_format_fn: register_map_format,
		};

		let mut api: map_format_api::Api =
			map_format_api::internal::create_map_format_api(core_fns);
		(self.inner.register_map_formats)(&mut api);

		self.map_formats = api_impl.finish();
	}

	pub fn supports_map_format(&self, format_name: &str) -> bool
	{
		return self.map_formats.contains_key(format_name);
	}

	pub fn get_parse_callback(&self, format_name: &str) -> Option<&MapParseCallback>
	{
		return self.map_formats.get(format_name);
	}

	pub fn get_supported_map_formats(&self) -> Vec<String>
	{
		return self.map_formats.keys().map(|item| item.clone()).collect();
	}
}

impl MapParseCallback
{
	pub fn parse(&self, data: &str, builder: &mut map_format_api::MapBlueprintBuilder)
	{
		(self.parse_fn)(&StringRef::new(data), builder);
	}
}

struct ApiImpl<'l>
{
	extension_name: &'l str,
	formats: HashMap<String, MapParseCallback>,
}

impl<'l> ApiImpl<'l>
{
	pub fn new(extension_name: &'l str) -> Self
	{
		return Self {
			extension_name: extension_name,
			formats: HashMap::new(),
		};
	}

	pub fn register_map_format(&mut self, format_name: &str, parse_fn: map_format_api::MapParseFn)
	{
		if let Some(_) = self.formats.insert(
			String::from(format_name),
			MapParseCallback { parse_fn: parse_fn },
		)
		{
			warn!(
				"Overriding existing registered parse function for extension {} map format \"{format_name}\"",
				self.extension_name
			);
		}
		else
		{
			debug!(
				"Extension {} registered support for map format \"{format_name}\"",
				self.extension_name
			);
		}
	}

	pub fn finish(self) -> HashMap<String, MapParseCallback>
	{
		return self.formats;
	}
}

unsafe extern "C" fn register_map_format(
	context: *mut c_void,
	format_name: &StringRef,
	parse_fn: map_format_api::MapParseFn,
)
{
	unsafe {
		(*context.cast::<ApiImpl>())
			.register_map_format(format_name.to_string().as_ref(), parse_fn);
	};
}

mod builder_extc
{
	use super::*;
	use bspextifc::builders::map_blueprint_builder::{
		IMapBlueprintBuilder, MapBlueprintBuilder as Builder,
	};
	use bspextifc::map_format_api::internal::CoreBuilderOperationErrorCode;
	use bspextifc::types::{DPlane, DVec2, DVec3, PortableOption};

	unsafe extern "C" fn set_failure(context: *mut c_void, description: &StringRef)
	{
		unsafe {
			(*context.cast::<Builder>()).set_failure(description.to_string());
		}
	}

	unsafe extern "C" fn set_failure_with_location(
		context: *mut c_void,
		line: usize,
		column: usize,
		description: &StringRef,
	)
	{
		unsafe {
			(*context.cast::<Builder>()).set_failure_with_location(
				line,
				column,
				description.to_string(),
			);
		}
	}

	unsafe extern "C" fn begin_entity(context: *mut c_void) -> CoreBuilderOperationErrorCode
	{
		return unsafe {
			CoreBuilderOperationErrorCode::from_result((*context.cast::<Builder>()).begin_entity())
		};
	}

	unsafe extern "C" fn end_entity(context: *mut c_void) -> CoreBuilderOperationErrorCode
	{
		return unsafe {
			CoreBuilderOperationErrorCode::from_result((*context.cast::<Builder>()).end_entity())
		};
	}

	unsafe extern "C" fn add_entity_keyvalue(
		context: *mut c_void,
		key: &StringRef,
		value: &StringRef,
	) -> CoreBuilderOperationErrorCode
	{
		return unsafe {
			CoreBuilderOperationErrorCode::from_result(
				(*context.cast::<Builder>())
					.add_entity_keyvalue(key.to_string(), value.to_string()),
			)
		};
	}

	unsafe extern "C" fn begin_brush(context: *mut c_void) -> CoreBuilderOperationErrorCode
	{
		return unsafe {
			CoreBuilderOperationErrorCode::from_result((*context.cast::<Builder>()).begin_brush())
		};
	}

	unsafe extern "C" fn end_brush(context: *mut c_void) -> CoreBuilderOperationErrorCode
	{
		return unsafe {
			CoreBuilderOperationErrorCode::from_result((*context.cast::<Builder>()).end_brush())
		};
	}

	unsafe extern "C" fn begin_brush_face(context: *mut c_void) -> CoreBuilderOperationErrorCode
	{
		return unsafe {
			CoreBuilderOperationErrorCode::from_result(
				(*context.cast::<Builder>()).begin_brush_face(),
			)
		};
	}

	unsafe extern "C" fn end_brush_face(context: *mut c_void) -> CoreBuilderOperationErrorCode
	{
		return unsafe {
			CoreBuilderOperationErrorCode::from_result(
				(*context.cast::<Builder>()).end_brush_face(),
			)
		};
	}

	unsafe extern "C" fn set_brush_face_plane(
		context: *mut c_void,
		plane: DPlane,
	) -> CoreBuilderOperationErrorCode
	{
		return unsafe {
			CoreBuilderOperationErrorCode::from_result(
				(*context.cast::<Builder>()).set_brush_face_plane(plane),
			)
		};
	}

	unsafe extern "C" fn set_brush_face_material(
		context: *mut c_void,
		material_name: &StringRef,
	) -> CoreBuilderOperationErrorCode
	{
		return unsafe {
			CoreBuilderOperationErrorCode::from_result(
				(*context.cast::<Builder>()).set_brush_face_material(material_name.to_string()),
			)
		};
	}

	unsafe extern "C" fn set_brush_face_material_axes(
		context: *mut c_void,
		u_unit_axis: DVec3,
		v_unit_axis: DVec3,
	) -> CoreBuilderOperationErrorCode
	{
		return unsafe {
			CoreBuilderOperationErrorCode::from_result(
				(*context.cast::<Builder>()).set_brush_face_material_axes(u_unit_axis, v_unit_axis),
			)
		};
	}

	unsafe extern "C" fn set_brush_face_material_translation(
		context: *mut c_void,
		translation: DVec2,
	) -> CoreBuilderOperationErrorCode
	{
		return unsafe {
			CoreBuilderOperationErrorCode::from_result(
				(*context.cast::<Builder>()).set_brush_face_material_translation(translation),
			)
		};
	}

	unsafe extern "C" fn set_brush_face_material_scale(
		context: *mut c_void,
		scale: DVec2,
	) -> CoreBuilderOperationErrorCode
	{
		return unsafe {
			CoreBuilderOperationErrorCode::from_result(
				(*context.cast::<Builder>()).set_brush_face_material_scale(scale),
			)
		};
	}

	unsafe extern "C" fn current_entity_index(context: *const c_void) -> PortableOption<usize>
	{
		return unsafe { (*context.cast::<Builder>()).current_entity_index().into() };
	}

	unsafe extern "C" fn current_brush_index(context: *const c_void) -> PortableOption<usize>
	{
		return unsafe { (*context.cast::<Builder>()).current_brush_index().into() };
	}

	unsafe extern "C" fn current_brush_face_index(context: *const c_void) -> PortableOption<usize>
	{
		return unsafe {
			(*context.cast::<Builder>())
				.current_brush_face_index()
				.into()
		};
	}

	unsafe extern "C" fn num_entities(context: *const c_void) -> usize
	{
		return unsafe { (*context.cast::<Builder>()).num_entities() };
	}

	unsafe extern "C" fn num_current_brushes(context: *const c_void) -> usize
	{
		return unsafe { (*context.cast::<Builder>()).num_current_brushes() };
	}

	unsafe extern "C" fn num_current_brush_faces(context: *const c_void) -> usize
	{
		return unsafe { (*context.cast::<Builder>()).num_current_brush_faces() };
	}
}
