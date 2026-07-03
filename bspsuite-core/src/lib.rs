// We allow dead code specifically in this crate,
// because it's a library under active development,
// and so the linter warnings for unused functions get quite noisy.
// This can be removed once we reach v1.0.0.
#![allow(dead_code)]

use const_cstr::{ConstCStr, const_cstr};
use constcat::concat;

mod compile_context;
mod compiler_error;
mod configs;
mod extensions;
mod io;
mod math;
mod model;
mod toolchain;

pub mod commands;
pub use io::{map_geometry_file, map_source_file};
pub use model::*;

pub static BUILD_IDENTIFIER: ConstCStr =
	const_cstr!(concat!(env!("BUILD_DATE"), " ", env!("VCS_HASH")));
