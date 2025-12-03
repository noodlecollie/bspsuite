use std::path::PathBuf;

use log::info;

use super::types::{BaseArgs, ResultCode};
use super::utils::wrap_panics;
use crate::extensions::ExtensionList;
use crate::toolchain::Toolchain;

#[repr(C)]
pub struct InfoArgs
{
	pub base: BaseArgs,
	pub list_map_formats: bool,
}

#[unsafe(no_mangle)]
pub extern "C" fn bspcore_run_info(args: &InfoArgs) -> ResultCode
{
	return wrap_panics(|| {
		let toolchain: Toolchain = Toolchain::new(&args.base.toolchain_root);
		let extensions: ExtensionList = toolchain.find_extensions();

		list_extensions(&extensions);

		if args.list_map_formats
		{
			// TODO: List formats
		}

		return ResultCode::Ok;
	});
}

fn list_extensions(extensions: &ExtensionList)
{
	info!("Extensions:");

	extensions.iter().for_each(|ext_ref| {
		let ext_name: &str = ext_ref.get_name();
		let ext = ext_ref
			.get_extension()
			.expect(&format!("Could not get ref to extension {ext_name}"));
		let ext_path: &PathBuf = ext.get_path();

		info!(
			"  {ext_name}: {}",
			ext_path.to_str().unwrap_or("<path error>")
		)
	});
}
