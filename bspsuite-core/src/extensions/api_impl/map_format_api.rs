use std::collections::HashMap;

use crate::extensions::FileFormatList;
use bspextifc::builders::map_source_builder::{BuilderError, Entity};
use bspextifc::map_format_api::{MapFormatApi, MapFormatApiCallbacks, MapParseFn};
use bspextifc::{
	builders::map_source_builder::MapSourceBuilder, map_format_api::BoxedMapFormatApi,
};
use bspffi::types::{XCSlice, XCStr};

struct MapFormatApiImpl<'l>
{
	extension_name: String,
	formats: &'l mut FileFormatList<MapParseCallback>,
}

impl<'l> MapFormatApiImpl<'l>
{
	pub fn new(extension_name: &str, formats: &'l mut FileFormatList<MapParseCallback>) -> Self
	{
		return Self {
			extension_name: extension_name.to_owned(),
			formats,
		};
	}
}

impl<'l> MapFormatApi for MapFormatApiImpl<'l>
{
	fn register_map_format(
		&mut self,
		format_name: &XCStr,
		file_extensions: &XCSlice<XCStr>,
		parse_fn: MapParseFn,
	)
	{
		self.formats.add(
			format_name.as_str(),
			file_extensions.as_slice(),
			MapParseCallback { parse_fn },
			false,
		);
	}
}

pub struct MapParseCallback
{
	// This callback must be encapsulated, since it's
	// copyable/cloneable and depends on the extension
	// library, but we have no way to codify this dependency!
	// Instead, we treat the callback as being owned
	// by the endpoint, which in turn is owned by the
	// extension. This struct purposefully does not implement Clone.
	parse_fn: MapParseFn,
}

pub struct MapFormatDefinition
{
	pub file_extensions: Vec<String>,
	pub parse_fn: MapParseCallback,
}

pub struct Endpoint
{
	inner: MapFormatApiCallbacks,
	map_formats: HashMap<String, MapFormatDefinition>,
}

impl Endpoint
{
	pub fn new(callbacks: MapFormatApiCallbacks) -> Self
	{
		return Self {
			inner: callbacks,
			map_formats: HashMap::new(),
		};
	}

	pub fn register_map_formats(&mut self, extension_name: &str)
	{
		let mut formats: FileFormatList<MapParseCallback> =
			FileFormatList::new(extension_name.into(), "map format".into());

		{
			let mut api_impl: BoxedMapFormatApi =
				BoxedMapFormatApi::new(MapFormatApiImpl::new(extension_name, &mut formats));

			(self.inner.register_map_formats)(&mut api_impl);
		}

		self.map_formats = formats
			.collect()
			.into_iter()
			.map(|(key, value)| {
				(
					key,
					MapFormatDefinition {
						file_extensions: value.1,
						parse_fn: value.0,
					},
				)
			})
			.collect();
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

impl MapFormatDefinition
{
	pub fn parse_map(&self, data: &str) -> Result<Vec<Entity>, BuilderError>
	{
		return MapSourceBuilder::run(|mut builder| {
			(self.parse_fn.parse_fn)(&XCStr::new(data), &mut builder);
		});
	}
}
