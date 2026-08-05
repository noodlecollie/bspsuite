//! Module for loading extensions and managing the features they present to the
//! rest of the crate.
//!
//! A BSPSuite extension is a shared library that registers interest in certain
//! APIs that the main compiler library offers. These APIs are mainly to do
//! things like load files of different formats.
//!
//! From the compiler's point of view, we want to know whether any extension
//! supports loading a particular file format, and if there is an extension that
//! supports it, we want to call a funtion to load the file.
//!
//! The [ExtensionCollection] struct is set up to deal with loading extensions,
//! and with exposing the different file format APIs they implement. The
//! process is as follows:
//!
//! * The `extensions` directory for the provided toolchain is scanned for
//!   shared libraries that might be BSPSuite extensions.
//! * For each library that is found, its interface is queried for
//!   compatibility.
//! * Compatible extensions are "probed", meaning the compiler library asks each
//!   extension to register for each API that it wants to use.
//! * Each of the required APIs is initialised for that extension. This is the
//!   step in which an extension will, for example, register loader callbacks
//!   for file formats that it supports.
//! * Once all extensions have been probed and their APIs initialised, the
//!   extension collection builds lists of all the APIs across all loaded
//!   extensions, to allow a loader callback for a particular file format to be
//!   looked up easily. This behaviour is implemented using the
//!   [ApiImplCollection] helper struct.

use std::collections::hash_map::Values;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use bspextifc::probe_api;
use bspextifc::resource_format_api::LoadImageFn;
use bspextifc::{
	EXTENSION_INFO_MAGIC, EXTENSION_INFO_VERSION, FFI_VERSION, SYMBOL_EXTENSION_INFO,
	SYMBOL_EXTENSION_INFO_VERSION,
};
use bspextifc::{ExtensionInfo, ExtensionInfoVersionType};

use crate::extensions::api_impl::map_format_api_impl::{MapFormatApiEndpoint, MapFormatDefinition};
use crate::extensions::api_impl::resource_format_api_impl::ResourceFormatApiEndpoint;
use crate::extensions::api_impl::vfs_api::VfsInitialiser;
use crate::extensions::api_impl::vfs_api_impl::VfsApiEndpoint;
use crate::extensions::api_impl::{ProbeApiImpl, ProbeRegistrationResults};
use crate::extensions::{FormatLoader, FormatLoaderApi, FormatSupportQuery};
use crate::{CompilerError, CompilerErrorCode};
use anyhow::{Context, Result, anyhow, bail, ensure};
use bspextifc::builders::map_source_builder::Entity;
use libloading::{Library, Symbol};
use log::{debug, trace, warn};
use self_cell::self_cell;
use target_lexicon::{HOST, OperatingSystem};

/// Returns the file extension used by libraries on the current platform,
/// without the leading full stop. On Windows this is `"dll"`, and on Linux this
/// is `"so"`.
pub const fn library_extension_for_platform() -> &'static str
{
	return match HOST.operating_system
	{
		OperatingSystem::Windows => "dll",
		OperatingSystem::Linux => "so",
		_ => panic!("Unsupported operating system"),
	};
}

/// Returns the naming prefix used by libraries on the current platform. On
/// Windows there is no prefix, and on Linux the prefix is `"lib"`.
pub const fn library_prefix_for_platform() -> &'static str
{
	return match HOST.operating_system
	{
		OperatingSystem::Windows => "",
		OperatingSystem::Linux => "lib",
		_ => panic!("Unsupported operating system"),
	};
}

pub(crate) type MapFormatImplCollection =
	ApiImplCollection<MapFormatDefinition, MapFormatApiEndpoint>;

pub(crate) type ResourceFormatImplCollection =
	ApiImplCollection<LoadImageFn, ResourceFormatApiEndpoint>;

pub(crate) type VfsFormatImplCollection = ApiImplCollection<VfsInitialiser, VfsApiEndpoint>;

