use crate::toolchain::Toolchain;
use log::warn;
use std::path::PathBuf;

pub enum ExtensionFeature
{
	DummyFeature,
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
		};
	}

	fn set_up_dummy_feature(self) -> Self
	{
		self.toolchain.extensions().iter().for_each(|extension| {
			if let Some(callbacks) = extension
				.get_dummy_api_callbacks()
				.expect("Could not get dummy API callbacks from extension")
			{
				callbacks
					.try_borrow_mut()
					.expect("Could not get reference to dummy API callbacks")
					.entry_point();
			}
		});

		return self;
	}
}
