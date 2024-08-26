const std = @import("std");
const types = @import("types.zig");

const Vec3 = types.Vec3;
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

normal: Vec3 = Vec3.zero,
dist: Float = 0.0,

// Asserts that normal vector is normalised
pub fn new(normal: Vec3, dist: Float) This {
    // We use length2() for this check since 1 squared is just 1 anyway,
    // so saves us a redundant sqrt.
    std.debug.assert(std.math.approxEqRel(Float, normal.length2(), 1.0, types.zero_epsilon));

    return .{
        .normal = normal.normalize(),
        .dist = dist,
    };
}

// Dir vector is normalised as part of this call.
// If your direction is already normalised, just
// call new().
pub fn newFromDir(dir: Vec3, dist: Float) This {
    const normal = dir.normalize();
    return if (types.vec3ApproxZero(normal)) This.null_plane else new(normal, dist);
}

pub fn eql(this: This, other: This) bool {
    if (this.isNull() and other.isNull()) {
        return true;
    }

    return this.normal.eql(other.normal) and this.dist == other.dist;
}

pub fn isNull(this: This) bool {
    return types.vec3ApproxZero(this.normal);
}

pub fn origin(this: This) Vec3 {
    return this.normal.scale(this.dist);
}

pub fn distanceToPoint(this: This, point: Vec3) Float {
    return point.dot(this.normal) - this.dist;
}

pub fn projectPointOnPlane(this: This, point: Vec3) Vec3 {
    if (this.isNull()) {
        return Vec3.zero;
    }

    const point_to_plane_origin = point.sub(this.origin());
    const dot_product = point_to_plane_origin.dot(this.normal);
    return point.sub(this.normal.scale(dot_product));
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

    try testing.expectEqual(plane.normal.x, 0.0);
    try testing.expectEqual(plane.normal.y, 0.0);
    try testing.expectEqual(plane.normal.z, 0.0);
    try testing.expectEqual(plane.dist, 0.0);

    try testing.expectEqual(Vec3.zero, plane.origin());
    try testing.expect(plane.isNull());
    try testing.expect(plane.eql(This.null_plane));
}

test "Plane origin is defined by its normal and distance" {
    const plane = This.new(Vec3.new(0.0, 0.0, 1.0), 12.0);
    try testing.expectEqual(Vec3.new(0.0, 0.0, 12.0), plane.origin());
}

test "Points can be classified as in front of, behind, or on a plane" {
    const plane = This.new(Vec3.new(1.0, 0.0, 0.0), 10.0);

    try testing.expectEqual(PointClassification.in_front_of_plane, plane.classifyPoint(Vec3.new(12.0, 1.0, 5.0)));
    try testing.expectEqual(PointClassification.behind_plane, plane.classifyPoint(Vec3.new(-2.0, 1.0, 5.0)));
    try testing.expectEqual(PointClassification.on_plane, plane.classifyPoint(Vec3.new(10.0, 1.0, 5.0)));

    try testing.expectEqual(PointClassification.on_plane, plane.classifyPointCustom(Vec3.new(10.1, 1.0, 5.0), 0.2));
}
