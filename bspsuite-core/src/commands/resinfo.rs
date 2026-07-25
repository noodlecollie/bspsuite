use anyhow::anyhow;

use super::types::{BaseArgs, ResultCode};
use super::utils::wrap_residual_errors;
use crate::compiler_error::CompilerError;
use crate::extensions::ExtensionList;
use crate::extensions::extension_routines;
use crate::toolchain::Toolchain;

pub struct ResinfoArgs
{
	pub base: BaseArgs,
	pub game: String,
	pub vfs_roots: Vec<String>,
	pub resource_path: String,
}

pub fn bspcore_run_resinfo(args: &ResinfoArgs) -> ResultCode
{
	return wrap_residual_errors(|| run_resinfo(args));
}

fn run_resinfo(args: &ResinfoArgs) -> Result<(), CompilerError>
{
	let toolchain: Toolchain = Toolchain::new(&args.base.toolchain_root);
	let extensions: ExtensionList = toolchain.find_extensions();
	extension_routines::register_all_formats(&extensions);

	// TODO: We now want to get a stat result for the resource in question.
	return Err(CompilerError::from_anyhow(
		crate::CompilerErrorCode::InternalError,
		anyhow!("TODO: Continue"),
	));
}
