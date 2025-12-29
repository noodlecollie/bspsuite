use std::path::PathBuf;

use bspsuite_ffim::types::{XCOption, XCStr};

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

#[repr(C)]
pub struct BaseArgs<'l>
{
	/// Directory under which the games and directories folders may
	/// be found. If this property is left invalid, the directory of
	/// the current executable is used.
	/// This should be fine for most cases, but if the bspcore library
	/// is being used as part of another application, it may not be
	/// adequate. In this case, the application should supply the
	/// relevant path here.
	pub toolchain_root: XCOption<XCStr<'l>>,
}

impl<'l> BaseArgs<'l>
{
	pub fn toolchain_root_path(&self) -> Option<PathBuf>
	{
		return match &self.toolchain_root
		{
			XCOption::Some(path) => Some(PathBuf::from(path.as_str())),
			XCOption::None => None,
		};
	}
}

impl<'l> Default for BaseArgs<'l>
{
	fn default() -> Self
	{
		return Self {
			toolchain_root: XCOption::None,
		};
	}
}
