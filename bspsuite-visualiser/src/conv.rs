use glam::{DVec2, DVec3};
use raylib::prelude as rl;

pub fn dvec3_to_vector3(vec: DVec3) -> rl::Vector3
{
	return rl::Vector3 {
		x: vec.x as f32,
		y: vec.y as f32,
		z: vec.z as f32,
	};
}

pub fn dvec2_to_vector2(vec: DVec2) -> rl::Vector2
{
	return rl::Vector2 {
		x: vec.x as f32,
		y: vec.y as f32,
	};
}
