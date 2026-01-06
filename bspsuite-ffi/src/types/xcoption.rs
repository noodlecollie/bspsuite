use crate::traits::RefMarshaller;

/// FFI-safe and extern "C"-safe counterpart to Rust's `Option` type.
///
/// Note that an `XCOption` is only intended to transport an optional value
/// across a library boundary, and is not intended to implement all the
/// functions that an `Option` does. If you want to use the value wrapped in an
/// `XCOption`, convert it to a full `Option` first.
#[repr(C)]
pub enum XCOption<T>
where
	T: Sized,
{
	None,
	Some(T),
}

impl<T> XCOption<T>
{
	/// Returns whether the `XCOption` contains a value.
	#[inline]
	#[must_use]
	pub fn is_some(&self) -> bool
	{
		return match self
		{
			XCOption::Some(_) => true,
			XCOption::None => false,
		};
	}

	/// Returns whether the `XCOption` is devoid of a value.
	#[inline]
	#[must_use]
	pub fn is_none(&self) -> bool
	{
		return !self.is_some();
	}

	/// Adapts the `XCOption<T>` to convert the internal value of `T` to a
	/// reference `&T`, and returns an `XCOption<&T>`.
	#[inline]
	#[must_use]
	pub fn as_ref(&self) -> XCOption<&T>
	{
		return match self
		{
			XCOption::Some(val) => XCOption::Some(val),
			XCOption::None => XCOption::None,
		};
	}

	/// Returns a Rust `Option<&T>` containing a reference the wrapped value.
	#[inline]
	#[must_use]
	pub fn as_ref_option(&self) -> Option<&T>
	{
		return match self
		{
			XCOption::Some(val) => Some(val),
			XCOption::None => None,
		};
	}

	/// Returns a Rust `Option<&U>`, where `U` is the native type being
	/// marshalled by the marshaller type `T`.
	///
	/// For example, this function allows an `XCOption<XCStr>` to be easily
	/// converted to an `Option<&str>`.
	#[inline]
	#[must_use]
	pub fn unmarshal_as_ref_option<'l, U>(&'l self) -> Option<&'l U>
	where
		T: RefMarshaller<'l, U>,
		U: ?Sized,
	{
		return match self
		{
			XCOption::Some(val) => Some(val.unmarshal_ref()),
			XCOption::None => None,
		};
	}

	/// Converts the `XCOption<T>` into a Rust `Option<T>` containing the
	/// wrapped value.
	#[inline]
	#[must_use]
	pub fn into_option(self) -> Option<T>
	{
		return match self
		{
			XCOption::Some(val) => Some(val),
			XCOption::None => None,
		};
	}
}

impl<T> From<T> for XCOption<T>
{
	/// Converts a `T` value to an `XCOption<T>`.
	#[inline]
	fn from(value: T) -> Self
	{
		return Self::Some(value);
	}
}

impl<T> From<Option<T>> for XCOption<T>
{
	/// Converts an `Option<T>` to an `XCOption<T>`.
	#[inline]
	fn from(value: Option<T>) -> Self
	{
		return match value
		{
			Some(val) => Self::Some(val),
			None => Self::None,
		};
	}
}

impl<T> Into<Option<T>> for XCOption<T>
{
	/// Converts an `XCOption<T>` to an `Option<T>`.
	#[inline]
	fn into(self) -> Option<T>
	{
		return self.into_option();
	}
}
