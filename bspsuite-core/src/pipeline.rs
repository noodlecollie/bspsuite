use super::map_format_registry::MapFormatRegistry;
use crate::toolchain::Toolchain;
use log::warn;
use std::path::PathBuf;

pub enum ExtensionFeature
{
	DummyFeature,
	MapFormatFeature,
}

pub struct PipelineBuilder
{
	toolchain: Toolchain,
	map_formats: Option<MapFormatRegistry>,
}

pub struct Pipeline
{
	toolchain: Toolchain,
	map_formats: MapFormatRegistry,
}

impl PipelineBuilder
{
	pub fn new(toolchain_root: &Option<PathBuf>) -> Self
	{
		return Self {
			toolchain: Toolchain::new(toolchain_root),
			map_formats: None,
		};
	}

	pub fn finalise(self) -> Pipeline
	{
		return Pipeline {
			toolchain: self.toolchain,
			map_formats: self.map_formats.unwrap_or_else(|| MapFormatRegistry::new()),
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
				callbacks.entry_point();
			}
		});

		return self;
	}

	fn register_map_formats(mut self) -> Self
	{
		let mut registry: MapFormatRegistry = MapFormatRegistry::new();

		self.toolchain.extensions().iter().for_each(|extension| {
			if let Some(callbacks) = &extension.get_api_callbacks().map_format_api_callbacks
			{
				for entry in callbacks.register_map_formats()
				{
					if let Err(err) = registry.insert(entry.format_name, entry.parse_fn)
					{
						warn!("{err}")
					}
				}
			}
		});

		self.map_formats = Some(registry);
		return self;
	}
}
