use std::collections::{HashMap, HashSet};

use crate::extensions::api_impl::map_format_api;
use crate::extensions::{ExtensionList, ExtensionRef};
use anyhow::{Result, bail};
use std::path::Path;

pub struct ExtensionForParsingMapFormat
{
	pub extension_name: String,
	pub map_format_name: String,
}

pub fn join_extension_names(extensions: Vec<&ExtensionRef>) -> String
{
	return extensions
		.iter()
		.map(|ext| ext.get_name())
		.collect::<Vec<&str>>()
		.join(", ");
}

// TODO: De-duplicate this logic
pub fn choose_extension_to_parse_map(
	list: &ExtensionList,
	input_file: &Path,
	allowed_formats: &Vec<&str>,
	map_format_override: &Option<&str>,
) -> Result<ExtensionForParsingMapFormat>
{
	let input_ext: Option<&str> = input_file
		.extension()
		.map(|ext_str| ext_str.to_str())
		.unwrap_or(None);

	if map_format_override.is_none()
	{
		if input_ext.is_some()
		{
			let input_ext: &str = input_ext.unwrap();

			let supported_exts: Vec<(&ExtensionRef, String)> =
				find_extensions_supporting_source_file_extension(list, input_ext, allowed_formats);

			if supported_exts.len() != 1
			{
				// TODO: Support better disambiguation in this case.
				if supported_exts.len() > 1
				{
					let matches_str: String = supported_exts
						.iter()
						.map(|(ext, fmt)| format!("{} (map format {fmt})", ext.get_name()))
						.collect::<Vec<String>>()
						.join(", ");

					bail!(
						"Input map file extension .{input_ext} supported by more than compiler extension: \
						{matches_str}. Unable to deduce which one to use."
					);
				}
				else
				{
					let indent: &'static str = "    ";

					bail!(
						"No compiler extensions recognised input map file with extension .{input_ext}\n\
						{indent}Formats allowed for game: {}\n\
						{indent}Formats supported by compiler: {}",
						allowed_formats.join(", "),
						supported_map_formats_and_file_extensions(list).join("; ")
					);
				}
			}

			return Ok(ExtensionForParsingMapFormat {
				extension_name: supported_exts[0].0.get_name().to_owned(),
				map_format_name: supported_exts[0].1.clone(),
			});
		}
		else
		{
			bail!(
				"Input map file {} has no extension, cannot infer format",
				input_file.display()
			);
		}
	}
	else
	{
		let map_format: &str = map_format_override.unwrap();
		let supported_exts: Vec<&ExtensionRef> =
			find_extensions_supporting_format(list, map_format, allowed_formats);

		if supported_exts.len() != 1
		{
			// TODO: Support better disambiguation in this case.
			if supported_exts.len() > 1
			{
				let matches_str: String = supported_exts
					.iter()
					.map(|ext| ext.get_name())
					.collect::<Vec<&str>>()
					.join(", ");

				bail!(
					"Map format {map_format} supported by more than compiler extension: \
					{matches_str}. Unable to deduce which one to use."
				);
			}
			else
			{
				let indent: &'static str = "    ";

				bail!(
					"No compiler extensions supported map format {map_format}.\n\
					{indent}Formats allowed for game: {}\n\
					{indent}Formats supported by compiler: {}",
					allowed_formats.join(", "),
					supported_map_formats(list).join(", ")
				);
			}
		}

		return Ok(ExtensionForParsingMapFormat {
			extension_name: supported_exts[0].get_name().to_owned(),
			map_format_name: map_format.to_owned(),
		});
	}
}

fn find_extensions_supporting_format<'l>(
	list: &'l ExtensionList,
	format_name: &str,
	allowed_formats: &Vec<&str>,
) -> Vec<&'l ExtensionRef>
{
	return list
		.iter()
		.filter(|ext| {
			extension_supports_format(ext, format_name) && allowed_formats.contains(&format_name)
		})
		.collect();
}

pub fn supported_map_formats_and_file_extensions(list: &ExtensionList) -> Vec<String>
{
	let mut format_to_exts: HashMap<String, HashSet<String>> = HashMap::new();

	list.iter().for_each(|ext| {
		let ext_ref = ext
			.get_extension()
			.expect("Could not get non-mutable reference to extension");

		let map_format_api: Option<&map_format_api::Endpoint> =
			ext_ref.get_api_endpoints().map_format_api.as_ref();

		if let Some(endpoint) = map_format_api
		{
			endpoint
				.get_supported_map_format_defs()
				.iter()
				.for_each(|(name, def)| {
					if !format_to_exts.contains_key(*name)
					{
						format_to_exts.insert((*name).to_owned(), HashSet::new());
					}

					let exts_hash: &mut HashSet<String> = format_to_exts.get_mut(*name).unwrap();

					for ext in &def.file_extensions
					{
						exts_hash.insert(ext.clone());
					}
				});
		}
	});

	return format_to_exts
		.into_iter()
		.map(|(fmt, exts)| {
			format!(
				"{fmt} (.{})",
				exts.into_iter().collect::<Vec<String>>().join(", .")
			)
		})
		.collect();
}

