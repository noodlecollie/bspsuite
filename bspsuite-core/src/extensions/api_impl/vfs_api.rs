use std::path::{Path, PathBuf};

use crate::extensions::{
	ApiCollector, ExtensionFileFormatCollection, FormatCollector, FormatLoaderEndpoint,
};
use crate::extensions::{FormatLoader, FormatSpec};
use crate::utils::path_extension;
use anyhow::{Result, anyhow, bail, ensure};
use bspextifc::vfs_api::{
	VfsApi, VfsApiCallbacks, VfsApiProvider, VfsFileErrorCode, VfsFileRecipient,
	VfsFileRecipientProvider, VfsFileStats, VfsImplCallbacks, VfsInitResultCode, VfsStatRecipient,
	VfsStatRecipientProvider,
};
use bspffi::types::{XCBytes, XCOption, XCSlice, XCStr};
use glob::glob;
use log::warn;

pub(crate) type VfsFormatImplCollection = ApiCollector<VfsInitialiser, VfsApiEndpoint>;
type ExtensionVfsTypeCollection = ExtensionFileFormatCollection<VfsImplCallbacks>;

#[derive(Debug, PartialEq)]
pub(crate) struct VfsFileStatResult
{
	pub parent_path: String,
	pub name: String,
	pub is_directory: bool,
	pub file_size: usize,
}

type VfsInitialiser = extern "C" fn(real_root_node: &XCStr) -> VfsInitResultCode;

struct VfsApiImpl<'l>
{
	extension_name: String,
	vfs_impls: &'l mut ExtensionVfsTypeCollection,
}

impl<'l> VfsApiImpl<'l>
{
	pub fn new(extension_name: &str, vfs_impls: &'l mut ExtensionVfsTypeCollection) -> Self
	{
		return Self {
			extension_name: extension_name.into(),
			vfs_impls,
		};
	}
}

impl<'l> VfsApi for VfsApiImpl<'l>
{
	fn register_vfs(
		&mut self,
		name: &XCStr,
		file_extensions: &XCOption<XCSlice<XCStr>>,
		callbacks: VfsImplCallbacks,
	)
	{
		let file_extensions: &[XCStr] = file_extensions
			.as_ref_option()
			.map_or(&[], |xc_slice| xc_slice.as_slice());

		self.vfs_impls
			.add(name.as_str(), file_extensions, callbacks, true);
	}
}

enum VfsFileStatResultWrapper
{
	Ok(VfsFileStatResult),
	Err(VfsFileErrorCode),
}

enum VfsFileLoadResultWrapper
{
	Ok(Vec<u8>),
	Err(VfsFileErrorCode),
}

struct VfsStatRecipientImpl<'l>
{
	result_wrapper: &'l mut Option<VfsFileStatResultWrapper>,
}

impl<'l> VfsStatRecipientImpl<'l>
{
	pub fn new(result_wrapper: &'l mut Option<VfsFileStatResultWrapper>) -> Self
	{
		return Self { result_wrapper };
	}
}

impl<'l> VfsStatRecipient for VfsStatRecipientImpl<'l>
{
	fn submit_stats(&mut self, stats: &VfsFileStats)
	{
		*self.result_wrapper = Some(VfsFileStatResultWrapper::Ok(stats.into()));
	}

	fn set_error(&mut self, code: VfsFileErrorCode)
	{
		*self.result_wrapper = Some(VfsFileStatResultWrapper::Err(code));
	}
}

struct VfsFileRecipientImpl<'l>
{
	result_wrapper: &'l mut Option<VfsFileLoadResultWrapper>,
}

impl<'l> VfsFileRecipientImpl<'l>
{
	pub fn new(result_wrapper: &'l mut Option<VfsFileLoadResultWrapper>) -> Self
	{
		return Self { result_wrapper };
	}
}

impl<'l> VfsFileRecipient for VfsFileRecipientImpl<'l>
{
	fn submit_bytes(&mut self, bytes: &XCBytes)
	{
		*self.result_wrapper = Some(VfsFileLoadResultWrapper::Ok(Vec::from(bytes.as_slice())));
	}

