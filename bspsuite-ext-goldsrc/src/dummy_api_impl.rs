use bspextifc::dummy_api::{BoxedThinTraitTest, DummyApi, DummyApiCallbacks, ThinTraitTest};
use log::info;

pub fn create_callbacks() -> DummyApiCallbacks
{
	return DummyApiCallbacks {
		entry_point: entry_point,
	};
}

extern "C" fn entry_point(api: &mut DummyApi)
{
	api.thin_trait_test(&BoxedThinTraitTest::new(ThinTraitImplementer::new()));
}

struct ThinTraitImplementer
{
	value: String,
}

impl ThinTraitImplementer
{
	pub fn new() -> Self
	{
		return Self {
			value: "Thin trait implementer's stored string".into(),
		};
	}
}

impl ThinTraitTest for ThinTraitImplementer
{
	fn print_hello_world(&self)
	{
		info!(
			"Hello world from bspsuite-ext-goldsrc! The stored value in this implementer struct is: \"{}\"",
			self.value
		);
	}
}
