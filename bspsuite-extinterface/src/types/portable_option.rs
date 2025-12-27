#[repr(C)]
pub enum PortableOption<T>
{
	None,
	Some(T),
}

impl<T> PortableOption<T>
{
	pub fn is_some(&self) -> bool
	{
		return match self
		{
			PortableOption::Some(_) => true,
			PortableOption::None => false,
		};
	}

	pub fn is_none(&self) -> bool
	{
		return !self.is_some();
	}

	pub fn unwrap(self) -> T
	{
		match self
		{
			PortableOption::Some(val) => return val,
			PortableOption::None => panic!("unwrap called with no value"),
		}
	}

	pub fn unwrap_ref(&self) -> &T
	{
		match self
		{
			PortableOption::Some(val) => return val,
			PortableOption::None => panic!("unwrap_ref called with no value"),
		}
	}
}

impl<T> From<Option<T>> for PortableOption<T>
{
	fn from(value: Option<T>) -> Self
	{
		return value.map_or(PortableOption::None, |val| PortableOption::Some(val));
	}
}

impl<T> From<T> for PortableOption<T>
{
	fn from(value: T) -> Self
	{
		return PortableOption::Some(value);
	}
}

// This function is needed even though the compiler documentation implies
// that it might not be. Not sure why the compiler doesn't automatically
// generate this function even though the docs say it should...
impl<T> Into<Option<T>> for PortableOption<T>
{
	fn into(self) -> Option<T>
	{
		return match self
		{
			PortableOption::None => None,
			PortableOption::Some(val) => Some(val),
		};
	}
}
