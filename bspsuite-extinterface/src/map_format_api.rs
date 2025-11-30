use super::api_info::ApiInfo;
use crate::StringRef;
use std::ffi::c_void;

pub const API_INFO: ApiInfo = ApiInfo::new("MapFormatApi", 1);
pub type RegisterMapFormatsFn = extern "C" fn(&mut Api);
pub type MapParseFn = extern "C" fn(); // TODO: Args

#[repr(C)]
pub struct Api<'l>
{
	fns: internal::CoreFns<'l>,
}

#[repr(C)]
pub struct Callbacks
{
	pub register_map_formats: RegisterMapFormatsFn,
}

impl<'l> Api<'l>
{
	pub fn register_map_format(&mut self, format_name: StringRef, parse_fn: MapParseFn)
	{
		unsafe { (self.fns.register_map_format_fn)(*self.fns.context, &format_name, parse_fn) };
	}
}

pub mod internal
{
	use super::*;

	#[repr(C)]
	pub struct CoreFns<'l>
	{
		pub context: &'l *mut c_void,

		pub register_map_format_fn: unsafe extern "C" fn(*mut c_void, &StringRef, MapParseFn),
	}

	// Called by the core library in order to create the dummy API struct.
	pub fn create_map_format_api<'l>(fns: internal::CoreFns<'l>) -> Api<'l>
	{
		return Api { fns: fns };
	}
}
