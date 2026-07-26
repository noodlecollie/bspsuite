use std::fs;
use std::path::PathBuf;
use std::slice::Iter;

use crate::extensions::extension::{Extension, ExtensionRef};
use anyhow::{Context, Result, ensure};
use log::{debug, error, warn};

pub struct ExtensionList
{
	extensions: Vec<ExtensionRef>,
}

impl ExtensionList
{
	pub fn extensions_directory(toolchain_root: &PathBuf) -> PathBuf
	{
		return toolchain_root.join("extensions");
	}

	pub fn new(toolchain_root: &PathBuf) -> Self
	{
		let mut out: Self = Self {
			extensions: Vec::new(),
		};

		out.load_extensions_from(toolchain_root);
		return out;
	}

	pub fn len(&self) -> usize
	{
		return self.extensions.len();
	}

	pub fn iter(&self) -> Iter<'_, ExtensionRef>
	{
		return self.extensions.iter();
	}

	pub fn find_by_name(&self, name: &str) -> Option<&ExtensionRef>
	{
		return self.extensions.iter().find(|ext| ext.get_name() == name);
	}

	pub fn for_each_ref_or_warn<F>(&self, op_desc: &str, mut f: F)
	where
		F: FnMut(&Extension) -> Result<()>,
	{
		self.for_each_or_warn(op_desc, |extension_ref| {
			extension_ref
				.get_extension()
				.with_context(|| "Failed to get reference to extension")
				.and_then(|ext_ref| f(&ext_ref))
		});
	}

	pub fn for_each_mut_ref_or_warn<F>(&self, op_desc: &str, mut f: F)
	where
		F: FnMut(&mut Extension) -> Result<()>,
	{
		self.for_each_or_warn(op_desc, |extension_ref| {
			extension_ref
				.get_extension_mut()
				.with_context(|| "Failed to get mutable reference to extension")
				.and_then(|mut ext_ref| f(&mut ext_ref))
		});
	}

	pub fn for_each_ref_or_error<F>(&self, op_desc: &str, mut f: F) -> Result<()>
	where
		F: FnMut(&Extension) -> Result<()>,
	{
		self.for_each_or_error(op_desc, |extension_ref| {
			extension_ref
				.get_extension()
				.with_context(|| "Failed to get reference to extension")
				.and_then(|ext_ref| f(&ext_ref))
		})
	}

	pub fn for_each_mut_ref_or_error<F>(&self, op_desc: &str, mut f: F) -> Result<()>
	where
		F: FnMut(&mut Extension) -> Result<()>,
	{
		self.for_each_or_error(op_desc, |extension_ref| {
			extension_ref
				.get_extension_mut()
				.with_context(|| "Failed to get mutable reference to extension")
				.and_then(|mut ext_ref| f(&mut ext_ref))
		})
	}

	pub fn for_each_or_warn<F>(&self, op_desc: &str, mut f: F)
	where
		F: FnMut(&ExtensionRef) -> Result<()>,
	{
		self.iter().for_each(|ext_ref| {
			if let Err(err) = f(ext_ref)
			{
				let ext_name: &str = ext_ref.get_name();
				warn!("{op_desc} failed for extension {ext_name}. {err}");
			}
		});
	}

	pub fn for_each_or_error<F>(&self, op_desc: &str, mut f: F) -> Result<()>
	where
		F: FnMut(&ExtensionRef) -> Result<()>,
	{
		let num_failures: usize = self.iter().fold(0, |fail_count, ext_ref| {
			fail_count
				+ f(ext_ref).map(|_| 0).unwrap_or_else(|err| {
					error!(
						"{op_desc} failed for extension {}. {err}",
						ext_ref.get_name()
					);
					1
				})
		});

		ensure!(
			num_failures == 0,
			format!("{op_desc} failed for {num_failures} extensions")
		);

		return Ok(());
	}

	fn load_extensions_from(&mut self, toolchain_root: &PathBuf)
	{
		let extensions_dir: PathBuf = ExtensionList::extensions_directory(toolchain_root);
		let extensions_result: Result<Vec<PathBuf>> =
			ExtensionList::find_extensions(&extensions_dir);

		if let Err(err) = extensions_result
		{
			warn!("Failed to look up extensions on disk. {}", err);
			return;
		}

		let extension_paths: Vec<PathBuf> = extensions_result.unwrap();

		debug!(
			"Found {} extensions in {}",
			extension_paths.len(),
			extensions_dir.to_str().unwrap()
		);

		let extensions: Vec<Result<ExtensionRef>> =
			ExtensionList::load_extensions(&extension_paths);

		for extension in extensions.iter().filter(|ext| ext.is_err())
		{
			// TODO: Better error logging
			let err = extension.as_ref().err().unwrap();
			let source = err.source();

			if let Some(source) = source
			{
				warn!("{err} Source error: {source}");
			}
			else
			{
				warn!("{err}");
			}
		}

		let mut extensions: Vec<ExtensionRef> =
			extensions.into_iter().filter_map(|ext| ext.ok()).collect();

		// Retain only the extensions where probe succeeds.
		extensions.retain_mut(|ext_ref| {
			let ext_name: String = String::from(ext_ref.get_name());
			debug!("Probing extension {ext_name}");

			// Getting a mutable ref should always succeed,
			// since no-one else is using the extensions yet.
			ext_ref
				.get_extension_mut()
				.expect("Could not get mutable ref to extension")
				.probe()
				.map(|_| true)
				.unwrap_or_else(|err| {
					warn!("Probe failed for extension {ext_name}. {err}");
					false
				})
		});

		self.extensions = extensions;
	}

	fn find_extensions(root: &PathBuf) -> Result<Vec<PathBuf>>
	{
		let entries: fs::ReadDir = fs::read_dir(root).with_context(|| {
			format!(
				"Could not read extensions from directory {}",
				root.to_str().unwrap()
			)
		})?;

		let file_ext: &str = Extension::library_extension_for_platform();
		let mut out_paths: Vec<PathBuf> = Vec::new();

		for entry in entries
		{
			if let Ok(entry) = entry
			{
				let path: PathBuf = entry.path();

				if let Some(ext) = path.extension()
					&& ext == file_ext
				{
					out_paths.push(path.clone());
				}
			}
		}

		return Ok(out_paths);
	}

	fn load_extensions(paths: &Vec<PathBuf>) -> Vec<Result<ExtensionRef>>
	{
		return paths
			.iter()
			.map(|path| {
				ExtensionRef::load(path).map_err(|err| {
					err.context(format!(
						"Failed to load extension {}",
						path.to_str().unwrap()
					))
				})
			})
			.collect();
	}
}
