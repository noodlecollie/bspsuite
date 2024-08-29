pub const Float = f64;
pub usingnamespace @import("zlm").SpecializeOn(Float);

// Taken from Quake compile tools:
// tolerance for comparing a floating
// point value against zero.
pub const zero_epsilon: Float = 0.0001;

pub fn vec3ApproxZero(vec: @This().Vec3) bool {
    return vec.approxEqAbs(@This().Vec3.zero, zero_epsilon);
}

// Irritatingly, the vector classes do have this function
// but it's not public, so we have to make our own...
pub fn vec3Field(vec: @This().Vec3, index: comptime_int) Float {
    switch (index) {
        0 => return vec.x,
        1 => return vec.y,
        2 => return vec.z,
        else => @compileError("index out of bounds!"),
    }
}
