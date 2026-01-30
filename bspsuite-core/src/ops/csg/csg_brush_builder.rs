use super::edge_collection::EdgeCollection;
use super::point_collection::PointCollection;
use crate::math::comparison::{values_are_equal, vectors_are_equal};
use crate::math::geometry::{
	LinePlaneIntersection, classify_point_against_plane, intersect_line_and_plane,
	intersect_planes, snap_point_to_nearest_integer_grid_point_if_close_enough,
};
use crate::math::{CompileTuningParameters, DLine3, DPlane3};
use crate::model::{MapCsgBrush, MapSourceBrush, MapSourceBrushFace};
use anyhow::{Context, Result, bail};
use glam::DVec3;

pub(super) struct CsgBrushBuilder<'l>
{
	source: &'l MapSourceBrush,
	params: &'l CompileTuningParameters,

	vertices: PointCollection,
	face_edges: Vec<EdgeCollection>,
}

impl<'l> CsgBrushBuilder<'l>
{
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
			self.build_face((face_index, face))?;
		}

		self.verify_geometry_and_handedness_of_all_face_edges()?;

		todo!(
			"Compute texture co-ordinates, and then construct brush object from computed geometry"
		);
	}

	fn build_face(&mut self, face: (usize, &MapSourceBrushFace)) -> Result<()>
	{
		// Compare against all faces after the current one,
		// since comparisons with faces before it will already have happened.
		for other_face_index in (face.0 + 1)..self.source.faces.len()
		{
			let other_face = (other_face_index, &self.source.faces[other_face_index]);

			self.compute_edges_from_faces(face, other_face)
				.with_context(|| {
					format!(
						"Failed to compute intersection of brush face {} with other face {}",
						face.0, other_face.0
					)
				})?;
		}

		return Ok(());
	}

	fn compute_edges_from_faces(
		&mut self,
		face_1: (usize, &MapSourceBrushFace),
		face_2: (usize, &MapSourceBrushFace),
	) -> Result<()>
	{
		let intersection: Option<DLine3> =
			intersect_planes(&face_1.1.plane, &face_2.1.plane, self.params.zero_epsilon);

		if intersection.is_none()
		{
			// Planes are parallel, nothing to do.
			return Ok(());
		}

		let intersection: DLine3 = intersection.unwrap();

		// Find the vertices for each end of the edge.
		let (bound_0, bound_1) = self.find_edge_bounds(&intersection, (face_1.0, face_2.0))?;

		// Snap these to integer grid points if close enough, and add to vertex
		// collection.
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

		let edge: (usize, usize) = (v0.0, v1.0);

		if let Err(err) = self.face_edges[face_1.0].add(edge)
		{
			bail!("Failed to add edge {} -> {}. {err}", v0.1, v1.1);
		}

		if let Err(err) = self.face_edges[face_2.0].add(edge)
		{
			bail!("Failed to add edge {} -> {}. {err}", v0.1, v1.1);
		}

		return Ok(());
	}

	fn verify_geometry_and_handedness_of_all_face_edges(&mut self) -> Result<()>
	{
		for (face_index, edges) in self.face_edges.iter_mut().enumerate()
		{
			if let Err(err) = edges.verify()
			{
				bail!("Invalid geometry computed for brush face {face_index}. {err}");
			}

			let normal: Option<DVec3> =
				edges.normal(self.vertices.points(), self.params.zero_epsilon);

			if normal.is_none()
			{
				bail!(
					"Invalid geometry computed for brush face {face_index}. Failed to compute normal from edges."
				);
			}

			let normal_dir: f64 = normal
				.unwrap()
				.dot(self.source.faces[face_index].plane.normal());

			// Sanity:
			if !values_are_equal(normal_dir.abs(), 1.0, self.params.zero_epsilon)
			{
				bail!(
					"Brush face {face_index} normal {} computed from edges did not align with plane normal {}. This should never happen!",
					normal.unwrap(),
					self.source.faces[face_index].plane.normal()
				);
			}

			if normal_dir < 0.0
			{
				// Need to reverse the edges so that they match the normal of the plane.
				edges.reverse();

				// Sanity:
				debug_assert!(vectors_are_equal(
					edges
						.normal(self.vertices.points(), self.params.zero_epsilon)
						.unwrap(),
					self.source.faces[face_index].plane.normal(),
					self.params.equal_point_radius_epsilon
				));
			}
		}

		return Ok(());
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

			let result: LinePlaneIntersection = intersect_line_and_plane(
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
				"Edge between brush faces {} and {} was not bounded",
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