	fn set_error(&mut self, code: VfsFileErrorCode)
	{
		*self.result_wrapper = Some(VfsFileLoadResultWrapper::Err(code));
	}
}

impl From<&VfsFileStats<'_>> for VfsFileStatResult
{
	fn from(value: &VfsFileStats) -> Self
	{
		return Self {
			parent_path: value.parent_path.to_string(),
			name: value.name.to_string(),
			is_directory: value.is_directory,
			file_size: value.file_size,
		};
	}
}

pub struct VfsApiEndpoint
{
	ext_name: String,
	inner: Option<VfsApiCallbacks>,

	// Each impl here is a different type of VFS (eg. directory, ZIP, WAD, etc.), by the order in
	// which they were registered.. Earlier impls will be queried before later impls.
	vfs_impls: Vec<(String, VfsImplCallbacks, Vec<String>)>,
}

impl VfsApiEndpoint
{
	pub fn new(extension_name: String, callbacks: Option<VfsApiCallbacks>) -> Self
	{
		return Self {
			ext_name: extension_name,
			inner: callbacks,
			vfs_impls: Vec::new(),
		};
	}

	pub fn get_registered_vfs_records(&self) -> Vec<(&str, &Vec<String>)>
	{
		return self
			.vfs_impls
			.iter()
			.map(|(vfs_type, _, file_extensions)| (vfs_type.as_str(), file_extensions))
			.collect();
	}

	pub fn get_supported_vfs_types(&self) -> Vec<&str>
	{
		return self.vfs_impls.iter().map(|item| item.0.as_str()).collect();
	}

	pub fn get_vfs_root_file_extensions(&self, vfs_type: &str) -> Option<&Vec<String>>
	{
		return self
			.vfs_impls
			.iter()
			.find_map(|item| (item.0 == vfs_type).then(|| &item.2));
	}

	pub fn exists(&self, sub_path: &str) -> bool
	{
		for (_, callbacks, _) in self.vfs_impls.iter().rev()
		{
			if (callbacks.exists)(&XCStr::from(sub_path))
			{
				return true;
			}
		}

		return false;
	}

	pub fn is_file(&self, sub_path: &str) -> bool
	{
		for (_, callbacks, _) in self.vfs_impls.iter().rev()
		{
			// The first impl that contains the item gets to decide what it is.
			if (callbacks.exists)(&XCStr::from(sub_path))
			{
				return (callbacks.is_file)(&XCStr::from(sub_path));
			}
		}

		return false;
	}

	pub fn is_directory(&self, sub_path: &str) -> bool
	{
		for (_, callbacks, _) in self.vfs_impls.iter().rev()
		{
			// The first impl that contains the item gets to decide what it is.
			if (callbacks.exists)(&XCStr::from(sub_path))
			{
				return (callbacks.is_directory)(&XCStr::from(sub_path));
			}
		}

		return false;
	}

	pub fn stat(&self, sub_path: &str) -> Result<Option<VfsFileStatResult>>
	{
		for (vfs_type, callbacks, _) in self.vfs_impls.iter().rev()
		{
			let mut result_wrapper: Option<VfsFileStatResultWrapper> = None;

			{
				let mut recipient: VfsStatRecipientProvider =
					VfsStatRecipientProvider::new(VfsStatRecipientImpl::new(&mut result_wrapper));

				(callbacks.stat)(&XCStr::from(sub_path), &mut recipient);
			}

			match result_wrapper
			{
				None =>
				{
					warn!(
						"Extension {} VFS impl {vfs_type} did not provide a stat result for {sub_path} - \
						this is an implementation error",
						self.ext_name,
					);

					continue;
				}
				Some(val) => match val
				{
					VfsFileStatResultWrapper::Ok(stat_result) => return Ok(Some(stat_result)),
					VfsFileStatResultWrapper::Err(err) => match err
					{
						// Skip VFSes where the path does not exist.
						VfsFileErrorCode::InvalidPath => continue,

						// We can't assume any other error is fine.
						_ => bail!("{err}"),
					},
				},
			}
		}

		// Not finding the item in any VFS is not an error.
		return Ok(None);
	}

