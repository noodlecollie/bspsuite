use crate::math::CompileTuningParameters;
use crate::math::comparison::points_are_equal_radial_sq;
use crate::model::{MapCsgBrush, MapCsgBrushFace, MapSourceBrush, MapSourceBrushFace};
use anyhow::Result;
use glam::DVec3;
use itertools::equal;

// In combination with the Stefan Hajnoczi paper (see the notes directory in
// this repo), and with
// https://math.stackexchange.com/questions/5120422/how-to-verify-whether-a-collection-of-3d-planes-forms-a-valid-convex-hull/5120722,
// the approach we follow here is:
// For each plane (face):
//   Check intersections with all other planes. For each intersection (edge):
//     Check intersections with all remaining planes to create vertex list.
//     Remove extraneous vertices (in front of any planes).
//     If edge vertices != 2, solid is not valid.
//     Add vertices and edges to respective faces.
//   If faace vertices < 3, solid is not valid.
//   Order face's vertices clockwise, using edges to link vertices.
//   Compute texture co-ordinates for each vertex.
// Normalise texture co-ordinates for all vertices in brush.

pub fn construct_brush(
	source: MapSourceBrush,
	params: &CompileTuningParameters,
) -> Result<MapCsgBrush>
{
	todo!();
}

struct CsgBrushBuilder<'l>
{
	source: &'l MapSourceBrush,
	params: &'l CompileTuningParameters,

	vertices: PointCollection,
}

impl<'l> CsgBrushBuilder<'l>
{
	pub fn build(
		source: &'l MapSourceBrush,
		params: &'l CompileTuningParameters,
	) -> Result<MapCsgBrush>
	{
		let builder = Self {
			source: source,
			params: params,
			vertices: PointCollection::new(params.equal_point_radius_epsilon),
		};

		return builder.build_internal();
	}

	fn build_internal(self) -> Result<MapCsgBrush>
	{
		for (face_index, face) in self.source.faces.iter().enumerate()
		{
			self.process_face((face_index, face))?;
		}

		todo!();
	}

	fn process_face(&self, face: (usize, &MapSourceBrushFace)) -> Result<()>
	{
		// Compare against all faces after the current one,
		// since comparisons with faces before it will already have happened.
		for other_face_index in (face.0 + 1)..self.source.faces.len()
		{
			let other_face = (other_face_index, &self.source.faces[other_face_index]);
			self.process_faces(face, other_face)?;
		}

		return Ok(());
	}

	fn process_faces(
		&self,
		face_1: (usize, &MapSourceBrushFace),
		face_2: (usize, &MapSourceBrushFace),
	) -> Result<()>
	{
		// TODO: Intersect the two face planes to produce a line
		todo!();
	}
}

struct PointCollection
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

	pub fn add(&mut self, point: DVec3) -> usize
	{
		return self.index_of(point).unwrap_or_else(|| {
			self.points_vec.push(point);
			return self.points_vec.len() - 1;
		});
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
