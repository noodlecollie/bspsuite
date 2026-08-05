use bspextifc::map_format_api::{MapFormatApi, MapFormatApiCallbacks, MapFormatApiProvider};
use bspffi::types::XCStr;
mod map_valve220;

#[cfg(test)]
mod tests;

pub fn create_callbacks() -> MapFormatApiCallbacks
{
	return MapFormatApiCallbacks {
		register_map_formats: register_map_formats,
	};
}

pub extern "C" fn register_map_formats(api: &mut MapFormatApiProvider)
{
	api.register_map_format(
		&"valve220".into(),
		&(&[XCStr::from("map")]).into(),
		map_valve220::parse,
	);
}