	pub fn load_file(&self, sub_path: &str) -> Result<Option<Vec<u8>>>
	{
		for (vfs_type, callbacks, _) in self.vfs_impls.iter().rev()
		{
			let mut result_wrapper: Option<VfsFileLoadResultWrapper> = None;

			{
				let mut recipient: VfsFileRecipientProvider =
					VfsFileRecipientProvider::new(VfsFileRecipientImpl::new(&mut result_wrapper));

				(callbacks.load_file)(&XCStr::from(sub_path), &mut recipient);
			}

			match result_wrapper
			{
				None =>
				{
					warn!(
						"Extension {} VFS impl {vfs_type} did not provide a result for loading file {sub_path} - \
						this is an implementation error",
						self.ext_name,
					);

					continue;
				}
				Some(val) => match val
				{
					VfsFileLoadResultWrapper::Ok(bytes) => return Ok(Some(bytes)),
					VfsFileLoadResultWrapper::Err(err) => match err
					{
						// Skip VFSes where the path does not exist.
						VfsFileErrorCode::InvalidPath => continue,

						// We can't assume any other error is fine.
						_ => bail!("{err}"),
					},
				},
			}
		}

		// Not finding the item in any VFS is not an error.
		return Ok(None);
	}
}

impl FormatLoaderEndpoint for VfsApiEndpoint
{
	fn register_supported_formats(&mut self) -> bool
	{
		self.inner
			.as_ref()
			.map(|callbacks| {
				let mut impls: ExtensionVfsTypeCollection =
					ExtensionVfsTypeCollection::new(self.ext_name.clone(), "VFS type".into());

				{
					let mut api_impl: VfsApiProvider =
						VfsApiProvider::new(VfsApiImpl::new(&self.ext_name, &mut impls));

					(callbacks.register_vfs_support)(&mut api_impl);
				}

				self.vfs_impls = impls.collect();
				true
			})
			.unwrap_or(false)
	}
}

// The "format" here is the package type that may be used as a VFS root.
impl FormatLoader<VfsInitialiser> for VfsApiEndpoint
{
	type LoaderOutput = VfsInitResultCode;

	fn extension_name(&self) -> &str
	{
		return &self.ext_name;
	}

	fn supported_formats(&self) -> Vec<FormatSpec>
	{
		return self
			.vfs_impls
			.iter()
			.map(|(vfs_type, _, file_extensions)| FormatSpec {
				format_name: vfs_type.clone(),
				associated_file_extensions: file_extensions.clone(),
			})
			.collect();
	}

	fn supports_loading_format(&self, format_name: &str) -> bool
	{
		return self
			.vfs_impls
			.iter()
			.find(|item| item.0 == format_name)
			.is_some();
	}

	fn supports_loading_format_from_file(&self, format_name: &str, file_extension: &str) -> bool
	{
		return self
			.vfs_impls
			.iter()
			.find(|item| item.0 == format_name)
			.map_or(false, |(_, _, file_extensions)| {
				file_extensions.contains(&file_extension.to_owned())
			});
	}

	fn supported_file_extensions_for_format(&self, format_name: &str) -> Option<Vec<&str>>
	{
		return self
			.vfs_impls
			.iter()
			.find(|item| item.0 == format_name)
			.map(|(_, _, file_extensions)| {
				file_extensions.iter().map(|str| str.as_str()).collect()
			});
	}

	fn load_if_supported<Callback>(
		&self,
		format_name: &str,
		callback: Callback,
	) -> anyhow::Result<Self::LoaderOutput>
	where
		Callback: Fn(&VfsInitialiser) -> anyhow::Result<Self::LoaderOutput>,
	{
		let (_, callbacks, _) = &self
			.vfs_impls
			.iter()
			.find(|item| item.0 == format_name)
			.ok_or_else(|| anyhow!(format!("VFS format {format_name} is not supported")))?;

		Ok((callback)(&callbacks.initialise)?)
	}
}

impl VfsFormatImplCollection
{
	pub fn initialise_specific(&self, extension_name: &str, root_path: &PathBuf) -> Result<()>
	{
		let root_str: &str = root_path
			.to_str()
			.ok_or_else(|| anyhow!("Could not convert VFS root {} to str", root_path.display()))?;

		let root_ext: Option<String> = path_extension(root_path)?.map(|str| str.to_owned());

		let endpoint = self
			.endpoint_from_extension(extension_name)
			.ok_or_else(|| anyhow!("No extension loaded with name {extension_name}"))?;

		return endpoint
			.get_registered_vfs_records()
			.iter()
			.try_for_each(|vfs_record| {
				let use_vfs: bool = match &root_ext
				{
					Some(ext) => vfs_record.1.contains(ext),
					None => vfs_record.1.is_empty(),
				};

				return if use_vfs
				{
					endpoint
						.load_if_supported(vfs_record.0, |init_fn| {
							VfsFormatImplCollection::run_initialiser(
								vfs_record.0,
								init_fn,
								root_str,
							)
						})
						// The only code that comes through will be VfsInitResultCode::Ok.
						.map(|_| ())
				}
				else
				{
					// Skip
					Ok(())
				};
			});
	}

