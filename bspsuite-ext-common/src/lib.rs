use bspextifc::probe_api::ProbeApi;
use bspextifc::{
	implement_extension_info, implement_extension_logger, log_api, probe_api, vfs_api,
};
use log::error;

mod vfs_impl;

implement_extension_info!(probe);
implement_extension_logger!(ExtensionLogger);

extern "C" fn probe(api: &mut probe_api::BoxedProbeApi) -> probe_api::ProbeResult
{
	if !set_up_logger(api)
	{
		return probe_api::ProbeResult::Failure;
	}

	if let Err(_) =
		api.register_vfs_api_callbacks(vfs_api::API_INFO.version, vfs_impl::create_callbacks())
	{
		error!("Failed to register for VFS API.");
		return probe_api::ProbeResult::Failure;
	}

	return probe_api::ProbeResult::Success;
}

fn set_up_logger(api: &mut probe_api::BoxedProbeApi) -> bool
{
	return api
		.request_log_api(log_api::API_INFO.version)
		.map(|api| ExtensionLogger::assign_static_logger(api).is_ok())
		.is_ok();
}
