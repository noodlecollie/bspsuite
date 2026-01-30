use crate::math::comparison::points_are_equal_radial_sq;
use crate::math::geometry::{
	LinePlaneIntersection, PointVsPlane, classify_point_against_plane,
	snap_point_to_nearest_integer_grid_point_if_close_enough,
};
use crate::math::{CompileTuningParameters, DLine3, DPlane3, geometry};
use crate::model::{
	MapCsgBrush, MapCsgBrushFace, MapCsgBrushFaceVertex, MapSourceBrush, MapSourceBrushFace,
};
use anyhow::{Result, bail};
use glam::DVec3;

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
//   If face vertices < 3, solid is not valid.
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
	face_edges: Vec<EdgeCollection>,
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
			face_edges: source.faces.iter().map(|_| EdgeCollection::new()).collect(),
		};

		return builder.build_internal();
	}

	fn build_internal(mut self) -> Result<MapCsgBrush>
	{
		for (face_index, face) in self.source.faces.iter().enumerate()
		{
			self.process_face((face_index, face))?;
		}

		todo!();
	}

	fn process_face(&mut self, face: (usize, &MapSourceBrushFace)) -> Result<()>
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
		&mut self,
		face_1: (usize, &MapSourceBrushFace),
		face_2: (usize, &MapSourceBrushFace),
	) -> Result<()>
	{
		let intersection: Option<DLine3> =
			geometry::intersect_planes(&face_1.1.plane, &face_2.1.plane, self.params.zero_epsilon);

		if intersection.is_none()
		{
			// Planes are parallel, nothing to do.
			return Ok(());
		}

		let intersection: DLine3 = intersection.unwrap();

		// Find the vertices for each end of the edge.
		let (bound_0, bound_1) = self.find_edge_bounds(&intersection, (face_1.0, face_2.0))?;

		let v0 = self
			.vertices
			.add(snap_point_to_nearest_integer_grid_point_if_close_enough(
				bound_0,
				self.params.equal_point_radius_epsilon,
			));

		let v1 = self
			.vertices
			.add(snap_point_to_nearest_integer_grid_point_if_close_enough(
				bound_1,
				self.params.equal_point_radius_epsilon,
			));

		if v0.0 == v1.0
		{
			bail!(
				"Vertices {bound_0} and {bound_1} were snapped to the same point {}",
				v0.1
			);
		}

		todo!();
	}

	// Intersect the edge with all faces in the brush to find the minimal edge span.
	fn find_edge_bounds(
		&self,
		edge: &DLine3,
		face_indices: (usize, usize),
	) -> Result<(DVec3, DVec3)>
	{
		struct Intersection<'l>
		{
			pub point: DVec3,
			pub from_plane: &'l DPlane3,
		}

		let mut intersections: (Option<Intersection>, Option<Intersection>) = (None, None);

		for (face_index, face) in self.source.faces.iter().enumerate()
		{
			if face_index == face_indices.0 || face_index == face_indices.1
			{
				continue;
			}

			let result: LinePlaneIntersection = geometry::intersect_line_and_plane(
				edge,
				&face.plane,
				self.params.zero_epsilon,
				self.params.contact_epsilon,
			);

			let intersection: Intersection = match result
			{
				LinePlaneIntersection::Point(point) => Intersection {
					point: point,
					from_plane: &face.plane,
				},
				_ => continue,
			};

			if intersections.0.is_none()
			{
				intersections.0 = Some(intersection);
				continue;
			}

			if intersections.1.is_none()
			{
				intersections.1 = Some(intersection);
				continue;
			}

			// Replace old points if they are in front of the new plane.

			let p0_in_front: bool = classify_point_against_plane(
				intersections.0.as_ref().unwrap().point,
				intersection.from_plane,
				self.params.contact_epsilon,
			)
			.is_in_front();

			let p1_in_front: bool = classify_point_against_plane(
				intersections.1.as_ref().unwrap().point,
				intersection.from_plane,
				self.params.contact_epsilon,
			)
			.is_in_front();

			if p0_in_front
			{
				intersections.0 = Some(intersection);

				if p1_in_front
				{
					// Both points were in front, so condense back down to one point.
					intersections.1 = None;
				}
			}
			else if p1_in_front
			{
				intersections.1 = Some(intersection);
			}
		}

		if intersections.0.is_none() || intersections.1.is_none()
		{
			bail!(
				"Edge between faces {} and {} was not bounded",
				face_indices.0,
				face_indices.1
			);
		}

		return Ok((
			intersections.0.unwrap().point,
			intersections.1.unwrap().point,
		));
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

struct EdgeCollection
{
	edges_vec: Vec<(usize, usize)>,
}

impl EdgeCollection
{
	pub fn new() -> Self
	{
		return Self {
			edges_vec: Vec::new(),
		};
	}

	pub fn add(&mut self, edge: (usize, usize)) -> Result<()>
	{
		// Each edge on a face connects two vertices, and each vertex should connect
		// only two edges. We try and insert edges into the list so that iterating up
		// the list iterates over connected chains of edges.

		if self.contains(&edge)
		{
			bail!("Duplicate edge");
		}

		for index in 0..self.edges_vec.len()
		{
			let existing: &(usize, usize) = &self.edges_vec[index];

			if existing.1 == edge.0
			{
				// Existing edge connects to the beginning of our new edge, so insert the new
				// edge in front.
				self.edges_vec.insert(index + 1, edge);
				return Ok(());
			}

			if existing.0 == edge.1
			{
				// Existing edge connects to the end of our new edge, so insert the edge behind.
				self.edges_vec.insert(index, edge);
				return Ok(());
			}

			if existing.0 == edge.0
			{
				// Existing edge connects to the end of our new edge, and the
				// new edge should be the other way around. Insert the edge behind.
				self.edges_vec.insert(index, (edge.1, edge.0));
				return Ok(());
			}

			if existing.1 == edge.1
			{
				// Existing edge connects to the beginning of our new edge, and
				// the new edge should be the other way around. Insert the new
				// edge in front.
				self.edges_vec.insert(index + 1, (edge.1, edge.0));
				return Ok(());
			}
		}

		// No matches, so just add on the end.
		self.edges_vec.push(edge);
		return Ok(());
	}

	pub fn contains(&self, edge: &(usize, usize)) -> bool
	{
		return self
			.edges_vec
			.iter()
			.find(|item| *item == edge || *item == &(edge.1, edge.0))
			.is_some();
	}

	// Returns a vector of slices, where each slice represents a chain of connected
	// edges. If the vector is empty, there are no edges.
	pub fn chains(&self) -> Vec<&[(usize, usize)]>
	{
		let mut out: Vec<&[(usize, usize)]> = Vec::new();
		let mut slice_indices: Option<(usize, usize)> = None;

		for index in 0..self.edges_vec.len()
		{
			if index == 0 || slice_indices.is_none()
			{
				slice_indices = Some((index, index));
				continue;
			}

			let edge: &(usize, usize) = &self.edges_vec[index];
			let last_edge: &(usize, usize) = &self.edges_vec[index - 1];

			if last_edge.1 == edge.0
			{
				// Lengthen the chain.
				slice_indices = Some((slice_indices.unwrap().0, index))
			}
			else
			{
				// Commit the chain and start a new one.
				let indices = slice_indices.unwrap();
				out.push(&self.edges_vec[indices.0..=indices.1]);

				slice_indices = Some((index, index));
			}
		}

		// Commit any remaining chain.
		if slice_indices.is_some()
		{
			let indices = slice_indices.unwrap();
			out.push(&self.edges_vec[indices.0..=indices.1]);
		}

		return out;
	}
}
