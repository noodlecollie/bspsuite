use crate::math::comparison::points_are_equal_radial_sq;
use glam::DVec3;

pub(super) struct PointCollection
{
	points_vec: Vec<DVec3>,
	equality_epsilon_squared: f64,
}

impl PointCollection
{
	pub fn new(equality_epsilon: f64) -> Self
	{
		return Self {
			points_vec: Vec::new(),
			equality_epsilon_squared: equality_epsilon * equality_epsilon,
		};
	}

	pub fn add(&mut self, point: DVec3) -> (usize, DVec3)
	{
		let index = self.index_of(point).unwrap_or_else(|| {
			self.points_vec.push(point);
			return self.points_vec.len() - 1;
		});

		return (index, self.points_vec[index]);
	}

	pub fn index_of(&self, point: DVec3) -> Option<usize>
	{
		for (index, existing) in self.points_vec.iter().enumerate()
		{
			if points_are_equal_radial_sq(point, *existing, self.equality_epsilon_squared)
			{
				return Some(index);
			}
		}

		return None;
	}

	pub fn contains(&self, point: DVec3) -> bool
	{
		return self.index_of(point).is_some();
	}

	pub fn points(&self) -> &Vec<DVec3>
	{
		return &self.points_vec;
	}
}

impl Into<Vec<DVec3>> for PointCollection
{
	fn into(self) -> Vec<DVec3>
	{
		return self.points_vec;
	}
}

#[cfg(test)]
mod tests
{
	use super::*;
	use crate::math::DEFAULT_EQUAL_POINT_RADIUS_EPSILON;

	#[test]
	fn identical_points()
	{
		let mut collection: PointCollection =
			PointCollection::new(DEFAULT_EQUAL_POINT_RADIUS_EPSILON);

		let vec: DVec3 = DVec3::new(1.0, 2.0, 3.0);
		let p0 = collection.add(vec);
		let p1 = collection.add(vec);

		assert_eq!(p0, p1);
		assert_eq!(collection.index_of(vec).unwrap(), p0.0);
	}

	#[test]
	fn close_enough_points()
	{
		let mut collection: PointCollection = PointCollection::new(0.5);

		let p0 = collection.add(DVec3::new(1.0, 2.0, 3.0));
		let p1 = collection.add(DVec3::new(1.1, 2.1, 3.0));

		assert_eq!(p0, p1);

		assert_eq!(
			collection.index_of(DVec3::new(1.0, 2.0, 3.0)).unwrap(),
			p0.0
		);
		assert_eq!(
			collection.index_of(DVec3::new(1.1, 2.1, 3.0)).unwrap(),
			p0.0
		);
		assert_eq!(
			collection.index_of(DVec3::new(1.0, 2.0, 3.0)).unwrap(),
			p1.0
		);
		assert_eq!(
			collection.index_of(DVec3::new(1.1, 2.1, 3.0)).unwrap(),
			p1.0
		);
	}

	#[test]
	fn distinct_points()
	{
		let mut collection: PointCollection = PointCollection::new(0.5);

		let p0 = collection.add(DVec3::new(1.0, 2.0, 3.0));
		let p1 = collection.add(DVec3::new(2.0, 2.0, 3.0));

		assert_ne!(p0, p1);

		assert_eq!(
			collection.index_of(DVec3::new(1.0, 2.0, 3.0)).unwrap(),
			p0.0
		);

		assert_eq!(
			collection.index_of(DVec3::new(2.0, 2.0, 3.0)).unwrap(),
			p1.0
		);
	}

	#[test]
	fn threshold_points()
	{
		let mut collection: PointCollection = PointCollection::new(0.5);

		let p0 = collection.add(DVec3::new(1.0, 2.0, 3.0));
		let p1 = collection.add(DVec3::new(1.5, 2.0, 3.0));

		assert_ne!(p0, p1);

		assert_eq!(
			collection.index_of(DVec3::new(1.0, 2.0, 3.0)).unwrap(),
			p0.0
		);

		assert_eq!(
			collection.index_of(DVec3::new(1.5, 2.0, 3.0)).unwrap(),
			p1.0
		);
	}
}
