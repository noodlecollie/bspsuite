use const_cstr::{ConstCStr, const_cstr};
use constcat::concat;

mod compile_context;
mod compiler_error;
mod configs;
mod extensions;
mod io;
mod math;
mod model;
mod ops;
mod toolchain;

pub mod commands;

pub static BUILD_IDENTIFIER: ConstCStr =
	const_cstr!(concat!(env!("BUILD_DATE"), " ", env!("VCS_HASH")));
