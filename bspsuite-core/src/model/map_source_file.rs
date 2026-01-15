use std::collections::HashMap;

use super::dplane3::DPlane3;
use bspextifc::builders::map_blueprint_builder::{Brush, BrushFace, Entity};
use bspextifc::types::{DPlane as ExtPlane, DVec2 as ExtVec2, DVec3 as ExtVec3};
use glam::{DVec2, DVec3};

pub struct MapSourceBrushFace
{
	pub plane: DPlane3,
	pub material_name: String,
	pub material_axes: (DVec3, DVec3),
	pub material_offset: DVec2,
	pub material_scale: DVec2,
}

pub struct MapSourceBrush
{
	pub faces: Vec<MapSourceBrushFace>,
}

pub struct MapSourceEntity
{
	pub brushes: Vec<MapSourceBrush>,
	pub keyvalues: HashMap<String, String>,
}

pub struct MapSourceFile
{
	pub entities: Vec<MapSourceEntity>,
}

impl From<BrushFace> for MapSourceBrushFace
{
	fn from(value: BrushFace) -> Self
	{
		return Self {
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
			faces: value.faces.into_iter().map(|face| face.into()).collect(),
		};
	}
}

impl From<Entity> for MapSourceEntity
{
	fn from(value: Entity) -> Self
	{
		return Self {
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
		return Self {
			entities: value.into_iter().map(|ent| ent.into()).collect(),
		};
	}
}

fn vec2(vec: ExtVec2) -> DVec2
{
	return DVec2 { x: vec.x, y: vec.y };
}

fn vec3(vec: ExtVec3) -> DVec3
{
	return DVec3 {
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
