mod compile;
mod info;
mod types;
mod utils;

use std::ffi::c_char;

pub use compile::{CompileArgs, bspcore_run_compile};
pub use info::{InfoArgs, bspcore_run_info};
pub use types::{BaseArgs, ResultCode};

use crate::BUILD_IDENTIFIER;

#[unsafe(no_mangle)]
pub extern "C" fn bspcore_get_build_identifier_string() -> *const c_char
{
	return BUILD_IDENTIFIER.as_ptr();
}
