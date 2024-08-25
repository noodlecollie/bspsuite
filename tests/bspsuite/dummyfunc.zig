const std = @import("std");
const dummyfunc = @import("bspsuite").dummyfunc;

test "Dummy test" {
    try std.testing.expectEqual(1, dummyfunc.return1());
}
