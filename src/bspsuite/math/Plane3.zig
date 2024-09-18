const std = @import("std");
const types = @import("types.zig");
const constants = @import("constants.zig");
const utils = @import("utils.zig");

const Vec3 = types.Vec3;
const Vec3Normal = @import("vec3normal.zig").Vec3Normal;
const Float = types.Float;
const testing = std.testing;

const This = @This();

pub const PointClassification = enum(i2) {
    behind_plane = -1,
    on_plane = 0,
    in_front_of_plane = 1,
};

const null_plane: This = .{};
const default_epsilon_point_on_plane: Float = 0.1;

normal: Vec3Normal = Vec3Normal.null_normal,
dist: Float = 0.0,

// Asserts that normal vector is normalised
pub fn new(normal: Vec3, dist: Float) This {
    // We use length2() for this check since 1 squared is just 1 anyway,
    // so saves us a redundant sqrt.
    std.debug.assert(std.math.approxEqRel(Float, normal.length2(), 1.0, constants.zero_epsilon));

    return .{
        .normal = Vec3Normal.createFromUnitVector(normal),
        .dist = dist,
    };
}

// Dir vector is normalised as part of this call.
// If your direction is already normalised, just
// call new().
pub fn newFromDir(dir: Vec3, dist: Float) This {
    const normal = dir.normalize();
    return if (utils.vec3ApproxZero(normal)) This.null_plane else new(normal, dist);
}

pub fn eql(this: This, other: This) bool {
    if (this.isNull() and other.isNull()) {
        return true;
    }

    return this.normal.eql(other.normal) and this.dist == other.dist;
}

pub fn isNull(this: This) bool {
    return this.normal.isNull();
}

pub fn origin(this: This) Vec3 {
    return this.normal.scale(this.dist);
}

pub fn negate(this: This) This {
    return .{ .normal = this.normal.negate(), .dist = this.dist };
}

// Returns the perpendicular distance from the plane to the point,
// in the direction of the plane's normal.
pub fn distanceToPoint(this: This, point: Vec3) Float {
    return this.normal.dot(point) - this.dist;
}

pub fn projectPointOnPlane(this: This, point: Vec3) Vec3 {
    // Avoid rounding errors by doing things this way:
    return switch (this.normal) {
        .axial => |axis| switch (axis) {
            .null_axis => point,
            .xpos => Vec3.new(this.dist, point.y, point.z),
            .ypos => Vec3.new(point.x, this.dist, point.z),
            .zpos => Vec3.new(point.x, point.y, this.dist),
            .xneg => Vec3.new(-this.dist, point.y, point.z),
            .yneg => Vec3.new(point.x, -this.dist, point.z),
            .zneg => Vec3.new(point.x, point.y, -this.dist),
        },
        .nonaxial => |norm_vec| {
            const plane_origin = norm_vec.scale(this.dist);
            const plane_origin_to_point = point.sub(plane_origin);
            const dot_product = plane_origin_to_point.dot(norm_vec);
            return point.sub(norm_vec.scale(dot_product));
        },
    };
}

// Returns null if there is no singular intersection point,
// either because the line is separate and parallel to the
// plane, or lies in the plane.
pub fn intersectionPointWithLine(this: This, p0: Vec3, p1: Vec3) ?Vec3 {
    if (this.isNull()) {
        return null;
    }

    // Direction unit vector of the line. normalize() already
    // caters for zero length (the vector will be left as zero).
    const line_dir = p1.sub(p0).normalize();

    const normal_dot_line_dir = this.normal.dot(line_dir);

    if (utils.floatApproxZero(normal_dot_line_dir)) {
        // Line is perpendicular to normal, so parallel to plane.
        return null;
    }

    // How many units along the line we go for every 1 unit
    // along the plane normal.
    const units_along_line_per_unit_along_normal = 1.0 / normal_dot_line_dir;

    // Distance from p0 to the plane along the plane normal.
    // distanceToPoint() gives us the distance from the plane
    // to p0, so we have to negate it.
    const dist_to_p0 = -this.distanceToPoint(p0);

    // How many units we have to go along the line to reach the plane.
    const units_along_line_to_reach_plane = units_along_line_per_unit_along_normal * dist_to_p0;

    // Add this many multiples of the line dir to the original point.
    const delta_from_p0 = line_dir.scale(units_along_line_to_reach_plane);
    return p0.add(delta_from_p0);
}

