const std = @import("std");
const math = @import("../math.zig");
const testing = std.testing;

const This = @This();
const Allocator = std.mem.Allocator;
const PointArray = std.ArrayList(math.Vec3);
const Plane3 = math.Plane3;
const Vec3 = math.Vec3;
const Vec3Normal = math.Vec3Normal;
const Float = math.Float;

pub const Error = Allocator.Error || error{
    TooManyPoints,
    OutOfRange,
};

// Max number of points allowed on a winding
pub const max_points: usize = 96;

// This is used as the max absolute value for certain
// attributes of a winding:
// - The maximum distance that the plane origin is allowed to be from (0,0,0).
// - The maximum distance that a point is allowed to be from the plane origin
//   along any basis vector.
pub const max_extent: Float = @as(Float, std.math.maxInt(i32));
pub const invalid_extent = max_extent + 1.0;

normal: Vec3Normal,
points: PointArray,

pub fn initEmpty(allocator: Allocator) This {
    return .{
        .normal = Vec3Normal.null_normal,
        .points = PointArray.init(allocator),
    };
}

pub fn initFromPlane(allocator: Allocator, plane: Plane3) !This {
    if (plane.isNull()) {
        return initEmpty(allocator);
    }

    if (plane.origin().length() > max_extent) {
        return Error.OutOfRange;
    }

    const basis_vectors = plane.basisVectors();

    std.debug.assert(!basis_vectors[0].eql(Vec3.zero));
    std.debug.assert(!basis_vectors[1].eql(Vec3.zero));

    var winding: This = .{
        .normal = plane.normal,
        .points = try PointArray.initCapacity(allocator, 4),
    };

    // These points are specified in anticlockwise order,
    // so that the cross product matches the normal direction.
    winding.points.appendSliceAssumeCapacity(&.{
        basis_vectors[0].scale(-invalid_extent).add(basis_vectors[1].scale(invalid_extent)),
        basis_vectors[0].scale(-invalid_extent).add(basis_vectors[1].scale(-invalid_extent)),
        basis_vectors[0].scale(invalid_extent).add(basis_vectors[1].scale(-invalid_extent)),
        basis_vectors[0].scale(invalid_extent).add(basis_vectors[1].scale(invalid_extent)),
    });

    return winding;
}

pub fn duplicate(this: This) Allocator.Error!This {
    return .{
        .allocator = this.allocator,
        .points = try this.points.clone(),
    };
}

pub fn deinit(this: *This) void {
    this.points.deinit();
}

pub fn pointCount(this: This) usize {
    return this.points.items.len;
}

pub fn isEmpty(this: This) bool {
    return this.points.items.len < 1;
}

pub fn getPoint(this: This, index: usize) Vec3 {
    return this.points.items[index];
}

// Splits this winding and returns another winding containing
// the points that were in front of the plane, or null if there
// were no points in front of the plane.
pub fn split(this: *This, plane: Plane3) !?This {
    if (this.isEmpty() or plane.isNull()) {
        // No change, and no additional winding.
        return null;
    }

    // Keep capacity for all existing points, to avoid reallocations
    // most of the time.

    var this_points = PointArray.initCapacity(this.points.allocator, this.points.items.len);
    errdefer this_points.deinit();

    var other_points = PointArray.init(this.points.allocator, this.points.items.len);
    errdefer other_points.deinit();

    // For each edge, we add all points but the last to the relevant lists,
    // as the last point is guaranteed to be used in the next iteration
    // (or to wrap around to the first point).
    for (this.points.items, 0..) |_, index| {
        const edge = this.getEdgeRef(index);
        const split_result = this.performEdgeSplit(edge, plane);

        switch (split_result) {
            .no_split => |edge_classification| switch (edge_classification) {
                .behind_plane, .on_plane => {
                    this_points.append(edge[0].pos);
                },
                .in_front_of_plane => {
                    other_points.append(edge[0].pos);
                },
            },
            .split => |split_info| {
                if (split_info.first_point_is_behind_plane) {
                    this_points.append(edge[0].pos);
                    this_points.append(split_info.split_point);

                    other_points.append(split_info.split_point);
                } else {
                    other_points.append(edge[0].pos);
                    other_points.append(split_info.split_point);

                    this_points.append(split_info.split_point);
                }
            },
        }

        if (this_points.items.len > max_points or other_points.items.len > max_points) {
            return Error.TooManyPoints;
        }
    }

    finalisePoints(&this_points);
    finalisePoints(&other_points);

    this.points.deinit();
    this.points = this_points;

    if (other_points.items.len < 1) {
        // There was no other winding, so don't return anything.
        other_points.deinit();
        return null;
    }

    return .{ .points = other_points };
}

const SplitEdgeInfo = struct {
    first_point_is_behind_plane: bool,
    split_point: Vec3,
};

const SplitResultTag = enum {
    no_split,
    split,
};

const SplitResult = union(SplitResultTag) {
    no_split: Plane3.PointClassification,
    split: SplitEdgeInfo,
};

const EdgeRef = [2]struct {
    pos: Vec3,
    index: usize,
};

fn initFromPoints(points: PointArray) This {
    return .{
        .allocator = points.allocator,
        .points = points,
    };
}

fn getEdgeRef(this: This, index: usize) EdgeRef {
    const next_index = (index + 1) % this.pointCount();

    return .{
        .{ .pos = this.points.items[index], .index = index },
        .{ .pos = this.points.items[next_index], .index = next_index },
    };
}

fn performEdgeSplit(edge: EdgeRef, plane: Plane3) SplitResult {
    const classifications: [2]Plane3.PointClassification = .{
        plane.classifyPoint(edge[0]),
        plane.classifyPoint(edge[1]),
    };

    if (classifications[0] == classifications[1] or //
        math.floatApproxZero((edge[1].pos.sub(edge[0].pos)).length()))
    {
        // There was no split.
        return .{ .no_split = classifications[0] };
    }

    // If a single point is on the plane, we just return the classification
    // of the other point, since the plane doesn't technically split the edge.
    if (classifications[0] == .on_plane) {
        return .{ .no_split = classifications[1] };
    } else if (classifications[1] == .on_plane) {
        return .{ .no_split = classifications[0] };
    }

    // The only option left here is for one point to be behind
    // the plane and the other point to be in front.
    const intersection_point = plane.intersectionPointWithLine(edge[0].pos, edge[1].pos) orelse unreachable;

    return .{ .split = .{
        .first_point_is_behind_plane = classifications[0] == .behind_plane,
        .split_point = intersection_point,
    } };
}

fn finalisePoints(points: *PointArray) void {
    if (points.items.len < 3) {
        // Not enough points to be valid, so clear out the list.
        points.clearAndFree();
    } else {
        // Reallocate capacity to length we actually got.
        points.shrinkAndFree(points.items.len);
    }
}

test "An empty winding holds zero points" {
    var winding = initEmpty(std.testing.allocator);
    defer winding.deinit();

    try testing.expectEqual(0, winding.pointCount());
}

test "A winding can be created from a plane" {
    var winding = try initFromPlane(std.testing.allocator, Plane3.newFromDir(Vec3.unitZ, 10.0));
    defer winding.deinit();

    try testing.expectEqual(4, winding.pointCount());
    try testing.expect(winding.normal.eql(Vec3Normal.createFromUnitVector(Vec3.unitZ)));

    const dir0 = winding.points.items[1].sub(winding.points.items[0]);
    const dir1 = winding.points.items[2].sub(winding.points.items[0]);
    const computed_normal = dir0.cross(dir1).normalize();

    try testing.expect(winding.normal.toVector().eql(computed_normal));
}
