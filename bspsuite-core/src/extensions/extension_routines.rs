use std::cell::RefMut;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::extensions::api_impl::map_format_api;
use crate::extensions::{Extension, ExtensionList, ExtensionRef, FormatLoader};
use crate::{CompilerError, CompilerErrorCode};
use anyhow::{Context, Result, anyhow, bail};
use bspextifc::builders::map_source_builder::Entity;

pub struct ExtensionAndSupportedFormat
{
	pub extension_name: String,
	pub format_name: String,
}

pub fn register_all_formats(list: &ExtensionList)
{
	register_vfs(list);
	register_map_formats(list);
	register_resource_formats(list);
}

pub fn register_all_formats_for_ext(extension: &mut Extension)
{
	register_vfs_for_ext(extension);
	register_map_formats_for_ext(extension);
	register_resource_formats_for_ext(extension);
}

pub fn register_map_formats(list: &ExtensionList)
{
	return list.for_each_mut_ref_or_warn("Registering map formats", |ext_ref| {
		register_map_formats_for_ext(ext_ref);
		Ok(())
	});
}

pub fn register_map_formats_for_ext(extension: &mut Extension)
{
	let ext_name: String = extension.get_name().into();
	let api_endpoints = extension.get_api_endpoints_mut();
	api_endpoints.map_format_api.register_map_formats(&ext_name);
}

pub fn register_resource_formats(list: &ExtensionList)
{
	return list.for_each_mut_ref_or_warn("Registering resource formats", |ext_ref| {
		register_resource_formats_for_ext(ext_ref);
		Ok(())
	});
}

pub fn register_resource_formats_for_ext(extension: &mut Extension)
{
	let ext_name: String = extension.get_name().into();
	let api_endpoints = extension.get_api_endpoints_mut();
	api_endpoints
		.resource_format_api
		.register_resource_formats(&ext_name);
}

pub fn register_vfs(list: &ExtensionList)
{
	return list.for_each_mut_ref_or_warn("Registering VFS implementations", |ext_ref| {
		register_vfs_for_ext(ext_ref);
		Ok(())
	});
}

pub fn register_vfs_for_ext(extension: &mut Extension)
{
	let ext_name: String = extension.get_name().into();
	let api_endpoints = extension.get_api_endpoints_mut();
	api_endpoints.vfs_api.register_vfs_impls(&ext_name);
}

pub fn choose_extension_to_parse_map(
	list: &ExtensionList,
	input_file: &Path,
	allowed_formats: &Vec<&str>,
	map_format_override: &Option<&str>,
) -> Result<ExtensionAndSupportedFormat>
{
	return map_format_override
		.map(|map_format| {
			choose_extension_to_parse_map_based_on_format_name(list, allowed_formats, map_format)
		})
		.unwrap_or_else(|| {
			choose_extension_to_parse_map_based_on_file_extension(list, input_file, allowed_formats)
		});
}

// Expects parse_using to be valid for the list, as the extension and map format
// should have been produced by a previous call to
// choose_extension_to_parse_map().
pub fn parse_map(
	full_path: &Path,
	list: &ExtensionList,
	input_data: &str,
	parse_using: &ExtensionAndSupportedFormat,
) -> Result<Vec<Entity>, CompilerError>
{
	let extension: &ExtensionRef = list
		.find_by_name(&parse_using.extension_name)
		.expect("Expected to be able to find appropriate extension to parse map");

	let ext_mut_ref: RefMut<Extension> = extension
		.get_extension_mut()
		.with_context(|| "Failed to acquire mutable reference to extension")
		.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::InternalError, err))?;

	let map_format_api: &map_format_api::Endpoint = &ext_mut_ref.get_api_endpoints().map_format_api;

	return map_format_api
		.load_if_supported(&parse_using.format_name, |def| {
			Ok(def.parse_map(input_data)?)
		})
		.map_err(|err| {
			CompilerError::from_anyhow(
				CompilerErrorCode::IoError,
				anyhow!("Failed to parse map {}. {err}", full_path.display()),
			)
		});
}

