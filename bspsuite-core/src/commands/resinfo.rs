use super::types::{BaseArgs, ResultCode};
use super::utils::wrap_residual_errors;
use crate::extensions::ExtensionCollection;
use crate::extensions::VfsFileStatResult;
use crate::toolchain::Toolchain;
use crate::{CompilerError, CompilerErrorCode};
use log::info;

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

	let stat: VfsFileStatResult = extensions
		.vfs_formats()
		.stat(&args.resource_path)
		.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::IoError, err))?;

	info!("Resource: {}", args.resource_path);
	info!("  Name: {}", stat.name);
	info!("  Parent: {}", stat.parent_path);
	info!("  Is directory: {}", stat.is_directory);

	if !stat.is_directory
	{
		info!("  Size: {}", stat.file_size);
	}

	Ok(())
}
