use bspextifc::StringRef;
use bspextifc::map_format_api::{Api, Callbacks};

mod goldsrcmap;

pub fn create_callbacks() -> Callbacks
{
	return Callbacks {
		register_map_formats: register_map_formats,
	};
}

pub extern "C" fn register_map_formats(api: &mut Api)
{
	api.register_map_format(StringRef::from("goldsrcmap"), goldsrcmap::parse);
}