fn choose_extension_to_parse_map_based_on_file_extension(
	list: &ExtensionList,
	input_file: &Path,
	allowed_formats: &Vec<&str>,
) -> Result<ExtensionAndSupportedFormat>
{
	let input_ext: Option<&str> = input_file
		.extension()
		.map(|ext_str| ext_str.to_str())
		.unwrap_or(None);

	if input_ext.is_some()
	{
		let input_ext: &str = input_ext.unwrap();

		let supported_exts: Vec<(&ExtensionRef, String)> =
			find_extensions_supporting_map_source_file_extension(list, input_ext, allowed_formats);

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
					all_supported_map_format_description_strings(list).join("; ")
				);
			}
		}

		return Ok(ExtensionAndSupportedFormat {
			extension_name: supported_exts[0].0.get_name().to_owned(),
			format_name: supported_exts[0].1.clone(),
		});
	}
	else
	{
		bail!(
			"Input map file {} has no file extension, cannot infer format",
			input_file.display()
		);
	}
}

fn choose_extension_to_parse_map_based_on_format_name(
	list: &ExtensionList,
	allowed_formats: &Vec<&str>,
	map_format: &str,
) -> Result<ExtensionAndSupportedFormat>
{
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
				all_supported_map_formats(list).join(", ")
			);
		}
	}

	return Ok(ExtensionAndSupportedFormat {
		extension_name: supported_exts[0].get_name().to_owned(),
		format_name: map_format.to_owned(),
	});
}

fn find_extensions_supporting_map_source_file_extension<'l>(
	list: &'l ExtensionList,
	file_extension: &str,
	allowed_formats: &Vec<&str>,
) -> Vec<(&'l ExtensionRef, String)>
{
	let extension_supported_formats: Vec<(&'l ExtensionRef, Vec<String>)> = list
		.iter()
		.filter_map(|ext| {
			let formats: Vec<String> =
				all_allowed_map_format_names_that_extension_supports_for_file_extension(
					ext,
					file_extension,
					allowed_formats,
				);

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

// Expects that no extension is being mutably accessed.
fn extension_supports_format(extension: &ExtensionRef, format_name: &str) -> bool
{
	let ext_ref = extension
		.get_extension()
		.expect("Could not get non-mutable reference to extension");

	return ext_ref
		.get_api_endpoints()
		.map_format_api
		.supports_loading_format(format_name);
}

// Expects that no extension is being mutably accessed.
fn all_allowed_map_format_names_that_extension_supports_for_file_extension(
	extension: &ExtensionRef,
	file_extension: &str,
	allowed_formats: &Vec<&str>,
) -> Vec<String>
{
	let ext_ref = extension
		.get_extension()
		.expect("Could not get non-mutable reference to extension");

	return ext_ref
		.get_api_endpoints()
		.map_format_api
		.supported_formats_for_file_extension(file_extension, allowed_formats);
}

fn all_supported_map_formats(list: &ExtensionList) -> Vec<String>
{
	let mut formats: HashSet<String> = HashSet::new();

	list.iter().for_each(|ext| {
		let ext_ref = ext
			.get_extension()
			.expect("Could not get non-mutable reference to extension");

		ext_ref
			.get_api_endpoints()
			.map_format_api
			.supported_format_names()
			.into_iter()
			.for_each(|fmt| {
				formats.insert(fmt);
			});
	});

	return formats.into_iter().collect();
}

// Strings contains the name of the map format, followed by all file types that
// the extension expects to represent the format.
// For example: "format_name (.map, .rmf, .txt)"
fn all_supported_map_format_description_strings(list: &ExtensionList) -> Vec<String>
{
	let mut format_to_exts: HashMap<String, HashSet<String>> = HashMap::new();

	list.iter().for_each(|ext| {
		let ext_ref = ext
			.get_extension()
			.expect("Could not get non-mutable reference to extension");

		ext_ref
			.get_api_endpoints()
			.map_format_api
			.supported_formats()
			.into_iter()
			.for_each(|spec| {
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
