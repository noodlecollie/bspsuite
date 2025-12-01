use const_cstr::{ConstCStr, const_cstr};
use constcat::concat;

mod compiler_error;
mod extensions;
mod model;
mod pipeline;
mod toolchain;
mod work_units;

pub mod commands;

pub static BUILD_IDENTIFIER: ConstCStr =
	const_cstr!(concat!(env!("BUILD_DATE"), " ", env!("VCS_HASH")));
