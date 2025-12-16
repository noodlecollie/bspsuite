#[repr(C)]
pub enum PortableOption<T>
{
	None,
	Some(T),
}

impl<T> From<Option<T>> for PortableOption<T>
{
	fn from(value: Option<T>) -> Self
	{
		return value.map_or(PortableOption::None, |val| PortableOption::Some(val));
	}
}

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
