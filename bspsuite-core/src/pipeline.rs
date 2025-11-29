use crate::extensions::dummy_api::call_dummy_api;
use crate::extensions::map_format_api::register_map_formats;
use crate::toolchain::Toolchain;
use std::path::PathBuf;

pub enum ExtensionFeature
{
	DummyFeature,
	MapFormatFeature,
}

pub struct PipelineBuilder
{
	toolchain: Toolchain,
}

pub struct Pipeline
{
	toolchain: Toolchain,
}

impl PipelineBuilder
{
	pub fn new(toolchain_root: &Option<PathBuf>) -> Self
	{
		return Self {
			toolchain: Toolchain::new(toolchain_root),
		};
	}

	pub fn finalise(self) -> Pipeline
	{
		return Pipeline {
			toolchain: self.toolchain,
		};
	}

	pub fn require_feature(self, feature: ExtensionFeature) -> Self
	{
		return match feature
		{
			ExtensionFeature::DummyFeature => self.set_up_dummy_feature(),
			ExtensionFeature::MapFormatFeature => self.register_map_formats(),
		};
	}

	fn set_up_dummy_feature(self) -> Self
	{
		self.toolchain.extensions().iter().for_each(|extension| {
			if let Some(callbacks) = &extension.get_api_callbacks().dummy_api_callbacks
			{
				call_dummy_api(callbacks.entry_point);
			}
		});

		return self;
	}

	fn register_map_formats(self) -> Self
	{
		self.toolchain.extensions().iter().for_each(|extension| {
			if let Some(callbacks) = &extension.get_api_callbacks().map_format_api_callbacks
			{
				register_map_formats(extension.get_name(), callbacks.register_map_formats);
			}
		});

		return self;
	}
}

impl Pipeline
{
	fn new(toolchain: Toolchain) -> Self
	{
		return Self {
			toolchain: toolchain,
		};
	}
}
