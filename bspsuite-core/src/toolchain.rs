use std::path::{Path, PathBuf};

use crate::extensions::ExtensionCollection;
use crate::{CompilerError, CompilerErrorCode};
use anyhow::{Result, ensure};
use log::debug;

pub(crate) struct Toolchain
{
	root: PathBuf,
	extension_collection: ExtensionCollection,
}

impl Toolchain
{
	pub fn new(toolchain_root: &Option<PathBuf>) -> Result<Self, CompilerError>
	{
		let root_path: PathBuf = if toolchain_root.is_some()
		{
			toolchain_root.as_ref().unwrap().clone()
		}
		else
		{
			Toolchain::infer_toolchain_root()
		};

		debug!(
			"Toolchain root path: {} ({})",
			root_path.display(),
			if toolchain_root.is_some()
			{
				"user-specified"
			}
			else
			{
				"inferred from executable directory"
			}
		);

		let extensions: ExtensionCollection =
			ExtensionCollection::load_extensions_in_directory(root_path.as_path())
				.map_err(|err| CompilerError::from_anyhow(CompilerErrorCode::IoError, err))?;

		return Ok(Self {
			root: root_path,
			extension_collection: extensions,
		});
	}

	pub fn root_path(&self) -> &PathBuf
	{
		return &self.root;
	}

	pub fn to_abs_path(&self, relative_path: &Path) -> Result<PathBuf>
	{
		ensure!(relative_path.is_relative(), "Expected path to be relative");
		return Ok(self.root.join(relative_path));
	}

	pub fn extensions(&self) -> &ExtensionCollection
	{
		return &self.extension_collection;
	}

	fn infer_toolchain_root() -> PathBuf
	{
		let exe_path: PathBuf =
			std::env::current_exe().expect("Could not get path to current executable");

		return exe_path.parent().unwrap().to_path_buf();
	}
}
