use std::collections::HashMap;

use crate::conv::{dvec2_to_vector2, dvec3_to_vector3};
use anyhow::{Result, anyhow, bail};
use bspcore::model::{MapGeomBrush, MapGeomBrushFace, MapGeomBrushFaceVertex};
use log::info;
use raylib::prelude as rl;

pub struct MeshVertexGenerator
{
	vertices: Vec<IntermediateVertex>,
	indices: Vec<u16>,
	colour: rl::Color,
}

#[derive(Debug)]
pub struct MeshComponents
{
	pub positions: Vec<rl::Vector3>,
	pub tex_coords: Vec<rl::Vector2>,
	pub normals: Vec<rl::Vector3>,
	pub colours: Vec<rl::Color>,
	pub indices: Vec<u16>,
}

struct IntermediateVertex
{
	position: rl::Vector3,
	tex_coord: rl::Vector2,
	normal: rl::Vector3,
}

impl MeshVertexGenerator
{
	pub fn from_brush(brush: &MapGeomBrush) -> Result<Self>
	{
		let mut generator: Self = Self {
			vertices: Vec::new(),
			indices: Vec::new(),
			colour: rl::Color::new(255, 255, 255, 255),
		};

		generator.add_brush(brush)?;
		return Ok(generator);
	}

	pub fn set_colour(&mut self, col: rl::Color)
	{
		self.colour = col;
	}

	pub fn into_components(self) -> MeshComponents
	{
		let mut positions: Vec<rl::Vector3> = Vec::with_capacity(self.vertices.len());
		let mut tex_coords: Vec<rl::Vector2> = Vec::with_capacity(self.vertices.len());
		let mut normals: Vec<rl::Vector3> = Vec::with_capacity(self.vertices.len());
		let mut colours: Vec<rl::Color> = Vec::with_capacity(self.vertices.len());

		for vert in self.vertices
		{
			positions.push(vert.position);
			tex_coords.push(vert.tex_coord);
			normals.push(vert.normal);
			colours.push(self.colour.clone());
		}

		return MeshComponents {
			positions,
			tex_coords,
			normals,
			colours,
			indices: self.indices,
		};
	}

	fn add_brush(&mut self, brush: &MapGeomBrush) -> Result<()>
	{
		for face in brush.faces.iter()
		{
			self.add_brush_face(brush, face)?;
		}

		return Ok(());
	}

	fn add_brush_face(&mut self, brush: &MapGeomBrush, face: &MapGeomBrushFace) -> Result<()>
	{
		if face.vertices.len() < 3
		{
			bail!(
				"Brush {} face {} had {} vertices when at least 3 were expected",
				brush.global_brush_index,
				face.global_face_index,
				face.vertices.len(),
			);
		}

		let normal: rl::Vector3 = dvec3_to_vector3(face.plane.normal());

		for vert in face.vertices.iter()
		{
			let index_in_brush: usize = vert.index_in_brush;

			if index_in_brush >= brush.vertices.len()
			{
				bail!(
					"Brush {} face {} vertex {} was out of range of brush vertices",
					brush.global_brush_index,
					face.global_face_index,
					index_in_brush,
				);
			}

			let index: usize = self.vertices.len();

			if index > u16::MAX as usize
			{
				bail!(
					"Brush {} face {} exceeded max number of vertices",
					brush.global_brush_index,
					face.global_face_index,
				);
			}

			let int_vert = IntermediateVertex {
				position: dvec3_to_vector3(brush.vertices[vert.index_in_brush]),
				tex_coord: dvec2_to_vector2(vert.tex_coord),
				normal,
			};

			self.vertices.push(int_vert);
		}

		let num_triangles: usize = self.vertices.len() - 2;
		self.indices.reserve(num_triangles * 3);

		for index in 0..num_triangles
		{
			self.indices.push(0);
			self.indices.push(index as u16 + 1);
			self.indices.push(index as u16 + 2);
		}

		return Ok(());
	}
}
