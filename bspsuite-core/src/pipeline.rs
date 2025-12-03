use crate::extensions::ExtensionRef;
use crate::toolchain::Toolchain;
use anyhow::Result;
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
		return self.wrap_ext_foreach_error("Setting up dummy feature", |ext_ref| {
			let mut ext_mut_ref = ext_ref.get_extension_mut()?;
			let api_endpoints = ext_mut_ref.get_api_endpoints_mut();

			if let Some(dummy_api) = &api_endpoints.dummy_api
			{
				dummy_api.entry_point();
			}

			Ok(())
		});
	}

	fn wrap_ext_foreach_error<F>(self, op_desc: &str, mut f: F) -> Self
	where
		F: FnMut(&ExtensionRef) -> Result<()>,
	{
		self.toolchain.extensions().iter().for_each(|ext_ref| {
			if let Err(err) = f(ext_ref)
			{
				warn!("{op_desc} failed. {err}");
			}
		});

		return self;
	}
}
