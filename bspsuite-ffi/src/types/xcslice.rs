use crate::traits::RefMarshaller;
use std::borrow::Borrow;
use std::marker::PhantomData;

/// FFI-safe and extern "C"-safe reference to a Rust slice.
///
/// An `XCSlice<T>` may only be constructed from a Rust slice of type `T`, and
/// may not live longer than the slice.
///
/// Internally, the slice is represented by a base pointer and a length.
#[repr(C)]
pub struct XCSlice<'l, T>
where
	T: Sized,
{
	begin: *const T,
	length: usize,
	phantom: PhantomData<&'l [T]>,
}

impl<'l, T> XCSlice<'l, T>
{
	/// Constructs a new `XCSlice<T>` from a `&[T]`.
	#[inline]
	#[must_use]
	pub fn new(slice: &'l [T]) -> Self
	{
		return Self {
			begin: slice.as_ptr(),
			length: slice.len(),
			phantom: PhantomData,
		};
	}

	/// Creates a slice reference `&[T]` from the `XCSlice<T>`.
	///
	/// The slice reference may not live longer than the `XCSlice` it was
	/// created from.
	#[inline]
	#[must_use]
	pub fn as_slice(&self) -> &'l [T]
	{
		// SAFETY:
		// The only way for this struct to be created is from an existing slice,
		// and this struct cannot live longer than the existing slice.
		// This means it's always OK to create a new slice that cannot live
		// longer than this struct. The data pointer and length here are also
		// taken directly from a valid slice, so the data itself is valid.
		return unsafe { std::slice::from_raw_parts(self.begin, self.length) };
	}
}

impl<'l, T> RefMarshaller<'l, [T]> for XCSlice<'l, T>
{
	#[inline]
	/// Marshals a slice of `T` elements.
	fn marshal_ref(value: &'l [T]) -> XCSlice<'l, T>
	{
		return XCSlice::new(value);
	}

	#[inline]
	/// Unmarshals to a slice of `T` elements.
	fn unmarshal_ref(&self) -> &[T]
	{
		return self.as_slice();
	}
}

impl<'l, T> From<&'l [T]> for XCSlice<'l, T>
{
	/// Converts a `&[T]` to an `XCSlice<T>`.
	#[inline]
	fn from(value: &'l [T]) -> Self
	{
		return XCSlice::new(value);
	}
}

impl<'l, T, const LENGTH: usize> From<&'l [T; LENGTH]> for XCSlice<'l, T>
{
	/// Converts a `&[T]` with a compile-time constant length to an
	/// `XCSlice<T>`.
	#[inline]
	fn from(value: &'l [T; LENGTH]) -> Self
	{
		return XCSlice::new(value.as_slice());
	}
}

impl<'l, T> Into<&'l [T]> for XCSlice<'l, T>
{
	/// Converts an `XCSlice<T>` to a `&[T]`.
	#[inline]
	fn into(self) -> &'l [T]
	{
		return self.as_slice();
	}
}

impl<'l, T> Borrow<[T]> for XCSlice<'l, T>
{
	/// Borrows a `&[T]` from the `XCSlice`.
	#[inline]
	fn borrow(&self) -> &'l [T]
	{
		return self.as_slice();
	}
}
