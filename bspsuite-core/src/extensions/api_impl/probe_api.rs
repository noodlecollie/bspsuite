use super::log_api_impl;
use bspextifc::ApiInfo;
use bspextifc::log_api::{API_INFO as LOG_API_INFO, LogApi};
use bspextifc::map_format_api::{API_INFO as MAP_FORMAT_API_INFO, MapFormatApiCallbacks};
use bspextifc::probe_api::{ProbeApi, RequestError};
use bspextifc::resource_format_api::{
	API_INFO as RESOURCE_FORMAT_API_INFO, ResourceFormatApiCallbacks,
};
use bspextifc::vfs_api::{API_INFO as VFS_API_INFO, VfsApiCallbacks};
use bspffi::types::XCStr;
use log::{error, trace};

pub(crate) struct ProbeApiImpl<'l>
{
	extension_name: XCStr<'l>,
	apis: &'l mut ExportedApis,
}

impl<'l> ProbeApiImpl<'l>
{
	pub fn new(extension_name: &'l str, apis: &'l mut ExportedApis) -> Self
	{
		return Self {
			extension_name: XCStr::from(extension_name),
			apis,
		};
	}
}

impl<'l> ProbeApi for ProbeApiImpl<'l>
{
	fn request_log_api(&mut self, requested_version: u64) -> Result<LogApi, RequestError>
	{
		return ExportedApis::request_get_api(
			self.extension_name.as_str(),
			&mut self.apis.log_api,
			requested_version,
		);
	}

	fn register_map_format_api_callbacks(
		&mut self,
		requested_version: u64,
		callbacks: MapFormatApiCallbacks,
	) -> Result<(), RequestError>
	{
		return ExportedApis::request_set_callbacks(
			self.extension_name.as_str(),
			&mut self.apis.map_format_callbacks,
			requested_version,
			callbacks,
		);
	}

	fn register_resource_format_api_callbacks(
		&mut self,
		requested_version: u64,
		callbacks: ResourceFormatApiCallbacks,
	) -> Result<(), RequestError>
	{
		return ExportedApis::request_set_callbacks(
			self.extension_name.as_str(),
			&mut self.apis.resource_format_callbacks,
			requested_version,
			callbacks,
		);
	}
}

pub(crate) struct ExportedApis
{
	pub log_api: ApiProvider<LogApi>,
	pub map_format_callbacks: CallbacksContainer<MapFormatApiCallbacks>,
	pub resource_format_callbacks: CallbacksContainer<ResourceFormatApiCallbacks>,
	pub vfs_callbacks: CallbacksContainer<VfsApiCallbacks>,
}

impl ExportedApis
{
	pub fn new() -> Self
	{
		return Self {
			log_api: ApiProvider::new(&LOG_API_INFO, log_api_impl::create_api()),
			map_format_callbacks: CallbacksContainer::new(&MAP_FORMAT_API_INFO),
			resource_format_callbacks: CallbacksContainer::new(&RESOURCE_FORMAT_API_INFO),
			vfs_callbacks: CallbacksContainer::new(&VFS_API_INFO),
		};
	}
}

pub(crate) enum ApiRequestError
{
	// Requested -> Actual
	MismatchedVersion((u64, u64)),
}

pub(crate) struct ApiProvider<T>
where
	T: Clone,
{
	name: &'static str,
	version: u64,
	api: T,
}

pub(crate) struct CallbacksContainer<T>
{
	name: &'static str,
	version: u64,
	callbacks: Option<T>,
}

impl<T> ApiProvider<T>
where
	T: Clone,
{
	pub fn new(api_info: &ApiInfo, api: T) -> Self
	{
		return Self {
			name: api_info.name.as_str(),
			version: api_info.version,
			api: api,
		};
	}

	pub fn get_name(&self) -> String
	{
		return self.name.to_string();
	}

	pub fn get_version(&self) -> u64
	{
		return self.version;
	}

	pub fn request_get_api(&self, requested_version: u64) -> Result<T, ApiRequestError>
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

impl<T> CallbacksContainer<T>
{
	pub fn new(api_info: &ApiInfo) -> Self
	{
		return Self {
			name: api_info.name.as_str(),
			version: api_info.version,
			callbacks: None,
		};
	}

	pub fn get_name(&self) -> String
	{
		return self.name.to_string();
	}

	pub fn get_version(&self) -> u64
	{
		return self.version;
	}

	pub fn request_set_callbacks(
		&mut self,
		requested_version: u64,
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

	pub fn take(self) -> Option<T>
	{
		return self.callbacks;
	}
}

impl ExportedApis
{
	pub fn request_get_api<T>(
		extension_name: &str,
		provider: &mut ApiProvider<T>,
		requested_version: u64,
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

	pub fn request_set_callbacks<T>(
		extension_name: &str,
		container: &mut CallbacksContainer<T>,
		requested_version: u64,
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

	pub fn process_result<T>(
		extension_name: &str,
		api_name: &str,
		version: u64,
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
