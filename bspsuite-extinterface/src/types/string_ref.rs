use std::ffi::c_uchar;
use std::marker::PhantomData;
use std::slice;

/// Shim wrapper to allow passing a string reference across a library boundary.
#[repr(C)]
pub struct StringRef<'l>
{
	begin: *const c_uchar,
	length: usize,
	phantom: PhantomData<&'l c_uchar>,
}

impl<'l> StringRef<'l>
{
	pub fn new(value: &'l str) -> Self
	{
		let slice = value.as_bytes();

		return Self {
			begin: slice.as_ptr(),
			length: slice.len(),
			phantom: PhantomData,
		};
	}

	// This function should always be safe. It relies on the assumptions that:
	// 1. Self was constructed from an &str containing valid UTF-8 characters.
	// 2. The original &str is still alive at the time of the call.
	// The String class should ensure that point 1 is true, and the Rust
	// compiler should ensure that point 2 is true.
	pub fn as_str(&self) -> &'l str
	{
		// SAFETY: Assumes the original &str is still alive.
		// Self is tied to the lifetime of the string ref used
		// to construct it, so this should be fine.
		let slice: &[u8] = unsafe { slice::from_raw_parts(self.begin, self.length) };

		// SAFETY: Assumes the original &str is composed of valid UTF-8 text.
		// Rust Strings already assume that this is true, and self can only
		// be constructed from a Rust string.
		return unsafe { str::from_utf8_unchecked(slice) };
	}
}

impl<'l> From<&'l str> for StringRef<'l>
{
	fn from(value: &'l str) -> Self
	{
		return StringRef::new(value);
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
	fn to_string(&self) -> String
	{
		return self.as_str().to_owned();
	}
}
