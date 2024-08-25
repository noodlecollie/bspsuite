const std = @import("std");
const math = @import("math.zig");
const testing = std.testing;

const This = @This();
const Allocator = std.mem.Allocator;
const ArrayType = std.ArrayListUnmanaged(math.Vec3);

const max_points: usize = 96;

allocator: Allocator,
points: ArrayType,

pub fn initEmpty(allocator: Allocator) !This {
    return .{
        .allocator = allocator,
        .points = try ArrayType.initCapacity(allocator, 0),
    };
}

pub fn initPoints(allocator: Allocator, points: []const math.Vec3) !This {
    if (points.len > This.max_points) {
        return error.TooManyPoints;
    }

    var winding: This = .{
        .allocator = allocator,
        .points = try ArrayType.initCapacity(allocator, points.len),
    };

    errdefer winding.deinit();

    try winding.points.appendSlice(winding.allocator, points);
    return winding;
}

pub fn duplicate(this: *const This) !This {
    return initPoints(this.allocator, this.points.items);
}

pub fn deinit(this: *This) void {
    this.points.deinit(this.allocator);
}

pub fn pointCount(this: *const This) usize {
    return this.points.items.len;
}

pub fn addPoint(this: *This, point: math.Vec3) !void {
    if (this.pointCount() >= This.max_points) {
        return error.TooManyPoints;
    }

    const new_item = try this.points.addOne(this.allocator);
    new_item = point;
}

pub fn getPoint(this: *const This, index: usize) ?math.Vec3 {
    if (index >= this.pointCount()) {
        return null;
    }

    return this.points.items[index];
}

test "An empty winding holds zero points" {
    var winding = try initEmpty(std.testing.allocator);
    defer winding.deinit();

    try testing.expectEqual(0, winding.pointCount());
}

test "A winding initialised with a slice of points holds these points" {
    var winding = try initPoints(std.testing.allocator, &.{
        math.Vec3.new(0.0, 0.0, 0.0),
        math.Vec3.new(1.0, 1.0, 1.0),
        math.Vec3.new(2.0, 2.0, 2.0),
    });

    defer winding.deinit();

    try testing.expectEqual(3, winding.pointCount());
    try testing.expectEqual(math.Vec3.new(0.0, 0.0, 0.0), winding.getPoint(0));
    try testing.expectEqual(math.Vec3.new(1.0, 1.0, 1.0), winding.getPoint(1));
    try testing.expectEqual(math.Vec3.new(2.0, 2.0, 2.0), winding.getPoint(2));
}

test "A duplicated winding holds a duplicated set of points from the original winding" {
    var winding = try initPoints(std.testing.allocator, &.{
        math.Vec3.new(0.0, 0.0, 0.0),
        math.Vec3.new(1.0, 1.0, 1.0),
        math.Vec3.new(2.0, 2.0, 2.0),
    });

    defer winding.deinit();

    var winding2 = try winding.duplicate();
    defer winding2.deinit();

    try testing.expectEqual(3, winding2.pointCount());
    try testing.expectEqual(math.Vec3.new(0.0, 0.0, 0.0), winding2.getPoint(0));
    try testing.expectEqual(math.Vec3.new(1.0, 1.0, 1.0), winding2.getPoint(1));
    try testing.expectEqual(math.Vec3.new(2.0, 2.0, 2.0), winding2.getPoint(2));
}
