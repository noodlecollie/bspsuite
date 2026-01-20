use std::collections::HashMap;

use super::dplane3::DPlane3;
use bspextifc::builders::map_source_builder::{Brush, BrushFace, Entity};
use bspextifc::types::{DPlane3 as ExtPlane, DVec2 as ExtVec2, DVec3 as ExtVec3};
use maths_rs::{Vec2d, Vec3d};

pub struct MapSourceBrushFace
{
	pub face_index: usize,
	pub plane: DPlane3,
	pub material_name: String,
	pub material_axes: (Vec3d, Vec3d),
	pub material_offset: Vec2d,
	pub material_scale: Vec2d,
}

pub struct MapSourceBrush
{
	pub brush_index: usize,
	pub faces: Vec<MapSourceBrushFace>,
}

pub struct MapSourceEntity
{
	pub entity_index: usize,
	pub brushes: Vec<MapSourceBrush>,
	pub keyvalues: HashMap<String, String>,
}

pub struct MapSourceFile
{
	pub entities: Vec<MapSourceEntity>,
}

impl MapSourceFile
{
	pub fn assign_indices(&mut self)
	{
		let mut current_brush: usize = 0;
		let mut current_face: usize = 0;

		for (entindex, entity) in self.entities.iter_mut().enumerate()
		{
			entity.entity_index = entindex;

			for brush in entity.brushes.iter_mut()
			{
				brush.brush_index = current_brush;
				assert!(current_brush < usize::MAX, "Overflowed max brush index");
				current_brush += 1;

				for face in brush.faces.iter_mut()
				{
					face.face_index = current_face;
					assert!(current_face < usize::MAX, "Overflowed max face index");
					current_face += 1;
				}
			}
		}
	}
}

impl From<BrushFace> for MapSourceBrushFace
{
	fn from(value: BrushFace) -> Self
	{
		return Self {
			face_index: 0, // Assigned later
			plane: plane3(value.plane),
			material_name: value.material_name,
			material_axes: (vec3(value.material_axes.0), vec3(value.material_axes.1)),
			material_offset: vec2(value.material_offset),
			material_scale: vec2(value.material_scale),
		};
	}
}

impl From<Brush> for MapSourceBrush
{
	fn from(value: Brush) -> Self
	{
		return Self {
			brush_index: 0, // Assigned later
			faces: value.faces.into_iter().map(|face| face.into()).collect(),
		};
	}
}

impl From<Entity> for MapSourceEntity
{
	fn from(value: Entity) -> Self
	{
		return Self {
			entity_index: 0, // Assigned later
			brushes: value
				.brushes
				.into_iter()
				.map(|brush| brush.into())
				.collect(),
			keyvalues: value.keyvalues,
		};
	}
}

impl From<Vec<Entity>> for MapSourceFile
{
	fn from(value: Vec<Entity>) -> Self
	{
		let mut out = Self {
			entities: value.into_iter().map(|ent| ent.into()).collect(),
		};

		out.assign_indices();
		return out;
	}
}

fn vec2(vec: ExtVec2) -> Vec2d
{
	return Vec2d { x: vec.x, y: vec.y };
}

fn vec3(vec: ExtVec3) -> Vec3d
{
	return Vec3d {
		x: vec.x,
		y: vec.y,
		z: vec.z,
	};
}

fn plane3(plane: ExtPlane) -> DPlane3
{
	return DPlane3 {
		normal: vec3(plane.normal),
		distance: plane.distance,
	};
}
