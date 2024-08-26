const std = @import("std");
const math = @import("math.zig");
const testing = std.testing;

const This = @This();
const Allocator = std.mem.Allocator;
const ArrayType = std.ArrayListUnmanaged(math.Vec3);
const Plane3 = math.Plane3;
const Vec3 = math.Vec3;

pub const Edge = [2]Vec3;

// Helper for computing new points when a winding is split by a plane.
const WindingClipper = struct {
    pub const ClipPointTag = enum {
        existing_point,
        new_point,
    };

    pub const ClipPoint = union(ClipPointTag) {
        existing_index: usize,
        new_point: Vec3,

        pub fn isNewPoint(this: ClipPoint) bool {
            return switch (this) {
                .existing_index => false,
                .new_index => true,
            };
        }
    };

    pub const ClipEdge = [2]ClipPoint;
    pub const EdgeArrayType = std.ArrayListUnmanaged(ClipEdge);

    allocator: Allocator,
    edges: EdgeArrayType,

    pub fn init(allocator: Allocator, capacity: usize) !@This() {
        return .{
            .allocator = allocator,
            .edges = try EdgeArrayType.initCapacity(Allocator, capacity),
        };
    }

    pub fn splitEdgeByPlane(this: @This(), edge: Edge, point_indices: [2]usize, plane: Plane3) !void {
        const discard_point: [2]bool = .{
            plane.classifyPoint(edge[0]) == .in_front_of_plane,
            plane.classifyPoint(edge[1]) == .in_front_of_plane,
        };

        if (discard_point[0] and discard_point[1]) {
            // Nothing to add.
            return;
        }

        const new_edge: *ClipEdge = try this.edges.addOne(this.allocator);

        if (!discard_point[0] and !discard_point[1]) {
            new_edge[0] = edge[0];
            new_edge[1] = edge[1];
            return;
        }

        if (discard_point[0]) {
            new_edge[0] = plane.projectPointOnPlane(edge[0]);
            new_edge[1] = point_indices[1];
        } else {
            new_edge[0] = point_indices[0];
            new_edge[1] = plane.projectPointOnPlane(edge[1]);
        }
    }

    pub fn edgeCount(this: @This()) usize {
        var count = 0;

        for (this.edges) |edge| {
            if (!edge[0].isNewPoint() and !edge[1].isNewPoint()) {
                // No new point was inserted for this edge.
                count += 1;
            } else {
                // A new point was inserted, so include it in the count.
                count += 2;
            }
        }

        return count;
    }
};

const max_points: usize = 96;

allocator: Allocator,
points: ArrayType,

pub fn initEmpty(allocator: Allocator) !This {
    return .{
        .allocator = allocator,
        .points = try ArrayType.initCapacity(allocator, 0),
    };
}

pub fn initPoints(allocator: Allocator, points: []const Vec3) !This {
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

pub fn duplicate(this: This) !This {
    return initPoints(this.allocator, this.points.items);
}

pub fn deinit(this: *This) void {
    this.points.deinit(this.allocator);
}

pub fn pointCount(this: This) usize {
    return this.points.items.len;
}

pub fn addPoint(this: *This, point: Vec3) !void {
    if (this.pointCount() >= This.max_points) {
        return error.TooManyPoints;
    }

    const new_item = try this.points.addOne(this.allocator);
    new_item = point;
}

pub fn getPoint(this: This, index: usize) ?Vec3 {
    if (index >= this.pointCount()) {
        return null;
    }

    return this.points.items[index];
}

pub fn getEdge(this: This, index: usize) ?Edge {
    if (index >= this.pointCount()) {
        return null;
    }

    return .{ this.points.items[index], this.points.items[index % this.pointCount()] };
}

// Removes any points on the winding that are strictly in front of the plane.
pub fn clip(this: *This, plane: Plane3) !void {
    if (plane.isNull()) {
        return;
    }

    // TODO: Use WindingClipper
}

test "An empty winding holds zero points" {
    var winding = try initEmpty(std.testing.allocator);
    defer winding.deinit();

    try testing.expectEqual(0, winding.pointCount());
}

test "A winding initialised with a slice of points holds these points" {
    var winding = try initPoints(std.testing.allocator, &.{
        Vec3.new(0.0, 0.0, 0.0),
        Vec3.new(1.0, 1.0, 1.0),
        Vec3.new(2.0, 2.0, 2.0),
    });

    defer winding.deinit();

    try testing.expectEqual(3, winding.pointCount());
    try testing.expectEqual(Vec3.new(0.0, 0.0, 0.0), winding.getPoint(0));
    try testing.expectEqual(Vec3.new(1.0, 1.0, 1.0), winding.getPoint(1));
    try testing.expectEqual(Vec3.new(2.0, 2.0, 2.0), winding.getPoint(2));
}

test "A duplicated winding holds a duplicated set of points from the original winding" {
    var winding = try initPoints(std.testing.allocator, &.{
        Vec3.new(0.0, 0.0, 0.0),
        Vec3.new(1.0, 1.0, 1.0),
        Vec3.new(2.0, 2.0, 2.0),
    });

    defer winding.deinit();

    var winding2 = try winding.duplicate();
    defer winding2.deinit();

    try testing.expectEqual(3, winding2.pointCount());
    try testing.expectEqual(Vec3.new(0.0, 0.0, 0.0), winding2.getPoint(0));
    try testing.expectEqual(Vec3.new(1.0, 1.0, 1.0), winding2.getPoint(1));
    try testing.expectEqual(Vec3.new(2.0, 2.0, 2.0), winding2.getPoint(2));
}
