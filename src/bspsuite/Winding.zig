const std = @import("std");
const math = @import("math.zig");

const Allocator = std.mem.Allocator;
const testing = std.testing;

pub fn Winding(comptime ValueType: type) type {
    return struct {
        const This = @This();
        const ArrayType = std.ArrayListUnmanaged(ValueType);

        const max_points: usize = 96;

        pub fn initEmpty(allocator: Allocator) !This {
            return .{
                .allocator = allocator,
                .points = try ArrayType.initCapacity(allocator, 0),
            };
        }

        pub fn initPoints(allocator: Allocator, points: []const ValueType) !This {
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

        pub fn deinit(this: *This) void {
            this.points.deinit(this.allocator);
        }

        pub fn pointCount(this: *const This) usize {
            return this.points.items.len;
        }

        pub fn addPoint(this: *This, point: ValueType) !void {
            if (this.pointCount() >= This.max_points) {
                return error.TooManyPoints;
            }

            const new_item = try this.points.addOne(this.allocator);
            new_item = point;
        }

        pub fn getPoint(this: *const This, index: usize) ?ValueType {
            if (index >= this.pointCount()) {
                return null;
            }

            return this.points.items[index];
        }

        allocator: Allocator,
        points: ArrayType,
    };
}

test "An empty winding holds zero points" {
    var winding = try Winding(math.Vec3).initEmpty(std.testing.allocator);
    defer winding.deinit();

    try testing.expectEqual(0, winding.pointCount());
}

test "A winding initialised with a slice of points holds these points" {
    var winding = try Winding(math.Vec3).initPoints(std.testing.allocator, &.{
        .{ 0.0, 0.0, 0.0 },
        .{ 1.0, 1.0, 1.0 },
        .{ 2.0, 2.0, 2.0 },
    });

    defer winding.deinit();

    try testing.expectEqual(3, winding.pointCount());
    try testing.expectEqual(math.Vec3{ 0.0, 0.0, 0.0 }, winding.getPoint(0));
    try testing.expectEqual(math.Vec3{ 1.0, 1.0, 1.0 }, winding.getPoint(1));
    try testing.expectEqual(math.Vec3{ 2.0, 2.0, 2.0 }, winding.getPoint(0));
}
