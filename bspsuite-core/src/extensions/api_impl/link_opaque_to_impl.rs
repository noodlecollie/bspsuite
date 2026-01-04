use bspsuite_ffi::types::internal::ContextPtr;
use std::cell::RefCell;

pub trait LinkOpaqueToImpl<'l, OpaqueType, ImplType>
{
	fn new_context(api_impl: &'l RefCell<ImplType>) -> ContextPtr<'l, OpaqueType>;
	fn to_impl(&self) -> &RefCell<ImplType>;
}
