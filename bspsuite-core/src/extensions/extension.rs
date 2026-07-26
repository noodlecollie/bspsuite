use std::cell::{BorrowError, BorrowMutError, Ref, RefCell, RefMut};
use std::path::PathBuf;

use super::api_impl::{ExportedApis, ProbeApiImpl};
use crate::extensions::api_impl::map_format_api_impl;
use crate::extensions::api_impl::resource_format_api_impl;
use crate::extensions::api_impl::vfs_api_impl;
use anyhow::{Context, Result};
use anyhow::{bail, ensure};
use bspextifc::probe_api;
use bspextifc::probe_api::{BoxedProbeApi, ProbeResult};
use bspextifc::{
	EXTENSION_INFO_VERSION, FFI_VERSION, SYMBOL_EXTENSION_INFO, SYMBOL_EXTENSION_INFO_VERSION,
};
use bspextifc::{ExtensionInfo, ExtensionInfoVersionType};
use libloading::{Library, Symbol};
use log::{debug, trace};
use target_lexicon::{HOST, OperatingSystem};

#[cfg(target_os = "linux")]
use libloading::os::unix::Symbol as UnsafeSymbol;
#[cfg(target_os = "windows")]
use libloading::os::windows::Symbol as UnsafeSymbol;

pub struct ApiEndpoints
{
	pub map_format_api: map_format_api_impl::Endpoint,
	pub resource_format_api: resource_format_api_impl::Endpoint,
	pub vfs_api: vfs_api_impl::Endpoint,
}

impl Default for ApiEndpoints
{
	fn default() -> Self
	{
		return Self {
			map_format_api: map_format_api_impl::Endpoint::new(None),
			resource_format_api: resource_format_api_impl::Endpoint::new(None),
			vfs_api: vfs_api_impl::Endpoint::new(None),
		};
	}
}

pub struct ExtensionRef
{
	name: String,
	extension: RefCell<Extension>,
}

// Wraps an extension, always providing access to its name.
// NB. For my Rust learnin's: An alternative to duplicating
// the name string would be to use an Rc<String> in both
// the reference and the extension. This might be worth it
// if the data we were duplicating was large, but given it's
// just a (likely short) string, it's probably not worth
// going down the Rc route.
impl ExtensionRef
{
	pub fn new(extension: Extension) -> Self
	{
		return Self {
			name: String::from(extension.get_name()),
			extension: RefCell::new(extension),
		};
	}

	pub fn load(path: &PathBuf) -> Result<Self>
	{
		return Extension::load(path).map(|ext| ExtensionRef::new(ext));
	}

	pub fn get_name(&self) -> &str
	{
		return &self.name;
	}

	pub fn get_extension(&self) -> Result<Ref<'_, Extension>, BorrowError>
	{
		return self.extension.try_borrow();
	}

	pub fn get_extension_mut(&self) -> Result<RefMut<'_, Extension>, BorrowMutError>
	{
		return self.extension.try_borrow_mut();
	}
}

pub struct Extension
{
	name: String,
	path: PathBuf,

	// This is just here to control the lifetime of the library.
	// By the time this object is constructed, we likely won't
	// need to query the library for anything else, so the member
	// might not be used.
	#[expect(dead_code)]
	library: Library,

	// Unsafe symbols are OK here PROVIDED that they are not
	// copied out of this struct. Here, the extension info
	// will not live longer than the library member above.
	extension_info: UnsafeSymbol<&'static bspextifc::ExtensionInfo>,

	api_endpoints: ApiEndpoints,
}

impl Extension
{
	pub const fn library_extension_for_platform() -> &'static str
	{
		return match HOST.operating_system
		{
			OperatingSystem::Windows => "dll",
			OperatingSystem::Linux => "so",
			_ => panic!("Unsupported operating system"),
		};
	}

	pub const fn library_prefix_for_platform() -> &'static str
	{
		return match HOST.operating_system
		{
			OperatingSystem::Windows => "",
			OperatingSystem::Linux => "lib",
			_ => panic!("Unsupported operating system"),
		};
	}

