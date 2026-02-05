use super::edge_collection::EdgeCollection;
use super::point_collection::PointCollection;
use crate::math::comparison::{values_are_equal, vectors_are_equal};
use crate::math::geometry::{
	LinePlaneIntersection, classify_point_against_plane, intersect_line_and_plane,
	intersect_planes, snap_point_to_nearest_integer_grid_point_if_close_enough,
	vector_to_unit_or_null,
};
use crate::math::{CompileTuningParameters, DLine3, DPlane3};
use crate::model::{
	MapCsgBrush, MapCsgBrushFace, MapCsgBrushFaceVertex, MapSourceBrush, MapSourceBrushFace,
};
use anyhow::{Context, Result, bail};
use glam::{DVec2, DVec3};

pub(super) struct CsgBrushBuilder<'l>
{
	source_brush: &'l MapSourceBrush,
	params: &'l CompileTuningParameters,
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
			source_brush: source,
			params: params,
		};

		return builder.build_internal();
	}

	fn build_internal(self) -> Result<MapCsgBrush>
	{
		let mut vertices: PointCollection =
			PointCollection::new(self.params.equal_point_radius_epsilon);

		let edges_by_face: Vec<EdgeCollection> =
			self.compute_edges_from_all_faces(&mut vertices)?;

		let face_edge_loops: Vec<Vec<usize>> =
			self.convert_edge_collections_to_edge_loops(edges_by_face, &vertices)?;

		let faces: Vec<MapCsgBrushFace> =
			self.finalise_faces(face_edge_loops, vertices.points())?;

		return Ok(MapCsgBrush {
			global_brush_index: self.source_brush.global_brush_index,
			vertices: vertices.into(),
			faces: faces,
		});
	}

	fn compute_edges_from_all_faces(
		&self,
		vertices: &mut PointCollection,
	) -> Result<Vec<EdgeCollection>>
	{
		let mut edges_by_face: Vec<EdgeCollection> = self
			.source_brush
			.faces
			.iter()
			.map(|_| EdgeCollection::new())
			.collect();

		for (face_index, face) in self.source_brush.faces.iter().enumerate()
		{
			// Compare against all faces after the current one,
			// since comparisons with faces before it will already have happened.
			for other_face_index in (face_index + 1)..self.source_brush.faces.len()
			{
				self.compute_edges_from_faces(
					&mut edges_by_face,
					vertices,
					(face_index, face),
					(
						other_face_index,
						&self.source_brush.faces[other_face_index]
					),
				)
				.with_context(|| {
					format!(
						"Failed to compute intersection of brush face {face_index} with other face {other_face_index}",
					)
				})?;
			}
		}

		return Ok(edges_by_face);
	}

	fn compute_edges_from_faces(
		&self,
		edges_by_face: &mut Vec<EdgeCollection>,
		vertices: &mut PointCollection,
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
		let v0 = vertices.add(snap_point_to_nearest_integer_grid_point_if_close_enough(
			bound_0,
			self.params.equal_point_radius_epsilon,
		));

		let v1 = vertices.add(snap_point_to_nearest_integer_grid_point_if_close_enough(
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

		if let Err(err) = edges_by_face[face_1.0].add(edge)
		{
			bail!("Failed to add edge {} -> {}. {err}", v0.1, v1.1);
		}

		if let Err(err) = edges_by_face[face_2.0].add(edge)
		{
			bail!("Failed to add edge {} -> {}. {err}", v0.1, v1.1);
		}

		return Ok(());
	}

	fn convert_edge_collections_to_edge_loops(
		&self,
		edge_collections: Vec<EdgeCollection>,
		vertices: &PointCollection,
	) -> Result<Vec<Vec<usize>>>
	{
		let mut out: Vec<Vec<usize>> = Vec::new();

		for (face_index, edges) in edge_collections.into_iter().enumerate()
		{
			if let Err(err) = edges.validate()
			{
				bail!("Invalid geometry computed for brush face {face_index}. {err}");
			}

			let mut edge_loop = edges.into_edge_loop().with_context(|| {
				format!("Invalid geometry computed for brush face {face_index}")
			})?;

			let normal: Option<DVec3> = self.compute_normal(&edge_loop, vertices);

			if normal.is_none()
			{
				bail!(
					"Invalid geometry computed for brush face {face_index}. Failed to compute normal from edges."
				);
			}

			let normal_dir: f64 = normal
				.unwrap()
				.dot(self.source_brush.faces[face_index].plane.normal());

			// Sanity:
			if !values_are_equal(normal_dir.abs(), 1.0, self.params.zero_epsilon)
			{
				bail!(
					"Brush face {face_index} normal {} computed from edges did not align with plane normal {}. This should never happen!",
					normal.unwrap(),
					self.source_brush.faces[face_index].plane.normal()
				);
			}

			if normal_dir < 0.0
			{
				// Need to reverse the edges so that they match the normal of the plane.
				edge_loop.reverse();

				// Sanity:
				debug_assert!(vectors_are_equal(
					self.compute_normal(&edge_loop, vertices).unwrap(),
					self.source_brush.faces[face_index].plane.normal(),
					self.params.equal_point_radius_epsilon
				));
			}

			out.push(edge_loop);
		}

		return Ok(out);
	}

	fn finalise_faces(
		&self,
		face_edge_loops: Vec<Vec<usize>>,
		vertices: &Vec<DVec3>,
	) -> Result<Vec<MapCsgBrushFace>>
	{
		let mut out: Vec<MapCsgBrushFace> = Vec::with_capacity(face_edge_loops.len());

		for (face_index, edges) in face_edge_loops.into_iter().enumerate()
		{
			out.push(self.finalise_face(face_index, edges, vertices));
		}

		return Ok(out);
	}

	fn finalise_face(
		&self,
		face_index: usize,
		edges: Vec<usize>,
		vertices: &Vec<DVec3>,
	) -> MapCsgBrushFace
	{
		let orig_face: &MapSourceBrushFace = &self.source_brush.faces[face_index];

		let mut face_vertices = edges
			.into_iter()
			.map(|vindex| MapCsgBrushFaceVertex {
				index_in_brush: vindex,
				tex_coord: DVec2::new(
					CsgBrushBuilder::texture_ordinate(
						vertices[vindex],
						orig_face.material_axes.0,
						64, // TODO: Actually read texture to get this!
						orig_face.material_scale.x,
						orig_face.material_offset.x,
					),
					CsgBrushBuilder::texture_ordinate(
						vertices[vindex],
						orig_face.material_axes.1,
						64, // TODO: Actually read texture to get this!
						orig_face.material_scale.y,
						orig_face.material_offset.y,
					),
				),
			})
			.collect();

		CsgBrushBuilder::normalise_all_texture_coordinates(&mut face_vertices);

		return MapCsgBrushFace {
			global_face_index: self.source_brush.faces[face_index].global_face_index,
			plane: orig_face.plane,
			vertices: face_vertices,
			material: orig_face.material_name.clone(),
		};
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

		for (face_index, face) in self.source_brush.faces.iter().enumerate()
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

	fn compute_normal(&self, edges: &Vec<usize>, vertices: &PointCollection) -> Option<DVec3>
	{
		let vertices: &Vec<DVec3> = vertices.points();

		if vertices.len() < 3 || edges.len() < 3
		{
			return None;
		}

		let vert_indices = (edges[0], edges[1], edges[2]);

		let verts = (
			vertices[vert_indices.0],
			vertices[vert_indices.1],
			vertices[vert_indices.2],
		);

		let edges = (verts.1 - verts.0, verts.2 - verts.0);

		let cross_product =
			vector_to_unit_or_null(edges.0.cross(edges.1), self.params.zero_epsilon);

		return if cross_product.1
		{
			Some(cross_product.0)
		}
		else
		{
			None
		};
	}

	// Because a face's vertices will likely be very far from the origin, their
	// texture co-ordinates will also be unnecessarily large. We want to measure the
	// shortest distance from the texture axis origin to any of the points on the
	// face, round it down to an integer, and substract this distance from the
	// texture co-ordinates of the points. This will minimise the distance between
	// the texture co-ordinates and the origin, and the point with the shortest
	// distance will be in the range (-1 1) (depending on what sign its
	// co-ordinate had). If any point has a co-ordinate between -1 and 1, there's no
	// point doing anything because the shortest co-ordinate distance would then end
	// up being 0.
	fn normalise_all_texture_coordinates(vertices: &mut Vec<MapCsgBrushFaceVertex>)
	{
		#[derive(Copy, Clone)]
		struct MinOrdinate
		{
			index: usize,
			abs_value: f64,
		}

		if vertices.len() < 1
		{
			return;
		}

		let init: MinOrdinate = MinOrdinate {
			index: usize::MAX,
			abs_value: f64::MAX,
		};

		let min_coords: (MinOrdinate, MinOrdinate) =
			vertices
				.iter()
				.enumerate()
				.fold((init, init), |acc, (index, vert)| {
					let abs: DVec2 = vert.tex_coord.abs();

					return (
						if abs.x < acc.0.abs_value
						{
							MinOrdinate {
								index: index,
								abs_value: abs.x,
							}
						}
						else
						{
							acc.0
						},
						if abs.y < acc.1.abs_value
						{
							MinOrdinate {
								index: index,
								abs_value: abs.y,
							}
						}
						else
						{
							acc.1
						},
					);
				});

		assert!(min_coords.0.index != usize::MAX && min_coords.1.index != usize::MAX);

		if min_coords.0.abs_value < 1.0 && min_coords.1.abs_value < 1.0
		{
			// No need to normalise.
			return;
		}

		// We now treat the values as non-abs, because it may be that we need to bring
		// negative co-ordinates back closer to zero.
		let offset: DVec2 = DVec2::new(
			vertices[min_coords.0.index].tex_coord.x,
			vertices[min_coords.1.index].tex_coord.y,
		)
		.trunc();

		for vert in vertices.iter_mut()
		{
			vert.tex_coord -= offset;
		}
	}

	fn texture_ordinate(
		pos: DVec3,
		tex_axis_dir: DVec3,
		image_dim: usize,
		texture_scale: f64,
		texture_offset: f64,
	) -> f64
	{
		let image_dim_float: f64 = image_dim as f64;

		return (pos.dot(tex_axis_dir) / (image_dim_float * texture_scale))
			+ (texture_offset / image_dim_float);
	}
}
