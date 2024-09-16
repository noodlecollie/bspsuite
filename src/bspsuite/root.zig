pub const math = @import("math.zig");
pub const geometry = @import("geometry.zig");

// Import all the tests that we want to run
test {
    _ = @import("math.zig");
    _ = @import("geometry.zig");
}
