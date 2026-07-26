use std::collections::HashMap;

use crate::extensions::FileFormatList;
use crate::extensions::{FormatLoader, FormatSpec};
use anyhow::Result;
use anyhow::anyhow;
use bspextifc::builders::map_source_builder::{BuilderError, Entity};
use bspextifc::map_format_api::{MapFormatApi, MapFormatApiCallbacks, ParseMapFn};
use bspextifc::{
	builders::map_source_builder::MapSourceBuilder, map_format_api::BoxedMapFormatApi,
};
use bspffi::types::{XCSlice, XCStr};

struct MapFormatApiImpl<'l>
{
	extension_name: String,
	formats: &'l mut FileFormatList<ParseMapFn>,
}

impl<'l> MapFormatApiImpl<'l>
{
	pub fn new(extension_name: &str, formats: &'l mut FileFormatList<ParseMapFn>) -> Self
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
		parse_fn: ParseMapFn,
	)
	{
		self.formats.add(
			format_name.as_str(),
			file_extensions.as_slice(),
			parse_fn,
			false,
		);
	}
}

pub struct MapFormatDefinition
{
	pub file_extensions: Vec<String>,
	pub parse_fn: ParseMapFn,
}

pub struct Endpoint
{
	inner: Option<MapFormatApiCallbacks>,
	map_formats: HashMap<String, MapFormatDefinition>,
}

impl Endpoint
{
	pub fn new(callbacks: Option<MapFormatApiCallbacks>) -> Self
	{
		return Self {
			inner: callbacks,
			map_formats: HashMap::new(),
		};
	}

	pub fn register_map_formats(&mut self, extension_name: &str) -> bool
	{
		self.inner
			.as_ref()
			.map(|callbacks| {
				let mut formats: FileFormatList<ParseMapFn> =
					FileFormatList::new(extension_name.into(), "map format".into());

				{
					let mut api_impl: BoxedMapFormatApi =
						BoxedMapFormatApi::new(MapFormatApiImpl::new(extension_name, &mut formats));

					(callbacks.register_map_formats)(&mut api_impl);
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

				true
			})
			.unwrap_or(false)
	}
}

impl MapFormatDefinition
{
	pub fn parse_map(&self, data: &str) -> Result<Vec<Entity>, BuilderError>
	{
		return MapSourceBuilder::run(|mut builder| {
			(self.parse_fn)(&XCStr::new(data), &mut builder);
		});
	}
}

impl FormatLoader<MapFormatDefinition> for Endpoint
{
	type LoaderOutput = Vec<Entity>;

	fn supported_formats(&self) -> Vec<FormatSpec>
	{
		return self
			.map_formats
			.iter()
			.map(|(key, value)| FormatSpec {
				format_name: key.clone(),
				associated_file_extensions: value.file_extensions.clone(),
			})
			.collect();
	}

	fn supports_loading_format(&self, format_name: &str) -> bool
	{
		return self.map_formats.contains_key(format_name);
	}

	fn supports_loading_format_from_file(&self, format_name: &str, file_extension: &str) -> bool
	{
		return self.map_formats.get(format_name).map_or(false, |def| {
			def.file_extensions.contains(&file_extension.to_owned())
		});
	}

	fn supported_file_extensions_for_format(&self, format_name: &str) -> Option<Vec<&str>>
	{
		return self.map_formats.get(format_name).map(|def| {
			def.file_extensions
				.iter()
				.map(|item| item.as_str())
				.collect()
		});
	}

	fn load_if_supported<Callback>(
		&self,
		format_name: &str,
		callback: Callback,
	) -> anyhow::Result<Self::LoaderOutput>
	where
		Callback: Fn(&MapFormatDefinition) -> anyhow::Result<Self::LoaderOutput>,
	{
		let def: &MapFormatDefinition = self
			.map_formats
			.get(format_name)
			.ok_or_else(|| anyhow!(format!("Map format {format_name} is not supported")))?;

		return Ok((callback)(&def)?);
	}
}
