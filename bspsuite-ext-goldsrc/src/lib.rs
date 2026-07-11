use bspextifc::{
	dummy_api, implement_extension_info, implement_extension_logger, log_api, map_format_api,
	probe_api,
};
use log::error;

mod dummy_api_impl;
mod map_formats;

implement_extension_info!(probe);
implement_extension_logger!(ExtensionLogger);

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
		return probe_api::ProbeResult::Failure;
	}

	if let Err(_) = api.register_dummy_api_callbacks(
		dummy_api::API_INFO.version,
		dummy_api_impl::create_callbacks(),
	)
	{
		error!("Failed to register for dummy API.");
		return probe_api::ProbeResult::Failure;
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
