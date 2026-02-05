use crate::traits::RefMarshaller;
use std::borrow::Borrow;
use std::ffi::c_uchar;
use std::marker::PhantomData;
use std::slice;

/// FFI-safe and extern "C"-safe reference to a Rust string.
///
/// An `XCStr` may only be constructed from a Rust `&str`, and may not live
/// longer than the `&str`.
///
/// Internally, the string is represented by a character pointer and a length.
#[repr(C)]
pub struct XCStr<'l>
{
	begin: *const c_uchar,
	length: usize,
	phantom: PhantomData<&'l str>,
}

impl<'l> XCStr<'l>
{
	/// Constructs a new `XCStr` from a `&str`.
	#[inline]
	#[must_use = "Constructed object was not used"]
	pub const fn new(value: &'l str) -> Self
	{
		let slice = value.as_bytes();

		return Self {
			begin: slice.as_ptr(),
			length: slice.len(),
			phantom: PhantomData,
		};
	}

	/// Creates a `&str` from the `XCstr`.
	///
	/// The `&str` may not live longer than the `XCStr` is was created from.
	#[inline]
	#[must_use = "Returned reference was not used"]
	pub fn as_str(&self) -> &'l str
	{
		// SAFETY: Assumes the original &str is still alive.
		// The lifetime 'l of the XCStr is tied to the lifetime
		// of the &str that was used to construct it.
		let slice: &[u8] = unsafe { slice::from_raw_parts(self.begin, self.length) };

		// SAFETY: Assumes the original &str is composed of valid UTF-8 text.
		// Rust Strings already assume that this is true, and self can only
		// be constructed from a Rust string.
		return unsafe { str::from_utf8_unchecked(slice) };
	}
}

impl<'l> RefMarshaller<'l, str> for XCStr<'l>
{
	/// Marshals a `&str`.
	#[inline]
	fn marshal_ref(value: &'l str) -> XCStr<'l>
	{
		return XCStr::new(value);
	}

	/// Unmarshals to a `&str`.
	#[inline]
	fn unmarshal_ref(&self) -> &str
	{
		return self.as_str();
	}
}

impl<'l> From<&'l str> for XCStr<'l>
{
	/// Converts a `&str` to a `XCStr`.
	#[inline]
	fn from(value: &'l str) -> Self
	{
		return XCStr::new(value);
	}
}

impl<'l> Borrow<str> for XCStr<'l>
{
	/// Borrows a `&str` from the `XCStr`.
	#[inline]
	fn borrow(&self) -> &'l str
	{
		return self.as_str();
	}
}

impl<'l> Into<String> for XCStr<'l>
{
	/// Converts an `XCStr` to a `String`.
	#[inline]
	fn into(self) -> String
	{
		return self.to_string();
	}
}

impl<'l> ToString for XCStr<'l>
{
	/// Creates a `String` from an `XCStr`.
	#[inline]
	fn to_string(&self) -> String
	{
		return self.as_str().to_owned();
	}
}
