use bspextifc::map_format_api::{Api, Callbacks};
use bspextifc::types::StringRef;

mod map_valve220;

#[cfg(test)]
mod test_resources;

pub fn create_callbacks() -> Callbacks
{
	return Callbacks {
		register_map_formats: register_map_formats,
	};
}

pub extern "C" fn register_map_formats(api: &mut Api)
{
	api.register_map_format(StringRef::from("map_valve220"), map_valve220::parse);
}
