use std::ffi::c_uchar;
use std::marker::PhantomData;

/// Shim wrapper to allow passing a slice of bytes across a library boundary.
/// The bytes are immutable.
#[repr(C)]
pub struct BytesRef<'l>
{
	begin: *const c_uchar,
	length: usize,
	phantom: PhantomData<&'l c_uchar>,
}

impl<'l> BytesRef<'l>
{
	pub fn new(slice: &'l [u8]) -> Self
	{
		return Self {
			begin: slice.as_ptr(),
			length: slice.len(),
			phantom: PhantomData,
		};
	}

	pub fn as_slice(&self) -> &'l [u8]
	{
		// The only way for this struct to be created is from an existing slice,
		// and this struct cannot live longer than the existing slice.
		// This means it's always OK to create a new slice that cannot live
		// longer than this struct. The data pointer and length here are also
		// taken directly from a valid slice, so the data itself is valid.
		return unsafe { std::slice::from_raw_parts(self.begin, self.length) };
	}
}
