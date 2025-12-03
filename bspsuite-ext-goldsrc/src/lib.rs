use bspextifc::log_api::{self, ExtensionLogger};
use bspextifc::{implement_extension_info, map_format_api, probe_api};
use log::error;

mod io;
mod map_formats;

implement_extension_info!(probe);

extern "C" fn probe(api: &mut probe_api::ProbeApi) -> probe_api::ProbeResult
{
	if !set_up_logger(api)
	{
		return probe_api::ProbeResult::Failure;
	}

	if let Err(_) = api.register_map_format_api_callbacks(
		map_format_api::API_INFO.version,
		map_formats::create_callbacks(),
	)
	{
		error!("Failed to register for map format API.");
	}

	return probe_api::ProbeResult::Success;
}

fn set_up_logger(api: &mut probe_api::ProbeApi) -> bool
{
	return api
		.request_log_api(log_api::API_INFO.version)
		.map(|api| ExtensionLogger::assign_static_logger(api).is_ok())
		.is_ok();
}
