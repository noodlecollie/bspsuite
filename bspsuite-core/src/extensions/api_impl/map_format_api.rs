use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use crate::extensions::ExtensionFileFormatCollection;
use crate::extensions::{
	ApiCollector, FormatCollector, FormatLoader, FormatLoaderEndpoint, FormatSpec,
};
use crate::{CompilerError, CompilerErrorCode};
use anyhow::{Result, anyhow, bail};
use bspextifc::builders::map_source_builder::{BuilderError, Entity};
use bspextifc::map_format_api::{MapFormatApi, MapFormatApiCallbacks, ParseMapFn};
use bspextifc::{
	builders::map_source_builder::MapSourceBuilder, map_format_api::MapFormatApiProvider,
};
use bspffi::types::{XCSlice, XCStr};

pub(crate) type MapFormatImplCollection = ApiCollector<MapFormatDefinition, MapFormatApiEndpoint>;
type ExtensionMapFormatCollection = ExtensionFileFormatCollection<ParseMapFn>;

pub struct MapFormatDefinition
{
	pub file_extensions: Vec<String>,
	pub parse_fn: ParseMapFn,
}

pub struct MapFormatApiEndpoint
{
	ext_name: String,
	inner: Option<MapFormatApiCallbacks>,
	map_formats: HashMap<String, MapFormatDefinition>,
}

struct MapFormatApiImpl<'l>
{
	extension_name: String,
	formats: &'l mut ExtensionMapFormatCollection,
}

impl<'l> MapFormatApiImpl<'l>
{
	pub fn new(extension_name: &str, formats: &'l mut ExtensionMapFormatCollection) -> Self
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

impl MapFormatApiEndpoint
{
	pub fn new(extension_name: String, callbacks: Option<MapFormatApiCallbacks>) -> Self
	{
		return Self {
			ext_name: extension_name,
			inner: callbacks,
			map_formats: HashMap::new(),
		};
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

impl FormatLoaderEndpoint for MapFormatApiEndpoint
{
	fn register_supported_formats(&mut self) -> bool
	{
		self.inner
			.as_ref()
			.map(|callbacks| {
				let mut formats: ExtensionMapFormatCollection =
					ExtensionMapFormatCollection::new(self.ext_name.clone(), "map format".into());

				{
					let mut api_impl: MapFormatApiProvider = MapFormatApiProvider::new(
						MapFormatApiImpl::new(&self.ext_name, &mut formats),
					);

					(callbacks.register_map_formats)(&mut api_impl);
				}

				self.map_formats = formats
					.collect()
					.into_iter()
					.map(|(format_name, parse_fn, file_extensions)| {
						(
							format_name,
							MapFormatDefinition {
								file_extensions,
								parse_fn,
							},
						)
					})
					.collect();

				true
			})
			.unwrap_or(false)
	}
}

impl FormatLoader<MapFormatDefinition> for MapFormatApiEndpoint
{
	type LoaderOutput = Vec<Entity>;

	fn extension_name(&self) -> &str
	{
		return &self.ext_name;
	}

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

impl MapFormatImplCollection
{
	pub fn parse_map(
		&self,
		map_path: &Path,
		input_data: &str,
		allowed_formats: &Option<&[&str]>,
		map_format_override: &Option<&str>,
	) -> Result<Vec<Entity>, CompilerError>
	{
		if let Some(override_format) = map_format_override
			&& let Some(formats) = allowed_formats
			&& !formats.contains(override_format)
		{
			return Err(CompilerError::from_anyhow(
				CompilerErrorCode::ArgumentError,
				anyhow!(
					"Map format {override_format} was not contained within list of allowed formats: {}",
					formats.join(", ")
				),
			));
		}

		let (endpoint, use_format): (Rc<MapFormatApiEndpoint>, String) = match map_format_override
		{
			Some(override_format) => (
				self.get_impl_for_format(*override_format).map_err(|err| {
					CompilerError::from_anyhow(CompilerErrorCode::ArgumentError, err)
				})?,
				(*override_format).to_owned(),
			),
			None => self
				.get_impl_with_format_from_file_extension(map_path, allowed_formats)
				.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::ArgumentError, err))?,
		};

		return endpoint
			.load_if_supported(&use_format, |def| Ok(def.parse_map(input_data)?))
			.map_err(|err| {
				CompilerError::from_anyhow(
					CompilerErrorCode::IoError,
					anyhow!("Failed to parse map {}. {err}", map_path.display()),
				)
			});
	}

	fn get_impl_for_format(&self, format: &str) -> Result<Rc<MapFormatApiEndpoint>>
	{
		let implementers: Vec<Rc<MapFormatApiEndpoint>> = self.endpoints_supporting_format(format);

		if implementers.len() != 1
		{
			// TODO: Support better disambiguation in this case.
			if implementers.len() > 1
			{
				let matches_str: String = implementers
					.iter()
					.map(|endpoint| endpoint.extension_name())
					.collect::<Vec<&str>>()
					.join(", ");

				bail!(
					"Map format {format} supported by more than compiler extension: \
					{matches_str}. Unable to deduce which one to use."
				);
			}
			else
			{
				let indent: &'static str = "    ";

				bail!(
					"No compiler extensions supported map format {format}.\n\
					{indent}Formats supported by compiler: {}",
					self.all_supported_formats().join(", "),
				);
			}
		}

		return Ok(implementers[0].clone());
	}

	fn get_impl_with_format_from_file_extension(
		&self,
		map_path: &Path,
		allowed_formats: &Option<&[&str]>,
	) -> Result<(Rc<MapFormatApiEndpoint>, String)>
	{
		let map_ext: &str = map_path
			.extension()
			.and_then(|ext_str| ext_str.to_str())
			.ok_or_else(|| anyhow!("Failed to deduce extension from map path"))?;

		let implementers: Vec<(Rc<MapFormatApiEndpoint>, String)> =
			self.all_formats_for_file_extension(map_ext, allowed_formats);

		if implementers.len() != 1
		{
			// TODO: Support better disambiguation in this case.
			if implementers.len() > 1
			{
				let matches_str: String = implementers
					.iter()
					.map(|(endpoint, format)| {
						format!("{} (format {format})", endpoint.extension_name(),)
					})
					.collect::<Vec<String>>()
					.join(", ");

				bail!(
					"Input map file extension .{map_ext} supported by more than one compiler extension: \
						{matches_str}. Unable to deduce which one to use."
				);
			}
			else
			{
				let indent: &'static str = "    ";

				bail!(
					"No compiler extensions recognised input map file with extension .{map_ext}\n\
						{indent}Formats allowed for game: {}\n\
						{indent}Formats supported by compiler: {}",
					allowed_formats
						.map(|list| list.join(", "))
						.unwrap_or("any".to_owned()),
					self.all_supported_format_extensions_desc()
				);
			}
		}

		return Ok(implementers[0].clone());
	}
}
