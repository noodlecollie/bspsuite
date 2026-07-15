use std::collections::HashMap;
use std::path::Path;

use crate::extensions::FileFormatList;
use crate::extensions::{FormatLoader, FormatSpec};
use anyhow::{Result, anyhow, ensure};
use bspextifc::vfs_api::{BoxedVfsApi, VfsApi, VfsApiCallbacks, VfsImplCallbacks};
use bspffi::types::{XCOption, XCSlice, XCStr};

pub type VfsInitialiser = extern "C" fn(real_root_node: &XCStr);

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
	vfs_impls: &'l mut FileFormatList<VfsImplCallbacks>,
}

impl<'l> VfsApiImpl<'l>
{
	pub fn new(extension_name: &str, vfs_impls: &'l mut FileFormatList<VfsImplCallbacks>) -> Self
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

pub struct Endpoint
{
	inner: VfsApiCallbacks,
	vfs_impls: HashMap<String, (VfsImplCallbacks, Vec<String>)>,
}

impl Endpoint
{
	pub fn new(callbacks: VfsApiCallbacks) -> Self
	{
		return Self {
			inner: callbacks,
			vfs_impls: HashMap::new(),
		};
	}

	pub fn register_vfs_impls(&mut self, extension_name: &str)
	{
		let mut impls: FileFormatList<VfsImplCallbacks> =
			FileFormatList::new(extension_name.into(), "VFS type".into());

		{
			let mut api_impl: BoxedVfsApi =
				BoxedVfsApi::new(VfsApiImpl::new(extension_name, &mut impls));

			(self.inner.register_vfs_support)(&mut api_impl);
		}

		self.vfs_impls = impls.collect();
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
}

// The "format" here is the package type that may be used as a VFS root.
impl FormatLoader<VfsInitialiser> for Endpoint
{
	type LoaderOutput = ();

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
