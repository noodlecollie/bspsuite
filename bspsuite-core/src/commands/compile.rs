use super::types::{BaseArgs, ResultCode};
use super::utils::wrap_panics;
use crate::extensions::{ExtensionList, extension_routines};
use crate::toolchain::Toolchain;
use bspextifc::StringRef;
use log::info;

#[repr(C)]
pub struct CompileArgs<'l>
{
	pub base: BaseArgs<'l>,
	pub input_file: StringRef<'l>,
}

#[unsafe(no_mangle)]
pub extern "C" fn bspcore_run_compile(args: &CompileArgs) -> ResultCode
{
	return wrap_panics(|| {
		let toolchain: Toolchain = Toolchain::new(&args.base.toolchain_root_path());
		let extensions: ExtensionList = toolchain.find_extensions();

		extension_routines::register_map_formats(&extensions);

		info!("Compile complete");
		return ResultCode::Ok;
	});
}
