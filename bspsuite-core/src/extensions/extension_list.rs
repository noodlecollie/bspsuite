use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::slice::Iter;

use crate::extensions::extension::{Extension, ExtensionRef};
use log::{debug, error, warn};

pub struct ExtensionList
{
	extensions: Vec<ExtensionRef>,
}

impl ExtensionList
{
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

	pub fn for_each_or_error<F>(&self, op_desc: &str, mut f: F)
	where
		F: FnMut(&ExtensionRef) -> Result<()>,
	{
		self.iter().for_each(|ext_ref| {
			if let Err(err) = f(ext_ref)
			{
				let ext_name: &str = ext_ref.get_name();
				error!("{op_desc} failed for extension {ext_name}. {err}");
			}
		});
	}

	fn load_extensions_from(&mut self, toolchain_root: &PathBuf)
	{
		let extensions_dir: PathBuf = toolchain_root.join("extensions");
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
