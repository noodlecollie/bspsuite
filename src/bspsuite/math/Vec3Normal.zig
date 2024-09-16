const std = @import("std");
const types = @import("types.zig");
const constants = @import("constants.zig");

const Vec3 = types.Vec3;
const Float = types.Float;
const testing = std.testing;

pub const Axis = enum(i3) {
    null_axis = 0,
    xpos = 1,
    ypos = 2,
    zpos = 3,
    xneg = -1,
    yneg = -2,
    zneg = -3,
};

pub const Axial = enum {
    axial,
    nonaxial,
};

// A class to represent a normal, with specific optimisations
// for normals which lie exactly in X, Y or Z. These optimisations
// help avoid rounding errors
pub const Vec3Normal = union(Axial) {
    axial: Axis,
    nonaxial: Vec3,

    pub const null_normal: Vec3Normal = .{ .axial = .null_axis };

    pub fn createOnAxis(axis: Axis) Vec3Normal {
        return .{ .axial = axis };
    }

    // Creates a normal from a unit vector (ie. a vector with length 1).
    // If the vector is exactly axial, generates an axial normal.
    // Asserts that the vector is null, or of length 1.
    pub fn createFromUnitVector(vec: Vec3) Vec3Normal {
        if (vec.eql(Vec3.zero)) {
            return .{ .axial = Axis.null_axis };
        }

        // We use length2() for this check since 1 squared is just 1 anyway,
        // so saves us a redundant sqrt.
        std.debug.assert(std.math.approxEqRel(Float, vec.length2(), 1.0, constants.zero_epsilon));

        if (vec.y == 0 and vec.z == 0) {
            if (vec.x == 1.0) {
                return .{ .axial = Axis.xpos };
            } else if (vec.x == -1.0) {
                return .{ .axial = Axis.xneg };
            }
        } else if (vec.x == 0 and vec.z == 0) {
            if (vec.y == 1.0) {
                return .{ .axial = Axis.ypos };
            } else if (vec.y == -1.0) {
                return .{ .axial = Axis.yneg };
            }
        } else if (vec.x == 0 and vec.y == 0) {
            if (vec.z == 1.0) {
                return .{ .axial = Axis.zpos };
            } else if (vec.z == -1.0) {
                return .{ .axial = Axis.zneg };
            }
        }

        return .{ .nonaxial = vec };
    }

    // Creates a normal from a unit vector (ie. a vector with length 1).
    // If the vector is close enough to axial given a numeric tolerance,
    // generates an axial normal.
    // Asserts that the vector is null, or of length 1.
    pub fn createFromUnitVectorApprox(vec: Vec3, tolerance: Float) Vec3Normal {
        if (vec.approxEqAbs(Vec3.zero, tolerance)) {
            return .{ .axial = Axis.null_axis };
        }

        // We use length2() for this check since 1 squared is just 1 anyway,
        // so saves us a redundant sqrt.
        std.debug.assert(std.math.approxEqRel(Float, vec.length2(), 1.0, constants.zero_epsilon));

        const x_is_zero: bool = std.math.approxEqAbs(Float, vec.x, 0.0, tolerance);
        const y_is_zero: bool = std.math.approxEqAbs(Float, vec.y, 0.0, tolerance);
        const z_is_zero: bool = std.math.approxEqAbs(Float, vec.z, 0.0, tolerance);

        if (y_is_zero and z_is_zero) {
            if (std.math.approxEqAbs(Float, vec.x - 1.0, 0.0, tolerance)) {
                return .{ .axial = Axis.xpos };
            } else if (std.math.approxEqAbs(Float, vec.x + 1.0, 0.0, tolerance)) {
                return .{ .axial = Axis.xneg };
            }
        } else if (x_is_zero and z_is_zero) {
            if (std.math.approxEqAbs(Float, vec.y - 1.0, 0.0, tolerance)) {
                return .{ .axial = Axis.ypos };
            } else if (std.math.approxEqAbs(Float, vec.y + 1.0, 0.0, tolerance)) {
                return .{ .axial = Axis.yneg };
            }
        } else if (x_is_zero and y_is_zero) {
            if (std.math.approxEqAbs(Float, vec.z - 1.0, 0.0, tolerance)) {
                return .{ .axial = Axis.zpos };
            } else if (std.math.approxEqAbs(Float, vec.z + 1.0, 0.0, tolerance)) {
                return .{ .axial = Axis.zneg };
            }
        }

        return .{ .nonaxial = vec };
    }

    // Creates a normal from a directional vector, which is automatically
    // converted to be of unit length.
    // If the vector is exactly axial, generates an axial normal.
    pub fn createFromVector(vec: Vec3) Vec3Normal {
        return createFromUnitVector(vec.normalize());
    }

    // Creates a normal from a directional vector, which is automatically
    // converted to be of unit length.
    // If the vector is close enough to axial given a numeric tolerance,
    // generates an axial normal.
    pub fn createFromVectorApprox(vec: Vec3, tolerance: Float) Vec3Normal {
        return createFromUnitVectorApprox(vec.normalize(), tolerance);
    }

    // Explicitly creates a non-axial normal, even if the vector does
    // lie along an axis.
    // Asserts that the vector is null, or of length 1.
    pub fn createNonAxialFromUnitVector(vec: Vec3) Vec3Normal {
        std.debug.assert(vec.eql(Vec3.zero) or std.math.approxEqRel(Float, vec.length2(), 1.0, constants.zero_epsilon));
        return .{ .nonaxial = vec };
    }

    // Explicitly creates a non-axial normal (which is automatically
    // converted to be of unit length), even if the vector does
    // lie along an axis.
    pub fn createNonAxialFromVector(vec: Vec3) Vec3Normal {
        return createNonAxialFromUnitVector(vec.normalize());
    }

    // Returns the vector representation of this normal.
    // This will always be a unit vector unless the normal
    // is null, in which case it will be a zero vector.
    pub fn toVector(this: Vec3Normal) Vec3 {
        return switch (this) {
            .axial => |axis| switch (axis) {
                .null_axis => Vec3.zero,
                .xpos => Vec3.new(1.0, 0.0, 0.0),
                .ypos => Vec3.new(0.0, 1.0, 0.0),
                .zpos => Vec3.new(0.0, 0.0, 1.0),
                .xneg => Vec3.new(-1.0, 0.0, 0.0),
                .yneg => Vec3.new(0.0, -1.0, 0.0),
                .zneg => Vec3.new(0.0, 0.0, -1.0),
            },
            .nonaxial => |normal| normal,
        };
    }

    // Returns whether the normal is null.
    // Creating a normal using .null_axis
    // should be preferred over a zero vector,
    // as the check is more reliable.
    pub fn isNull(this: Vec3Normal) bool {
        return switch (this) {
            .axial => |axis| axis == .null_axis,
            .nonaxial => |vec| vec.eql(Vec3.zero),
        };
    }

    // Returns whether the normal is axial, ie.
    // it is explicitly defined to be along the X,
    // Y or Z axis. A null normal is not axial.
    pub fn isAxial(this: Vec3Normal) bool {
        return switch (this) {
            .axial => |axis| axis != .null_axis,
            .nonaxial => false,
        };
    }

    // Returns whether this normal is strictly equal to another normal.
    // Both Normals must be of the same type, ie. either axial or
    // non-axial.
    pub fn eql(this: Vec3Normal, other: Vec3Normal) bool {
        return switch (this) {
            .axial => |axis| {
                if (!other.isAxial()) {
                    return false;
                }

                switch (other) {
                    .axial => |other_axis| return axis == other_axis,
                    .nonaxial => unreachable,
                }
            },
            .nonaxial => |vec| {
                return !other.isAxial() and vec.eql(other.toVector());
            },
        };
    }

    // Returns whether this normal is equivalent to another normal,
    // ie. whether the vectors they generate are equal.
    pub fn equivalent(this: Vec3Normal, other: Vec3Normal) bool {
        return this.toVector().eql(other.toVector());
    }

    // Returns the normal's vector scaled by a specific value.
    // This helps avoid rounding errors.
    pub fn scale(this: Vec3Normal, value: Float) Vec3 {
        return switch (this) {
            .axial => |axis| switch (axis) {
                .null_axis => Vec3.zero,
                .xpos => Vec3.new(value, 0.0, 0.0),
                .ypos => Vec3.new(0.0, value, 0.0),
                .zpos => Vec3.new(0.0, 0.0, value),
                .xneg => Vec3.new(-value, 0.0, 0.0),
                .yneg => Vec3.new(0.0, -value, 0.0),
                .zneg => Vec3.new(0.0, 0.0, -value),
            },
            .nonaxial => |vec| vec.scale(value),
        };
    }

    pub fn dot(this: Vec3Normal, other: Vec3) Float {
        return switch (this) {
            .axial => |axis| switch (axis) {
                .null_axis => 0,
                .xpos => other.x,
                .ypos => other.y,
                .zpos => other.z,
                .xneg => -other.x,
                .yneg => -other.y,
                .zneg => -other.z,
            },
            .nonaxial => |vec| vec.dot(other),
        };
    }

    pub fn negate(this: Vec3Normal) Vec3Normal {
        return switch (this.normal) {
            .axial => |axis| switch (axis) {
                .null_axis => .{ .axial = .null_axis },
                .xpos => .{ .axial = .xneg },
                .ypos => .{ .axial = .yneg },
                .zpos => .{ .axial = .zneg },
                .xneg => .{ .axial = .xpos },
                .yneg => .{ .axial = .ypos },
                .zneg => .{ .axial = .zpos },
            },
            .nonaxial => |norm_vec| .{ .nonaxial = norm_vec.neg() },
        };
    }
};

