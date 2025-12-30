use super::opaque_ptr::OpaqueMutPtr;
use bspextifc::dummy_api::internal::{FfiTable, create_dummy_api};
use bspextifc::dummy_api::{DummyApi, DummyApiCallbacks};
use std::ffi::c_void;

pub struct DummyApiEndpoint
{
	inner: DummyApiCallbacks,
}

impl DummyApiEndpoint
{
	pub fn new(callbacks: DummyApiCallbacks) -> Self
	{
		return Self { inner: callbacks };
	}

	pub fn call_entry_point(&self)
	{
		let mut api_impl: DummyApiImpl = DummyApiImpl::new(42);
		let mut context: OpaqueMutPtr<DummyApiImpl> = OpaqueMutPtr::new(&mut api_impl);
		let ffi_table: FfiTable = ffi_impl::create_ffi_table(&mut context);
		let mut api: DummyApi = create_dummy_api(ffi_table);
		(self.inner.entry_point)(&mut api);
	}
}

struct DummyApiImpl
{
	magic_number: i32,
	numbers: Vec<i32>,
}

impl DummyApiImpl
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

mod ffi_impl
{
	use super::*;

	pub(super) fn create_ffi_table<'l>(api_impl: &'l mut OpaqueMutPtr<DummyApiImpl>)
	-> FfiTable<'l>
	{
		return FfiTable {
			context: api_impl.as_mut_void_ref(),
			store_number_fn: store_number,
			get_magic_number_fn: get_magic_number,
		};
	}

	unsafe extern "C" fn store_number(context: *mut c_void, value: i32)
	{
		unsafe { (*context.cast::<DummyApiImpl>()).store_number(value) };
	}

	unsafe extern "C" fn get_magic_number(context: *const c_void) -> i32
	{
		return unsafe { (*context.cast::<DummyApiImpl>()).get_magic_number() };
	}
}
