use crate::extensions::api_impl::{dummy_api_impl, log_api_impl, map_format_api_impl};
use anyhow::{Context, Result, bail};
use bspextifc::probe_api::ProbeResult;
use bspextifc::probe_api::internal::{ApiProvider, CallbacksContainer, ExportedApis};
use bspextifc::{
	EXTENSION_INFO_VERSION, ExtensionInfo, ExtensionInfoVersionType, SYMBOL_EXTENSION_INFO,
	SYMBOL_EXTENSION_INFO_VERSION, dummy_api, log_api, map_format_api, probe_api,
};
use libloading::{Library, Symbol};
use log::{debug, trace};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use target_lexicon::{HOST, OperatingSystem};

#[cfg(target_os = "linux")]
use libloading::os::unix::Symbol as UnsafeSymbol;
#[cfg(target_os = "windows")]
use libloading::os::windows::Symbol as UnsafeSymbol;

type MapFormatParsers = HashMap<String, map_format_api::MapParseFn>;
pub type CallbackRef<T> = Rc<RefCell<T>>;

pub struct ApiCallbacks
{
	pub dummy_api_callbacks: Option<CallbackRef<dummy_api_impl::Callbacks>>,
	pub map_format_api_callbacks: Option<CallbackRef<map_format_api_impl::Callbacks>>,
}

impl Default for ApiCallbacks
{
	fn default() -> Self
	{
		return Self {
			dummy_api_callbacks: None,
			map_format_api_callbacks: None,
		};
	}
}

#[derive(Clone)]
pub struct Extension
{
	const_data: Rc<ExtConstData>,
	mutable_data: Rc<RefCell<ExtMutData>>,
}

struct ExtConstData
{
	name: String,

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
}

struct ExtMutData
{
	api_callbacks: ApiCallbacks,
	map_formats: MapFormatParsers,
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

		if extension_info_version != EXTENSION_INFO_VERSION
		{
			bail!(
				"Expected extension info version {EXTENSION_INFO_VERSION} but got version {extension_info_version}"
			);
		}

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

		if probe_api_version != probe_api::API_VERSION
		{
			bail!(
				"Required interface version {}, \
					but extension provided interface version {probe_api_version}.",
				probe_api::API_VERSION
			);
		}

		let name: String =
			Extension::compute_library_name(path.file_stem().unwrap().to_str().unwrap());

		let extension: Self = Self {
			const_data: Rc::new(ExtConstData {
				name: name,
				library: library,
				extension_info: extension_info_symbol,
			}),
			mutable_data: Rc::new(RefCell::new(ExtMutData {
				api_callbacks: ApiCallbacks::default(),
				map_formats: MapFormatParsers::new(),
			})),
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
		return &self.const_data.name;
	}

	pub fn get_dummy_api_callbacks(&self)
	-> Result<Option<CallbackRef<dummy_api_impl::Callbacks>>>
	{
		let data: Ref<'_, ExtMutData> = self.borrow_mutdata()?;
		return Ok(data.api_callbacks.dummy_api_callbacks.clone());
	}

	pub fn get_map_format_callbacks(
		&self,
	) -> Result<Option<CallbackRef<map_format_api_impl::Callbacks>>>
	{
		let data: Ref<'_, ExtMutData> = self.borrow_mutdata()?;
		return Ok(data.api_callbacks.map_format_api_callbacks.clone());
	}

	pub fn probe(&mut self) -> Result<()>
	{
		let mut data: RefMut<'_, ExtMutData> = self.borrow_mutdata_mut()?;
		let callbacks: ExportedApis = self.probe_and_return_callbacks()?;

		let api_callbacks: ApiCallbacks = ApiCallbacks {
			dummy_api_callbacks: callbacks
				.dummy_callbacks
				.take()
				.map(|cb| Extension::new_callbacks(dummy_api_impl::Callbacks::new(cb))),
			map_format_api_callbacks: callbacks
				.map_format_callbacks
				.take()
				.map(|cb| Extension::new_callbacks(map_format_api_impl::Callbacks::new(cb))),
		};

		data.api_callbacks = api_callbacks;
		return Ok(());
	}

