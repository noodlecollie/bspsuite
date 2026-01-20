use maths_rs::Vec3d;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DPlane3
{
	pub normal: Vec3d,
	pub distance: f64,
}

impl DPlane3
{
	pub const NULL: Self = Self::new_xyzd(0.0, 0.0, 0.0, 0.0);

	#[inline]
	#[must_use]
	pub const fn new(normal: Vec3d, distance: f64) -> Self
	{
		return DPlane3 { normal, distance };
	}

	#[inline]
	#[must_use]
	pub const fn new_xyzd(x: f64, y: f64, z: f64, d: f64) -> Self
	{
		return Self::new(Vec3d { x: x, y: y, z: z }, d);
	}

	#[inline]
	#[must_use]
	pub fn is_null(&self) -> bool
	{
		return self == &DPlane3::NULL;
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn construct_null_plane()
	{
		let null1: DPlane3 = DPlane3 {
			normal: Vec3d::new(0.0, 0.0, 0.0),
			distance: 0.0,
		};

		assert!(null1.is_null());

		let null2: DPlane3 = DPlane3::new(Vec3d::new(0.0, 0.0, 0.0), 0.0);
		assert!(null2.is_null());

		let null3: DPlane3 = DPlane3::new_xyzd(0.0, 0.0, 0.0, 0.0);
		assert!(null3.is_null());

		let null4: DPlane3 = DPlane3::NULL.clone();
		assert!(null4.is_null());

		assert!(DPlane3::NULL.is_null());
		assert_eq!(&null1, &DPlane3::NULL);
		assert_eq!(&null2, &DPlane3::NULL);
		assert_eq!(&null3, &DPlane3::NULL);
		assert_eq!(&null4, &DPlane3::NULL);
	}
}
