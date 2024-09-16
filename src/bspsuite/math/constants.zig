const types = @import("types.zig");

// Taken from Quake compile tools:
// tolerance for comparing a floating
// point value against zero.
pub const zero_epsilon: types.Float = 0.0001;
