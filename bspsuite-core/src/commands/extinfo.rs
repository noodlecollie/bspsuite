use std::collections::HashMap;
use std::ops::DerefMut;
use std::path::PathBuf;

use log::{error, info};

use super::types::{BaseArgs, ResultCode};
use super::utils::wrap_panics;
use crate::extensions::{ApiEndpoints, Extension, ExtensionList, ExtensionRef};
use crate::toolchain::Toolchain;

pub struct ExtinfoArgs
{
	pub base: BaseArgs,
	pub extension_name: Option<String>,
}

#[unsafe(no_mangle)]
pub extern "C" fn bspcore_run_extinfo(args: &ExtinfoArgs) -> ResultCode
{
	return wrap_panics(|| {
		let toolchain: Toolchain = Toolchain::new(&args.base.toolchain_root);
		let extensions: ExtensionList = toolchain.find_extensions();

		if args.extension_name.is_none()
		{
			list_extensions(toolchain.root_path(), &extensions);
			return ResultCode::Ok;
		}

		let ext_name: &str = args.extension_name.as_ref().unwrap();
		let found_ext: Option<&ExtensionRef> = extensions.find_by_name(&ext_name);

		if found_ext.is_none()
		{
			error!("Could not find extension with name \"{ext_name}\"");
			return ResultCode::ArgumentError;
		}

		extension_info(found_ext.unwrap());
		return ResultCode::Ok;
	});
}

fn extension_info(ext_ref: &ExtensionRef)
{
	let mut extension = ext_ref.get_extension_mut().unwrap();
	let name: String = extension.get_name().to_string();
	let path: String = extension
		.get_path()
		.to_str()
		.unwrap_or("<path error>")
		.to_string();

	let formats: Vec<String> = get_map_formats(extension.deref_mut());
	let formats_str: String = formats.join(", ");

	info!("Extension: {name}");
	info!("  Path: {path}");
	info!("  Map formats: {formats_str}");
}

fn get_map_formats(extension: &mut Extension) -> Vec<String>
{
	let name: String = extension.get_name().to_string();
	let api_endpoints: &mut ApiEndpoints = extension.get_api_endpoints_mut();

	if let Some(map_format_api) = &mut api_endpoints.map_format_api
	{
		map_format_api.register_map_formats(&name);
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

fn list_extensions(toolchain_root: &PathBuf, extensions: &ExtensionList)
{
	let ext_dir: PathBuf = ExtensionList::extensions_directory(toolchain_root);
	info!("Extensions found in {}:", ext_dir.to_str().unwrap());

	if extensions.len() < 1
	{
		info!("  None.");
		return;
	}

	let mut ext_paths: HashMap<String, String> = HashMap::new();
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

		ext_paths.insert(
			String::from(ext_name),
			String::from(
				ext_path
					.file_name()
					.map_or(None, |os_str| os_str.to_str())
					.unwrap_or("<path error>"),
			),
		);
	});

	ext_paths.iter().for_each(|(name, filename)| {
		let padding: String = String::from(" ").repeat(max_name_length - name.len());

		info!("  {name}:{padding} {}", filename)
	});
}
