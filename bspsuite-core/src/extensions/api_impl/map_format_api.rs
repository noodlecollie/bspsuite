use super::opaque_ptr::OpaqueMutPtr;
use bspextifc::map_format_api;
use bspextifc::types::StringRef;
use log::{debug, warn};
use std::collections::HashMap;
use std::ffi::c_void;

pub struct MapParseCallback
{
	// This callback must be encapsulated, since it's
	// copyable/cloneable and depends on the extension
	// library, but we have no way to codify this dependency!
	// Instead, we treat the callback as being owned
	// by the endpoint, which in turn is owned by the
	// extension.
	parse_fn: map_format_api::MapParseFn,
}

pub struct Endpoint
{
	inner: map_format_api::Callbacks,
	map_formats: HashMap<String, MapParseCallback>,
}

impl Endpoint
{
	pub fn new(callbacks: map_format_api::Callbacks) -> Self
	{
		return Self {
			inner: callbacks,
			map_formats: HashMap::new(),
		};
	}

	pub fn register_map_formats(&mut self, extension_name: &str)
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

		self.map_formats = api_impl.finish();
	}

	pub fn supports_map_format(&self, format_name: &str) -> bool
	{
		return self.map_formats.contains_key(format_name);
	}

	pub fn get_parse_callback(&self, format_name: &str) -> Option<&MapParseCallback>
	{
		return self.map_formats.get(format_name);
	}

	pub fn get_supported_map_formats(&self) -> Vec<String>
	{
		return self.map_formats.keys().map(|item| item.clone()).collect();
	}
}

impl MapParseCallback
{
	pub fn parse(&self, data: &str, builder: &mut map_format_api::MapBlueprintBuilder)
	{
		(self.parse_fn)(&StringRef::new(data), builder);
	}
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
				"Extension {} registered support for map format \"{format_name}\"",
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
