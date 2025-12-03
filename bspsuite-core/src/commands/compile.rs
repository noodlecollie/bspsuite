use crate::extensions::ExtensionList;
use crate::toolchain::Toolchain;

use super::types::{BaseArgs, ResultCode};
use super::utils::wrap_panics;
use crate::extensions::extension_routines;
use log::info;
use std::path::PathBuf;

#[repr(C)]
pub struct CompileArgs
{
	pub base: BaseArgs,
	pub input_file: PathBuf,
}

#[unsafe(no_mangle)]
pub extern "C" fn bspcore_run_compile(args: &CompileArgs) -> ResultCode
{
	return wrap_panics(|| {
		let toolchain: Toolchain = Toolchain::new(&args.base.toolchain_root);
		let extensions: ExtensionList = toolchain.find_extensions();

		extension_routines::register_map_formats(&extensions);

		info!("Compile complete");
		return ResultCode::Ok;
	});
}