/// Struct to hold all extensions found for the current toolchain, along with
/// convenience maps referencing all the constructed API endpoints.
pub(crate) struct ExtensionCollection
{
	extensions: HashMap<String, Extension>,

	map_formats: MapFormatImplCollection,
	image_formats: ResourceFormatImplCollection,
	vfs_formats: VfsFormatImplCollection,
}

/// Struct representing a loaded extension library.
pub(crate) struct Extension
{
	name: String,
	path: PathBuf,
	library_and_data: LibraryAndData,
}

/// Struct holding all data that belongs to a specific extension.
pub(crate) struct ExtensionData<'l>
{
	marker: PhantomData<&'l Library>,

	extension_info: ExtensionInfo,
	api_endpoints: ExtensionApis<'l>,
}

/// Helper struct that holds Rcs to all implementations of a particular
/// [FormatLoader] API, across all loaded extensions.
pub(crate) struct ApiImplCollection<LoaderInterface, ApiImpl: FormatLoader<LoaderInterface>>
{
	marker: PhantomData<LoaderInterface>,
	implementers: Vec<Rc<ApiImpl>>,
}

impl<LoaderInterface, ApiImpl: FormatLoader<LoaderInterface>>
	ApiImplCollection<LoaderInterface, ApiImpl>
{
	pub fn new<F: Fn(&ExtensionData) -> Rc<ApiImpl>>(
		extensions: &HashMap<String, Extension>,
		query_fn: F,
	) -> Self
	{
		return Self {
			marker: PhantomData,
			implementers: extensions
				.iter()
				.map(|(_, extension)| {
					extension
						.library_and_data
						.with_dependent(|_, data| query_fn(data))
				})
				.collect(),
		};
	}
}

impl<LoaderInterface, ApiImpl: FormatLoader<LoaderInterface>> FormatSupportQuery<ApiImpl>
	for ApiImplCollection<LoaderInterface, ApiImpl>
{
	fn implementers_supporting_format(&self, format_name: &str) -> Vec<Rc<ApiImpl>>
	{
		return self
			.implementers
			.iter()
			.filter_map(|api_impl| {
				api_impl
					.supports_loading_format(format_name)
					.then_some(api_impl.clone())
			})
			.collect();
	}

	fn implementers_supporting_file_extension(
		&self,
		file_extension: &str,
		format_whitelist: &Option<&[&str]>,
	) -> Vec<(Rc<ApiImpl>, Vec<String>)>
	{
		return self
			.implementers
			.iter()
			.filter_map(|api_impl| {
				let formats =
					api_impl.supported_formats_for_file_extension(file_extension, format_whitelist);
				(!formats.is_empty()).then(|| (api_impl.clone(), formats))
			})
			.collect();
	}

	fn implementer_from_extension(&self, extension_name: &str) -> Option<Rc<ApiImpl>>
	{
		return self
			.implementers
			.iter()
			.find(|api_impl| api_impl.extension_name() == extension_name)
			.map(|rc| rc.clone());
	}

	fn all_supported_formats(&self) -> Vec<String>
	{
		let mut format_set: HashSet<String> = HashSet::new();

		self.implementers.iter().for_each(|endpoint| {
			endpoint.supported_format_names().iter().for_each(|name| {
				format_set.insert(name.clone());
			})
		});

		return format_set.into_iter().collect();
	}

	fn all_supported_format_extensions(&self) -> HashMap<String, HashSet<String>>
	{
		let mut format_to_exts: HashMap<String, HashSet<String>> = HashMap::new();

		self.implementers.iter().for_each(|endpoint| {
			endpoint.supported_formats().into_iter().for_each(|spec| {
				if !format_to_exts.contains_key(&spec.format_name)
				{
					format_to_exts.insert(spec.format_name.clone(), HashSet::new());
				}

				let exts_hash: &mut HashSet<String> =
					format_to_exts.get_mut(&spec.format_name).unwrap();

				spec.associated_file_extensions.into_iter().for_each(|ext| {
					exts_hash.insert(ext);
				});
			});
		});

		return format_to_exts;
	}
}

