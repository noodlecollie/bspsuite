use super::opaque_ptr::OpaqueMutPtr;
use bspextifc::map_format_api;
use bspsuite_ffi::types::{XCSlice, XCStr};
use itertools::Itertools;
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

pub struct MapFormatDefinition
{
	pub file_extensions: Vec<String>,
	pub parse_fn: MapParseCallback,
}

pub struct Endpoint
{
	inner: map_format_api::Callbacks,
	map_formats: HashMap<String, MapFormatDefinition>,
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

		let core_fns: map_format_api::internal::ApiFfiTable =
			map_format_api::internal::ApiFfiTable {
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

	pub fn supports_map_format_with_file_extension(
		&self,
		format_name: &str,
		file_extension: &str,
	) -> bool
	{
		return self
			.get_definition(format_name)
			.map(|def| def.file_extensions.contains(&file_extension.to_owned()))
			.unwrap_or(false);
	}

	pub fn get_definition(&self, format_name: &str) -> Option<&MapFormatDefinition>
	{
		return self.map_formats.get(format_name);
	}

	pub fn get_definition_supported_file_extensions(
		&self,
		format_name: &str,
	) -> Option<&Vec<String>>
	{
		return self
			.get_definition(format_name)
			.map(|def| &def.file_extensions);
	}

	pub fn get_supported_map_formats(&self) -> Vec<String>
	{
		return self.map_formats.keys().map(|item| item.clone()).collect();
	}

	pub fn get_supported_map_format_defs(&self) -> Vec<(&str, &MapFormatDefinition)>
	{
		return self
			.map_formats
			.iter()
			.map(|(key, val)| (key.as_str(), val))
			.collect();
	}
}

impl MapParseCallback
{
	pub fn parse(&self, data: &str, builder: &mut map_format_api::MapBlueprintBuilderApi)
	{
		(self.parse_fn)(&XCStr::new(data), builder);
	}
}

struct ApiImpl<'l>
{
	extension_name: &'l str,
	formats: HashMap<String, MapFormatDefinition>,
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

	pub fn register_map_format(
		&mut self,
		format_name: &str,
		file_extensions: &[XCStr], // TODO: Slice of &str?
		parse_fn: map_format_api::MapParseFn,
	)
	{
		if file_extensions.is_empty()
		{
			warn!(
				"Extension {} specified no file extensions for map format {format_name}. \
				This format will be ignored.",
				self.extension_name
			);

			return;
		}

		// We want to do a few things here:
		// - Trim leading and trailing whitespace
		// - Trim leading dots, in case people specify ".map" instead of "map"
		// - Remove any items that end up being empty after these operations
		// - Remove duplicates
		let extension_strings: Vec<String> = file_extensions
			.iter()
			.map(|item| item.as_str().trim().trim_start_matches(".").to_string())
			.filter(|item| !item.is_empty())
			.unique()
			.collect();

		if extension_strings.is_empty()
		{
			warn!(
				"After removing invalid file extensions, extension {} was left with no valid file extensions \
				for map format {format_name}. This format will be ignored.",
				self.extension_name
			);

			return;
		}

		if extension_strings.len() < file_extensions.len()
		{
			warn!(
				"Extension {} provided {} empty, duplicated, or otherwise invalid file extensions for map format \
				{format_name}. These will be ignored.",
				self.extension_name,
				file_extensions.len() - extension_strings.len()
			);
		}

		if let Some(_) = self.formats.insert(
			String::from(format_name),
			MapFormatDefinition {
				file_extensions: extension_strings,
				parse_fn: MapParseCallback { parse_fn: parse_fn },
			},
		)
		{
			warn!(
				"Overriding existing registration for extension {} map format \"{format_name}\"",
				self.extension_name
			);
		}

		if log::max_level() >= log::LevelFilter::Debug
		{
			let all_extensions: String = self
				.formats
				.get(format_name)
				.unwrap()
				.file_extensions
				.join(", ");

			debug!(
				"Extension {} registered support for map format {format_name}, with \
				file extensions: {all_extensions}",
				self.extension_name
			);
		}
	}

	pub fn finish(self) -> HashMap<String, MapFormatDefinition>
	{
		return self.formats;
	}
}

unsafe extern "C" fn register_map_format(
	context: *mut c_void,
	format_name: &XCStr,
	file_extensions: &XCSlice<XCStr>,
	parse_fn: map_format_api::MapParseFn,
)
{
	unsafe {
		(*context.cast::<ApiImpl>()).register_map_format(
			format_name.to_string().as_ref(),
			file_extensions.as_slice(),
			parse_fn,
		);
	};
}

mod builder_extc
{
	use super::*;
	use bspextifc::builders::map_blueprint_builder::{
		IMapBlueprintBuilder, MapBlueprintBuilder as Builder,
	};
	use bspextifc::map_format_api::MapParseFn;
	use bspextifc::map_format_api::internal::BuilderErrorCode;
	use bspextifc::types::{DPlane, DVec2, DVec3};
	use bspsuite_ffi::types::XCOption;

