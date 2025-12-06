pub struct ApiInfo
{
	pub name: &'static str,
	pub version: usize,
}

impl ApiInfo
{
	pub const fn new(name: &'static str, version: usize) -> Self
	{
		return Self {
			name: name,
			version: version,
		};
	}
}
