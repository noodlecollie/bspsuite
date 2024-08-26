pub const Float = f64;
pub usingnamespace @import("zlm").SpecializeOn(Float);

// Taken from Quake compile tools:
// tolerance for comparing a floating
// point value against zero.
pub const zero_epsilon: Float = 0.0001;

pub fn vec3ApproxZero(vec: @This().Vec3) bool {
    return vec.approxEqAbs(@This().Vec3.zero, zero_epsilon);
}
