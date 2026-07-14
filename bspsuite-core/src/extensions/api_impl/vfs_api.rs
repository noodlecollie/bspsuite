use std::collections::HashMap;
use std::path::Path;

use crate::extensions::FileFormatList;
use anyhow::{Result, anyhow, ensure};
use bspextifc::vfs_api::{BoxedVfsApi, VfsApi, VfsApiCallbacks, VfsImplCallbacks};
use bspffi::types::{XCOption, XCSlice, XCStr};

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

	pub fn get_supported_vfs_types(&self) -> Vec<String>
	{
		return self.vfs_impls.keys().cloned().collect();
	}

	// An empty list means that the VFS requires a disk directory as its root.
	pub fn get_vfs_root_file_extensions(&self, vfs_type: &str) -> Option<&Vec<String>>
	{
		return self.vfs_impls.get(vfs_type).map(|item| &item.1);
	}
}
