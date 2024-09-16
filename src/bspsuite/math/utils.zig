const std = @import("std");
const types = @import("types.zig");
const constants = @import("constants.zig");

pub fn floatApproxZero(value: types.Float) bool {
    return std.math.approxEqAbs(types.Float, value, 0.0, constants.zero_epsilon);
}

pub fn vec3ApproxZero(vec: types.Vec3) bool {
    return vec.approxEqAbs(types.Vec3.zero, constants.zero_epsilon);
}

// Irritatingly, the vector classes do have this function
// but it's not public, so we have to make our own...
pub fn vec3Field(vec: types.Vec3, index: comptime_int) types.Float {
    switch (index) {
        0 => return vec.x,
        1 => return vec.y,
        2 => return vec.z,
        else => @compileError("Index out of bounds"),
    }
}
