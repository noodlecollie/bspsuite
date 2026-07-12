use std::collections::HashMap;
use std::ops::DerefMut;
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use log::info;

use super::types::{BaseArgs, ResultCode};
use super::utils::wrap_residual_errors;
use crate::extensions::extension_routines;
use crate::extensions::{Extension, ExtensionList, ExtensionRef};
use crate::toolchain::Toolchain;
use crate::{CompilerError, CompilerErrorCode};

pub struct ExtinfoArgs
{
	pub base: BaseArgs,
	pub extension_name: Option<String>,
}

pub fn bspcore_run_extinfo(args: &ExtinfoArgs) -> ResultCode
{
	return wrap_residual_errors(|| run_extinfo(args));
}

fn run_extinfo(args: &ExtinfoArgs) -> Result<(), CompilerError>
{
	let toolchain: Toolchain = Toolchain::new(&args.base.toolchain_root);
	let extensions: ExtensionList = toolchain.find_extensions();

	if args.extension_name.is_none()
	{
		list_extensions(toolchain.root_path(), &extensions);
		return Ok(());
	}

	let ext_name: &str = args.extension_name.as_ref().unwrap();
	let found_ext: Option<&ExtensionRef> = extensions.find_by_name(&ext_name);

	if found_ext.is_none()
	{
		return Err(CompilerError::from_anyhow(
			CompilerErrorCode::ArgumentError,
			anyhow!("Could not find extension with name \"{ext_name}\""),
		));
	}

	return extension_info(found_ext.unwrap())
		.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::InternalError, err));
}

fn extension_info(ext_ref: &ExtensionRef) -> Result<()>
{
	let mut extension = ext_ref.get_extension_mut().unwrap();
	let name: String = extension.get_name().to_string();
	let path: PathBuf = extension.get_path().into();

	extension_routines::register_all_formats_for_ext(extension.deref_mut())?;

	let resource_formats: HashMap<String, Vec<String>> =
		get_resource_formats(extension.deref_mut());
	let map_formats: Vec<String> = get_map_formats(extension.deref_mut());

	let formats_str: String = if !map_formats.is_empty()
	{
		map_formats.join(", ")
	}
	else
	{
		"None".into()
	};

	info!("Extension: {name}");
	info!("  Path: {}", path.display());
	info!("  Map formats: {formats_str}");

	for (key, value) in resource_formats.iter()
	{
		let value_str: String = if !value.is_empty()
		{
			value.join(", ")
		}
		else
		{
			"None".into()
		};

		info!("  {key} formats: {}", value_str);
	}

	Ok(())
}

fn get_map_formats(extension: &mut Extension) -> Vec<String>
{
	if let Some(map_format_api) = &mut extension.get_api_endpoints_mut().map_format_api
	{
		return map_format_api
			.get_supported_map_format_defs()
			.iter()
			.map(|(name, def)| {
				if !def.file_extensions.is_empty()
				{
					format!("{name} (.{})", def.file_extensions.join(", ."))
				}
				else
				{
					name.to_string()
				}
			})
			.collect();
	}
	else
	{
		return Vec::new();
	}
}

fn get_resource_formats(extension: &mut Extension) -> HashMap<String, Vec<String>>
{
	if let Some(resource_format_api) = &mut extension.get_api_endpoints_mut().resource_format_api
	{
		let image_formats: Vec<String> = resource_format_api
			.get_supported_image_format_defs()
			.iter()
			.map(|(name, def)| {
				if !def.file_extensions.is_empty()
				{
					format!("{name} (.{})", def.file_extensions.join(", ."))
				}
				else
				{
					name.to_string()
				}
			})
			.collect();

		let mut formats_map: HashMap<String, Vec<String>> = HashMap::new();
		formats_map.insert("Image".into(), image_formats);

		return formats_map;
	}
	else
	{
		return HashMap::new();
	}
}

fn list_extensions(toolchain_root: &PathBuf, extensions: &ExtensionList)
{
	let ext_dir: PathBuf = ExtensionList::extensions_directory(toolchain_root);
	info!("Extensions found in {}:", ext_dir.display());

	if extensions.len() < 1
	{
		info!("  None.");
		return;
	}

	let mut ext_paths: HashMap<String, PathBuf> = HashMap::new();
	let mut max_name_length: usize = 0;

	extensions.iter().for_each(|ext_ref| {
		let ext_name: &str = ext_ref.get_name();
		let ext = ext_ref
			.get_extension()
			.expect(&format!("Could not get ref to extension {ext_name}"));
		let ext_path: &PathBuf = ext.get_path();

		if ext_name.len() > max_name_length
		{
			max_name_length = ext_name.len()
		}

		ext_paths.insert(ext_name.into(), ext_path.into());
	});

	ext_paths.iter().for_each(|(name, filename)| {
		let padding: String = String::from(" ").repeat(max_name_length - name.len());

		info!("  {name}:{padding} {}", filename.display())
	});
}
