use crate::extensions::api_impl::{dummy_api_impl, log_api_impl, map_format_api_impl};
use anyhow::{Context, Result, bail, ensure};
use bspextifc::probe_api::ProbeResult;
use bspextifc::probe_api::internal::{ApiProvider, CallbacksContainer, ExportedApis};
use bspextifc::{
	EXTENSION_INFO_VERSION, ExtensionInfo, ExtensionInfoVersionType, SYMBOL_EXTENSION_INFO,
	SYMBOL_EXTENSION_INFO_VERSION, dummy_api, log_api, map_format_api, probe_api,
};
use libloading::{Library, Symbol};
use log::{debug, trace};
use std::cell::{BorrowError, BorrowMutError, Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::path::PathBuf;
use target_lexicon::{HOST, OperatingSystem};

#[cfg(target_os = "linux")]
use libloading::os::unix::Symbol as UnsafeSymbol;
#[cfg(target_os = "windows")]
use libloading::os::windows::Symbol as UnsafeSymbol;

type MapFormatParsers = HashMap<String, map_format_api::MapParseFn>;

pub struct ApiEndpoints
{
	pub dummy_api: Option<dummy_api_impl::Endpoint>,
	pub map_format_api: Option<map_format_api_impl::Endpoint>,
}

impl Default for ApiEndpoints
{
	fn default() -> Self
	{
		return Self {
			dummy_api: None,
			map_format_api: None,
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

		trace!(
			"Extension {} reported probe API version {probe_api_version}",
			path.to_str().unwrap()
		);

		ensure!(
			probe_api_version == probe_api::API_VERSION,
			"Required interface version {}, but extension provided interface version {probe_api_version}.",
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
			"Loaded extension: {} ({})",
			extension.get_name(),
			path.to_str().unwrap()
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
			dummy_api: callbacks
				.dummy_callbacks
				.take()
				.map(|cb| dummy_api_impl::Endpoint::new(cb)),
			map_format_api: callbacks
				.map_format_callbacks
				.take()
				.map(|cb| map_format_api_impl::Endpoint::new(cb)),
		};

		self.api_endpoints = api_endpoints;
		return Ok(());
	}

	fn probe_and_return_callbacks(&self) -> Result<ExportedApis>
	{
		let mut exported_apis: ExportedApis = Extension::create_exported_apis();
		let mut probe: probe_api::ProbeApi =
			probe_api::internal::create_probe_api(&self.name, &mut exported_apis);

		let probe_result: ProbeResult = (self.extension_info.probe_fn)(&mut probe);

		if let ProbeResult::Failure = probe_result
		{
			bail!("Extension {} failed probe call", self.name);
		}

		return Ok(exported_apis);
	}

	fn create_exported_apis() -> ExportedApis
	{
		return ExportedApis {
			log_api: ApiProvider::new(&log_api::API_INFO, log_api_impl::create_api()),
			dummy_callbacks: CallbacksContainer::new(&dummy_api::API_INFO),
			map_format_callbacks: CallbacksContainer::new(&map_format_api::API_INFO),
		};
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
