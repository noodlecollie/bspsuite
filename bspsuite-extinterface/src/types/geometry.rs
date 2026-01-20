//! This module contains simple FFI-safe types used to describe geometry.
//! These are only intended for being passed across the dynamic library
//! boundary. You will probably want to convert these to proper objects (eg.
//! vectors from a library like glam or maths_rs) before you use them.

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DVec2
{
	pub x: f64,
	pub y: f64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DVec3
{
	pub x: f64,
	pub y: f64,
	pub z: f64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DVec4
{
	pub x: f64,
	pub y: f64,
	pub z: f64,
	pub w: f64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DPlane
{
	pub normal: DVec3,
	pub distance: f64,
}

impl DVec2
{
	pub const NULL: Self = Self::new(0.0, 0.0);

	#[inline]
	#[must_use]
	pub const fn new(x: f64, y: f64) -> Self
	{
		return Self { x: x, y: y };
	}
}

impl PartialEq for DVec2
{
	fn eq(&self, other: &Self) -> bool
	{
		return self.x == other.x && self.y == other.y;
	}
}

impl DVec3
{
	pub const NULL: Self = Self::new(0.0, 0.0, 0.0);

	#[inline]
	#[must_use]
	pub const fn new(x: f64, y: f64, z: f64) -> Self
	{
		return Self { x: x, y: y, z: z };
	}
}

impl PartialEq for DVec3
{
	fn eq(&self, other: &Self) -> bool
	{
		return self.x == other.x && self.y == other.y && self.z == other.z;
	}
}

impl DVec4
{
	pub const NULL: Self = Self::new(0.0, 0.0, 0.0, 0.0);

	#[inline]
	#[must_use]
	pub const fn new(x: f64, y: f64, z: f64, w: f64) -> Self
	{
		return Self {
			x: x,
			y: y,
			z: z,
			w: w,
		};
	}
}

impl PartialEq for DVec4
{
	fn eq(&self, other: &Self) -> bool
	{
		return self.x == other.x && self.y == other.y && self.z == other.z && self.w == other.w;
	}
}

impl DPlane
{
	pub const NULL: Self = Self::new(DVec3::NULL, 0.0);

	#[inline]
	#[must_use]
	pub const fn new(normal: DVec3, distance: f64) -> Self
	{
		return Self {
			normal: normal,
			distance: distance,
		};
	}

	#[inline]
	#[must_use]
	pub const fn new_xyzd(x: f64, y: f64, z: f64, distance: f64) -> Self
	{
		return DPlane::new(DVec3::new(x, y, z), distance);
	}
}

impl PartialEq for DPlane
{
	fn eq(&self, other: &Self) -> bool
	{
		if self.normal != other.normal
		{
			return false;
		}

		if self.normal == DVec3::NULL
		{
			// Distance does not matter.
			return true;
		}

		return self.distance == other.distance;
	}
}
