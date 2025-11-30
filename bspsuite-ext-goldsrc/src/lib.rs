use bspextifc::log_api::{self, ExtensionLogger};
use bspextifc::{dummy_api, implement_extension_info, probe_api};
use log::{error, info};

mod io;

implement_extension_info!(probe);

extern "C" fn probe(api: &mut probe_api::ProbeApi) -> probe_api::ProbeResult
{
	if !set_up_logger(api)
	{
		return probe_api::ProbeResult::Failure;
	}

	let dummy_callbacks: dummy_api::Callbacks = dummy_api::Callbacks {
		entry_point: dummyapi_entry_point,
	};

	if let Err(_) = api.register_dummy_api_callbacks(dummy_api::API_INFO.version, dummy_callbacks)
	{
		error!("Failed to register for dummy API");
		return probe_api::ProbeResult::Failure;
	}

	return probe_api::ProbeResult::Success;
}

extern "C" fn dummyapi_entry_point(api: &mut dummy_api::Api)
{
	let magic_number: i32 = api.get_magic_number();
	info!("Magic number from dummy API: {magic_number}");

	info!("Storing number 99 in dummy API");
	api.store_number(99);
}

fn set_up_logger(api: &mut probe_api::ProbeApi) -> bool
{
	return api
		.request_log_api(log_api::API_INFO.version)
		.map(|api| ExtensionLogger::assign_static_logger(api).is_ok())
		.is_ok();
}
