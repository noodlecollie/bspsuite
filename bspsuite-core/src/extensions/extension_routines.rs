use crate::extensions::api_impl::map_format_api;
use crate::extensions::{ExtensionList, ExtensionRef};

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

pub fn find_extensions_supporting_format<'l>(
	list: &'l ExtensionList,
	format_name: &str,
) -> Vec<&'l ExtensionRef>
{
	return list
		.iter()
		.filter(|ext| extension_supports_format(ext, format_name))
		.collect();
}

pub fn find_extensions_supporting_source_file_extension<'l>(
	list: &'l ExtensionList,
	file_extension: &str,
	allowed_formats: &Vec<&str>,
) -> Vec<(&'l ExtensionRef, String)>
{
	let extension_supported_formats: Vec<(&'l ExtensionRef, Vec<String>)> = list
		.iter()
		.filter_map(|ext| {
			let formats: Vec<String> =
				extension_map_formats_for_file_extension(ext, file_extension, allowed_formats);

			if !formats.is_empty()
			{
				Some((ext, formats))
			}
			else
			{
				None
			}
		})
		.collect();

	let flat_supported_formats: Vec<Vec<(&'l ExtensionRef, String)>> = extension_supported_formats
		.into_iter()
		.map(|(ext, fmts)| {
			let flat_fmts: Vec<(&'l ExtensionRef, String)> =
				fmts.into_iter().map(|fmt| (ext, fmt)).collect();
			return flat_fmts;
		})
		.collect();

	let mut out: Vec<(&'l ExtensionRef, String)> = Vec::new();

	// Not sure if there's a more functional way of doing this,
	// but I gave it a good go and it got complicated,
	// so we're going imperative instead.
	for fmts in flat_supported_formats
	{
		out.reserve(fmts.len());
		out.extend(fmts);
	}

	return out;
}

// Expects that no extension is being mutably accessed.
fn extension_supports_format(extension: &ExtensionRef, format_name: &str) -> bool
{
	let ext_ref = extension
		.get_extension()
		.expect("Could not get non-mutable reference to extension");

	let map_format_api: Option<&map_format_api::Endpoint> =
		ext_ref.get_api_endpoints().map_format_api.as_ref();

	return map_format_api
		.map(|api| api.supports_map_format(format_name))
		.unwrap_or(false);
}

// Expects that no extension is being mutably accessed.
fn extension_map_formats_for_file_extension(
	extension: &ExtensionRef,
	file_extension: &str,
	allowed_formats: &Vec<&str>,
) -> Vec<String>
{
	let ext_ref = extension
		.get_extension()
		.expect("Could not get non-mutable reference to extension");

	let map_format_api: Option<&map_format_api::Endpoint> =
		ext_ref.get_api_endpoints().map_format_api.as_ref();

	if map_format_api.is_none()
	{
		return Vec::new();
	}

	let map_format_api: &map_format_api::Endpoint = map_format_api.unwrap();
	let file_extension_string: String = file_extension.to_owned();

	return map_format_api
		.get_supported_map_formats()
		.into_iter()
		.filter(|fmt| allowed_formats.contains(&fmt.as_ref()))
		.filter(|fmt| {
			map_format_api
				.get_definition_supported_file_extensions(fmt)
				.unwrap()
				.contains(&file_extension_string)
		})
		.collect();
}
