mod compile;
mod extinfo;
mod types;
mod utils;

use std::ffi::c_char;

pub use compile::{CompileArgs, bspcore_run_compile};
pub use extinfo::{ExtinfoArgs, bspcore_run_extinfo};
pub use types::{BaseArgs, ResultCode};

use crate::BUILD_IDENTIFIER;

#[unsafe(no_mangle)]
pub extern "C" fn bspcore_get_build_identifier_string() -> *const c_char
{
	return BUILD_IDENTIFIER.as_ptr();
}
