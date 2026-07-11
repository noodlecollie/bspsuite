use std::collections::HashMap;

use bspextifc::builders::map_source_builder::{BuilderError, Entity};
use bspextifc::map_format_api;
use bspextifc::map_format_api::MapFormatApi;
use bspextifc::{
	builders::map_source_builder::MapSourceBuilder, map_format_api::BoxedMapFormatApi,
};
use bspffi::types::{XCSlice, XCStr};
use itertools::Itertools;
use log::{debug, warn};

struct MapFormatApiImpl<'l>
{
	extension_name: String,
	formats: &'l mut HashMap<String, MapFormatDefinition>,
}

impl<'l> MapFormatApiImpl<'l>
{
	pub fn new(extension_name: &str, formats: &'l mut HashMap<String, MapFormatDefinition>)
	-> Self
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
		parse_fn: map_format_api::MapParseFn,
	)
	{
		let format_name: &str = format_name.as_str();
		let file_extensions: &[XCStr] = file_extensions.as_slice();

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
}

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
		let mut formats: HashMap<String, MapFormatDefinition> = HashMap::new();

		{
			let mut api_impl: BoxedMapFormatApi =
				BoxedMapFormatApi::new(MapFormatApiImpl::new(extension_name, &mut formats));

			(self.inner.register_map_formats)(&mut api_impl);
		}

		self.map_formats = formats;
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
	pub fn parse_map(&self, data: &str) -> Result<Vec<Entity>, BuilderError>
	{
		return MapSourceBuilder::run(|mut builder| {
			(self.parse_fn.parse_fn)(&XCStr::new(data), &mut builder);
		});
	}
}
