use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use log::info;

use crate::commands::utils::wrap_residual_errors;
use crate::commands::{BaseArgs, ResultCode};
use crate::extensions::{Extension, ExtensionCollection, FormatCollector, FormatLoader};
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
	let toolchain: Toolchain = Toolchain::new(&args.base.toolchain_root)?;

	if args.extension_name.is_none()
	{
		list_extensions(&toolchain);
		return Ok(());
	}

	let ext_name: &str = args.extension_name.as_ref().unwrap();
	let extensions: &ExtensionCollection = toolchain.extensions();

	return match extensions.get_extension(&ext_name)
	{
		Some(ext) => extension_info(&extensions, ext)
			.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::InternalError, err)),
		None => Err(CompilerError::from_anyhow(
			CompilerErrorCode::ArgumentError,
			anyhow!("Could not find extension with name \"{ext_name}\""),
		)),
	};
}

fn extension_info(extensions: &ExtensionCollection, extension: &Extension) -> Result<()>
{
	let name: &str = extension.name();
	let path: &Path = extension.path();

	let vfs_types: Vec<String> = get_vfs_types(extensions, name);
	let resource_formats: HashMap<String, Vec<String>> = get_resource_formats(extensions, name);
	let map_formats: Vec<String> = get_map_formats(extensions, name);

	info!("Extension: {name}");
	info!("  Path: {}", path.display());
	info!("  VFS types: {}", display_string(vfs_types));
	info!("  Map formats: {}", display_string(map_formats));

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

fn get_vfs_types(extensions: &ExtensionCollection, name: &str) -> Vec<String>
{
	return extensions
		.vfs_formats()
		.endpoint_from_extension(name)
		.unwrap()
		.get_supported_vfs_types()
		.into_iter()
		.map(|str| str.to_owned())
		.collect();
}

fn get_map_formats(extensions: &ExtensionCollection, name: &str) -> Vec<String>
{
	return extensions
		.map_formats()
		.endpoint_from_extension(name)
		.unwrap()
		.supported_formats()
		.into_iter()
		.map(|spec| -> String {
			if !spec.associated_file_extensions.is_empty()
			{
				format!(
					"{} (.{})",
					spec.format_name,
					spec.associated_file_extensions.join(", .")
				)
				.into()
			}
			else
			{
				spec.format_name.to_string()
			}
		})
		.collect();
}

fn get_resource_formats(
	extensions: &ExtensionCollection,
	name: &str,
) -> HashMap<String, Vec<String>>
{
	let mut formats_map: HashMap<String, Vec<String>> = HashMap::new();

	let image_formats: Vec<String> = extensions
		.image_formats()
		.endpoint_from_extension(name)
		.unwrap()
		.get_supported_image_format_defs()
		.iter()
		.map(|(name, def)| -> String {
			if !def.file_extensions.is_empty()
			{
				format!("{name} (.{})", def.file_extensions.join(", .")).into()
			}
			else
			{
				name.to_string()
			}
		})
		.collect();

	formats_map.insert("Image".into(), image_formats);
	return formats_map;
}

fn list_extensions(toolchain: &Toolchain)
{
	let extensions: &ExtensionCollection = toolchain.extensions();
	let ext_dir: PathBuf = ExtensionCollection::extensions_directory(toolchain.root_path());
	info!("Extensions found in {}:", ext_dir.display());

	if extensions.num_extensions() < 1
	{
		info!("  None.");
		return;
	}

	let mut ext_paths: HashMap<String, PathBuf> = HashMap::new();
	let mut max_name_length: usize = 0;

	extensions.extensions_iter().for_each(|extension| {
		let ext_name: &str = extension.name();
		let ext_path: &Path = extension.path();

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

fn display_string(vec: Vec<String>) -> String
{
	return if !vec.is_empty()
	{
		vec.join(", ")
	}
	else
	{
		"None".into()
	};
}