	fn probe_and_return_callbacks(&self) -> Result<ExportedApis>
	{
		let mut exported_apis: ExportedApis = Extension::create_exported_apis();
		let mut probe: probe_api::ProbeApi =
			probe_api::internal::create_probe_api(&self.const_data.name, &mut exported_apis);

		let probe_result: ProbeResult = (self.const_data.extension_info.probe_fn)(&mut probe);

		if let ProbeResult::Failure = probe_result
		{
			bail!("Extension {} failed probe call", self.const_data.name);
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

	fn borrow_mutdata(&self) -> Result<Ref<'_, ExtMutData>>
	{
		return self
			.mutable_data
			.try_borrow()
			.with_context(|| "Failed to acquire non-mutable ref to internal data");
	}

	fn borrow_mutdata_mut(&self) -> Result<RefMut<'_, ExtMutData>>
	{
		return self
			.mutable_data
			.try_borrow_mut()
			.with_context(|| "Failed to acquire mutable ref to internal data");
	}

	fn new_callbacks<T>(callbacks: T) -> CallbackRef<T>
	{
		return Rc::new(RefCell::new(callbacks));
	}
}

// impl Extension
// {
// 	pub fn load(path: &PathBuf) -> Result<ExtensionRef>
// 	{
// 		let library: Library = unsafe { Library::new(path.as_os_str()) }?;

// 		let extension_info_version_symbol: UnsafeSymbol<&'static
// ExtensionInfoVersionType> = 			unsafe {
// Extension::get_unsafe_symbol(&library, SYMBOL_EXTENSION_INFO_VERSION) }
// 				.with_context(|| {
// 					format!("Failed to look up extension info version symbol in extension
// library") 				})?;

// 		let extension_info_version: ExtensionInfoVersionType =
// **extension_info_version_symbol;

// 		if extension_info_version != EXTENSION_INFO_VERSION
// 		{
// 			bail!(
// 				"Expected extension info version {EXTENSION_INFO_VERSION} but got version
// {extension_info_version}" 			);
// 		}

// 		let extension_info_symbol: UnsafeSymbol<&'static ExtensionInfo> =
// 			unsafe { Extension::get_unsafe_symbol(&library, SYMBOL_EXTENSION_INFO)
// }.with_context( 				|| format!("Failed to look up extension info symbol in
// extension library"), 			)?;

// 		let extension_info: &ExtensionInfo = *extension_info_symbol;
// 		let probe_api_version: usize = extension_info.probe_api_version;

// 		trace!(
// 			"Extension {} reported probe API version {probe_api_version}",
// 			path.to_str().unwrap()
// 		);

// 		if probe_api_version != probe_api::API_VERSION
// 		{
// 			bail!(
// 				"Required interface version {}, \
// 				but extension provided interface version {probe_api_version}.",
// 				probe_api::API_VERSION
// 			);
// 		}

// 		let name: String =
// 			Extension::compute_library_name(path.file_stem().unwrap().to_str().
// unwrap());

// 		let extension: Self = Self {
// 			name: name,
// 			library: library,
// 			extension_info: extension_info_symbol,
// 			api_callbacks: ApiCallbacks::default(),
// 			map_formats: MapFormatParsers::new(),
// 		};

// 		debug!(
// 			"Loaded extension: {} ({})",
// 			extension.get_name(),
// 			path.to_str().unwrap()
// 		);

// 		return Ok(Extension::new_ref(extension));
// 	}

// 	pub fn get_name(&self) -> &str
// 	{
// 		return &self.name;
// 	}

// 	pub fn probe(&mut self) -> Result<()>
// 	{
// 		let result: Result<ExportedApis> = self.probe_and_return_callbacks();

// 		if let Err(err) = result
// 		{
// 			return Err(err);
// 		}

// 		let api_callbacks: ApiCallbacks =
// 			result.map_or(ApiCallbacks::default(), |callbacks| ApiCallbacks {
// 				dummy_api_callbacks: callbacks
// 					.dummy_callbacks
// 					.take_callbacks()
// 					.map(|cb| dummy_api_impl::Callbacks::new(cb)),
// 				map_format_api_callbacks: callbacks
// 					.map_format_callbacks
// 					.take_callbacks()
// 					.map(|cb| map_format_api_impl::Callbacks::new(cb)),
// 			});

// 		self.api_callbacks = api_callbacks;

// 		return Ok(());
// 	}

// 	pub fn get_dummy_api_callbacks(&self) -> Option<&dummy_api_impl::Callbacks>
// 	{
// 		return self.api_callbacks.dummy_api_callbacks.as_ref();
// 	}

// 	pub fn get_map_format_callbacks(&self) ->
// Option<&map_format_api_impl::Callbacks> 	{
// 		return self.api_callbacks.map_format_api_callbacks.as_ref();
// 	}

// 	fn new_ref(extension: Extension) -> ExtensionRef
// 	{
// 		return Rc::new(RefCell::new(extension));
// 	}

// 	fn probe_and_return_callbacks(&self) -> Result<ExportedApis>
// 	{
// 		let mut exported_apis: ExportedApis = Extension::create_exported_apis();
// 		let mut probe: probe_api::ProbeApi =
// 			probe_api::internal::create_probe_api(&self.name, &mut exported_apis);

// 		let probe_result: ProbeResult = (self.extension_info.probe_fn)(&mut probe);

// 		if let ProbeResult::Failure = probe_result
// 		{
// 			bail!("Extension {} failed probe call", self.name);
// 		}

// 		return Ok(exported_apis);
// 	}

// 	fn create_exported_apis() -> ExportedApis
// 	{
// 		return ExportedApis {
// 			log_api: ApiProvider::new(&log_api::API_INFO, log_api_impl::create_api()),
// 			dummy_callbacks: CallbacksContainer::new(&dummy_api::API_INFO),
// 			map_format_callbacks: CallbacksContainer::new(&map_format_api::API_INFO),
// 		};
// 	}

// 	fn compute_library_name(filename_stem: &str) -> String
// 	{
// 		let prefix: &str = Extension::library_prefix_for_platform();
// 		return filename_stem[prefix.len()..].to_string();
// 	}

// 	// It is the caller's responsibility that the symbol is not used after the
// 	// library is unloaded.
// 	unsafe fn get_unsafe_symbol<T>(library: &Library, name: &[u8]) ->
// Result<UnsafeSymbol<T>> 	{
// 		let symbol: Symbol<T> = unsafe { library.get(name) }?;
// 		return unsafe { Ok(symbol.into_raw()) };
// 	}

// 	pub const fn library_extension_for_platform() -> &'static str
// 	{
// 		return match HOST.operating_system
// 		{
// 			OperatingSystem::Windows => "dll",
// 			OperatingSystem::Linux => "so",
// 			_ => panic!("Unsupported operating system"),
// 		};
// 	}

// 	pub const fn library_prefix_for_platform() -> &'static str
// 	{
// 		return match HOST.operating_system
// 		{
// 			OperatingSystem::Windows => "",
// 			OperatingSystem::Linux => "lib",
// 			_ => panic!("Unsupported operating system"),
// 		};
// 	}
// }
