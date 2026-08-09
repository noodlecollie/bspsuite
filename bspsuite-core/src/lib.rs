// We allow dead code specifically in this crate,
// because it's a library under active development,
// and so the linter warnings for unused functions get quite noisy.
// This can be removed once we reach v1.0.0.
#![allow(dead_code)]

use constcat::concat;
pub static BUILD_IDENTIFIER: &str = concat!(env!("BUILD_DATE"), " ", env!("VCS_HASH"));

// We have one level of public modules, which act as categories under
// the root bspsuite module. Don't nest other public modules within these - we'd
// like to keep a maximum of two namespace levels before the actual item being
// used, eg. bspsuite::model::MapGeomFile.
pub mod commands;
pub mod configs;
pub mod extensions;
pub mod io;
pub mod math;
pub mod model;

pub(crate) mod utils;

pub use compiler_error::{CompilerError, CompilerErrorCode};

mod compile_context;
mod compiler_error;
mod toolchain;
