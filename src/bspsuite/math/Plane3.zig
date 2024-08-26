const std = @import("std");
const types = @import("types.zig");

const Vec3 = types.Vec3;
const Float = types.Float;
const testing = std.testing;

const This = @This();

const PointClassification = enum(i2) {
    behind_plane = -1,
    on_plane = 0,
    in_front_of_plane = 1,
};

const null_plane: This = .{};
const epsilon_point_on_plane: Float = 0.1;

normal: Vec3 = Vec3.zero,
dist: Float = 0.0,

pub fn new(normal: Vec3, dist: Float) This {
    return .{
        .normal = normal,
        .dist = dist,
    };
}

pub fn eql(this: This, other: This) bool {
    return this.normal.eql(other.normal) and this.dist == other.dist;
}

pub fn isNull(this: This) bool {
    return this.eql(This.null_plane);
}

pub fn origin(this: This) Vec3 {
    return this.normal.scale(this.dist);
}

pub fn distanceToPoint(this: This, point: Vec3) Float {
    return point.dot(this.normal) - this.dist;
}

pub fn classifyPoint(this: This, point: Vec3) PointClassification {
    return classifyPointCustom(this, point, This.epsilon_point_on_plane);
}

pub fn classifyPointCustom(this: This, point: Vec3, epsilon: Float) PointClassification {
    const dist = this.distanceTo(point);

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
}