// TODO: Refactor this file to relocate these bits
impl MapFormatImplCollection
{
	pub fn parse_map(
		&self,
		map_path: &Path,
		input_data: &str,
		allowed_formats: &Option<&[&str]>,
		map_format_override: &Option<&str>,
	) -> Result<Vec<Entity>, CompilerError>
	{
		if let Some(override_format) = map_format_override
			&& let Some(formats) = allowed_formats
			&& !formats.contains(override_format)
		{
			return Err(CompilerError::from_anyhow(
				CompilerErrorCode::ArgumentError,
				anyhow!(
					"Map format {override_format} was not contained within list of allowed formats: {}",
					formats.join(", ")
				),
			));
		}

		let (endpoint, use_format): (Rc<MapFormatApiEndpoint>, String) = match map_format_override
		{
			Some(override_format) => (
				self.get_impl_for_format(*override_format).map_err(|err| {
					CompilerError::from_anyhow(CompilerErrorCode::ArgumentError, err)
				})?,
				(*override_format).to_owned(),
			),
			None => self
				.get_impl_with_format_from_file_extension(map_path, allowed_formats)
				.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::ArgumentError, err))?,
		};

		return endpoint
			.load_if_supported(&use_format, |def| Ok(def.parse_map(input_data)?))
			.map_err(|err| {
				CompilerError::from_anyhow(
					CompilerErrorCode::IoError,
					anyhow!("Failed to parse map {}. {err}", map_path.display()),
				)
			});
	}

	fn get_impl_for_format(&self, format: &str) -> Result<Rc<MapFormatApiEndpoint>>
	{
		let implementers: Vec<Rc<MapFormatApiEndpoint>> =
			self.implementers_supporting_format(format);

		if implementers.len() != 1
		{
			// TODO: Support better disambiguation in this case.
			if implementers.len() > 1
			{
				let matches_str: String = implementers
					.iter()
					.map(|endpoint| endpoint.extension_name())
					.collect::<Vec<&str>>()
					.join(", ");

				bail!(
					"Map format {format} supported by more than compiler extension: \
					{matches_str}. Unable to deduce which one to use."
				);
			}
			else
			{
				let indent: &'static str = "    ";

				bail!(
					"No compiler extensions supported map format {format}.\n\
					{indent}Formats supported by compiler: {}",
					self.all_supported_formats().join(", "),
				);
			}
		}

		return Ok(implementers[0].clone());
	}

	fn get_impl_with_format_from_file_extension(
		&self,
		map_path: &Path,
		allowed_formats: &Option<&[&str]>,
	) -> Result<(Rc<MapFormatApiEndpoint>, String)>
	{
		let map_ext: &str = map_path
			.extension()
			.and_then(|ext_str| ext_str.to_str())
			.ok_or_else(|| anyhow!("Failed to deduce extension from map path"))?;

		let implementers: Vec<(Rc<MapFormatApiEndpoint>, String)> =
			self.all_formats_for_file_extension(map_ext, allowed_formats);

		if implementers.len() != 1
		{
			// TODO: Support better disambiguation in this case.
			if implementers.len() > 1
			{
				let matches_str: String = implementers
					.iter()
					.map(|(endpoint, format)| {
						format!("{} (format {format})", endpoint.extension_name(),)
					})
					.collect::<Vec<String>>()
					.join(", ");

				bail!(
					"Input map file extension .{map_ext} supported by more than one compiler extension: \
						{matches_str}. Unable to deduce which one to use."
				);
			}
			else
			{
				let indent: &'static str = "    ";

				bail!(
					"No compiler extensions recognised input map file with extension .{map_ext}\n\
						{indent}Formats allowed for game: {}\n\
						{indent}Formats supported by compiler: {}",
					allowed_formats
						.map(|list| list.join(", "))
						.unwrap_or("any".to_owned()),
					self.all_supported_format_extensions_desc()
				);
			}
		}

		return Ok(implementers[0].clone());
	}
}

