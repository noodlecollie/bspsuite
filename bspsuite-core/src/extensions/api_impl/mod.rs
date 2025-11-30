mod opaque_ptr;

pub mod dummy_api;
pub mod log_api;
pub mod map_format_api;

pub use {
	dummy_api as dummy_api_impl, log_api as log_api_impl, map_format_api as map_format_api_impl,
};
