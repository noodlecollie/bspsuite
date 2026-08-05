use anyhow::anyhow;

use super::types::{BaseArgs, ResultCode};
use super::utils::wrap_residual_errors;
use crate::extensions::ExtensionCollection;
use crate::toolchain::Toolchain;
use crate::{CompilerError, CompilerErrorCode};

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
	let toolchain: Toolchain = Toolchain::new(&args.base.toolchain_root)?;
	let extensions: ExtensionCollection =
		ExtensionCollection::load_extensions_from(toolchain.root_path().as_path())
			.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::IoError, err))?;

	// TODO: We now want to get a stat result for the resource in question.
	return Err(CompilerError::from_anyhow(
		crate::CompilerErrorCode::InternalError,
		anyhow!("TODO: Continue"),
	));
}