// These basis vectors are right-handed. If the normal is pointing in the
// thumb direction, the first basis vector will point in the index finger
// direction, and the second in the middle finger direction.
pub fn basisVectors(this: This) [2]Vec3 {
    return switch (this.normal) {
        .axial => |axis| switch (axis) {
            .null_axis => .{ Vec3.new(0.0, 0.0, 0.0), Vec3.new(0.0, 0.0, 0.0) },
            .xpos => .{ Vec3.new(0.0, 1.0, 0.0), Vec3.new(0.0, 0.0, 1.0) },
            .ypos => .{ Vec3.new(-1.0, 0.0, 0.0), Vec3.new(0.0, 0.0, 1.0) },
            .zpos => .{ Vec3.new(1.0, 0.0, 0.0), Vec3.new(0.0, 1.0, 0.0) },
            .xneg => .{ Vec3.new(0.0, -1.0, 0.0), Vec3.new(0.0, 0.0, 1.0) },
            .yneg => .{ Vec3.new(1.0, 0.0, 0.0), Vec3.new(0.0, 0.0, 1.0) },
            .zneg => .{ Vec3.new(1.0, 0.0, 0.0), Vec3.new(0.0, -1.0, 0.0) },
        },
        .nonaxial => |norm_vec| {
            if (utils.vec3ApproxZero(norm_vec)) {
                return .{ Vec3.new(0.0, 0.0, 0.0), Vec3.new(0.0, 0.0, 0.0) };
            }

            const first_basis_dir: Vec3 = choose_basis: {
                const abs_x = @abs(norm_vec.x);
                const abs_y = @abs(norm_vec.y);
                const abs_z = @abs(norm_vec.z);

                if (abs_z >= abs_x and abs_z >= abs_y) {
                    break :choose_basis Vec3.new(1.0, 0.0, 0.0);
                } else if (abs_y >= abs_x and abs_y >= abs_z) {
                    break :choose_basis Vec3.new(-1.0 * std.math.sign(norm_vec.y), 0.0, 0.0);
                } else {
                    break :choose_basis Vec3.new(0.0, std.math.sign(norm_vec.x), 0.0);
                }
            };

            const plane_origin = this.origin();
            const first_plane_point = this.projectPointOnPlane(plane_origin.add(first_basis_dir));
            const first_basis = first_plane_point.sub(plane_origin).normalize();

            std.debug.assert(!utils.vec3ApproxZero(first_basis));

            return .{ first_basis, norm_vec.cross(first_basis) };
        },
    };
}

pub fn classifyPoint(this: This, point: Vec3) PointClassification {
    return classifyPointCustom(this, point, This.default_epsilon_point_on_plane);
}

pub fn classifyPointCustom(this: This, point: Vec3, epsilon: Float) PointClassification {
    const dist = this.distanceToPoint(point);

    if (dist < -epsilon) {
        return .behind_plane;
    } else if (dist > epsilon) {
        return .in_front_of_plane;
    } else {
        return .on_plane;
    }
}

test "Null plane contains only zero values" {
    const plane = This.null_plane;
    const normal = plane.normal.toVector();

    try testing.expectEqual(normal.x, 0.0);
    try testing.expectEqual(normal.y, 0.0);
    try testing.expectEqual(normal.z, 0.0);
    try testing.expectEqual(plane.dist, 0.0);

    try testing.expectEqual(Vec3.zero, plane.origin());
    try testing.expect(plane.isNull());
    try testing.expect(plane.eql(This.null_plane));
    try testing.expect(plane.normal.isNull());
}

