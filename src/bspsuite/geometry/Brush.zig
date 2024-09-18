const std = @import("std");
const math = @import("../math.zig");
const testing = std.testing;

const This = @This();
const Allocator = std.mem.Allocator;
const Float = math.Float;
const Vec3 = math.Vec3;
const Vec3Normal = math.Vec3Normal;
const VertexArray = std.ArrayList(Vec3);
const FaceArray = std.ArrayList(Face);
const Winding = @import("Winding.zig");

const max_faces: usize = 64;
const max_vertices: usize = 128;

pub const Error = Allocator.Error || error{
    TooManyVertices,
    TooManyIndices,
    TooManyFaces,
};

const Face = struct {
    const IndexArray = std.ArrayList(usize);
    const max_indices: usize = 64;

    normal: Vec3Normal,
    vert_indices: IndexArray,

    pub fn init(allocator: Allocator, normal: Vec3Normal) Face {
        return .{
            .normal = normal,
            .vert_indices = IndexArray.init(allocator),
        };
    }

    pub fn deinit(this: Face) void {
        this.vert_indices.deinit();
    }

    pub fn appendIndex(this: *Face, index: usize) !void {
        if (this.vert_indices.items.len >= max_indices) {
            return Error.TooManyIndices;
        }

        this.vert_indices.append(index);
    }
};

allocator: Allocator,
vertices: VertexArray,
faces: FaceArray,

pub fn initEmpty(allocator: Allocator) This {
    return .{
        .allocator = allocator,
        .vertices = VertexArray.init(allocator),
        .faces = FaceArray.init(allocator),
    };
}

pub fn deinit(this: This) void {
    for (this.faces.items) |face| {
        face.deinit();
    }

    this.faces.deinit();
    this.vertices.deinit();
}

pub fn vertexCount(this: This) usize {
    return this.vertices.items.len;
}

pub fn faceCount(this: This) usize {
    return this.faces.items.len;
}

pub fn addFace(this: *This, winding: Winding) Error!void {
    if (winding.isEmpty()) {
        return;
    }

    if (this.faces.items.len >= max_faces) {
        return Error.TooManyFaces;
    }

    var face: *Face = this.faces.addOne();
    face = Face.init(this.allocator, winding.normal);

    const old_vertex_count: usize = this.vertices.items.len;
    errdefer this.resetVerticesAndPopFace(old_vertex_count);

    try this.addVerticesToFace(face, winding);
}

fn addVerticesToFace(this: *This, face: *Face, winding: Winding) !void {
    for (winding.points.items) |vertex| {
        if (this.indexOfVertex(vertex)) |existing_index| {
            try face.appendIndex(existing_index);
            continue;
        }

        if (this.vertices.items.len >= max_vertices) {
            return Error.TooManyVertices;
        }

        const new_index: usize = this.vertices.items.len;

        try this.vertices.append(vertex);
        try face.appendIndex(new_index);
    }
}

fn indexOfVertex(this: This, in_vertex: Vec3) ?usize {
    for (this.vertices.items, 0..) |index, vertex| {
        if (math.utils.vec3ApproxEqual(vertex, in_vertex)) {
            return index;
        }
    }

    return null;
}

fn resetVerticesAndPopFace(this: *This, old_count: usize) void {
    this.faces.items[this.faces.items.len - 1].deinit();
    this.faces.shrinkAndFree(this.faces.items.len - 1);
    this.vertices.shrinkAndFree(old_count);
}

test "An empty brush contains no vertices or faces" {
    var brush = This.initEmpty(testing.allocator);
    defer brush.deinit();

    try testing.expectEqual(0, brush.vertexCount());
    try testing.expectEqual(0, brush.faceCount());
}
