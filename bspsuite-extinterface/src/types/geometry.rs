//! This module contains simple FFI-safe types used to describe geometry.
//! These are only intended for being passed across the dynamic library
//! boundary. You will probably want to convert these to proper objects (eg.
//! Glam vectors) before you use them.

pub struct DVec3
{
	pub x: f64,
	pub y: f64,
	pub z: f64,
}

pub struct DVec4
{
	pub x: f64,
	pub y: f64,
	pub z: f64,
	pub w: f64,
}

pub struct DPlane
{
	pub normal: DVec3,
	pub distance: f64,
}

impl DVec3
{
	pub fn new(x: f64, y: f64, z: f64) -> Self
	{
		return Self { x: x, y: y, z: z };
	}
}

impl DVec4
{
	pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self
	{
		return Self {
			x: x,
			y: y,
			z: z,
			w: w,
		};
	}
}

impl DPlane
{
	pub fn new(normal: DVec3, distance: f64) -> Self
	{
		return Self {
			normal: normal,
			distance: distance,
		};
	}

	pub fn new_xyzd(x: f64, y: f64, z: f64, distance: f64) -> Self
	{
		return DPlane::new(DVec3::new(x, y, z), distance);
	}
}