test "Plane origin is defined by its normal and distance" {
    const plane = This.new(Vec3.new(0.0, 0.0, 1.0), 12.0);
    try testing.expectEqual(Vec3.new(0.0, 0.0, 12.0), plane.origin());
}

test "Points can be classified as in front of, behind, or on a plane" {
    const plane = This.new(Vec3.unitX, 10.0);

    try testing.expectEqual(PointClassification.in_front_of_plane, plane.classifyPoint(Vec3.new(12.0, 1.0, 5.0)));
    try testing.expectEqual(PointClassification.behind_plane, plane.classifyPoint(Vec3.new(-2.0, 1.0, 5.0)));
    try testing.expectEqual(PointClassification.on_plane, plane.classifyPoint(Vec3.new(10.0, 1.0, 5.0)));

    try testing.expectEqual(PointClassification.on_plane, plane.classifyPointCustom(Vec3.new(10.1, 1.0, 5.0), 0.2));
}

test "Points can be projected onto a plane via its normal" {
    const plane = This.newFromDir(Vec3.new(1.0, 1.0, 0.0), 0.0);
    const point = Vec3.new(-1.0, 2.0, 1.0);
    const expected_projected_point = Vec3.new(-1.5, 1.5, 1.0);
    const actual_projected_point = plane.projectPointOnPlane(point);

    try testing.expect(expected_projected_point.approxEqRel(actual_projected_point, constants.zero_epsilon));
}

test "The intersection point can be determined between the plane and a line" {
    const p0 = Vec3.new(-1.0, 2.0, 1.0);
    const p1 = Vec3.new(-2.0, 1.0, 1.0);

    try testing.expect(This.null_plane.intersectionPointWithLine(p0, p1) == null);

    const plane = This.newFromDir(Vec3.new(1.0, 1.0, 0.0), 0.0);
    const expected_intersection_point = Vec3.new(-1.5, 1.5, 1.0);
    const actual_intersection_point: Vec3 = plane.intersectionPointWithLine(p0, p1) orelse unreachable;

    try testing.expect(expected_intersection_point.approxEqRel(actual_intersection_point, constants.zero_epsilon));

    const p2 = Vec3.new(-2.0, 3.0, 2.0);
    const no_intersection_point: ?Vec3 = plane.intersectionPointWithLine(p0, p2);

    try testing.expect(no_intersection_point == null);
}

test "Basis vectors can be generated from a plane" {
    const null_basis_vectors = This.null_plane.basisVectors();

    try testing.expect(null_basis_vectors[0].eql(Vec3.zero));
    try testing.expect(null_basis_vectors[1].eql(Vec3.zero));

    {
        const plane = This.newFromDir(Vec3.new(1.0, 1.0, 0.0), 0.0);
        const basis_vectors: [2]Vec3 = plane.basisVectors();
        const expected_basis_right = Vec3.new(-1.0, 1.0, 0.0).normalize();
        const expected_basis_up = Vec3.unitZ;

        try testing.expect(basis_vectors[0].approxEqRel(expected_basis_right, constants.zero_epsilon));
        try testing.expect(basis_vectors[1].approxEqRel(expected_basis_up, constants.zero_epsilon));
    }

    {
        const inverse_plane = This.newFromDir(Vec3.new(-1.0, -1.0, 0.0), 0.0);
        const basis_vectors: [2]Vec3 = inverse_plane.basisVectors();
        const expected_basis_right = Vec3.new(1.0, -1.0, 0.0).normalize();
        const expected_basis_up = Vec3.unitZ;

        try testing.expect(basis_vectors[0].approxEqRel(expected_basis_right, constants.zero_epsilon));
        try testing.expect(basis_vectors[1].approxEqRel(expected_basis_up, constants.zero_epsilon));
    }
}
