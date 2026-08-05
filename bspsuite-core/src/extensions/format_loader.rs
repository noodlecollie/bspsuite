use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::rc::Rc;

use crate::extensions::{Extension, ExtensionData};
use anyhow;

/// Convenience struct to map a file format name to a list of associated file
/// extensions.
pub(crate) struct FormatSpec
{
	pub format_name: String,
	pub associated_file_extensions: Vec<String>,
}

/// Trait to facilitate invoking file format registration for a particular
/// extension.
pub(crate) trait FormatLoaderEndpoint
{
	/// If an extension has indicated support for this API, calls the extension
	/// to register its supported formats, and returns true.
	/// If the extension has not indicated suppport for this API, does nothing
	/// and returns false.
	fn register_supported_formats(&mut self) -> bool;
}

pub(crate) trait FormatLoader<LoaderInterface>
{
	type LoaderOutput;

	/// Returns the name of the extension that is linked to this loader.
	fn extension_name(&self) -> &str;

	/// Returns a list of all supported formats, along with the file extensions
	/// they are associated with.
	fn supported_formats(&self) -> Vec<FormatSpec>;

	/// Returns whether the implementer supports loading data in the given
	/// format.
	fn supports_loading_format(&self, format_name: &str) -> bool;

	/// Returns whether the implementer supports loading data in the given
	/// format, and if so, whether it associates the given file extension with
	/// this format.
	fn supports_loading_format_from_file(&self, format_name: &str, file_extension: &str) -> bool;

	/// Returns a list of file extensions for the given format if it is
	/// supported, or None if the format is not supported.
	fn supported_file_extensions_for_format(&self, format_name: &str) -> Option<Vec<&str>>;

	/// If the given format is supported, calls the provided callback so that
	/// loading can take place. The argument to the callback is the interface
	/// that the implementer provides in order to load a file of this format.
	/// Returns Ok if the operation succeeds, or an error otherwise.
	fn load_if_supported<Callback>(
		&self,
		format_name: &str,
		callback: Callback,
	) -> anyhow::Result<Self::LoaderOutput>
	where
		Callback: Fn(&LoaderInterface) -> anyhow::Result<Self::LoaderOutput>;

	/// Convenience for obtaining just the format names from
	/// [FormatLoader<LoaderInterface>::supported_formats].
	fn supported_format_names(&self) -> Vec<String>
	{
		return self
			.supported_formats()
			.into_iter()
			.map(|spec| spec.format_name)
			.collect();
	}

	/// Convenience for obtaining format names and associated extensions from
	/// [FormatLoader<LoaderInterface>::supported_formats]. Each description
	/// string is in the form: "format (.ext1, .ext2, ...)"
	fn supported_format_descriptions(&self) -> Vec<String>
	{
		return self
			.supported_formats()
			.into_iter()
			.map(|spec| {
				format!(
					"{} ({})",
					spec.format_name,
					spec.associated_file_extensions.join(", ")
				)
			})
			.collect();
	}

	/// Returns all formats that have the provided file extension associated
	/// with them. If a whitelist is provided, only the formats present in the
	/// whitelist will be considered.
	fn supported_formats_for_file_extension(
		&self,
		file_extension: &str,
		format_whitelist: &Option<&[&str]>,
	) -> Vec<String>
	{
		// An annoying workaround to the fact that we can't call .contains(&str) on a
		// Vec<String>, since it wants &String instead...
		let file_extension: String = file_extension.to_owned();

		return self
			.supported_formats()
			.iter()
			.filter(|spec| {
				let in_whitelist: bool = format_whitelist
					.map(|list| list.contains(&spec.format_name.as_ref()))
					.unwrap_or(true);

				in_whitelist && spec.associated_file_extensions.contains(&file_extension)
			})
			.map(|spec| spec.format_name.clone())
			.collect();
	}
}

// TODO: Make a helper type to hold the varying data types that we get back from
// these functions, and then add helper functions to this type to convert it to
// strings/lists of strings
pub(crate) trait FormatCollector<ApiEndpoint>
{
	/// Returns a vector of endpoints which support loading the specified
	/// format.
	fn endpoints_supporting_format(&self, format_name: &str) -> Vec<Rc<ApiEndpoint>>;

	/// Returns a vector of endpoints which support loading the specified
	/// file extension in a format that is present in the whitelist. The first
	/// item in each tuple is the endpoint, and the second item is the
	/// formats that the file extension is mapped to.
	fn endpoints_supporting_file_extension(
		&self,
		file_extension: &str,
		format_whitelist: &Option<&[&str]>,
	) -> Vec<(Rc<ApiEndpoint>, Vec<String>)>;

