use super::types::{BaseArgs, ResultCode};
use super::utils::wrap_panics;
use crate::pipeline::{ExtensionFeature, PipelineBuilder};
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
		let pipeline = PipelineBuilder::new(&args.base.toolchain_root)
			.require_feature(ExtensionFeature::DummyFeature)
			.finalise();

		info!("Compile complete");
		return ResultCode::Ok;
	});
}
