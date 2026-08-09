use std::fmt::Display;

use crate::ApiInfo;
use bspffi::types::{XCBytes, XCOption, XCSlice, XCStr};
use thin_trait_object::thin_trait_object;

/// Version of the VFS API. This should be passed when registering callbacks
/// with [register_vfs_api_callbacks].
pub const API_INFO: ApiInfo = ApiInfo::new("VfsApi", 1);

/// Extension function to be called when the compiler is querying for
/// supported VFSes. The extension should use the provided boxed [VfsApi]
/// interface to tell the compiler about the VFSes it supports.
pub type RegisterVfsSupportFn = extern "C" fn(&mut VfsApiProvider);

/// Callbacks implemented by this extension when requesting access to the VFS
/// API.
#[repr(C)]
pub struct VfsApiCallbacks
{
	/// Called when the compiler is querying for supported VFSes. See
	/// [RegisterVfsSupportFn].
	pub register_vfs_support: RegisterVfsSupportFn,
}

/// API used to register an extension's support for a VFS.
#[thin_trait_object(drop_abi = "C", trait_object(pub VfsApiProvider))]
pub trait VfsApi
{
	/// Registers support for a VFS under a given `name`. This name may be used
	/// in a game config to request that resources be loaded through this VFS.
	///
	/// If the VFS root should be a file with a particular extension, the
	/// supported extensions should be specified in the `file_extensions`
	/// argument. If the VFS root should be a directory on disk, this argument
	/// should be set to None.
	///
	/// The provided `callbacks` will be called when the compiler needs to
	/// interact with the VFS.
	fn register_vfs(
		&mut self,
		name: &XCStr,
		file_extensions: &XCOption<XCSlice<XCStr>>,
		callbacks: VfsImplCallbacks,
	);
}

/// Set of functions that an extension must implement for a VFS.
///
/// Apart from the [initialise] function, all paths provided to callback
/// functions in this struct are expected to be virtual, ie. to refer to files
/// within the VFS rather than to files on the physical disk. Paths should use a
/// forward slash (`/`) as a delimiter, and by convention should not begin with
/// a slash.
#[repr(C)]
pub struct VfsImplCallbacks
{
	/// Called when a VFS instance is first initialised. `real_root_node` is the
	/// path to a file or directory on the physical disk that should serve as
	/// the root of the VFS. The initialised VFS is expected to persist until
	/// the extension library is unloaded.
	///
	/// This function may be called multiple times if multiple roots are
	/// encountered on disk. VFS instances created later are expected to take
	/// precedence over VFS instances created earlier. This means that if two
	/// VFS instances contain a file with the same path, the file from the later
	/// instance should take precedence over that from the earlier instance.
	/// This behaviour can be thought of as analogous to a file system overlay.
	pub initialise: extern "C" fn(real_root_node: &XCStr) -> VfsInitResultCode,

	/// Returns true if a file or directory at the given `path` exists, or false
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
	pub stat: extern "C" fn(path: &XCStr, recipient: &mut VfsStatRecipientProvider),

	/// Loads all of the data from the file at the given path. The data should
	/// be submitted to the recipient, or an error code provided if the
	/// operation fails.
	pub load_file: extern "C" fn(path: &XCStr, recipient: &mut VfsFileRecipientProvider),
}

/// Code representing the result of initialising a VFS.
#[repr(C)]
#[derive(Debug)]
pub enum VfsInitResultCode
{
	/// Initialised OK.
	Ok,

	/// An unspecified internal error occurred.
	InternalError,

	/// The path to the VFS root on disk was not valid.
	InvalidRootPath,

	/// The provided VFS root has already been submitted before.
	RootAlreadyInUse,

	/// The provided VFS root path overlaps with another root that is already in
	/// use.
	RootsOverlap,
}

