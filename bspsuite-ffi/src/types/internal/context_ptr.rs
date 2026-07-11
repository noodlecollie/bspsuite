use std::cell::RefCell;
use std::ffi::c_void;
use std::marker::PhantomData;

/// Wraps a reference to an OpaqueType object. The reference is stored
/// internally as a C void* pointer.
pub struct ContextPtr<'l, OpaqueType>
{
	ptr: *const c_void,
	phantom: PhantomData<&'l OpaqueType>,
}

impl<'l, OpaqueType> ContextPtr<'l, OpaqueType>
{
	// SAFETY: This function is unsafe because it is designed to erase the type of
	// the context reference passed to it. The OpaqueType stands in to replace it as
	// far as the compiler is concerned, but OpaqueType and Ctx are unrelated. The
	// user of the returned struct has the responsibility of knowing what the type
	// Ctx was when this function was called.
	#[inline]
	#[must_use = "Constructed object was not used"]
	pub unsafe fn new<Ctx>(context: &'l RefCell<Ctx>) -> Self
	{
		return Self {
			ptr: core::ptr::from_ref(context) as *const c_void,
			phantom: PhantomData,
		};
	}

	// SAFETY: The pointer returned by this function may be copied and end up living
	// longer than the original context object. It is the responsibility of the
	// caller to manage this risk.
	// The caller must also know what the type of the pointer was when the new()
	// function was originally called.
	#[inline]
	#[must_use = "Returned pointer was not used"]
	pub unsafe fn as_void_ptr(&self) -> *const c_void
	{
		return self.ptr;
	}
}
