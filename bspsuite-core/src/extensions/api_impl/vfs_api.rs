use std::collections::HashMap;
use std::path::Path;

use crate::extensions::{ApiCollector, ExtensionFileFormatCollection, FormatLoaderEndpoint};
use crate::extensions::{FormatLoader, FormatSpec};
use anyhow::{Result, anyhow, bail, ensure};
use bspextifc::vfs_api::{
	VfsApi, VfsApiCallbacks, VfsApiProvider, VfsFileErrorCode, VfsFileStats, VfsImplCallbacks,
	VfsInitResultCode, VfsStatRecipient, VfsStatRecipientProvider,
};
use bspffi::types::{XCOption, XCSlice, XCStr};
use log::warn;

pub(crate) type VfsFormatImplCollection = ApiCollector<VfsInitialiser, VfsApiEndpoint>;

pub(crate) struct VfsFileStatResult
{
	pub parent_path: String,
	pub name: String,
	pub is_directory: bool,
	pub file_size: usize,
}

type VfsInitialiser = extern "C" fn(real_root_node: &XCStr) -> VfsInitResultCode;

struct VfsInstance
{
	callbacks: VfsImplCallbacks,
	initialised: bool,
}

impl VfsInstance
{
	pub fn new(callbacks: VfsImplCallbacks) -> Self
	{
		return Self {
			callbacks,
			initialised: false,
		};
	}

	pub fn initialise(&mut self, root: &Path) -> Result<()>
	{
		ensure!(!self.initialised, "VFS was already initialised");

		let path_str: &str = root
			.to_str()
			.ok_or_else(|| anyhow!("Failed to convert path to str"))?;
		(self.callbacks.initialise)(&path_str.into());
		self.initialised = true;

		Ok(())
	}
}

struct VfsApiImpl<'l>
{
	extension_name: String,
	vfs_impls: &'l mut ExtensionFileFormatCollection<VfsImplCallbacks>,
}

impl<'l> VfsApiImpl<'l>
{
	pub fn new(
		extension_name: &str,
		vfs_impls: &'l mut ExtensionFileFormatCollection<VfsImplCallbacks>,
	) -> Self
	{
		return Self {
			extension_name: extension_name.into(),
			vfs_impls,
		};
	}
}

impl<'l> VfsApi for VfsApiImpl<'l>
{
	fn register_vfs(
		&mut self,
		name: &XCStr,
		file_extensions: &XCOption<XCSlice<XCStr>>,
		callbacks: VfsImplCallbacks,
	)
	{
		let file_extensions: &[XCStr] = file_extensions
			.as_ref_option()
			.map_or(&[], |xc_slice| xc_slice.as_slice());

		self.vfs_impls
			.add(name.as_str(), file_extensions, callbacks, true);
	}
}

enum VfsFileStatResultWrapper
{
	Ok(VfsFileStatResult),
	Err(VfsFileErrorCode),
}

struct VfsStatRecipientImpl<'l>
{
	result_wrapper: &'l mut Option<VfsFileStatResultWrapper>,
}

impl<'l> VfsStatRecipientImpl<'l>
{
	pub fn new(result_wrapper: &'l mut Option<VfsFileStatResultWrapper>) -> Self
	{
		return Self { result_wrapper };
	}
}

impl<'l> VfsStatRecipient for VfsStatRecipientImpl<'l>
{
	fn submit_stats(&mut self, stats: &VfsFileStats)
	{
		*self.result_wrapper = Some(VfsFileStatResultWrapper::Ok(stats.into()));
	}

	fn set_error(&mut self, code: VfsFileErrorCode)
	{
		*self.result_wrapper = Some(VfsFileStatResultWrapper::Err(code));
	}
}

impl From<&VfsFileStats<'_>> for VfsFileStatResult
{
	fn from(value: &VfsFileStats) -> Self
	{
		return Self {
			parent_path: value.parent_path.to_string(),
			name: value.name.to_string(),
			is_directory: value.is_directory,
			file_size: value.file_size,
		};
	}
}

pub struct VfsApiEndpoint
{
	ext_name: String,
	inner: Option<VfsApiCallbacks>,
	vfs_impls: HashMap<String, (VfsImplCallbacks, Vec<String>)>,
}

impl VfsApiEndpoint
{
	pub fn new(extension_name: String, callbacks: Option<VfsApiCallbacks>) -> Self
	{
		return Self {
			ext_name: extension_name,
			inner: callbacks,
			vfs_impls: HashMap::new(),
		};
	}

