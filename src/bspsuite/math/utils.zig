const std = @import("std");
const types = @import("types.zig");
const constants = @import("constants.zig");

pub fn floatApproxZero(value: types.Float) bool {
    return std.math.approxEqAbs(types.Float, value, 0.0, constants.zero_epsilon);
}

pub fn vec3ApproxZero(vec: types.Vec3) bool {
    return vec.approxEqAbs(types.Vec3.zero, constants.zero_epsilon);
}

pub fn vec3ApproxEqual(a: types.Vec3, b: types.Vec3) bool {
    inline for (0..3) |index| {
        const a_val: types.Float = vec3Field(a, index);
        const b_val: types.Float = vec3Field(b, index);

        const a_near_zero = @abs(a_val) < constants.zero_epsilon;
        const b_near_zero = @abs(b_val) < constants.zero_epsilon;

        if (a_near_zero != b_near_zero) {
            // We know these are not equal.
            return false;
        }

        if (a_near_zero and !std.math.approxEqAbs(types.Float, a_val, b_val, constants.zero_epsilon)) {
            // Both were near zero, but neither were considered equal.
            return false;
        }

        if (!a_near_zero and !std.math.approxEqRel(types.Float, a_val, b_val, constants.zero_epsilon)) {
            // Neither were near zero, and neither were considered equal.
            return false;
        }
    }

    // Passed all checks.
    return true;
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