	pub fn parse_map(data: &XCStr, parse_fn: &MapParseFn) -> Builder
	{
		let mut local_builder: Builder = Builder::new();
		let mut context: OpaqueMutPtr<Builder> = OpaqueMutPtr::new(&mut local_builder);

		let mut external_builder =
			bspextifc::map_format_api::internal::create_map_blueprint_builder_api(
				bspextifc::map_format_api::internal::BuilderFfiTable {
					context: context.as_mut_void_ref(),
					set_failure: set_failure,
					set_failure_with_location: set_failure_with_location,
					begin_entity: begin_entity,
					end_entity: end_entity,
					add_entity_keyvalue: add_entity_keyvalue,
					begin_brush: begin_brush,
					end_brush: end_brush,
					begin_brush_face: begin_brush_face,
					end_brush_face: end_brush_face,
					set_brush_face_plane: set_brush_face_plane,
					set_brush_face_material: set_brush_face_material,
					set_brush_face_material_axes: set_brush_face_material_axes,
					set_brush_face_material_translation: set_brush_face_material_translation,
					set_brush_face_material_scale: set_brush_face_material_scale,
					current_entity_index: current_entity_index,
					current_brush_index: current_brush_index,
					current_brush_face_index: current_brush_face_index,
					num_entities: num_entities,
					num_current_brushes: num_current_brushes,
					num_current_brush_faces: num_current_brush_faces,
				},
			);

		parse_fn(data, &mut external_builder);
		return local_builder;
	}

	unsafe extern "C" fn set_failure(context: *mut c_void, description: &XCStr)
	{
		unsafe {
			(*context.cast::<Builder>()).set_failure(description.to_string());
		}
	}

	unsafe extern "C" fn set_failure_with_location(
		context: *mut c_void,
		line: usize,
		column: usize,
		description: &XCStr,
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

	unsafe extern "C" fn begin_entity(context: *mut c_void) -> BuilderErrorCode
	{
		return unsafe { BuilderErrorCode::from((*context.cast::<Builder>()).begin_entity()) };
	}

	unsafe extern "C" fn end_entity(context: *mut c_void) -> BuilderErrorCode
	{
		return unsafe { BuilderErrorCode::from((*context.cast::<Builder>()).end_entity()) };
	}

	unsafe extern "C" fn add_entity_keyvalue(
		context: *mut c_void,
		key: &XCStr,
		value: &XCStr,
	) -> BuilderErrorCode
	{
		return unsafe {
			BuilderErrorCode::from(
				(*context.cast::<Builder>())
					.add_entity_keyvalue(key.to_string(), value.to_string()),
			)
		};
	}

	unsafe extern "C" fn begin_brush(context: *mut c_void) -> BuilderErrorCode
	{
		return unsafe { BuilderErrorCode::from((*context.cast::<Builder>()).begin_brush()) };
	}

	unsafe extern "C" fn end_brush(context: *mut c_void) -> BuilderErrorCode
	{
		return unsafe { BuilderErrorCode::from((*context.cast::<Builder>()).end_brush()) };
	}

	unsafe extern "C" fn begin_brush_face(context: *mut c_void) -> BuilderErrorCode
	{
		return unsafe { BuilderErrorCode::from((*context.cast::<Builder>()).begin_brush_face()) };
	}

	unsafe extern "C" fn end_brush_face(context: *mut c_void) -> BuilderErrorCode
	{
		return unsafe { BuilderErrorCode::from((*context.cast::<Builder>()).end_brush_face()) };
	}

	unsafe extern "C" fn set_brush_face_plane(
		context: *mut c_void,
		plane: DPlane,
	) -> BuilderErrorCode
	{
		return unsafe {
			BuilderErrorCode::from((*context.cast::<Builder>()).set_brush_face_plane(plane))
		};
	}

	unsafe extern "C" fn set_brush_face_material(
		context: *mut c_void,
		material_name: &XCStr,
	) -> BuilderErrorCode
	{
		return unsafe {
			BuilderErrorCode::from(
				(*context.cast::<Builder>()).set_brush_face_material(material_name.to_string()),
			)
		};
	}

	unsafe extern "C" fn set_brush_face_material_axes(
		context: *mut c_void,
		u_unit_axis: DVec3,
		v_unit_axis: DVec3,
	) -> BuilderErrorCode
	{
		return unsafe {
			BuilderErrorCode::from(
				(*context.cast::<Builder>()).set_brush_face_material_axes(u_unit_axis, v_unit_axis),
			)
		};
	}

	unsafe extern "C" fn set_brush_face_material_translation(
		context: *mut c_void,
		translation: DVec2,
	) -> BuilderErrorCode
	{
		return unsafe {
			BuilderErrorCode::from(
				(*context.cast::<Builder>()).set_brush_face_material_translation(translation),
			)
		};
	}

	unsafe extern "C" fn set_brush_face_material_scale(
		context: *mut c_void,
		scale: DVec2,
	) -> BuilderErrorCode
	{
		return unsafe {
			BuilderErrorCode::from(
				(*context.cast::<Builder>()).set_brush_face_material_scale(scale),
			)
		};
	}

	unsafe extern "C" fn current_entity_index(context: *const c_void) -> XCOption<usize>
	{
		return unsafe { (*context.cast::<Builder>()).current_entity_index().into() };
	}

	unsafe extern "C" fn current_brush_index(context: *const c_void) -> XCOption<usize>
	{
		return unsafe { (*context.cast::<Builder>()).current_brush_index().into() };
	}

	unsafe extern "C" fn current_brush_face_index(context: *const c_void) -> XCOption<usize>
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
