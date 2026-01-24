use glam::DVec3;

pub const fn dvec3_length_sq(vec: DVec3) -> f64
{
	return (vec.x * vec.x) + (vec.y * vec.y) + (vec.z * vec.z);
}

pub const fn dvec3_add(a: DVec3, b: DVec3) -> DVec3
{
	return DVec3::new(a.x + b.x, a.y + b.y, a.z + b.z);
}

pub const fn dvec3_subtract(a: DVec3, b: DVec3) -> DVec3
{
	return DVec3::new(a.x - b.x, a.y - b.y, a.z - b.z);
}
