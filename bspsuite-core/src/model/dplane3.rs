use glam::DVec3;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DPlane3
{
	pub normal: DVec3,
	pub distance: f64,
}

impl DPlane3
{
	pub const NULL: Self = Self::new_xyzd(0.0, 0.0, 0.0, 0.0);

	#[inline]
	#[must_use = "Constructed plane was not used"]
	pub const fn new(normal: DVec3, distance: f64) -> Self
	{
		return DPlane3 { normal, distance };
	}

	#[inline]
	#[must_use = "Constructed plane was not used"]
	pub const fn new_xyzd(x: f64, y: f64, z: f64, d: f64) -> Self
	{
		return Self::new(DVec3::new(x, y, z), d);
	}

	#[inline]
	pub fn is_null(&self) -> bool
	{
		return self == &DPlane3::NULL;
	}

	#[inline]
	pub fn origin(&self) -> DVec3
	{
		return self.normal * self.distance;
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
			normal: DVec3::new(0.0, 0.0, 0.0),
			distance: 0.0,
		};

		assert!(null1.is_null());

		let null2: DPlane3 = DPlane3::new(DVec3::new(0.0, 0.0, 0.0), 0.0);
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
