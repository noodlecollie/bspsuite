use crate::ApiInfo;
use crate::builders::map_source_builder::BoxedMapSourceBuilderApi;
use bspffi::types::{XCSlice, XCStr};
use thin_trait_object::thin_trait_object;

pub const API_INFO: ApiInfo = ApiInfo::new("MapFormatApi", 1);
pub type RegisterMapFormatsFn = extern "C" fn(&mut BoxedMapFormatApi);
pub type MapParseFn = extern "C" fn(&XCStr, &mut BoxedMapSourceBuilderApi);

#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum OperationError
{
	/// A required operation had not been started.
	OperationNotStarted,

	/// A previous operation had not been completed before starting a new one.
	OperationNotFinished,
}

#[repr(C)]
pub struct MapFormatApiCallbacks
{
	pub register_map_formats: RegisterMapFormatsFn,
}

// TODO: Docs
#[thin_trait_object(drop_abi = "C")]
pub trait MapFormatApi
{
	fn register_map_format(
		&mut self,
		format_name: &XCStr,
		file_extensions: &XCSlice<XCStr>,
		parse_fn: MapParseFn,
	);
}
