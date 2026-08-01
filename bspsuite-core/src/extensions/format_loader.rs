use std::rc::Rc;

use anyhow;

pub(crate) struct FormatSpec
{
	pub format_name: String,
	pub associated_file_extensions: Vec<String>,
}

pub(crate) trait FormatLoaderApi
{
	/// If an extension has indicated support for this API, calls the extension
	/// to register its supported formats, and returns true.
	/// If the extension has not indicated suppport for this API, does nothing
	/// and returns false.
	fn register_supported_formats(&mut self, extension_name: &str) -> bool;
}

pub(crate) trait FormatLoader<LoaderInterface>
{
	type LoaderOutput;

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
	/// [supported_formats].
	fn supported_format_names(&self) -> Vec<String>
	{
		return self
			.supported_formats()
			.into_iter()
			.map(|spec| spec.format_name)
			.collect();
	}

	/// Returns all formats that have the provided file extension associated
	/// with them. If a whitelist is provided, only the formats present in the
	/// whitelist will be considered.
	fn supported_formats_for_file_extension(
		&self,
		file_extension: &str,
		format_whitelist: &Vec<&str>,
	) -> Vec<String>
	{
		// An annoying workaround to the fact that we can't call .contains(&str) on a
		// Vec<String>, since it wants &String instead...
		let file_extension: String = file_extension.to_owned();

		return self
			.supported_formats()
			.iter()
			.filter(|spec| {
				(format_whitelist.is_empty()
					|| format_whitelist.contains(&spec.format_name.as_ref()))
					&& spec.associated_file_extensions.contains(&file_extension)
			})
			.map(|spec| spec.format_name.clone())
			.collect();
	}
}

pub(crate) trait FormatSupportQuery<ApiImpl>
{
	/// Returns a vector of implementers which support loading the specified
	/// format.
	fn implementers_supporting_format(&self, format_name: &str) -> Vec<Rc<ApiImpl>>;

	/// Returns a vector of possible file formats that the given file extension
	/// may apply to.
	fn format_for_file_extension(&self, file_extension: &str) -> Vec<String>;
}
