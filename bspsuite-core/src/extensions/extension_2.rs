use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::fs;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use bspextifc::probe_api;
use bspextifc::{
	EXTENSION_INFO_MAGIC, EXTENSION_INFO_VERSION, FFI_VERSION, SYMBOL_EXTENSION_INFO,
	SYMBOL_EXTENSION_INFO_VERSION,
};
use bspextifc::{ExtensionInfo, ExtensionInfoVersionType};

use crate::extensions::FormatLoaderApi;
use crate::extensions::api_impl::map_format_api_impl::Endpoint as MapFormatApiEndpoint;
use crate::extensions::api_impl::resource_format_api_impl::Endpoint as ResourceFormatApiEndpoint;
use crate::extensions::api_impl::vfs_api_impl::Endpoint as VfsApiEndpoint;
use crate::extensions::api_impl::{ExportedApis, ProbeApiImpl};
use anyhow::{Context, Result, bail, ensure};
use log::{debug, trace, warn};
use self_cell::self_cell;
use target_lexicon::{HOST, OperatingSystem};

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

/// Struct to hold all extensions found for the current toolchain.
pub struct ExtensionCollection
{
	extensions: HashMap<String, Extension>,
}

/// Struct that holds API callbacks and other info that an extension has
/// provided. The struct's lifetime is bound by that of the extension's shared
/// library.
pub struct ExtensionApis<'l>
{
	marker: PhantomData<&'l Library>,

	pub map_format_api: MapFormatApiEndpoint,
	pub resource_format_api: ResourceFormatApiEndpoint,
	pub vfs_api: VfsApiEndpoint,
}

struct Extension
{
	name: String,
	path: PathBuf,
	library_and_symbols: SharedLibrary,
}

struct SharedLibrarySymbols<'l>
{
	extension_info: Symbol<'l, ExtensionInfo>,
	api_endpoints: ExtensionApis<'l>,
}

self_cell!(
	struct SharedLibrary
	{
		owner: Library,

		#[covariant]
		dependent: SharedLibrarySymbols,
	}
);

/// Helper for registering formats for a loader. When this helper is unwrapped
/// by calling register_and_consume(), it ensures that the format loader asks
/// its extension for the formats it supports. This design is so that we don't
/// end up with one big function where all formats are registered, and a
/// developer could forget to add a new format loader to this list - in order to
/// get at the container, format registration must be run.
struct FormatRegisterHelper<Container: FormatLoaderApi>(Container);

impl<Container: FormatLoaderApi> FormatRegisterHelper<Container>
{
	pub fn register_and_consume(mut self, extension_name: &str) -> Container
	{
		self.0.register_supported_formats(extension_name);
		return self.0;
	}
}

/// Struct containing all the endpoints where supported file formats have yet to
/// be registered. Call register_and_transform() to ask for all formats to be
/// registered, and to get the final ExtensionApis struct at the other end.
struct UnregisteredApiEndpoints<'l>
{
	marker: PhantomData<&'l Library>,

	pub map_format_api: FormatRegisterHelper<MapFormatApiEndpoint>,
	pub resource_format_api: FormatRegisterHelper<ResourceFormatApiEndpoint>,
	pub vfs_api: FormatRegisterHelper<VfsApiEndpoint>,
}

impl<'l> UnregisteredApiEndpoints<'l>
{
	pub fn register_and_transform(self, extension_name: &str) -> ExtensionApis<'l>
	{
		return ExtensionApis {
			marker: PhantomData,
			map_format_api: self.map_format_api.register_and_consume(extension_name),
			resource_format_api: self
				.resource_format_api
				.register_and_consume(extension_name),
			vfs_api: self.vfs_api.register_and_consume(extension_name),
		};
	}
}