	/// Returns the API endpoint that belongs to the given extension, or None if
	/// there is no extension with this name.
	fn endpoint_from_extension(&self, extension_name: &str) -> Option<Rc<ApiEndpoint>>;

	/// Returns a vector of all format names supported by any implementer.
	fn all_supported_formats(&self) -> Vec<String>;

	/// Returns all supported formats, and all their associated file
	/// extensions across all compiler extensions.
	fn all_supported_format_extensions(&self) -> HashMap<String, HashSet<String>>;

	/// Returns a string listing all supported format names with their
	/// associated file extensions.
	fn all_supported_format_extensions_desc(&self) -> String
	{
		return self
			.all_supported_format_extensions()
			.into_iter()
			.map(|(format, exts)| {
				format!(
					"{format} (.{})",
					exts.into_iter().collect::<Vec<String>>().join(", .")
				)
			})
			.collect::<Vec<String>>()
			.join("; ");
	}

	/// Returns a vector of endpoints which support loading the specified
	/// file extension in a format that is present in the whitelist. This is
	/// similar to
	/// [FormatSupportQuery<ApiImpl>::endpoints_supporting_file_extension],
	/// except that each item in the vector corresponds to a single format.
	fn all_formats_for_file_extension(
		&self,
		file_extension: &str,
		format_whitelist: &Option<&[&str]>,
	) -> Vec<(Rc<ApiEndpoint>, String)>
	{
		let mut out: Vec<(Rc<ApiEndpoint>, String)> = Vec::new();

		self.endpoints_supporting_file_extension(file_extension, format_whitelist)
			.into_iter()
			.for_each(|tuple| {
				tuple
					.1
					.into_iter()
					.for_each(|format| out.push((tuple.0.clone(), format)));
			});

		return out;
	}
}

/// Helper struct that holds Rcs to all implementations of a particular
/// [FormatLoader] API, across all loaded extensions.
pub(crate) struct ApiCollector<LoaderInterface, ApiEndpoint: FormatLoader<LoaderInterface>>
{
	marker: PhantomData<LoaderInterface>,
	endpoints: Vec<Rc<ApiEndpoint>>,
}

impl<LoaderInterface, ApiImpl: FormatLoader<LoaderInterface>> ApiCollector<LoaderInterface, ApiImpl>
{
	pub fn new<F: Fn(&ExtensionData) -> Rc<ApiImpl>>(
		extensions: &HashMap<String, Extension>,
		query_fn: F,
	) -> Self
	{
		return Self {
			marker: PhantomData,
			endpoints: extensions
				.iter()
				.map(|(_, extension)| extension.with_data(|data| query_fn(data)))
				.collect(),
		};
	}
}

impl<LoaderInterface, ApiImpl: FormatLoader<LoaderInterface>> FormatCollector<ApiImpl>
	for ApiCollector<LoaderInterface, ApiImpl>
{
	fn endpoints_supporting_format(&self, format_name: &str) -> Vec<Rc<ApiImpl>>
	{
		return self
			.endpoints
			.iter()
			.filter_map(|api_impl| {
				api_impl
					.supports_loading_format(format_name)
					.then_some(api_impl.clone())
			})
			.collect();
	}

	fn endpoints_supporting_file_extension(
		&self,
		file_extension: &str,
		format_whitelist: &Option<&[&str]>,
	) -> Vec<(Rc<ApiImpl>, Vec<String>)>
	{
		return self
			.endpoints
			.iter()
			.filter_map(|api_impl| {
				let formats =
					api_impl.supported_formats_for_file_extension(file_extension, format_whitelist);
				(!formats.is_empty()).then(|| (api_impl.clone(), formats))
			})
			.collect();
	}

	fn endpoint_from_extension(&self, extension_name: &str) -> Option<Rc<ApiImpl>>
	{
		return self
			.endpoints
			.iter()
			.find(|api_impl| api_impl.extension_name() == extension_name)
			.map(|rc| rc.clone());
	}

	fn all_supported_formats(&self) -> Vec<String>
	{
		let mut format_set: HashSet<String> = HashSet::new();

		self.endpoints.iter().for_each(|endpoint| {
			endpoint.supported_format_names().iter().for_each(|name| {
				format_set.insert(name.clone());
			})
		});

		return format_set.into_iter().collect();
	}

	fn all_supported_format_extensions(&self) -> HashMap<String, HashSet<String>>
	{
		let mut format_to_exts: HashMap<String, HashSet<String>> = HashMap::new();

		self.endpoints.iter().for_each(|endpoint| {
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
