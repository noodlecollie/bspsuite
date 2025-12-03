use std::collections::HashMap;
use std::path::PathBuf;

use log::{error, info};

use super::types::{BaseArgs, ResultCode};
use super::utils::wrap_panics;
use crate::extensions::{ExtensionList, ExtensionRef};
use crate::toolchain::Toolchain;

#[repr(C)]
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
		let found_ext: Option<&ExtensionRef> = extensions.find_by_name(ext_name);

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
	let extension = ext_ref.get_extension().unwrap();

	info!("Extension");
	info!("=========");

	info!("  Name: {}", extension.get_name());
	info!(
		"  Path: {}",
		extension.get_path().to_str().unwrap_or("<path error>")
	);

	// TODO: List supported map formats
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
