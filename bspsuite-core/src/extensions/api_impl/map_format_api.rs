use super::opaque_ptr::OpaqueMutPtr;
use crate::extensions::extension::ExtensionRc;
use crate::extensions::strong_callbacks::StrongCallbacks;
use bspextifc::map_format_api::{MapFormatApi, MapParseFn};
use bspextifc::{StringRef, map_format_api};
use log::{debug, warn};
use std::ffi::c_void;

pub struct MapFormatStrongCallbacks
{
	cb: StrongCallbacks<map_format_api::MapFormatCallbacks>,
}

impl MapFormatStrongCallbacks
{
	pub fn new(extension: ExtensionRc, callbacks: map_format_api::MapFormatCallbacks) -> Self
	{
		return Self {
			cb: StrongCallbacks::new(extension, callbacks),
		};
	}

	pub fn register_map_formats(&self) -> Vec<MapFormatEntry>
	{
		let mut api_impl: MapFormatApiImpl = MapFormatApiImpl::new(self.cb.extension().get_name());
		let mut context: OpaqueMutPtr<MapFormatApiImpl> = OpaqueMutPtr::new(&mut api_impl);

		let core_fns: map_format_api::internal::MapFormatApiCoreFns =
			map_format_api::internal::MapFormatApiCoreFns {
				context: context.as_mut_void_ref(),
				register_map_format_fn: register_map_format,
			};

		let mut api: MapFormatApi = map_format_api::internal::create_map_format_api(core_fns);
		(self.cb.register_map_formats)(&mut api);

		return api_impl.finish();
	}
}

pub struct MapFormatEntry
{
	pub format_name: String,
	pub parse_fn: MapParseFn,
}

struct MapFormatApiImpl<'l>
{
	extension_name: &'l str,
	formats: Vec<MapFormatEntry>,
}

impl MapFormatEntry
{
	pub fn new(format_name: String, parse_fn: MapParseFn) -> Self
	{
		return Self {
			format_name: format_name,
			parse_fn: parse_fn,
		};
	}
}

impl<'l> MapFormatApiImpl<'l>
{
	pub fn new(extension_name: &'l str) -> Self
	{
		return Self {
			extension_name: extension_name,
			formats: Vec::new(),
		};
	}

	pub fn register_map_format(&mut self, format_name: String, parse_fn: MapParseFn)
	{
		if let Some(entry) = self
			.formats
			.iter_mut()
			.find(|entry| entry.format_name == format_name)
		{
			warn!(
				"Overriding existing registered parse function for extension {} map format {format_name}",
				self.extension_name
			);
			entry.parse_fn = parse_fn;
		}
		else
		{
			debug!(
				"Extension {} registered support for map format {format_name}",
				self.extension_name
			);

			self.formats
				.push(MapFormatEntry::new(format_name, parse_fn));
		}
	}

	pub fn finish(self) -> Vec<MapFormatEntry>
	{
		return self.formats;
	}
}

unsafe extern "C" fn register_map_format(
	context: *mut c_void,
	format_name: &StringRef,
	parse_fn: MapParseFn,
)
{
	unsafe {
		(*context.cast::<MapFormatApiImpl>())
			.register_map_format(format_name.to_string(), parse_fn);
	};
}
