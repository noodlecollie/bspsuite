use std::ffi::c_void;
use std::marker::PhantomData;

// Note that is is not the creation of a pointer that's considered unsafe
// behaviour, only the dereferencing of it.

pub struct OpaquePtr<'l, T>
{
	data: *const c_void,
	phantom: PhantomData<&'l T>,
}

pub struct OpaqueMutPtr<'l, T>
{
	data: *mut c_void,
	phantom: PhantomData<&'l T>,
}

impl<T> OpaquePtr<'_, T>
{
	pub fn new(obj: &T) -> Self
	{
		return Self {
			data: obj as *const T as *const c_void,
			phantom: PhantomData,
		};
	}

	pub fn as_ref(&self) -> &T
	{
		let typed_ptr: *const T = self.data as *const T;

		// SAFETY: We know the type of the data, and we know
		// that the object is still valid based on our lifetime.
		return unsafe { &*typed_ptr };
	}

	pub fn as_void_ref(&self) -> &*const c_void
	{
		return &self.data;
	}
}

impl<T> OpaqueMutPtr<'_, T>
{
	pub fn new(obj: &mut T) -> Self
	{
		return Self {
			data: obj as *mut T as *mut c_void,
			phantom: PhantomData,
		};
	}

	pub fn as_ref(&self) -> &T
	{
		let typed_ptr: *const T = self.data as *const T;

		// SAFETY: We know the type of the data, and we know
		// that the object is still valid based on our lifetime.
		return unsafe { &*typed_ptr };
	}

	pub fn as_mut_ref(&mut self) -> &mut T
	{
		let typed_ptr: *mut T = self.data as *mut T;

		// SAFETY: We know the type of the data, and we know
		// that the object is still valid based on our lifetime.
		return unsafe { &mut *typed_ptr };
	}

	pub fn as_mut_void_ref(&mut self) -> &*mut c_void
	{
		return &self.data;
	}
}
