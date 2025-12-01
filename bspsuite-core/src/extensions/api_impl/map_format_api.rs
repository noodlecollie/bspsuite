use super::opaque_ptr::OpaqueMutPtr;
use bspextifc::{StringRef, map_format_api};
use log::{debug, warn};
use std::collections::HashMap;
use std::ffi::c_void;

pub struct Callbacks
{
	inner: map_format_api::Callbacks,
}

pub struct MapParseCallback
{
	parse_fn: map_format_api::MapParseFn,
}

impl Callbacks
{
	pub fn new(callbacks: map_format_api::Callbacks) -> Self
	{
		return Self { inner: callbacks };
	}

	pub fn register_map_formats(&self, extension_name: &str) -> HashMap<String, MapParseCallback>
	{
		let mut api_impl: ApiImpl = ApiImpl::new(extension_name);
		let mut context: OpaqueMutPtr<ApiImpl> = OpaqueMutPtr::new(&mut api_impl);

		let core_fns: map_format_api::internal::CoreFns = map_format_api::internal::CoreFns {
			context: context.as_mut_void_ref(),
			register_map_format_fn: register_map_format,
		};

		let mut api: map_format_api::Api =
			map_format_api::internal::create_map_format_api(core_fns);
		(self.inner.register_map_formats)(&mut api);

		return api_impl.finish();
	}
}

impl MapParseCallback
{
}

struct ApiImpl<'l>
{
	extension_name: &'l str,
	formats: HashMap<String, MapParseCallback>,
}

impl<'l> ApiImpl<'l>
{
	pub fn new(extesion_name: &'l str) -> Self
	{
		return Self {
			extension_name: extesion_name,
			formats: HashMap::new(),
		};
	}

	pub fn register_map_format(&mut self, format_name: &str, parse_fn: map_format_api::MapParseFn)
	{
		if let Some(_) = self.formats.insert(
			String::from(format_name),
			MapParseCallback { parse_fn: parse_fn },
		)
		{
			warn!(
				"Overriding existing registered parse function for extension {} map format {format_name}",
				self.extension_name
			);
		}
		else
		{
			debug!(
				"Extension {} registered support for map format {format_name}",
				self.extension_name
			);
		}
	}

	pub fn finish(self) -> HashMap<String, MapParseCallback>
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
		(*context.cast::<ApiImpl>())
			.register_map_format(format_name.to_string().as_ref(), parse_fn);
	};
}