impl<'l> From<ExportedApis> for UnregisteredApiEndpoints<'l>
{
	fn from(value: ExportedApis) -> Self
	{
		return Self {
			marker: PhantomData,

			map_format_api: FormatRegisterHelper(MapFormatApiEndpoint::new(
				value.map_format_callbacks.into(),
			)),

			resource_format_api: FormatRegisterHelper(ResourceFormatApiEndpoint::new(
				value.resource_format_callbacks.into(),
			)),

			vfs_api: FormatRegisterHelper(VfsApiEndpoint::new(value.vfs_callbacks.into())),
		};
	}
}

impl<'l> Default for ExtensionApis<'l>
{
	fn default() -> Self
	{
		return Self {
			marker: PhantomData,
			map_format_api: MapFormatApiEndpoint::new(None),
			resource_format_api: ResourceFormatApiEndpoint::new(None),
			vfs_api: VfsApiEndpoint::new(None),
		};
	}
}

impl ExtensionCollection
{
	pub fn extensions_directory(toolchain_root: &PathBuf) -> PathBuf
	{
		return toolchain_root.join("extensions");
	}

	fn load_extensions_from(toolchain_root: &PathBuf) -> Result<Self>
	{
		let extensions_dir: PathBuf = ExtensionCollection::extensions_directory(toolchain_root);
		let extension_paths: Vec<PathBuf> =
			ExtensionCollection::find_extensions(&extensions_dir)
				.with_context(|| "Failed to look up extensions on disk")?;

		debug!(
			"Found {} extensions in {}",
			extension_paths.len(),
			extensions_dir.display()
		);

		// Do the initial load and filter out the extensions that failed.
		let extensions: Vec<Extension> =
			ExtensionCollection::load_extension_libraries(&extension_paths)
				.into_iter()
				.filter_map(ExtensionCollection::log_and_prune_errors)
				.collect();

		// Then probe each extension so that we know all the APIs they register for.
		let extensions: Vec<Extension> = extensions
			.into_iter()
			.map(|mut extension| -> Result<Extension> {
				extension
					.library_and_symbols
					.with_dependent_mut(|_, symbols| -> Result<()> {
						ExtensionCollection::probe(&extension.name, symbols)
					})?;

				Ok(extension)
			})
			.filter_map(ExtensionCollection::log_and_prune_errors)
			.collect();

		// Finally, set up a hash map by name for each extension.
		let hash_map: HashMap<String, Extension> = extensions
			.into_iter()
			.map(|extension| (extension.name.clone(), extension))
			.collect();

		return Ok(Self {
			extensions: hash_map,
		});
	}

	fn find_extensions(root: &PathBuf) -> Result<Vec<PathBuf>>
	{
		let entries: fs::ReadDir = fs::read_dir(root).with_context(|| {
			format!(
				"Could not read extensions from directory {}",
				root.display()
			)
		})?;

		let file_ext: &str = library_extension_for_platform();

		let paths_for_file_ext: Vec<PathBuf> = entries
			.into_iter()
			// Only the successful entries
			.filter_map(|entry| entry.ok())
			.filter_map(|entry| {
				let path: PathBuf = entry.path();
				// Transform into the entry path if it has an
				// extension that matches what we want.
				path.extension()
					.map(|ext| ext == file_ext)
					.unwrap_or(false)
					.then(|| path)
			})
			.collect();

		return Ok(paths_for_file_ext);
	}

	fn load_extension_libraries(paths: &Vec<PathBuf>) -> Vec<Result<Extension>>
	{
		return paths
			.iter()
			.map(|path| {
				Extension::load(path).map_err(|err| {
					err.context(format!("Failed to load extension {}", path.display()))
				})
			})
			.collect();
	}

	fn probe(name: &str, symbols: &mut SharedLibrarySymbols) -> Result<()>
	{
		// The first step probes the extension for what it supports.
		// The extension can call register_X_api() to indicate that it supports this
		// API.
		let unregistered_endpoints: UnregisteredApiEndpoints = {
			let mut exported_apis: ExportedApis = ExportedApis::new();

			{
				let mut probe: probe_api::BoxedProbeApi =
					probe_api::BoxedProbeApi::new(ProbeApiImpl::new(name, &mut exported_apis));

				let probe_result: probe_api::ProbeResult =
					(symbols.extension_info.probe_fn)(&mut probe);

				if let probe_api::ProbeResult::Failure = probe_result
				{
					bail!("Extension {name} failed probe call");
				}
			}

			exported_apis.into()
		};

		// The second step sets up each API that the extension has indicated it
		// supports. For the map format API, for example, this would involve asking
		// the extension which map formats it supports, and storing the callback
		// provided for each format.
		symbols.api_endpoints = unregistered_endpoints.register_and_transform(name);
		return Ok(());
	}

