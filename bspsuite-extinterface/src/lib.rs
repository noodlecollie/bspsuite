// Code architecture of this module is informed by
// https://users.rust-lang.org/t/linking-issues-when-designing-a-dynamic-plugin-based-architecture/136388

mod apis;
mod build_generated;

pub mod builders;
pub mod types;

pub use apis::{ApiInfo, dummy_api, log_api, map_format_api, probe_api};

/// Struct whose sole responsibility is to expose a versioned entry point API to
/// users of an extension.
#[repr(C)]
pub struct ExtensionInfo
{
	pub ffi_version: u64,
	pub probe_api_version: usize,
	pub probe_fn: probe_api::ExtFnProbe,
}

/// Name of the library symbol that exposes the extension's interface
/// information.
pub const SYMBOL_EXTENSION_INFO: &[u8] = b"bspsuite_ext_info";

/// Name of the library symbol that exposes the version of the extension
/// information struct.
pub const SYMBOL_EXTENSION_INFO_VERSION: &[u8] = b"bspsuite_ext_info_version";

/// Type used to report the version of the extension info struct.
pub type ExtensionInfoVersionType = u64;

/// The version of the extension info struct that we expect to read.
pub const EXTENSION_INFO_VERSION: ExtensionInfoVersionType = 1;

// The version identifier for the underlying FFI mechanics.
// This is set at build time for this crate.
// In base 10, the format is [major][minor] with three digits for each
// component, 000-999.
pub const FFI_VERSION: u64 = build_generated::FFI_VERSION;

// The full version identifier for the underlying FFI mechanics,
// including the patch component.
// // In base 10, the format is [major][minor][patch] with three digits for each
// component, 000-999.
pub const FFI_FULL_VERSION: u64 = build_generated::FFI_FULL_VERSION;

/// Macro for implementing the required extension info symbols into a shared
/// library. Extensions should always use this macro, and should not attempt to
/// construct the info manually.
#[macro_export]
macro_rules! implement_extension_info {
	($probe:expr) => {
		#[doc(hidden)]
		#[allow(non_upper_case_globals)]
		#[unsafe(no_mangle)]
		pub static bspsuite_ext_info_version: $crate::ExtensionInfoVersionType =
			$crate::EXTENSION_INFO_VERSION;

		#[doc(hidden)]
		#[allow(non_upper_case_globals)]
		#[unsafe(no_mangle)]
		pub static bspsuite_ext_info: $crate::ExtensionInfo = $crate::ExtensionInfo {
			ffi_version: $crate::FFI_VERSION,
			probe_api_version: $crate::probe_api::API_VERSION,
			probe_fn: $probe,
		};
	};
}
