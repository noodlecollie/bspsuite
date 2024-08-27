const std = @import("std");
const math = @import("math.zig");
const testing = std.testing;

const This = @This();
const Allocator = std.mem.Allocator;
const ArrayType = std.ArrayListUnmanaged(math.Vec3);
const Plane3 = math.Plane3;
const Vec3 = math.Vec3;

pub const Edge = [2]Vec3;

pub const Error = Allocator.Error || error{
    TooManyPoints,
    InvalidPlane,
};

// Helper for computing new points when a winding is split by a plane.
const WindingClipper = struct {
    pub const ClipPointTag = enum {
        existing_index,
        new_point,
    };

    pub const ClipPoint = union(ClipPointTag) {
        existing_index: usize,
        new_point: Vec3,

        pub fn isNewPoint(this: ClipPoint) bool {
            return switch (this) {
                .existing_index => false,
                .new_point => true,
            };
        }

        pub fn getPoint(this: ClipPoint, existing_points: []Vec3) Vec3 {
            return switch (this) {
                .existing_index => |index| existing_points[index],
                .new_point => |value| value,
            };
        }
    };

    pub const ClipEdge = [2]ClipPoint;
    pub const EdgeArrayType = std.ArrayListUnmanaged(ClipEdge);

    allocator: Allocator,
    edges: EdgeArrayType,

    pub fn init(allocator: Allocator, capacity: usize) Allocator.Error!WindingClipper {
        return .{
            .allocator = allocator,
            .edges = try EdgeArrayType.initCapacity(allocator, capacity),
        };
    }

    pub fn deinit(this: *WindingClipper) void {
        this.edges.deinit(this.allocator);
    }

    pub fn splitEdgeByPlane(this: *WindingClipper, edge: Edge, point_indices: [2]usize, plane: Plane3) Allocator.Error!void {
        const discard_point: [2]bool = .{
            plane.classifyPoint(edge[0]) == .in_front_of_plane,
            plane.classifyPoint(edge[1]) == .in_front_of_plane,
        };

        if (discard_point[0] and discard_point[1]) {
            // Nothing to add.
            return;
        }

        const new_edge: *ClipEdge = try this.edges.addOne(this.allocator);

        if (discard_point[0] and !discard_point[1]) {
            new_edge[0] = .{ .new_point = plane.projectPointOnPlane(edge[0]) };
            new_edge[1] = .{ .existing_index = point_indices[1] };
        } else if (discard_point[1] and !discard_point[0]) {
            new_edge[0] = .{ .existing_index = point_indices[0] };
            new_edge[1] = .{ .new_point = plane.projectPointOnPlane(edge[1]) };
        } else {
            new_edge[0] = .{ .existing_index = point_indices[0] };
            new_edge[1] = .{ .existing_index = point_indices[1] };
        }
    }

    pub fn pointCount(this: WindingClipper) usize {
        var count: usize = 0;

        // For an edge where the end point is new, we need to count
        // both points. Otherwise, we can just count the first point,
        // as the second will be catered for by the next edge.
        for (this.edges.items) |edge| {
            count += if (!edge[0].isNewPoint() and edge[1].isNewPoint()) 2 else 1;
        }

        return count;
    }
};

const max_points: usize = 96;

allocator: Allocator,
points: ArrayType,

pub fn initEmpty(allocator: Allocator) Allocator.Error!This {
    return .{
        .allocator = allocator,
        .points = try ArrayType.initCapacity(allocator, 0),
    };
}

pub fn initPoints(allocator: Allocator, points: []const Vec3) Error!This {
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

pub fn duplicate(this: This) Error!This {
    return initPoints(this.allocator, this.points.items);
}

pub fn deinit(this: *This) void {
    this.points.deinit(this.allocator);
}

pub fn pointCount(this: This) usize {
    return this.points.items.len;
}

pub fn addPoint(this: *This, point: Vec3) Error!void {
    if (this.pointCount() >= This.max_points) {
        return error.TooManyPoints;
    }

    const new_item = try this.points.addOne(this.allocator);
    new_item = point;
}

pub fn getPoint(this: This, index: usize) Vec3 {
    return this.points.items[index];
}

pub fn getEdge(this: This, index: usize) Edge {
    return .{ this.points.items[index], this.points.items[(index + 1) % this.pointCount()] };
}

// Removes any points on the winding that are strictly in front of the plane.
pub fn clip(this: *This, plane: Plane3) Error!void {
    if (plane.isNull()) {
        return error.InvalidPlane;
    }

    var clipper: WindingClipper = try WindingClipper.init(this.allocator, this.pointCount());
    defer clipper.deinit();

    for (0..this.pointCount()) |index| {
        const next_index: usize = (index + 1) % this.pointCount();
        const edge: Edge = this.getEdge(index);

        try clipper.splitEdgeByPlane(edge, .{ index, next_index }, plane);
    }

    const new_point_count: usize = clipper.pointCount();

    if (new_point_count > This.max_points) {
        return error.TooManyPoints;
    }

    var new_points: ArrayType = try ArrayType.initCapacity(this.allocator, new_point_count);
    errdefer new_points.deinit(this.allocator);

    const existing_points_slice: []Vec3 = this.points.items[0..this.points.items.len];

    for (clipper.edges.items) |edge| {
        const first_point: *Vec3 = new_points.addOneAssumeCapacity();
        first_point.* = edge[0].getPoint(existing_points_slice);

        if (!edge[0].isNewPoint() and edge[1].isNewPoint()) {
            const second_point: *Vec3 = new_points.addOneAssumeCapacity();
            second_point.* = edge[1].getPoint(existing_points_slice);
        }
    }

    this.points.deinit(this.allocator);
    this.points = new_points;
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

test "Clipping a winding creates new points appropriately" {
    var winding = try initPoints(std.testing.allocator, &.{
        Vec3.new(-1.0, -1.0, 0.0),
        Vec3.new(1.0, -1.0, 0.0),
        Vec3.new(1.0, 1.0, 0.0),
        Vec3.new(-1.0, 1.0, 0.0),
    });

    defer winding.deinit();

    const clip_plane: Plane3 = Plane3.new(Vec3.new(1.0, 0.0, 0.0), 0.0);
    try winding.clip(clip_plane);

    try testing.expectEqual(4, winding.pointCount());
    try testing.expectEqual(Vec3.new(-1.0, -1.0, 0.0), winding.getPoint(0));
    try testing.expectEqual(Vec3.new(0.0, -1.0, 0.0), winding.getPoint(1));
    try testing.expectEqual(Vec3.new(0.0, 1.0, 0.0), winding.getPoint(2));
    try testing.expectEqual(Vec3.new(-1.0, 1.0, 0.0), winding.getPoint(3));
}
