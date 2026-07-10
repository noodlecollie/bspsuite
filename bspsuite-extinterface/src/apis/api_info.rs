use bspffi::types::XCStr;

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