	fn log_and_prune_errors<T>(result: Result<T>) -> Option<T>
	{
		if let Err(err) = result.as_ref()
		{
			let source = err.source();

			if let Some(source) = source
			{
				warn!("{err} Source error: {source}");
			}
			else
			{
				warn!("{err}");
			}
		}

		result.ok()
	}
}

impl Extension
{
	fn load(path: &PathBuf) -> Result<Extension>
	{
		// SAFETY: It is up to the library to be well-behaved when running init and
		// shutdown routines. There's not much we can do to guarantee that from this
		// side.
		let library: Library = unsafe { Library::new(path.as_os_str()) }?;

		// SAFETY: Again, it is up to the library to implement this symbol properly, and
		// we have no way of enforcing this. However, if the version symbol is garbage,
		// it is very unlikely to match the expected version, which will fail the
		// loading process.
		let extension_info_version_symbol: Symbol<ExtensionInfoVersionType> =
			unsafe { library.get(SYMBOL_EXTENSION_INFO_VERSION) }.with_context(|| {
				format!("Failed to look up extension info version symbol in extension library")
			})?;

		let extension_info_version: ExtensionInfoVersionType = *extension_info_version_symbol;

		ensure!(
			extension_info_version == EXTENSION_INFO_VERSION,
			"Expected extension info version {EXTENSION_INFO_VERSION} but got version {extension_info_version}"
		);

		let shared_library: SharedLibrary = SharedLibrary::try_new(library, |lib_ref| {
			// SAFETY: Again, it is up to the library to implement this symbol properly, and
			// we have no way of enforcing this. However, if the magic in the struct is
			// garbage and does not match what is expected, the loading process will fail.
			let extension_info_symbol: Symbol<ExtensionInfo> =
				unsafe { lib_ref.get(SYMBOL_EXTENSION_INFO) }.with_context(|| {
					format!("Failed to look up extension info symbol in extension library")
				})?;

			let extension_info: &ExtensionInfo = &*extension_info_symbol;
			let magic: u32 = extension_info.magic;
			let probe_api_version: usize = extension_info.probe_api_version;
			let ffi_api_version: u64 = extension_info.ffi_version;

			trace!(
				"Extension {} reported FFI version {ffi_api_version:0>6}, probe API version {probe_api_version}",
				path.to_str().unwrap()
			);

			ensure!(
				magic == EXTENSION_INFO_MAGIC,
				"Required magic {EXTENSION_INFO_MAGIC:0>8}, but extension provided magic {magic:0>8}.",
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

			// We return this from the closure, and it's added into the overall
			// SharedLibrary struct instance.
			Ok(SharedLibrarySymbols {
				extension_info: extension_info_symbol,
				api_endpoints: ExtensionApis::default(),
			})
		})?;

		let name: String = Extension::compute_library_name(path.as_path());

		let extension: Self = Self {
			name: name,
			path: path.clone(),
			library_and_symbols: shared_library,
		};

		debug!(
			"Loaded extension: {} from {}",
			extension.name,
			extension.path.display()
		);

		return Ok(extension);
	}

	fn compute_library_name(path: &Path) -> String
	{
		let prefix: &str = library_prefix_for_platform();

		let filename_stem = path.file_stem().and_then(|stem| stem.to_str()).expect(
			format!(
				"Could not extract file stem from extension path {}",
				path.display()
			)
			.as_str(),
		);

		return filename_stem[prefix.len()..].to_string();
	}
}