test "A null normal represents a zero vector" {
    const normal: Vec3Normal = Vec3Normal.null_normal;

    try testing.expect(normal.isNull());
    try testing.expect(!normal.isAxial());
    try testing.expectEqual(normal.toVector(), Vec3.zero);

    const normal2 = Vec3Normal.createOnAxis(.null_axis);

    try testing.expect(normal2.isNull());
    try testing.expect(!normal2.isAxial());
    try testing.expectEqual(normal2.toVector(), Vec3.zero);
}

test "An axial normal represents a unit vector in a specific axis" {
    try testing.expectEqual(Vec3Normal.createOnAxis(.null_axis).toVector(), Vec3.zero);
    try testing.expectEqual(Vec3Normal.createOnAxis(.xpos).toVector(), Vec3.new(1.0, 0.0, 0.0));
    try testing.expectEqual(Vec3Normal.createOnAxis(.ypos).toVector(), Vec3.new(0.0, 1.0, 0.0));
    try testing.expectEqual(Vec3Normal.createOnAxis(.zpos).toVector(), Vec3.new(0.0, 0.0, 1.0));
    try testing.expectEqual(Vec3Normal.createOnAxis(.xneg).toVector(), Vec3.new(-1.0, 0.0, 0.0));
    try testing.expectEqual(Vec3Normal.createOnAxis(.yneg).toVector(), Vec3.new(0.0, -1.0, 0.0));
    try testing.expectEqual(Vec3Normal.createOnAxis(.zneg).toVector(), Vec3.new(0.0, 0.0, -1.0));
}

test "A normal from a vector close to a unit vector can create an axial normal if desired" {
    const normal: Vec3Normal = Vec3Normal.createFromVectorApprox(Vec3.new(0.99, 0.01, 0.0), 0.05);
    try testing.expect(normal.isAxial());
    try testing.expectEqual(normal.toVector(), Vec3.unitX);
}

test "A normal from a vector close to a unit vector can also create a non-axial normal if desired" {
    const normal: Vec3Normal = Vec3Normal.createFromVector(Vec3.new(0.99, 0.1, 0.0));

    try testing.expect(!normal.isAxial());
    try testing.expectEqual(normal.toVector(), Vec3.new(0.99, 0.1, 0.0).normalize());
}

test "A non-axial normal can be created from a vector on an axis" {
    const normal: Vec3Normal = Vec3Normal.createNonAxialFromVector(Vec3.unitZ);

    try testing.expect(!normal.isAxial());
    try testing.expectEqual(normal.toVector(), Vec3.unitZ);
}

test "Normals can be compared for equality" {
    const normal1: Vec3Normal = Vec3Normal.createOnAxis(.ypos);
    const normal2: Vec3Normal = Vec3Normal.createFromVector(Vec3.unitY);

    try testing.expect(normal1.eql(normal2));
}
