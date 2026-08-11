use bspffi::types::XCStr;
use itertools::Itertools;
use log;
use log::{debug, warn};

pub(super) struct ExtensionFileFormatCollection<Handler>
{
	extension_name: String,
	format_desc: String,
	formats: Vec<(String, Handler, Vec<String>)>,
}

impl<Handler> ExtensionFileFormatCollection<Handler>
{
	pub fn new(extension_name: String, format_desc: String) -> Self
	{
		return Self {
			extension_name,
			format_desc,
			formats: Vec::new(),
		};
	}

	pub fn add(
		&mut self,
		format_name: &str,
		file_extensions: &[XCStr],
		handler: Handler,
		allow_empty_exts_slice: bool,
	) -> bool
	{
		if file_extensions.is_empty() && !allow_empty_exts_slice
		{
			warn!(
				"Extension {} specified no file extensions for {} {format_name}. \
				This request will be ignored.",
				self.extension_name, self.format_desc
			);

			return false;
		}

		let extension_strings: Vec<String> = if !file_extensions.is_empty()
		{
			// We want to do a few things here:
			// - Trim leading and trailing whitespace
			// - Trim leading dots, in case people specify ".map" instead of "map"
			// - Remove any items that end up being empty after these operations
			// - Remove duplicates
			let extension_strings: Vec<String> = file_extensions
				.iter()
				.map(|item| item.as_str().trim().trim_start_matches(".").to_string())
				.filter(|item| !item.is_empty())
				.unique()
				.collect();

			if extension_strings.is_empty()
			{
				warn!(
					"After removing invalid file extensions, extension {} was left with no valid file extensions \
					for {} {format_name}. This request will be ignored.",
					self.extension_name, self.format_desc
				);

				return false;
			}

			extension_strings
		}
		else
		{
			Vec::new()
		};

		if extension_strings.len() < file_extensions.len()
		{
			warn!(
				"Extension {} provided {} empty, duplicated, or otherwise invalid file extensions for {} \
				{format_name}. These will be ignored.",
				self.extension_name,
				file_extensions.len() - extension_strings.len(),
				self.format_desc,
			);
		}

		if let Some(index) = self
			.formats
			.iter()
			.find_position(|item| item.0 == format_name)
			.map(|(index, _)| index)
		{
			warn!(
				"Overriding existing registration for extension {} {} \"{format_name}\"",
				self.extension_name, self.format_desc,
			);

			self.formats.remove(index);
		}

		self.formats
			.push((format_name.to_string(), handler, extension_strings));

		if log::max_level() >= log::LevelFilter::Debug
		{
			let all_extensions: String = self.formats[self.formats.len() - 1].2.join(", ");

			if all_extensions.is_empty()
			{
				debug!(
					"Extension {} registered support for {} \"{format_name}\"",
					self.extension_name, self.format_desc
				);
			}
			else
			{
				debug!(
					"Extension {} registered support for {} \"{format_name}\", with \
				file extensions: {all_extensions}",
					self.extension_name, self.format_desc
				);
			}
		}

		return true;
	}

	pub fn collect(self) -> Vec<(String, Handler, Vec<String>)>
	{
		return self.formats;
	}
}
