pub const Float = f64;

// We *shouldn't* need to use an explicit @Vector() for this,
// since apparently the compiler is pretty good at automatically
// applying vector operations where possible.
pub const Vec3 = [3]Float;
