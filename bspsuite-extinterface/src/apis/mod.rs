use bspffi::types::XCStr;

pub mod log_api;
pub mod map_format_api;
pub mod probe_api;
pub mod resource_format_api;

pub struct ApiInfo
{
	pub name: XCStr<'static>,
	pub version: u64,
}

impl ApiInfo
{
	pub const fn new(name: &'static str, version: u64) -> Self
	{
		return Self {
			name: XCStr::new(name),
			version: version,
		};
	}
}
