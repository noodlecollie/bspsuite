use crate::extensions::extension::{Extension, ExtensionRc};

// Pairs a resource (eg. a callback) with an extension,
// so that the extension will always be valid for the
// lifetime of the resource.
pub struct ExtensionResource<T>
{
	extension_rc: ExtensionRc,
	resource: T,
}

impl<T> ExtensionResource<T>
{
	pub fn new(extension: ExtensionRc, resource: T) -> Self
	{
		return Self {
			extension_rc: extension,
			resource: resource,
		};
	}

	pub fn extension(&self) -> &Extension
	{
		return self.extension_rc.as_ref();
	}

	pub fn extension_rc(&self) -> ExtensionRc
	{
		return self.extension_rc.clone();
	}
}

impl<T> std::ops::Deref for ExtensionResource<T>
{
	type Target = T;

	fn deref(&self) -> &Self::Target
	{
		return &self.resource;
	}
}