	pub fn get_registered_vfs_records(&self) -> Vec<(&str, &Vec<String>)>
	{
		return self
			.vfs_impls
			.iter()
			.map(|(key, value)| (key.as_str(), &value.1))
			.collect();
	}

	pub fn get_supported_vfs_types(&self) -> Vec<&str>
	{
		return self.vfs_impls.keys().map(|key| key.as_str()).collect();
	}

	pub fn get_vfs_root_file_extensions(&self, vfs_type: &str) -> Option<&Vec<String>>
	{
		return self.vfs_impls.get(vfs_type).map(|item| &item.1);
	}

	pub fn stat(&self, sub_path: &str) -> Option<VfsFileStatResult>
	{
		for vfs_impl in self.vfs_impls.iter()
		{
			let mut result_wrapper: Option<VfsFileStatResultWrapper> = None;

			{
				let mut recipient: VfsStatRecipientProvider =
					VfsStatRecipientProvider::new(VfsStatRecipientImpl::new(&mut result_wrapper));

				(vfs_impl.1.0.stat)(&XCStr::from(sub_path), &mut recipient);
			}

			match result_wrapper
			{
				None =>
				{
					warn!(
						"Extension {} VFS impl {} did not provide a stat result for {sub_path} - \
						this is an implementation error",
						self.ext_name, vfs_impl.0
					);

					continue;
				}
				Some(val) => match val
				{
					VfsFileStatResultWrapper::Ok(stat_result) => return Some(stat_result),
					VfsFileStatResultWrapper::Err(_) => continue,
				},
			}
		}

		return None;
	}
}

impl FormatLoaderEndpoint for VfsApiEndpoint
{
	fn register_supported_formats(&mut self) -> bool
	{
		self.inner
			.as_ref()
			.map(|callbacks| {
				let mut impls: ExtensionFileFormatCollection<VfsImplCallbacks> =
					ExtensionFileFormatCollection::new(self.ext_name.clone(), "VFS type".into());

				{
					let mut api_impl: VfsApiProvider =
						VfsApiProvider::new(VfsApiImpl::new(&self.ext_name, &mut impls));

					(callbacks.register_vfs_support)(&mut api_impl);
				}

				self.vfs_impls = impls.collect();
				true
			})
			.unwrap_or(false)
	}
}

// The "format" here is the package type that may be used as a VFS root.
impl FormatLoader<VfsInitialiser> for VfsApiEndpoint
{
	type LoaderOutput = VfsInitResultCode;

	fn extension_name(&self) -> &str
	{
		return &self.ext_name;
	}

	fn supported_formats(&self) -> Vec<FormatSpec>
	{
		return self
			.vfs_impls
			.iter()
			.map(|(key, value)| FormatSpec {
				format_name: key.clone(),
				associated_file_extensions: value.1.clone(),
			})
			.collect();
	}

	fn supports_loading_format(&self, format_name: &str) -> bool
	{
		return self.vfs_impls.contains_key(format_name);
	}

	fn supports_loading_format_from_file(&self, format_name: &str, file_extension: &str) -> bool
	{
		return self
			.vfs_impls
			.get(format_name)
			.map_or(false, |vfs| vfs.1.contains(&file_extension.to_owned()));
	}

	fn supported_file_extensions_for_format(&self, format_name: &str) -> Option<Vec<&str>>
	{
		return self
			.vfs_impls
			.get(format_name)
			.map(|vfs| vfs.1.iter().map(|str| str.as_str()).collect());
	}

	fn load_if_supported<Callback>(
		&self,
		format_name: &str,
		callback: Callback,
	) -> anyhow::Result<Self::LoaderOutput>
	where
		Callback: Fn(&VfsInitialiser) -> anyhow::Result<Self::LoaderOutput>,
	{
		let vfs = &self
			.vfs_impls
			.get(format_name)
			.ok_or_else(|| anyhow!(format!("VFS format {format_name} is not supported")))?;

		Ok((callback)(&vfs.0.initialise)?)
	}
}

impl VfsFormatImplCollection
{
	pub fn stat(&self, sub_path: &str) -> Result<VfsFileStatResult>
	{
		for endpoint in self.endpoints.iter()
		{
			let result: Option<VfsFileStatResult> = endpoint.stat(sub_path);

			if result.is_some()
			{
				return Ok(result.unwrap());
			}
		}

		bail!("{sub_path} was not found");
	}
}
