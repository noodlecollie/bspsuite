use std::marker::PhantomData;

/// Shim wrapper to allow passing a slice of bytes across a library boundary.
/// The bytes are immutable.
#[repr(C)]
pub struct SliceRef<'l, T>
{
	begin: *const T,
	length: usize,
	phantom: PhantomData<&'l T>,
}

impl<'l, T> SliceRef<'l, T>
{
	pub fn new(slice: &'l [T]) -> Self
	{
		return Self {
			begin: slice.as_ptr(),
			length: slice.len(),
			phantom: PhantomData,
		};
	}

	pub fn as_slice(&self) -> &'l [T]
	{
		// The only way for this struct to be created is from an existing slice,
		// and this struct cannot live longer than the existing slice.
		// This means it's always OK to create a new slice that cannot live
		// longer than this struct. The data pointer and length here are also
		// taken directly from a valid slice, so the data itself is valid.
		return unsafe { std::slice::from_raw_parts(self.begin, self.length) };
	}
}

impl<'l, T> From<&'l [T]> for SliceRef<'l, T>
{
	fn from(value: &'l [T]) -> Self
	{
		return SliceRef::new(value);
	}
}

impl<'l, T, const LENGTH: usize> From<&'l [T; LENGTH]> for SliceRef<'l, T>
{
	fn from(value: &'l [T; LENGTH]) -> Self
	{
		return SliceRef::new(value.as_slice());
	}
}
