use std::cell::RefCell;

use bspextifc::dummy_api::internal::{FfiTable, create_dummy_api};
use bspextifc::dummy_api::{DummyApi, DummyApiCallbacks};

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
		let api_impl: RefCell<DummyApiImpl> = RefCell::new(DummyApiImpl::new(42));
		let ffi_table: FfiTable = ffi_impl::create_ffi_table(&api_impl);
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
	use crate::extensions::api_impl::{LinkOpaqueToImpl, link_opaque_to_impl};
	use bspextifc::dummy_api::internal::{Ctx, OpaqueContext};
	use bspsuite_ffi::types::internal::ContextPtr;

	link_opaque_to_impl!(OpaqueContext, DummyApiImpl);

	pub(super) fn create_ffi_table<'l>(api_impl: &'l RefCell<DummyApiImpl>) -> FfiTable<'l>
	{
		return FfiTable {
			context: Ctx::new_context(api_impl),
			store_number_fn: store_number,
			get_magic_number_fn: get_magic_number,
		};
	}

	unsafe extern "C" fn store_number(context: &mut Ctx, value: i32)
	{
		context.to_impl().borrow_mut().store_number(value);
	}

	unsafe extern "C" fn get_magic_number(context: &Ctx) -> i32
	{
		return context.to_impl().borrow().get_magic_number();
	}
}
