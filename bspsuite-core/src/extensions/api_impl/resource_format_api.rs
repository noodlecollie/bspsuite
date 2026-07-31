use std::collections::HashMap;

use crate::extensions::{FileFormatList, FormatLoaderApi};
use crate::extensions::{FormatLoader, FormatSpec};
use anyhow::anyhow;
use bspextifc::resource_format_api::{
	BoxedResourceFormatApi, LoadImageFn, ResourceFormatApi, ResourceFormatApiCallbacks,
};
use bspffi::types::{XCSlice, XCStr};

pub struct ImageFormatDefinition
{
	pub file_extensions: Vec<String>,
	pub load_fn: LoadImageFn,
}

struct ResourceFormatsCollector
{
	image_formats: FileFormatList<LoadImageFn>,
}

struct ResourceFormatApiImpl<'l>
{
	extension_name: String,
	formats: &'l mut ResourceFormatsCollector,
}

pub struct Endpoint
{
	inner: Option<ResourceFormatApiCallbacks>,
	image_formats: HashMap<String, ImageFormatDefinition>,
}

impl<'l> ResourceFormatApiImpl<'l>
{
	pub fn new(extension_name: &str, formats: &'l mut ResourceFormatsCollector) -> Self
	{
		return Self {
			extension_name: extension_name.into(),
			formats,
		};
	}
}

impl ResourceFormatsCollector
{
	pub fn new(extension_name: &str) -> Self
	{
		return Self {
			image_formats: FileFormatList::new(extension_name.into(), "image format".into()),
		};
	}
}

impl<'l> ResourceFormatApi for ResourceFormatApiImpl<'l>
{
	fn register_image_format(
		&mut self,
		format_name: &XCStr,
		file_extensions: &XCSlice<XCStr>,
		load_fn: LoadImageFn,
	)
	{
		self.formats.image_formats.add(
			format_name.as_str(),
			file_extensions.as_slice(),
			load_fn,
			false,
		);
	}
}

impl Endpoint
{
	pub fn new(callbacks: Option<ResourceFormatApiCallbacks>) -> Self
	{
		return Self {
			inner: callbacks,
			image_formats: HashMap::new(),
		};
	}

	pub fn supports_image_format(&self, format_name: &str) -> bool
	{
		return self.image_formats.contains_key(format_name);
	}

	pub fn get_image_format_definition(&self, format_name: &str) -> Option<&ImageFormatDefinition>
	{
		return self.image_formats.get(format_name);
	}

	pub fn supports_image_format_with_file_extension(
		&self,
		format_name: &str,
		file_extension: &str,
	) -> bool
	{
		return self
			.get_image_format_definition(format_name)
			.map(|def| def.file_extensions.contains(&file_extension.to_owned()))
			.unwrap_or(false);
	}

	pub fn get_image_format_definition_supported_file_extensions(
		&self,
		format_name: &str,
	) -> Option<&Vec<String>>
	{
		return self
			.get_image_format_definition(format_name)
			.map(|def| &def.file_extensions);
	}

	pub fn get_supported_image_formats(&self) -> Vec<String>
	{
		return self.image_formats.keys().map(|item| item.clone()).collect();
	}

	pub fn get_supported_image_format_defs(&self) -> Vec<(&str, &ImageFormatDefinition)>
	{
		return self
			.image_formats
			.iter()
			.map(|(key, val)| (key.as_str(), val))
			.collect();
	}
}

impl FormatLoaderApi for Endpoint
{
	fn register_supported_formats(&mut self, extension_name: &str) -> bool
	{
		self.inner
			.as_ref()
			.map(|callbacks| {
				let mut formats: ResourceFormatsCollector =
					ResourceFormatsCollector::new(extension_name.into());

				{
					let mut api_impl: BoxedResourceFormatApi = BoxedResourceFormatApi::new(
						ResourceFormatApiImpl::new(extension_name, &mut formats),
					);

					(callbacks.register_resource_formats)(&mut api_impl);
				}

				self.image_formats = formats
					.image_formats
					.collect()
					.into_iter()
					.map(|(key, value)| {
						(
							key,
							ImageFormatDefinition {
								file_extensions: value.1,
								load_fn: value.0,
							},
						)
					})
					.collect();
				true
			})
			.unwrap_or(false)
	}
}

impl FormatLoader<LoadImageFn> for Endpoint
{
	type LoaderOutput = ();

	fn supported_formats(&self) -> Vec<FormatSpec>
	{
		return self
			.image_formats
			.iter()
			.map(|(key, value)| FormatSpec {
				format_name: key.to_owned(),
				associated_file_extensions: value.file_extensions.clone(),
			})
			.collect();
	}

	fn supports_loading_format(&self, format_name: &str) -> bool
	{
		return self.image_formats.contains_key(format_name);
	}

	fn supports_loading_format_from_file(&self, format_name: &str, file_extension: &str) -> bool
	{
		return self.image_formats.get(format_name).map_or(false, |def| {
			def.file_extensions.contains(&file_extension.to_owned())
		});
	}

	fn supported_file_extensions_for_format(&self, format_name: &str) -> Option<Vec<&str>>
	{
		return self
			.image_formats
			.get(format_name)
			.map(|def| def.file_extensions.iter().map(|str| str.as_str()).collect());
	}

	fn load_if_supported<Callback>(
		&self,
		format_name: &str,
		callback: Callback,
	) -> anyhow::Result<Self::LoaderOutput>
	where
		Callback: Fn(&LoadImageFn) -> anyhow::Result<Self::LoaderOutput>,
	{
		let def: &ImageFormatDefinition = self
			.image_formats
			.get(format_name)
			.ok_or_else(|| anyhow!(format!("Image format {format_name} is not supported")))?;

		Ok((callback)(&def.load_fn)?)
	}
}