pub fn supported_map_formats(list: &ExtensionList) -> Vec<String>
{
	let mut formats: HashSet<String> = HashSet::new();

	list.iter().for_each(|ext| {
		let ext_ref = ext
			.get_extension()
			.expect("Could not get non-mutable reference to extension");

		let map_format_api: Option<&map_format_api::Endpoint> =
			ext_ref.get_api_endpoints().map_format_api.as_ref();

		if let Some(endpoint) = map_format_api
		{
			endpoint.get_supported_map_formats().iter().for_each(|fmt| {
				formats.insert(fmt.clone());
			});
		}
	});

	return formats.into_iter().collect();
}

pub fn register_map_formats(list: &ExtensionList)
{
	return list.for_each_or_warn("Registering map formats", |ext_ref| {
		let mut ext_mut_ref = ext_ref.get_extension_mut()?;
		let ext_name: String = String::from(ext_mut_ref.get_name());
		let api_endpoints = ext_mut_ref.get_api_endpoints_mut();

		if let Some(map_format_api) = &mut api_endpoints.map_format_api
		{
			map_format_api.register_map_formats(&ext_name);
		}

		Ok(())
	});
}

pub fn find_extensions_supporting_source_file_extension<'l>(
	list: &'l ExtensionList,
	file_extension: &str,
	allowed_formats: &Vec<&str>,
) -> Vec<(&'l ExtensionRef, String)>
{
	let extension_supported_formats: Vec<(&'l ExtensionRef, Vec<String>)> = list
		.iter()
		.filter_map(|ext| {
			let formats: Vec<String> =
				extension_map_formats_for_file_extension(ext, file_extension, allowed_formats);

			if !formats.is_empty()
			{
				Some((ext, formats))
			}
			else
			{
				None
			}
		})
		.collect();

	let flat_supported_formats: Vec<Vec<(&'l ExtensionRef, String)>> = extension_supported_formats
		.into_iter()
		.map(|(ext, fmts)| {
			let flat_fmts: Vec<(&'l ExtensionRef, String)> =
				fmts.into_iter().map(|fmt| (ext, fmt)).collect();
			return flat_fmts;
		})
		.collect();

	let mut out: Vec<(&'l ExtensionRef, String)> = Vec::new();

	// Not sure if there's a more functional way of doing this,
	// but I gave it a good go and it got complicated,
	// so we're going imperative instead.
	for fmts in flat_supported_formats
	{
		out.reserve(fmts.len());
		out.extend(fmts);
	}

	return out;
}

// Expects that no extension is being mutably accessed.
fn extension_supports_format(extension: &ExtensionRef, format_name: &str) -> bool
{
	let ext_ref = extension
		.get_extension()
		.expect("Could not get non-mutable reference to extension");

	let map_format_api: Option<&map_format_api::Endpoint> =
		ext_ref.get_api_endpoints().map_format_api.as_ref();

	return map_format_api
		.map(|api| api.supports_map_format(format_name))
		.unwrap_or(false);
}

// Expects that no extension is being mutably accessed.
fn extension_supports_format_and_maybe_file_ext(
	extension: &ExtensionRef,
	format_name: &str,
	file_extension: Option<&str>,
) -> bool
{
	let ext_ref = extension
		.get_extension()
		.expect("Could not get non-mutable reference to extension");

	let map_format_api: Option<&map_format_api::Endpoint> =
		ext_ref.get_api_endpoints().map_format_api.as_ref();

	return map_format_api
		.map(|api| {
			file_extension
				.map(|file_ext| api.supports_map_format_with_file_extension(format_name, file_ext))
				.unwrap_or_else(|| api.supports_map_format(format_name))
		})
		.unwrap_or(false);
}

// Expects that no extension is being mutably accessed.
fn extension_map_formats_for_file_extension(
	extension: &ExtensionRef,
	file_extension: &str,
	allowed_formats: &Vec<&str>,
) -> Vec<String>
{
	let ext_ref = extension
		.get_extension()
		.expect("Could not get non-mutable reference to extension");

	let map_format_api: Option<&map_format_api::Endpoint> =
		ext_ref.get_api_endpoints().map_format_api.as_ref();

	if map_format_api.is_none()
	{
		return Vec::new();
	}

	let map_format_api: &map_format_api::Endpoint = map_format_api.unwrap();
	let file_extension_string: String = file_extension.to_owned();

	return map_format_api
		.get_supported_map_formats()
		.into_iter()
		.filter(|fmt| allowed_formats.contains(&fmt.as_ref()))
		.filter(|fmt| {
			map_format_api
				.get_definition_supported_file_extensions(fmt)
				.unwrap()
				.contains(&file_extension_string)
		})
		.collect();
}
