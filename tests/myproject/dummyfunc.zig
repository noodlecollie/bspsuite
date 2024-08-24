const std = @import("std");
const dummyfunc = @import("myproject").dummyfunc;

test "Dummy test" {
    try std.testing.expectEqual(1, dummyfunc.return1());
}
