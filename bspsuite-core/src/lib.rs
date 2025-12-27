use const_cstr::{ConstCStr, const_cstr};
use constcat::concat;

mod compiler_error;
mod extensions;
mod game_configs;
mod model;
mod toolchain;

pub mod commands;

pub static BUILD_IDENTIFIER: ConstCStr =
	const_cstr!(concat!(env!("BUILD_DATE"), " ", env!("VCS_HASH")));
