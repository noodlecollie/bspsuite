use crate::extensions::ExtensionList;

pub fn register_map_formats(list: &ExtensionList)
{
	return list.for_each_or_warn("Registering map formats", |ext_ref| {
		let mut ext_mut_ref = ext_ref.get_extension_mut()?;
		let ext_name: String = String::from(ext_mut_ref.get_name());
		let api_endpoints = ext_mut_ref.get_api_endpoints_mut();

		if let Some(map_format_api) = &mut api_endpoints.map_format_api
		{
			map_format_api.register_map_formats(&ext_name);
		}

		Ok(())
	});
}
