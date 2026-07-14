use std::collections::HashMap;

use bspextifc::vfs_api::{BoxedVfsApi, VfsApi, VfsApiCallbacks, VfsImplCallbacks};
use bspffi::types::XCStr;
use log::{debug, warn};

struct VfsApiImpl<'l>
{
	extension_name: String,
	vfs_map: &'l mut HashMap<String, VfsImplCallbacks>,
}

impl<'l> VfsApiImpl<'l>
{
	pub fn new(extension_name: &str, vfs_map: &'l mut HashMap<String, VfsImplCallbacks>) -> Self
	{
		return Self {
			extension_name: extension_name.into(),
			vfs_map,
		};
	}
}

impl<'l> VfsApi for VfsApiImpl<'l>
{
	fn register_vfs(&mut self, name: &XCStr, callbacks: VfsImplCallbacks)
	{
		if let Some(_) = self.vfs_map.insert(name.as_str().into(), callbacks)
		{
			warn!(
				"Overriding existing registration for extension {} VFS type \"{}\"",
				self.extension_name,
				name.as_str()
			);
		}

		debug!(
			"Extension {} registered support for VFS type {}",
			self.extension_name,
			name.as_str()
		);
	}
}

pub struct Endpoint
{
	inner: VfsApiCallbacks,
	vfs_impls: HashMap<String, VfsImplCallbacks>,
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
		let mut impls: HashMap<String, VfsImplCallbacks> = HashMap::new();

		{
			let mut api_impl: BoxedVfsApi =
				BoxedVfsApi::new(VfsApiImpl::new(extension_name, &mut impls));

			(self.inner.register_vfs_support)(&mut api_impl);
		}

		self.vfs_impls = impls;
	}

	pub fn get_supported_vfs_types(&self) -> Vec<String>
	{
		return self.vfs_impls.keys().cloned().collect();
	}
}
