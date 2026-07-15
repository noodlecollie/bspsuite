use bspextifc::vfs_api::{BoxedVfsApi, VfsApi, VfsApiCallbacks, VfsImplCallbacks};
use bspffi::types::{XCOption, XCStr};

mod directory_vfs;

pub fn create_callbacks() -> VfsApiCallbacks
{
	return VfsApiCallbacks {
		register_vfs_support: register_vfs_support,
	};
}

pub extern "C" fn register_vfs_support(api: &mut BoxedVfsApi)
{
	api.register_vfs(
		&XCStr::from("directory"),
		&XCOption::None,
		VfsImplCallbacks {
			initialise: directory_vfs::initialise,
			exists: directory_vfs::exists,
			is_file: directory_vfs::is_file,
			is_directory: directory_vfs::is_directory,
			stat: directory_vfs::stat,
			load_file: directory_vfs::load_file,
		},
	);
}
