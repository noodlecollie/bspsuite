use bspextifc::resource_format_api::{BoxedResourceFormatApi, ResourceFormatApiCallbacks};

pub fn create_callbacks() -> ResourceFormatApiCallbacks
{
	return ResourceFormatApiCallbacks {
		register_resource_formats: register_resource_formats,
	};
}

extern "C" fn register_resource_formats(api: &mut BoxedResourceFormatApi)
{
}
