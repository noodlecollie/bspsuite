use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use bspffi::types::{XCOption, XCStr};

#[derive(Copy, Clone, Debug, strum::Display)]
#[repr(C)]
pub enum ResultCode
{
	/// Execution completed successfully.
	Ok = 0,

	/// Some unexpected error occurred during execution. This should never
	/// usually happen.
	InternalError = 1,

	/// The arguments provided when invoking the operation were not valid.
	ArgumentError = 2,

	/// There was an error configuring the compiler.
	ConfigError = 3,

	/// There was an error reading from or writing to disk.
	IoError = 4,
}

pub struct BaseArgs
{
	/// Directory under which the games and directories folders may
	/// be found. If this property is left invalid, the directory of
	/// the current executable is used.
	/// This should be fine for most cases, but if the bspcore library
	/// is being used as part of another application, it may not be
	/// adequate. In this case, the application should supply the
	/// relevant path here.
	pub toolchain_root: Option<PathBuf>,
}

impl Default for BaseArgs
{
	fn default() -> Self
	{
		return Self {
			toolchain_root: None,
		};
	}
}

pub struct InputPathMetadata
{
	pub full_path: PathBuf,
	pub directory_path: PathBuf,
	pub file_name: String,
	pub file_stem: String,
	pub file_ext: String,
}

impl InputPathMetadata
{
	pub fn new(input_path: &Path) -> Result<InputPathMetadata>
	{
		let directory_path: PathBuf = input_path
			.parent()
			.ok_or_else(|| anyhow!("Failed to compute directory of input file"))?
			.to_path_buf();

		let file_name: String = input_path
			.file_name()
			.ok_or_else(|| anyhow!("Failed to compute file name of input file"))?
			.to_str()
			.ok_or_else(|| anyhow!("Failed to convert file name of input file to string"))?
			.to_string();

		let file_stem: String = PathBuf::from(&file_name)
			.file_stem()
			.ok_or_else(|| anyhow!("Failed to compute file stem of input file"))?
			.to_str()
			.ok_or_else(|| anyhow!("Failed to convert file stem of input file to string"))?
			.to_string();

		let file_ext: String = PathBuf::from(&file_name)
			.extension()
			.ok_or_else(|| anyhow!("Failed to compute file extension of input file"))?
			.to_str()
			.ok_or_else(|| anyhow!("Failed to convert file extension of input file to string"))?
			.to_string();

		return Ok(InputPathMetadata {
			full_path: input_path.to_path_buf(),
			directory_path: directory_path,
			file_name: file_name,
			file_stem: file_stem,
			file_ext: file_ext,
		});
	}
}
