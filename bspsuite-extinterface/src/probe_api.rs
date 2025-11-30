use super::string_ref::StringRef;
use super::{dummy_api, log_api, map_format_api};
use log::{error, trace};
use std::result::Result;

pub const API_VERSION: usize = 1;
pub type ExtFnProbe = extern "C" fn(&mut ProbeApi) -> ProbeResult;

/// Enum representing a failure to provide a requested API to the caller
/// extension.
#[repr(C)]
pub enum RequestError
{
	/// The provided version did not match the version of the available API.
	/// The inner value of this enum item is the actual version available.
	VersionDidNotMatch(usize),
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

#[repr(C)]
pub struct ProbeApi<'l>
{
	extension_name: StringRef<'l>,
	apis: &'l mut internal::ExportedApis,
}

impl<'l> ProbeApi<'l>
{
	pub fn request_log_api(
		&mut self,
		requested_version: usize,
	) -> Result<log_api::Api, RequestError>
	{
		return internal::ExportedApis::request_get_api(
			self.extension_name.to_string().as_str(),
			&mut self.apis.log_api,
			requested_version,
		);
	}

	pub fn register_dummy_api_callbacks(
		&mut self,
		requested_version: usize,
		callbacks: dummy_api::Callbacks,
	) -> Result<(), RequestError>
	{
		return internal::ExportedApis::request_set_callbacks(
			self.extension_name.to_string().as_str(),
			&mut self.apis.dummy_callbacks,
			requested_version,
			callbacks,
		);
	}

	pub fn register_map_format_api_callbacks(
		&mut self,
		requested_version: usize,
		callbacks: map_format_api::Callbacks,
	) -> Result<(), RequestError>
	{
		return internal::ExportedApis::request_set_callbacks(
			self.extension_name.to_string().as_str(),
			&mut self.apis.map_format_callbacks,
			requested_version,
			callbacks,
		);
	}
}

#[doc(hidden)]
pub mod internal
{
	use super::*;
	use crate::api_info::ApiInfo;
	use crate::map_format_api;

	#[doc(hidden)]
	#[repr(C)]
	pub enum ApiRequestError
	{
		// Requested -> Actual
		MismatchedVersion((usize, usize)),
	}

	#[doc(hidden)]
	#[repr(C)]
	pub struct ApiProvider<T>
	where
		T: Clone,
	{
		name: StringRef<'static>,
		version: usize,
		api: T,
	}

	#[doc(hidden)]
	#[repr(C)]
	pub struct CallbacksContainer<T>
	{
		name: StringRef<'static>,
		version: usize,
		callbacks: Option<T>,
	}

	#[doc(hidden)]
	#[repr(C)]
	pub struct ExportedApis
	{
		pub log_api: ApiProvider<log_api::Api>,
		pub dummy_callbacks: CallbacksContainer<dummy_api::Callbacks>,
		pub map_format_callbacks: CallbacksContainer<map_format_api::Callbacks>,
	}

	#[doc(hidden)]
	impl<T> ApiProvider<T>
	where
		T: Clone,
	{
		#[doc(hidden)]
		pub fn new(api_info: &ApiInfo, api: T) -> Self
		{
			return Self {
				name: StringRef::from(api_info.name),
				version: api_info.version,
				api: api,
			};
		}

		#[doc(hidden)]
		pub fn get_name(&self) -> String
		{
			return self.name.to_string();
		}

		#[doc(hidden)]
		pub fn get_version(&self) -> usize
		{
			return self.version;
		}

		#[doc(hidden)]
		pub fn request_get_api(&self, requested_version: usize) -> Result<T, ApiRequestError>
		{
			if requested_version != self.version
			{
				return Err(ApiRequestError::MismatchedVersion((
					requested_version,
					self.version,
				)));
			}

			return Ok(self.api.clone());
		}
	}

	#[doc(hidden)]
	impl<T> CallbacksContainer<T>
	{
		#[doc(hidden)]
		pub fn new(api_info: &ApiInfo) -> Self
		{
			return Self {
				name: StringRef::from(api_info.name),
				version: api_info.version,
				callbacks: None,
			};
		}

		#[doc(hidden)]
		pub fn get_name(&self) -> String
		{
			return self.name.to_string();
		}

		#[doc(hidden)]
		pub fn get_version(&self) -> usize
		{
			return self.version;
		}

		#[doc(hidden)]
		pub fn request_set_callbacks(
			&mut self,
			requested_version: usize,
			callbacks: T,
		) -> Result<(), ApiRequestError>
		{
			if requested_version != self.version
			{
				return Err(ApiRequestError::MismatchedVersion((
					requested_version,
					self.version,
				)));
			}

			self.callbacks = Some(callbacks);
			return Ok(());
		}

		#[doc(hidden)]
		pub fn take_callbacks(self) -> Option<T>
		{
			return self.callbacks;
		}
	}

	#[doc(hidden)]
	impl ExportedApis
	{
		#[doc(hidden)]
		pub fn request_get_api<T>(
			extension_name: &str,
			provider: &mut ApiProvider<T>,
			requested_version: usize,
		) -> Result<T, RequestError>
		where
			T: Clone,
		{
			return ExportedApis::process_result(
				extension_name,
				provider.get_name().as_str(),
				provider.get_version(),
				provider.request_get_api(requested_version),
			);
		}

		#[doc(hidden)]
		pub fn request_set_callbacks<T>(
			extension_name: &str,
			container: &mut CallbacksContainer<T>,
			requested_version: usize,
			callbacks: T,
		) -> Result<(), RequestError>
		{
			return ExportedApis::process_result(
				extension_name,
				container.get_name().as_str(),
				container.get_version(),
				container.request_set_callbacks(requested_version, callbacks),
			);
		}

		#[doc(hidden)]
		pub fn process_result<T>(
			extension_name: &str,
			api_name: &str,
			version: usize,
			result: Result<T, ApiRequestError>,
		) -> Result<T, RequestError>
		{
			if let Err(req_err) = &result
			{
				match req_err
				{
					ApiRequestError::MismatchedVersion((requested, actual)) =>
					{
						error!(
							"Extension {extension_name} failed request for {api_name}. Requested version was {requested}, but the provided version is {actual}",
						);
					}
				}
			}
			else
			{
				trace!(
					"Extension {extension_name} successfully requested version {version} of {api_name}",
				);
			}

			return result.map_err(|res| match res
			{
				ApiRequestError::MismatchedVersion((_, actual)) =>
				{
					RequestError::VersionDidNotMatch(actual)
				}
			});
		}
	}

	pub fn create_probe_api<'l>(
		extension_name: &'l str,
		apis: &'l mut internal::ExportedApis,
	) -> ProbeApi<'l>
	{
		return ProbeApi {
			extension_name: StringRef::from(extension_name),
			apis: apis,
		};
	}
}
