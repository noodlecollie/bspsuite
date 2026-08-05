use crate::ApiInfo;
use crate::builders::map_source_builder::MapSourceBuilderApiProvider;
use bspffi::types::{XCSlice, XCStr};
use thin_trait_object::thin_trait_object;

pub const API_INFO: ApiInfo = ApiInfo::new("MapFormatApi", 1);
pub type RegisterMapFormatsFn = extern "C" fn(&mut MapFormatApiProvider);
pub type ParseMapFn = extern "C" fn(&XCStr, &mut MapSourceBuilderApiProvider);

#[repr(C)]
pub struct MapFormatApiCallbacks
{
	pub register_map_formats: RegisterMapFormatsFn,
}

// TODO: Docs
#[thin_trait_object(drop_abi = "C", trait_object(pub MapFormatApiProvider))]
pub trait MapFormatApi
{
	fn register_map_format(
		&mut self,
		format_name: &XCStr,
		file_extensions: &XCSlice<XCStr>,
		parse_fn: ParseMapFn,
	);
}
