use std::collections::HashMap;

use crate::math::geometry::{snap_normal_to_axis_if_close_enough, vector_to_unit_or_null};
use crate::math::{CompileTuningParameters, DPlane3};
use bspextifc::builders::map_source_builder::{Brush, BrushFace, Entity};
use bspextifc::types::{DPlane3 as ExtPlane, DVec2 as ExtVec2, DVec3 as ExtVec3};
use glam::{DVec2, DVec3};

pub struct MapSourceBrushFace
{
	pub global_face_index: usize,
	pub plane: DPlane3,
	pub material_name: String,
	pub material_axes: (DVec3, DVec3),
	pub material_offset: DVec2,
	pub material_scale: DVec2,
}

pub struct MapSourceBrush
{
	pub global_brush_index: usize,
	pub faces: Vec<MapSourceBrushFace>,
}

pub struct MapSourceEntity
{
	pub global_entity_index: usize,
	pub brushes: Vec<MapSourceBrush>,
	pub keyvalues: HashMap<String, String>,
}

pub struct MapSourceFile
{
	pub entities: Vec<MapSourceEntity>,
}

impl MapSourceFile
{
	pub fn create(entities: Vec<Entity>, compile_params: &CompileTuningParameters) -> Self
	{
		let mut out = Self {
			entities: entities
				.into_iter()
				.map(|ent| into_entity(ent, compile_params))
				.collect(),
		};

		out.assign_global_indices();
		return out;
	}

	pub fn assign_global_indices(&mut self)
	{
		let mut current_brush: usize = 0;
		let mut current_face: usize = 0;

		for (entindex, entity) in self.entities.iter_mut().enumerate()
		{
			entity.global_entity_index = entindex;

			for brush in entity.brushes.iter_mut()
			{
				brush.global_brush_index = current_brush;
				assert!(current_brush < usize::MAX, "Overflowed max brush index");
				current_brush += 1;

				for face in brush.faces.iter_mut()
				{
					face.global_face_index = current_face;
					assert!(current_face < usize::MAX, "Overflowed max face index");
					current_face += 1;
				}
			}
		}
	}
}

fn into_face(input: BrushFace, compile_params: &CompileTuningParameters) -> MapSourceBrushFace
{
	return MapSourceBrushFace {
		global_face_index: 0, // Assigned later
		plane: into_plane3_snapped(
			input.plane,
			compile_params.equal_vector_component_epsilon,
			compile_params.zero_epsilon,
		),
		material_name: input.material_name,
		material_axes: (
			into_vec3(input.material_axes.0),
			into_vec3(input.material_axes.1),
		),
		material_offset: into_vec2(input.material_offset),
		material_scale: into_vec2(input.material_scale),
	};
}

fn into_brush(input: Brush, compile_params: &CompileTuningParameters) -> MapSourceBrush
{
	return MapSourceBrush {
		global_brush_index: 0, // Assigned later
		faces: input
			.faces
			.into_iter()
			.map(|face| into_face(face, compile_params))
			.collect(),
	};
}

fn into_entity(input: Entity, compile_params: &CompileTuningParameters) -> MapSourceEntity
{
	return MapSourceEntity {
		global_entity_index: 0, // Assigned later
		brushes: input
			.brushes
			.into_iter()
			.map(|brush| into_brush(brush, compile_params))
			.collect(),
		keyvalues: input.keyvalues,
	};
}

fn into_vec2(vec: ExtVec2) -> DVec2
{
	return DVec2::new(vec.x, vec.y);
}

fn into_vec3(vec: ExtVec3) -> DVec3
{
	return DVec3::new(vec.x, vec.y, vec.z);
}

fn into_plane3_snapped(plane: ExtPlane, component_epsilon: f64, zero_epsilon: f64) -> DPlane3
{
	let normal: DVec3 = vector_to_unit_or_null(into_vec3(plane.normal), zero_epsilon).0;

	return DPlane3::new_unchecked(
		snap_normal_to_axis_if_close_enough(normal, component_epsilon),
		plane.distance,
	);
}
