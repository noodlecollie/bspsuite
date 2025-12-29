use crate::types::{XCOption, XCSlice, XCStr};

impl<'l> From<Option<&'l str>> for XCOption<XCStr<'l>>
{
	/// Converts an `Option<&str>` to an `XCOption<XCStr>`.
	fn from(value: Option<&'l str>) -> Self
	{
		return match value
		{
			Some(val) => Self::Some(XCStr::new(val)),
			None => Self::None,
		};
	}
}

impl<'l> Into<Option<&'l str>> for XCOption<XCStr<'l>>
{
	/// Converts an `XCOption<XCStr>` to an `Option<&str>`.
	fn into(self) -> Option<&'l str>
	{
		return match self
		{
			XCOption::Some(val) => Some(val.as_str()),
			XCOption::None => None,
		};
	}
}

impl<'l, T> From<Option<&'l [T]>> for XCOption<XCSlice<'l, T>>
{
	/// Converts an `Option<&[T]>` to an `XCOption<XCSlice[T]>`.
	fn from(value: Option<&'l [T]>) -> Self
	{
		return match value
		{
			Some(val) => Self::Some(XCSlice::new(val)),
			None => Self::None,
		};
	}
}

impl<'l, T> Into<Option<&'l [T]>> for XCOption<XCSlice<'l, T>>
{
	/// Converts an `XCOption<XCSlice<T>>` to an `Option<&[T]>`.
	fn into(self) -> Option<&'l [T]>
	{
		return match self
		{
			XCOption::Some(val) => Some(val.as_slice()),
			XCOption::None => None,
		};
	}
}