	pub fn load(path: &PathBuf) -> Result<Extension>
	{
		let library: Library = unsafe { Library::new(path.as_os_str()) }?;

		let extension_info_version_symbol: UnsafeSymbol<&'static ExtensionInfoVersionType> =
			unsafe { Extension::get_unsafe_symbol(&library, SYMBOL_EXTENSION_INFO_VERSION) }
				.with_context(|| {
					format!("Failed to look up extension info version symbol in extension library")
				})?;

		let extension_info_version: ExtensionInfoVersionType = **extension_info_version_symbol;

		ensure!(
			extension_info_version == EXTENSION_INFO_VERSION,
			"Expected extension info version {EXTENSION_INFO_VERSION} but got version {extension_info_version}"
		);

		let extension_info_symbol: UnsafeSymbol<&'static ExtensionInfo> =
			unsafe { Extension::get_unsafe_symbol(&library, SYMBOL_EXTENSION_INFO) }.with_context(
				|| format!("Failed to look up extension info symbol in extension library"),
			)?;

		let extension_info: &ExtensionInfo = *extension_info_symbol;
		let probe_api_version: usize = extension_info.probe_api_version;
		let ffi_api_version: u64 = extension_info.ffi_version;

		trace!(
			"Extension {} reported FFI version {ffi_api_version:0>6}, probe API version {probe_api_version}",
			path.to_str().unwrap()
		);

		ensure!(
			ffi_api_version == FFI_VERSION,
			"Required FFI version {FFI_VERSION:0>6}, but extension provided FFI version {ffi_api_version:0>6}.",
		);

		ensure!(
			probe_api_version == probe_api::API_VERSION,
			"Required probe API version {}, but extension provided probe API version {probe_api_version}.",
			probe_api::API_VERSION
		);

		let name: String =
			Extension::compute_library_name(path.file_stem().unwrap().to_str().unwrap());

		let extension: Self = Self {
			name: name,
			path: path.clone(),
			library: library,
			extension_info: extension_info_symbol,
			api_endpoints: ApiEndpoints::default(),
		};

		debug!(
			"Loaded extension: {} from {}",
			extension.get_name(),
			path.display()
		);

		return Ok(extension);
	}

	pub fn get_name(&self) -> &str
	{
		return &self.name;
	}

	pub fn get_path(&self) -> &PathBuf
	{
		return &self.path;
	}

	pub fn get_api_endpoints(&self) -> &ApiEndpoints
	{
		return &self.api_endpoints;
	}

	pub fn get_api_endpoints_mut(&mut self) -> &mut ApiEndpoints
	{
		return &mut self.api_endpoints;
	}

	pub fn probe(&mut self) -> Result<()>
	{
		let callbacks: ExportedApis = self.probe_and_return_callbacks()?;

		let api_endpoints: ApiEndpoints = ApiEndpoints {
			map_format_api: map_format_api_impl::Endpoint::new(
				callbacks.map_format_callbacks.take(),
			),
			resource_format_api: resource_format_api_impl::Endpoint::new(
				callbacks.resource_format_callbacks.take(),
			),
			vfs_api: vfs_api_impl::Endpoint::new(callbacks.vfs_callbacks.take()),
		};

		self.api_endpoints = api_endpoints;
		return Ok(());
	}

	fn probe_and_return_callbacks(&self) -> Result<ExportedApis>
	{
		let mut exported_apis: ExportedApis = ExportedApis::new();

		{
			let mut probe: BoxedProbeApi =
				BoxedProbeApi::new(ProbeApiImpl::new(&self.name, &mut exported_apis));
			let probe_result: ProbeResult = (self.extension_info.probe_fn)(&mut probe);

			if let ProbeResult::Failure = probe_result
			{
				bail!("Extension {} failed probe call", self.name);
			}
		}

		return Ok(exported_apis);
	}

	fn compute_library_name(filename_stem: &str) -> String
	{
		let prefix: &str = Extension::library_prefix_for_platform();
		return filename_stem[prefix.len()..].to_string();
	}

	// It is the caller's responsibility that the symbol is not used after the
	// library is unloaded.
	unsafe fn get_unsafe_symbol<T>(library: &Library, name: &[u8]) -> Result<UnsafeSymbol<T>>
	{
		let symbol: Symbol<T> = unsafe { library.get(name) }?;
		return unsafe { Ok(symbol.into_raw()) };
	}
}