impl VfsInitResultCode
{
	fn as_str(&self) -> &str
	{
		return match self
		{
			VfsInitResultCode::Ok => "Initialised OK",
			VfsInitResultCode::InternalError => "An internal error occurred",
			VfsInitResultCode::InvalidRootPath => "The provided root path was not valid",
			VfsInitResultCode::RootAlreadyInUse =>
			{
				"The provided root path was already submitted for a previous VFS"
			}
			VfsInitResultCode::RootsOverlap =>
			{
				"The provided root path overlapped with the root of a previous VFS"
			}
		};
	}
}

impl Display for VfsInitResultCode
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
	{
		write!(f, "{}", self.as_str())
	}
}

/// Error code produced when interacting with a VFS file.
#[repr(C)]
#[derive(Debug)]
pub enum VfsFileErrorCode
{
	/// An unspecified internal error occurred.
	InternalError,

	/// The path to the file was not valid.
	InvalidPath,

	/// An error occured with the underlying IO system.
	IoError,
}

impl VfsFileErrorCode
{
	fn as_str(&self) -> &str
	{
		return match self
		{
			VfsFileErrorCode::InternalError => "An internal error occurred",
			VfsFileErrorCode::InvalidPath => "The provided path was not valid",
			VfsFileErrorCode::IoError => "An I/O error occurred when accessing the path",
		};
	}
}

impl Display for VfsFileErrorCode
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
	{
		write!(f, "{}", self.as_str())
	}
}

/// Interface that receives the contents of a file loaded from the VFS.
///
/// Since returning a `Result<Vec<u8>>` is not FFI-safe, this FFI interface
/// serves as the recipient of the loaded bytes. Call
/// [VfsFileRecipient::submit_bytes] if the file was loaded successfully, or
/// call [VfsFileRecipient::set_error] to provide an appropriate code
/// if an error occurs.
#[thin_trait_object(drop_abi = "C", trait_object(pub VfsFileRecipientProvider))]
pub trait VfsFileRecipient
{
	/// Receives the `bytes` loaded from the file. The extension retains
	/// ownership of the data; if the compiler wants to keep and use the data,
	/// it will make a copy.
	///
	/// If [VfsFileRecipient::set_error] was called earlier, calling this
	/// function will clear the error.
	fn submit_bytes(&mut self, bytes: &XCBytes);

	/// Marks this file loading request as having failed, with the provided
	/// `code`.
	///
	/// If [VfsFileRecipient::submit_bytes] was called earlier, calling this
	/// function will invalidate the data and the error will take precedence.
	fn set_error(&mut self, code: VfsFileErrorCode);
}

/// Struct holding details about a file or directory queried through stat.
#[repr(C)]
pub struct VfsFileStats<'l>
{
	/// Path to the parent node in the filesystem, or an empty string if these
	/// stats represent the root. This path should not begin with a slash.
	pub parent_path: XCStr<'l>,

	/// The name of the file or directory being queried.
	/// `"{parent_path}/{name}"` represents the item's entire path.
	pub name: XCStr<'l>,

	/// True if the item is a directory, false if it's a file.
	pub is_directory: bool,

	/// The size of the file in bytes, or 0 if the item is a directory.
	pub file_size: usize,
}

/// Interface that receives the results of a [VfsImplCallbacks::stat] call.
///
/// Since returning a `Result<VfsFileStats>` is not FFI-safe, and the
/// [VfsFileStats] struct contains references to data it does not own, this FFI
/// interface serves as the recipient of the stat results. Call
/// [VfsStatRecipient::submit_stats] if the stat call completed successfully, or
/// call [VfsStatRecipient::set_error] to provide an appropriate code
/// if an error occurs.
#[thin_trait_object(drop_abi = "C", trait_object(pub VfsStatRecipientProvider))]
pub trait VfsStatRecipient
{
	/// Receives the `stats` resulting from the request.
	///
	/// If [VfsStatRecipient::set_error] was called earlier, calling this
	/// function will clear the error.
	fn submit_stats(&mut self, stats: &VfsFileStats);

	/// Marks this stat request as having failed, with the provided `code`.
	///
	/// If [VfsStatRecipient::submit_stats] was called earlier, calling this
	/// function will invalidate the result and the error will take precedence.
	fn set_error(&mut self, code: VfsFileErrorCode);
}
