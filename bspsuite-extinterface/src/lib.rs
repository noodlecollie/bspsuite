// Code architecture of this module is informed by
// https://users.rust-lang.org/t/linking-issues-when-designing-a-dynamic-plugin-based-architecture/136388

mod apis;

pub mod containers;
pub mod types;

pub use apis::{ApiInfo, dummy_api, log_api, map_format_api, probe_api};

/// Struct whose sole responsibility is to expose a versioned entry point API to
/// users of an extension.
#[repr(C)]
pub struct ExtensionInfo
{
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
pub type ExtensionInfoVersionType = usize;

/// The version of the extension info struct that we expect to read.
pub const EXTENSION_INFO_VERSION: ExtensionInfoVersionType = 1;

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
			probe_api_version: $crate::probe_api::API_VERSION,
			probe_fn: $probe,
		};
	};
}
