use super::extensions::ExtensionList;
use log::debug;
use std::path::PathBuf;

pub struct Toolchain
{
	root: PathBuf,
}

impl Toolchain
{
	pub fn new(toolchain_root: &Option<PathBuf>) -> Self
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
			root_path.to_str().unwrap_or("<unknown>"),
			if toolchain_root.is_some()
			{
				"user-specified"
			}
			else
			{
				"inferred from executable directory"
			}
		);

		return Self {
			root: root_path.clone(),
		};
	}

	pub fn root_path(&self) -> &PathBuf
	{
		return &self.root;
	}

	pub fn find_extensions(&self) -> ExtensionList
	{
		return ExtensionList::new(&self.root);
	}

	fn infer_toolchain_root() -> PathBuf
	{
		let exe_path: PathBuf =
			std::env::current_exe().expect("Could not get path to current executable");

		return exe_path.parent().unwrap().to_path_buf();
	}
}
