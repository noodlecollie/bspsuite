pub const math = @import("math.zig");
pub const Winding = @import("Winding.zig");

// Import all the tests that we want to run
test {
    _ = @import("Winding.zig");
    _ = @import("math.zig");
}
