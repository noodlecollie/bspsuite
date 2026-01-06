use bspffi::types::XCStr;

pub struct ApiInfo
{
	pub name: XCStr<'static>,
	pub version: usize,
}

impl ApiInfo
{
	pub const fn new(name: &'static str, version: usize) -> Self
	{
		return Self {
			name: XCStr::new(name),
			version: version,
		};
	}
}
