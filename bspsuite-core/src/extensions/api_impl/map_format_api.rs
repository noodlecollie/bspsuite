use std::cell::RefCell;
use std::collections::HashMap;

use bspextifc::builders::map_blueprint_builder::MapBlueprintBuilder;
use bspextifc::map_format_api;
use bspextifc::map_format_api::internal::{
	ApiFfiTable, BuilderFfiTable, create_map_blueprint_builder_api, create_map_format_api,
};
use bspffi::types::{XCSlice, XCStr};
use itertools::Itertools;
use log::{debug, warn};

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
	inner: map_format_api::MapFormatApiCallbacks,
	map_formats: HashMap<String, MapFormatDefinition>,
}

impl Endpoint
{
	pub fn new(callbacks: map_format_api::MapFormatApiCallbacks) -> Self
	{
		return Self {
			inner: callbacks,
			map_formats: HashMap::new(),
		};
	}

	pub fn register_map_formats(&mut self, extension_name: &str)
	{
		let api_impl: RefCell<ApiImpl> = RefCell::new(ApiImpl::new(extension_name));
		let ffi_table: ApiFfiTable = ffi_impl::create_api_ffi_table(&api_impl);
		let mut api: map_format_api::MapFormatApi = create_map_format_api(ffi_table);

		(self.inner.register_map_formats)(&mut api);
		self.map_formats = api_impl.into_inner().finish();
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

struct ApiImpl
{
	extension_name: String,
	formats: HashMap<String, MapFormatDefinition>,
}

impl ApiImpl
{
	pub fn new(extension_name: &str) -> Self
	{
		return Self {
			extension_name: extension_name.to_owned(),
			formats: HashMap::new(),
		};
	}

	pub fn register_map_format(
		&mut self,
		format_name: &str,
		file_extensions: &[XCStr],
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

impl MapFormatDefinition
{
	pub fn parse_map(&self, data: &str) -> MapBlueprintBuilder
	{
		let builder_impl: RefCell<MapBlueprintBuilder> = RefCell::new(MapBlueprintBuilder::new());
		let ffi_table: BuilderFfiTable = ffi_impl::create_builder_ffi_table(&builder_impl);
		let mut builder_api: map_format_api::MapBlueprintBuilderApi =
			create_map_blueprint_builder_api(ffi_table);

		(self.parse_fn.parse_fn)(&XCStr::new(data), &mut builder_api);
		return builder_impl.into_inner();
	}
}

mod ffi_impl
{
	use super::*;
	use crate::extensions::api_impl::{LinkOpaqueToImpl, link_opaque_to_impl};
	use bspextifc::builders::map_blueprint_builder::{
		IMapBlueprintBuilder, MapBlueprintBuilder as BuilderImpl,
	};
	use bspextifc::map_format_api::internal::{
		ApiCtx, ApiFfiTable, ApiOpaqueContext, BuilderCtx, BuilderErrorCode, BuilderFfiTable,
		BuilderOpaqueContext,
	};
	use bspextifc::types::{DPlane, DVec2, DVec3};
	use bspffi::types::XCOption;
	use bspffi::types::internal::ContextPtr;

	link_opaque_to_impl!(ApiOpaqueContext, ApiImpl);
	link_opaque_to_impl!(BuilderOpaqueContext, BuilderImpl);

	pub(super) fn create_api_ffi_table<'l>(api_impl: &'l RefCell<ApiImpl>) -> ApiFfiTable<'l>
	{
		return ApiFfiTable {
			context: ApiCtx::new_context(api_impl),
			register_map_format_fn: register_map_format,
		};
	}

	pub(super) fn create_builder_ffi_table<'l>(
		api_impl: &'l RefCell<BuilderImpl>,
	) -> BuilderFfiTable<'l>
	{
		return BuilderFfiTable {
			context: BuilderCtx::new_context(api_impl),
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
		};
	}

	unsafe extern "C" fn register_map_format(
		context: &mut ApiCtx,
		format_name: &XCStr,
		file_extensions: &XCSlice<XCStr>,
		parse_fn: map_format_api::MapParseFn,
	)
	{
		context.to_impl().borrow_mut().register_map_format(
			format_name.to_string().as_ref(),
			file_extensions.as_slice(),
			parse_fn,
		);
	}

	unsafe extern "C" fn set_failure(context: &mut BuilderCtx, description: &XCStr)
	{
		context
			.to_impl()
			.borrow_mut()
			.set_failure(description.to_string());
	}

	unsafe extern "C" fn set_failure_with_location(
		context: &mut BuilderCtx,
		line: usize,
		column: usize,
		description: &XCStr,
	)
	{
		context.to_impl().borrow_mut().set_failure_with_location(
			line,
			column,
			description.to_string(),
		);
	}

	unsafe extern "C" fn begin_entity(context: &mut BuilderCtx) -> BuilderErrorCode
	{
		return context.to_impl().borrow_mut().begin_entity().into();
	}

	unsafe extern "C" fn end_entity(context: &mut BuilderCtx) -> BuilderErrorCode
	{
		return context.to_impl().borrow_mut().end_entity().into();
	}

	unsafe extern "C" fn add_entity_keyvalue(
		context: &mut BuilderCtx,
		key: &XCStr,
		value: &XCStr,
	) -> BuilderErrorCode
	{
		return context
			.to_impl()
			.borrow_mut()
			.add_entity_keyvalue(key.to_string(), value.to_string())
			.into();
	}

	unsafe extern "C" fn begin_brush(context: &mut BuilderCtx) -> BuilderErrorCode
	{
		return context.to_impl().borrow_mut().begin_brush().into();
	}

	unsafe extern "C" fn end_brush(context: &mut BuilderCtx) -> BuilderErrorCode
	{
		return context.to_impl().borrow_mut().end_brush().into();
	}

	unsafe extern "C" fn begin_brush_face(context: &mut BuilderCtx) -> BuilderErrorCode
	{
		return context.to_impl().borrow_mut().begin_brush_face().into();
	}

	unsafe extern "C" fn end_brush_face(context: &mut BuilderCtx) -> BuilderErrorCode
	{
		return context.to_impl().borrow_mut().end_brush_face().into();
	}

	unsafe extern "C" fn set_brush_face_plane(
		context: &mut BuilderCtx,
		plane: DPlane,
	) -> BuilderErrorCode
	{
		return context
			.to_impl()
			.borrow_mut()
			.set_brush_face_plane(plane)
			.into();
	}

	unsafe extern "C" fn set_brush_face_material(
		context: &mut BuilderCtx,
		material_name: &XCStr,
	) -> BuilderErrorCode
	{
		return context
			.to_impl()
			.borrow_mut()
			.set_brush_face_material(material_name.to_string())
			.into();
	}

	unsafe extern "C" fn set_brush_face_material_axes(
		context: &mut BuilderCtx,
		u_unit_axis: DVec3,
		v_unit_axis: DVec3,
	) -> BuilderErrorCode
	{
		return context
			.to_impl()
			.borrow_mut()
			.set_brush_face_material_axes(u_unit_axis, v_unit_axis)
			.into();
	}

	unsafe extern "C" fn set_brush_face_material_translation(
		context: &mut BuilderCtx,
		translation: DVec2,
	) -> BuilderErrorCode
	{
		return context
			.to_impl()
			.borrow_mut()
			.set_brush_face_material_translation(translation)
			.into();
	}

	unsafe extern "C" fn set_brush_face_material_scale(
		context: &mut BuilderCtx,
		scale: DVec2,
	) -> BuilderErrorCode
	{
		return context
			.to_impl()
			.borrow_mut()
			.set_brush_face_material_scale(scale)
			.into();
	}

	unsafe extern "C" fn current_entity_index(context: &BuilderCtx) -> XCOption<usize>
	{
		return context.to_impl().borrow().current_entity_index().into();
	}

	unsafe extern "C" fn current_brush_index(context: &BuilderCtx) -> XCOption<usize>
	{
		return context.to_impl().borrow().current_brush_index().into();
	}

	unsafe extern "C" fn current_brush_face_index(context: &BuilderCtx) -> XCOption<usize>
	{
		return context.to_impl().borrow().current_brush_face_index().into();
	}

	unsafe extern "C" fn num_entities(context: &BuilderCtx) -> usize
	{
		return context.to_impl().borrow().num_entities();
	}

	unsafe extern "C" fn num_current_brushes(context: &BuilderCtx) -> usize
	{
		return context.to_impl().borrow().num_current_brushes();
	}

	unsafe extern "C" fn num_current_brush_faces(context: &BuilderCtx) -> usize
	{
		return context.to_impl().borrow().num_current_brush_faces();
	}
}
