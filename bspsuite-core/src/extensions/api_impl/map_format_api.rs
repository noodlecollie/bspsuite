use super::opaque_ptr::OpaqueMutPtr;
use crate::extensions::extension::ExtensionRc;
use crate::extensions::extension_resource::ExtensionResource as StrongCb;
use bspextifc::{StringRef, map_format_api};
use log::{debug, warn};
use std::ffi::c_void;

pub struct StrongCallbacks
{
	cb: StrongCb<map_format_api::Callbacks>,
}

impl StrongCallbacks
{
	pub fn new(extension: ExtensionRc, callbacks: map_format_api::Callbacks) -> Self
	{
		return Self {
			cb: StrongCb::new(extension, callbacks),
		};
	}

	pub fn register_map_formats(&self) -> Vec<MapFormatEntry>
	{
		let mut api_impl: ApiImpl = ApiImpl::new(self.cb.extension_rc());
		let mut context: OpaqueMutPtr<ApiImpl> = OpaqueMutPtr::new(&mut api_impl);

		let core_fns: map_format_api::internal::CoreFns = map_format_api::internal::CoreFns {
			context: context.as_mut_void_ref(),
			register_map_format_fn: register_map_format,
		};

		let mut api: map_format_api::Api =
			map_format_api::internal::create_map_format_api(core_fns);
		(self.cb.register_map_formats)(&mut api);

		return api_impl.finish();
	}
}

pub struct MapFormatEntry
{
	pub format_name: String,
	pub parse_fn: StrongCb<map_format_api::MapParseFn>,
}

struct ApiImpl
{
	extension: ExtensionRc,
	formats: Vec<MapFormatEntry>,
}

impl MapFormatEntry
{
	pub fn new(
		extension: ExtensionRc,
		format_name: String,
		parse_fn: map_format_api::MapParseFn,
	) -> Self
	{
		return Self {
			format_name: format_name,
			parse_fn: StrongCb::new(extension, parse_fn),
		};
	}
}

impl ApiImpl
{
	pub fn new(extesion: ExtensionRc) -> Self
	{
		return Self {
			extension: extesion,
			formats: Vec::new(),
		};
	}

	pub fn register_map_format(&mut self, format_name: String, parse_fn: map_format_api::MapParseFn)
	{
		if let Some(entry) = self
			.formats
			.iter_mut()
			.find(|entry| entry.format_name == format_name)
		{
			warn!(
				"Overriding existing registered parse function for extension {} map format {format_name}",
				self.extension.get_name()
			);
			entry.parse_fn = StrongCb::new(self.extension.clone(), parse_fn);
		}
		else
		{
			debug!(
				"Extension {} registered support for map format {format_name}",
				self.extension.get_name()
			);

			self.formats.push(MapFormatEntry::new(
				self.extension.clone(),
				format_name,
				parse_fn,
			));
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
	parse_fn: map_format_api::MapParseFn,
)
{
	unsafe {
		(*context.cast::<ApiImpl>()).register_map_format(format_name.to_string(), parse_fn);
	};
}
