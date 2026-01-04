use bspsuite_ffi::types::internal::ContextPtr;
use std::cell::RefCell;

pub trait LinkOpaqueToImpl<'l, OpaqueType, ImplType>
{
	fn new_context(api_impl: &'l RefCell<ImplType>) -> ContextPtr<'l, OpaqueType>;
	fn to_impl(&self) -> &RefCell<ImplType>;
}

// Annoying that this has to be a macro, but I couldn't get a nice impl for
// LinkOpaqueToImpl to work automatically just based on generic types. Until my
// Rust improves, this will do.
macro_rules! link_opaque_to_impl {
	($opaque:ty, $impl:ty) => {
		impl<'l> LinkOpaqueToImpl<'l, $opaque, $impl> for ContextPtr<'l, $opaque>
		{
			fn new_context(api_impl: &'l RefCell<$impl>) -> ContextPtr<'l, $opaque>
			{
				unsafe {
					return ContextPtr::new(api_impl);
				}
			}

			fn to_impl(&self) -> &RefCell<$impl>
			{
				unsafe {
					return *self.as_void_ptr().cast::<&RefCell<$impl>>();
				}
			}
		}
	};
}

pub(crate) use link_opaque_to_impl;
