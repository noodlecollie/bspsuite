use crate::apis::log_api::LogApi;
use crate::apis::map_format_api::MapFormatApiCallbacks;
use crate::resource_format_api::ResourceFormatApiCallbacks;
use crate::vfs_api::VfsApiCallbacks;
use std::result::Result;

use thin_trait_object::thin_trait_object;

pub const API_VERSION: usize = 1;
pub type ProbeExtensionFn = extern "C" fn(&mut ProbeApiProvider) -> ProbeResult;

/// Enum representing a failure to provide a requested API to the caller
/// extension.
#[repr(C)]
pub enum RequestError
{
	/// The provided version did not match the version of the available API.
	/// The inner value of this enum item is the actual version available.
	VersionDidNotMatch(u64),
}

/// Enum representing the result of a probe call to an extension.
#[repr(C)]
pub enum ProbeResult
{
	/// The extension was able to obtain all the APIs that it needed. This
	/// result still covers cases where an extension is not able to get every
	/// single API that it asks for, but is still able to operate correctly.
	Success,

	/// The extension was not able to obtain all the APIs it needed to function
	/// correctly.
	Failure,
}

// TODO: Docs
#[thin_trait_object(drop_abi = "C", trait_object(pub ProbeApiProvider))]
pub trait ProbeApi
{
	fn request_log_api(&mut self, requested_version: u64) -> Result<LogApi, RequestError>;

	fn register_map_format_api_callbacks(
		&mut self,
		requested_version: u64,
		callbacks: MapFormatApiCallbacks,
	) -> Result<(), RequestError>;

	fn register_resource_format_api_callbacks(
		&mut self,
		requested_version: u64,
		callbacks: ResourceFormatApiCallbacks,
	) -> Result<(), RequestError>;

	fn register_vfs_api_callbacks(
		&mut self,
		requested_version: u64,
		callbacks: VfsApiCallbacks,
	) -> Result<(), RequestError>;
}
