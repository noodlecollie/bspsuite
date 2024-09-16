const std = @import("std");
const Winding = @import("bspsuite").Winding;

test "Dummy test" {
    var winding = Winding.initEmpty(std.testing.allocator);
    defer winding.deinit();

    try std.testing.expectEqual(0, winding.pointCount());
}
