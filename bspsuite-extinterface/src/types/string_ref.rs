use std::ffi::c_uchar;
use std::marker::PhantomData;
use std::ops::Range;
use std::slice;

/// Shim wrapper to allow passing a string reference across a library boundary.
/// Requires taking ownership of the string in order to access its data.
#[repr(C)]
pub struct StringRef<'l>
{
	begin: *const c_uchar,
	length: usize,
	phantom: PhantomData<&'l c_uchar>,
}

impl<'l> From<&str> for StringRef<'l>
{
	fn from(value: &str) -> Self
	{
		let range: Range<*const c_uchar> = value.as_bytes().as_ptr_range();

		return Self {
			begin: range.start,
			length: (range.end as usize) - (range.start as usize),
			phantom: PhantomData,
		};
	}
}

impl<'l> Into<String> for StringRef<'l>
{
	fn into(self) -> String
	{
		return self.to_string();
	}
}

impl<'l> ToString for StringRef<'l>
{
	// This function *should* always be safe. It relies on the assumptions that:
	// 1. Self was constructed from an &str containing valid UTF-8 characters.
	// 2. The original &str is still alive at the time of the call.
	// The String class should ensure that point 1 is true, and the Rust
	// compiler should ensure that point 2 is true.
	fn to_string(&self) -> String
	{
		// SAFETY: Assumes the original &str is still alive.
		// Self is tied to the lifetime of the string ref used
		// to construct it, so this should be fine.
		let str_slice = unsafe { slice::from_raw_parts(self.begin, self.length) };

		// SAFETY: Assumes the original &str is composed of valid UTF-8 text.
		// Rust Strings already assume that this is true, and self can only
		// be constructed from a Rust string.
		return unsafe { String::from_utf8_unchecked(Vec::from(str_slice)) };
	}
}
