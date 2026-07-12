use std::collections::HashMap;

use crate::extensions::FileFormatList;
use bspextifc::resource_format_api::{
	BoxedResourceFormatApi, LoadImageFn, ResourceFormatApi, ResourceFormatApiCallbacks,
};
use bspffi::types::{XCSlice, XCStr};

// TODO: Re-evaluate this. It might be better to make a utility newtype that
// wraps a function and prevents it from being copied/cloned, and which is also
// callable.
pub struct ImageLoadCallback
{
	// This callback must be encapsulated, since it's
	// copyable/cloneable and depends on the extension
	// library, but we have no way to codify this dependency!
	// Instead, we treat the callback as being owned
	// by the endpoint, which in turn is owned by the
	// extension. This struct purposefully does not implement Clone.
	load_fn: LoadImageFn,
}

pub struct ImageFormatDefinition
{
	pub file_extensions: Vec<String>,
	pub load_fn: ImageLoadCallback,
}

struct ResourceFormatsCollector
{
	image_formats: FileFormatList<ImageLoadCallback>,
}

struct ResourceFormatApiImpl<'l>
{
	extension_name: String,
	formats: &'l mut ResourceFormatsCollector,
}

pub struct Endpoint
{
	inner: ResourceFormatApiCallbacks,
	image_formats: HashMap<String, ImageFormatDefinition>,
}

impl<'l> ResourceFormatApiImpl<'l>
{
	pub fn new(extension_name: &str, formats: &'l mut ResourceFormatsCollector) -> Self
	{
		return Self {
			extension_name: extension_name.to_owned(),
			formats,
		};
	}
}

impl ResourceFormatsCollector
{
	pub fn new(extension_name: &str) -> Self
	{
		return Self {
			image_formats: FileFormatList::new(extension_name.into(), "image".into()),
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
			ImageLoadCallback { load_fn },
		);
	}
}

impl Endpoint
{
	pub fn new(callbacks: ResourceFormatApiCallbacks) -> Self
	{
		return Self {
			inner: callbacks,
			image_formats: HashMap::new(),
		};
	}

	pub fn register_resource_formats(&mut self, extension_name: &str)
	{
		let mut formats: ResourceFormatsCollector =
			ResourceFormatsCollector::new(extension_name.into());

		{
			let mut api_impl: BoxedResourceFormatApi = BoxedResourceFormatApi::new(
				ResourceFormatApiImpl::new(extension_name, &mut formats),
			);

			(self.inner.register_resource_formats)(&mut api_impl);
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
