use super::opaque_ptr::OpaqueMutPtr;
use bspextifc::dummy_api;
use std::ffi::c_void;

pub struct Endpoint
{
	inner: dummy_api::Callbacks,
}

impl Endpoint
{
	pub fn new(callbacks: dummy_api::Callbacks) -> Self
	{
		return Self { inner: callbacks };
	}

	pub fn entry_point(&self)
	{
		let mut api_impl: ApiImpl = ApiImpl::new(42);
		let mut context: OpaqueMutPtr<ApiImpl> = OpaqueMutPtr::new(&mut api_impl);

		let core_fns: dummy_api::internal::CoreFns = dummy_api::internal::CoreFns {
			context: context.as_mut_void_ref(),
			store_number_fn: store_number,
			get_magic_number_fn: get_magic_number,
		};

		let mut api: dummy_api::Api = dummy_api::internal::create_dummy_api(core_fns);
		(self.inner.entry_point)(&mut api);
	}
}

struct ApiImpl
{
	magic_number: i32,
	numbers: Vec<i32>,
}

impl ApiImpl
{
	pub fn new(magic_number: i32) -> Self
	{
		return Self {
			magic_number: magic_number,
			numbers: Vec::new(),
		};
	}

	pub fn get_magic_number(&self) -> i32
	{
		return self.magic_number;
	}

	pub fn store_number(&mut self, value: i32)
	{
		self.numbers.push(value);
	}
}

unsafe extern "C" fn store_number(context: *mut c_void, value: i32)
{
	unsafe { (*context.cast::<ApiImpl>()).store_number(value) };
}

unsafe extern "C" fn get_magic_number(context: *const c_void) -> i32
{
	return unsafe { (*context.cast::<ApiImpl>()).get_magic_number() };
}