/// Struct that holds API callbacks and other info that an extension has
/// provided. The struct's lifetime is bound by that of the extension's shared
/// library.
struct ExtensionApis<'l>
{
	marker: PhantomData<&'l Library>,

	pub map_format_api: Rc<MapFormatApiEndpoint>,
	pub resource_format_api: Rc<ResourceFormatApiEndpoint>,
	pub vfs_api: Rc<VfsApiEndpoint>,
}

impl<'l> ExtensionApis<'l>
{
	pub fn construct_empty(extension_name: &str) -> Self
	{
		return Self {
			marker: PhantomData,
			map_format_api: Rc::new(MapFormatApiEndpoint::new(extension_name.into(), None)),
			resource_format_api: Rc::new(ResourceFormatApiEndpoint::new(
				extension_name.into(),
				None,
			)),
			vfs_api: Rc::new(VfsApiEndpoint::new(extension_name.into(), None)),
		};
	}
}

self_cell!(
	struct LibraryAndData
	{
		owner: Library,

		#[covariant]
		dependent: ExtensionData,
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
	pub fn register_and_consume(mut self, extension_name: &str) -> Rc<Container>
	{
		self.0.register_supported_formats(extension_name);
		return Rc::new(self.0);
	}
}

/// Struct containing all the endpoints where supported file formats have yet to
/// be registered. Call register_and_transform() to ask for all formats to be
/// registered, and to get the final ExtensionApis struct at the other end.
struct UnregisteredApiEndpoints<'l>
{
	marker: PhantomData<&'l Library>,

	ext_name: String,
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

	pub fn from_exported_apis(extension_name: &str, apis: ProbeRegistrationResults) -> Self
	{
		return Self {
			marker: PhantomData,

			ext_name: extension_name.into(),

			map_format_api: FormatRegisterHelper(MapFormatApiEndpoint::new(
				extension_name.into(),
				apis.map_format_callbacks.into(),
			)),

			resource_format_api: FormatRegisterHelper(ResourceFormatApiEndpoint::new(
				extension_name.into(),
				apis.resource_format_callbacks.into(),
			)),

			vfs_api: FormatRegisterHelper(VfsApiEndpoint::new(
				extension_name.into(),
				apis.vfs_callbacks.into(),
			)),
		};
	}
}

impl ExtensionCollection
{
	pub fn extensions_directory(toolchain_root: &Path) -> PathBuf
	{
		return toolchain_root.join("extensions");
	}

	pub fn map_formats(&self) -> &MapFormatImplCollection
	{
		return &self.map_formats;
	}

	pub fn image_formats(&self) -> &ResourceFormatImplCollection
	{
		return &self.image_formats;
	}

	pub fn vfs_formats(&self) -> &VfsFormatImplCollection
	{
		return &self.vfs_formats;
	}

	pub fn extensions_iter(&self) -> Values<'_, String, Extension>
	{
		return self.extensions.values();
	}

	pub fn num_extensions(&self) -> usize
	{
		return self.extensions.len();
	}

	pub fn get_extension(&self, name: &str) -> Option<&Extension>
	{
		return self.extensions.get(name);
	}

	pub fn load_extensions_from(toolchain_root: &Path) -> Result<Self>
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
					.library_and_data
					.with_dependent_mut(|_, data| -> Result<()> {
						ExtensionCollection::probe(&extension.name, data)
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

		return Ok(ExtensionCollection::build_from_extensions(hash_map));
	}

	fn build_from_extensions(extensions: HashMap<String, Extension>) -> Self
	{
		return Self {
			map_formats: ApiImplCollection::new(&extensions, |data| {
				data.api_endpoints.map_format_api.clone()
			}),
			image_formats: ApiImplCollection::new(&extensions, |data| {
				data.api_endpoints.resource_format_api.clone()
			}),
			vfs_formats: ApiImplCollection::new(&extensions, |data| {
				data.api_endpoints.vfs_api.clone()
			}),
			extensions,
		};
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

	fn probe(name: &str, data: &mut ExtensionData) -> Result<()>
	{
		// The first step probes the extension for what it supports.
		// The extension can call register_X_api() to indicate that it supports this
		// API.
		let unregistered_endpoints: UnregisteredApiEndpoints = {
			let mut exported_apis: ProbeRegistrationResults = ProbeRegistrationResults::new();

			{
				let mut probe: probe_api::ProbeApiProvider =
					probe_api::ProbeApiProvider::new(ProbeApiImpl::new(name, &mut exported_apis));

				let probe_result: probe_api::ProbeResult =
					(data.extension_info.probe_fn)(&mut probe);

				if let probe_api::ProbeResult::Failure = probe_result
				{
					bail!("Extension {name} failed probe call");
				}
			}

			UnregisteredApiEndpoints::from_exported_apis(name, exported_apis)
		};

		// The second step sets up each API that the extension has indicated it
		// supports. For the map format API, for example, this would involve asking
		// the extension which map formats it supports, and storing the callback
		// provided for each format.
		data.api_endpoints = unregistered_endpoints.register_and_transform(name);
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
	pub fn name(&self) -> &str
	{
		return &self.name;
	}

	pub fn path(&self) -> &Path
	{
		return self.path.as_path();
	}

	fn load(path: &PathBuf) -> Result<Extension>
	{
		let name: String = Extension::compute_library_name(path.as_path());

		// SAFETY: It is up to the library to be well-behaved when running init and
		// shutdown routines. There's not much we can do to guarantee that from this
		// side.
		let library: Library = unsafe { Library::new(path.as_os_str()) }?;

		// SAFETY: Again, it is up to the library to implement this symbol properly, and
		// we have no way of enforcing this. However, if the version symbol is garbage,
		// it is very unlikely to match the expected version, which will fail the
		// loading process.
		// Important note: variable symbols are always pointers! If you deref a
		// non-pointer symbol, you'll just get back the address of the value rather
		// than the value itself, or the symbol will completely fail to load.
		let extension_info_version_symbol: Symbol<*const ExtensionInfoVersionType> =
			unsafe { library.get(SYMBOL_EXTENSION_INFO_VERSION) }.with_context(|| {
				format!(
					"Failed to look up extension info version symbol \"{}\" in extension library",
					String::from_utf8_lossy(SYMBOL_EXTENSION_INFO_VERSION)
				)
			})?;

		// SAFETY: The library for this symbol is still alive, as above, so we can
		// deref.
		let extension_info_version: ExtensionInfoVersionType =
			unsafe { **extension_info_version_symbol };

		ensure!(
			extension_info_version == EXTENSION_INFO_VERSION,
			"Expected extension info version {EXTENSION_INFO_VERSION} but got version {extension_info_version}"
		);

		let shared_library: LibraryAndData = LibraryAndData::try_new(library, |lib_ref| {
			// SAFETY: Again, it is up to the library to implement this symbol properly, and
			// we have no way of enforcing this. However, if the magic in the struct is
			// garbage and does not match what is expected, the loading process will fail.
			let extension_info_symbol: Symbol<*const ExtensionInfo> =
				unsafe { lib_ref.get(SYMBOL_EXTENSION_INFO) }.with_context(|| {
					format!(
						"Failed to look up extension info symbol \"{}\" in extension library",
						String::from_utf8_lossy(SYMBOL_EXTENSION_INFO)
					)
				})?;

			// SAFETY: The library for this symbol is still alive, as above, so we can
			// deref.
			let extension_info: &ExtensionInfo = unsafe { &**extension_info_symbol };

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
			Ok(ExtensionData {
				marker: PhantomData,

				// Manually copy this struct, so that we don't rely on how clone() may be
				// implemented for it.
				extension_info: ExtensionInfo {
					magic,
					ffi_version: ffi_api_version,
					probe_api_version,
					probe_fn: extension_info.probe_fn,
				},
				api_endpoints: ExtensionApis::construct_empty(&name),
			})
		})?;

		let extension: Self = Self {
			name: name,
			path: path.clone(),
			library_and_data: shared_library,
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
