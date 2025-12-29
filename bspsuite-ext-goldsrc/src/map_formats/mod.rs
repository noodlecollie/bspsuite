use bspextifc::map_format_api::{Api, Callbacks};
use bspsuite_ffi::types::XCStr;
mod map_valve220;

#[cfg(test)]
mod tests;

pub fn create_callbacks() -> Callbacks
{
	return Callbacks {
		register_map_formats: register_map_formats,
	};
}

pub extern "C" fn register_map_formats(api: &mut Api)
{
	api.register_map_format(
		&"valve220".into(),
		&(&[XCStr::from("map")]).into(),
		map_valve220::parse,
	);
}
