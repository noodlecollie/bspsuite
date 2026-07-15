use crate::ApiInfo;
use bspffi::types::{XCBytes, XCOption, XCSlice, XCStr};
use thin_trait_object::thin_trait_object;

pub const API_INFO: ApiInfo = ApiInfo::new("VfsApi", 1);

pub type RegisterVfsSupportFn = extern "C" fn(&mut BoxedVfsApi);

/// Callbacks implemented by this extension when requesting access to the VFS
/// API.
#[repr(C)]
pub struct VfsApiCallbacks
{
	/// Extension function to be called when the compiler is querying for
	/// supported VFSes. The extension should use the provided BoxedVfsApi
	/// interface to tell the compiler about the VFSes it supports.
	pub register_vfs_support: RegisterVfsSupportFn,
}

/// API used to register an extension's support for a VFS.
#[thin_trait_object(drop_abi = "C")]
pub trait VfsApi
{
	/// Registers support for a FVS under a given name. This name may be used in
	/// a game config to request that resources be loaded through this VFS.
	/// If the VFS root should be a file with a particular extension, the
	/// supported extensions should be specified in the file_extensions
	/// argument. If the VFS root should be a directory on disk, this argument
	/// should be set to None.
	fn register_vfs(
		&mut self,
		name: &XCStr,
		file_extensions: &XCOption<XCSlice<XCStr>>,
		callbacks: VfsImplCallbacks,
	);
}

/// Set of functions that an extension must implement for a VFS.
#[repr(C)]
pub struct VfsImplCallbacks
{
	/// Called when the VFS is first initialised. real_root_node is the path to
	/// a file or directory on the physical disk that should serve as the root
	/// of the VFS. The VFS is expected to persist until the extension library
	/// is unloaded.
	pub initialise: extern "C" fn(real_root_node: &XCStr) -> VfsInitResultCode,

	/// Returns true if a file or directory at the given path exists, or false
	/// otherwise.
	pub exists: extern "C" fn(path: &XCStr) -> bool,

	/// Returns true if there is a file at the given path, or false otherwise.
	/// Importantly, this should return false for directories.
	pub is_file: extern "C" fn(path: &XCStr) -> bool,

	/// Returns true if there is a directory at the given path, or false
	/// otherwise. Importantly, this should return false for files.
	pub is_directory: extern "C" fn(path: &XCStr) -> bool,

	/// Provides stats about the file or directory at the given path. The stats
	/// should be submitted to the recipient, or an error code provided if the
	/// operation fails.
	pub stat: extern "C" fn(path: &XCStr, recipient: &mut BoxedVfsStatRecipient),

	/// Loads all of the data from the file at the given path. The data should
	/// be submitted to the recipient, or an error code provided if the
	/// operation fails.
	pub load_file: extern "C" fn(path: &XCStr, recipient: &mut BoxedVfsFileRecipient),
}

#[repr(C)]
pub enum VfsInitResultCode
{
	Ok,
	InternalError,
	InvalidRootPath,
	RootAlreadyInUse,
}

#[repr(C)]
pub enum VfsFileErrorCode
{
	InternalError,
	InvalidPath,
}

#[repr(C)]
pub enum VfsFileRecipientResult
{
	Ok,
	AlreadyHasResult,
}

// TODO: Docs
#[thin_trait_object(drop_abi = "C")]
pub trait VfsFileRecipient
{
	fn submit_bytes(&mut self, bytes: &XCBytes) -> VfsFileRecipientResult;
	fn set_error(&mut self, code: VfsFileErrorCode);
}

/// Struct holding details about a file or directory queried through stat.
#[repr(C)]
pub struct VfsFileStats<'l>
{
	/// Path to the parent node in the filesystem, or an empty string if these
	/// stats represent the root.
	parent_path: XCStr<'l>,

	/// The name of the file or directory being queried.
	/// `"{parent_path}/{name}"` represents the item's entire path.
	name: XCStr<'l>,

	/// True if the item is a directory, false if it's a file.
	is_directory: bool,

	/// The size of the file in bytes, or 0 if the item is a directory.
	file_size: usize,
}

// TODO: Docs
#[thin_trait_object(drop_abi = "C")]
pub trait VfsStatRecipient
{
	fn submit_stats(&mut self, stats: &VfsFileStats) -> VfsFileRecipientResult;
	fn set_error(&mut self, code: VfsFileErrorCode);
}