	pub fn initialise_all(&self, game_dir: &Path) -> Result<()>
	{
		ensure!(
			game_dir.is_absolute(),
			"VFS root directory must be absolute"
		);

		let game_dir_str: &str = game_dir.to_str().ok_or_else(|| {
			anyhow!(
				"Could not convert game directory {} to str",
				game_dir.display()
			)
		})?;

		return self.endpoints.iter().try_for_each(|endpoint| {
			endpoint
				.supported_formats()
				.into_iter()
				.try_for_each(|format_spec| {
					let roots: Vec<PathBuf> = if !format_spec.associated_file_extensions.is_empty()
					{
						VfsFormatImplCollection::find_roots_with_exts(
							game_dir_str,
							format_spec.associated_file_extensions.as_slice(),
						)
					}
					else
					{
						vec![game_dir.to_path_buf()]
					};

					roots.into_iter().try_for_each(|root| {
						let root_str: &str = root.to_str().ok_or_else(|| {
							anyhow!("Could not convert VFS root {} to str", root.display())
						})?;

						endpoint
							.load_if_supported(&format_spec.format_name, |init_fn| {
								VfsFormatImplCollection::run_initialiser(
									&format_spec.format_name,
									init_fn,
									root_str,
								)
							})
							// The only code that comes through will be VfsInitResultCode::Ok.
							.map(|_| ())
					})
				})
		});
	}

	pub fn exists(&self, sub_path: &str) -> bool
	{
		for endpoint in self.endpoints.iter()
		{
			if endpoint.exists(sub_path)
			{
				return true;
			}
		}

		return false;
	}

	pub fn is_directory(&self, sub_path: &str) -> bool
	{
		for endpoint in self.endpoints.iter()
		{
			if endpoint.is_directory(sub_path)
			{
				return true;
			}
		}

		return false;
	}

	pub fn is_file(&self, sub_path: &str) -> bool
	{
		for endpoint in self.endpoints.iter()
		{
			if endpoint.is_file(sub_path)
			{
				return true;
			}
		}

		return false;
	}

	pub fn stat(&self, sub_path: &str) -> Result<VfsFileStatResult>
	{
		for endpoint in self.endpoints.iter()
		{
			let result: Option<VfsFileStatResult> = endpoint.stat(sub_path)?;

			if result.is_some()
			{
				return Ok(result.unwrap());
			}
		}

		bail!("{sub_path} was not found");
	}

	pub fn load_file(&self, sub_path: &str) -> Result<Vec<u8>>
	{
		for endpoint in self.endpoints.iter()
		{
			let result: Option<Vec<u8>> = endpoint.load_file(sub_path)?;

			if result.is_some()
			{
				return Ok(result.unwrap());
			}
		}

		bail!("{sub_path} was not found");
	}

	fn run_initialiser(
		format_name: &str,
		init_fn: &VfsInitialiser,
		root_str: &str,
	) -> Result<VfsInitResultCode>
	{
		let result_code: VfsInitResultCode = init_fn(&XCStr::from(root_str));

		return match result_code
		{
			VfsInitResultCode::Ok => Ok(VfsInitResultCode::Ok),
			_ => Err(anyhow!(
				"Failed to initialise {format_name} VFS from {root_str}: {result_code}",
			)),
		};
	}

	fn find_roots_with_exts(root: &str, exts: &[String]) -> Vec<PathBuf>
	{
		let mut roots: Vec<PathBuf> = Vec::new();

		for ext in exts.iter()
		{
			let glob_path: String = format!("{root}/*.{ext}");

			let paths = glob(&glob_path)
				.map(|paths| {
					paths
						.into_iter()
						.filter_map(|entry| {
							if let Err(err) = &entry
							{
								warn!("Error encountred in VFS root glob for {glob_path}: {err}");
							}

							entry.ok()
						})
						.collect()
				})
				.unwrap_or_else(|err| {
					warn!("Failed to execute glob {glob_path} for VFS roots: {err}");
					Vec::new()
				});

			roots.extend(paths);
		}

		return roots;
	}
}
