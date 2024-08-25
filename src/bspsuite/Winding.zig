const std = @import("std");
const math = @import("math.zig");

const Allocator = std.mem.Allocator;
const testing = std.testing;

pub fn Winding(comptime ValueType: type) type {
    return struct {
        const This = @This();
        const ArrayType = std.ArrayListUnmanaged(ValueType);

        const max_points: usize = 96;

        allocator: Allocator,
        points: ArrayType,

        pub fn initEmpty(allocator: Allocator) !Winding(ValueType) {
            return .{
                .allocator = allocator,
                .points = try ArrayType.initCapacity(allocator, 0),
            };
        }

        pub fn deinit(this: *This) void {
            this.points.deinit(this.allocator);
        }

        pub fn pointCount(this: *const This) usize {
            return this.points.items.len;
        }
    };
}

test "An empty winding holds zero points" {
    const Winding3f = Winding(math.Vec3);
    var winding: Winding3f = try Winding3f.initEmpty(std.testing.allocator);
    defer winding.deinit();

    try testing.expectEqual(0, winding.pointCount());
}
