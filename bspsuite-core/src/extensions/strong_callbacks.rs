use crate::extensions::extension::{Extension, ExtensionRc};

pub struct StrongCallbacks<T>
{
	extension_rc: ExtensionRc,
	callbacks: T,
}

impl<T> StrongCallbacks<T>
{
	pub fn new(extension: ExtensionRc, callbacks: T) -> Self
	{
		return Self {
			extension_rc: extension,
			callbacks: callbacks,
		};
	}

	pub fn extension(&self) -> &Extension
	{
		return self.extension_rc.as_ref();
	}
}

impl<T> std::ops::Deref for StrongCallbacks<T>
{
	type Target = T;

	fn deref(&self) -> &Self::Target
	{
		return &self.callbacks;
	}
}
