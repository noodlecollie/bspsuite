const std = @import("std");

var in_target: std.Build.ResolvedTarget = undefined;
var in_optimize: std.builtin.OptimizeMode = undefined;
var lib_bspsuite: *std.Build.Step.Compile = undefined;

// Although this function looks imperative, note that its job is to
// declaratively construct a build graph that will be executed by an external
// runner.
pub fn build(b: *std.Build) !void {
    // Standard target options allows the person running `zig build` to choose
    // what target to build for. Here we do not override the defaults, which
    // means any target is allowed, and the default is native. Other options
    // for restricting supported target set are available.
    in_target = b.standardTargetOptions(.{});

    // Standard optimization options allow the person running `zig build` to select
    // between Debug, ReleaseSafe, ReleaseFast, and ReleaseSmall. Here we do not
    // set a preferred release mode, allowing the user to decide how to optimize.
    in_optimize = b.standardOptimizeOption(.{});

    const zlm = b.dependency("zlm", .{});

    lib_bspsuite = b.addStaticLibrary(.{
        .name = "bspsuite",
        .root_source_file = b.path("src/bspsuite/root.zig"),
        .target = in_target,
        .optimize = in_optimize,
    });

    lib_bspsuite.root_module.addImport("zlm", zlm.module("zlm"));

    const lib_unit_tests = b.addTest(.{
        .root_source_file = b.path("src/bspsuite/root.zig"),
        .target = in_target,
        .optimize = in_optimize,
    });

    lib_unit_tests.root_module.addImport("zlm", zlm.module("zlm"));

    const run_lib_unit_tests = b.addRunArtifact(lib_unit_tests);
    const install_lib_unit_tests = b.addInstallArtifact(lib_unit_tests, .{});

    // This declares intent for the library to be installed into the standard
    // location when the user invokes the "install" step (the default step when
    // running `zig build`).
    b.installArtifact(lib_bspsuite);

    // const exe = b.addExecutable(.{
    //     .name = "bspsuite",
    //     .root_source_file = b.path("src/main.zig"),
    //     .target = target,
    //     .optimize = optimize,
    // });

    // // This declares intent for the executable to be installed into the
    // // standard location when the user invokes the "install" step (the default
    // // step when running `zig build`).
    // b.installArtifact(exe);

    // // This *creates* a Run step in the build graph, to be executed when another
    // // step is evaluated that depends on it. The next line below will establish
    // // such a dependency.
    // const run_cmd = b.addRunArtifact(exe);

    // // By making the run step depend on the install step, it will be run from the
    // // installation directory rather than directly from within the cache directory.
    // // This is not necessary, however, if the application depends on other installed
    // // files, this ensures they will be present and in the expected location.
    // run_cmd.step.dependOn(b.getInstallStep());

    // // This allows the user to pass arguments to the application in the build
    // // command itself, like this: `zig build run -- arg1 arg2 etc`
    // if (b.args) |args| {
    //     run_cmd.addArgs(args);
    // }

    // // This creates a build step. It will be visible in the `zig build --help` menu,
    // // and can be selected like this: `zig build run`
    // // This will evaluate the `run` step rather than the default, which is "install".
    // const run_step = b.step("run", "Run the app");
    // run_step.dependOn(&run_cmd.step);

    // Creates a step for unit testing. This only builds the test executable
    // but does not run it.
    const separate_unit_tests = b.addTest(.{
        .root_source_file = b.path("tests/bspsuite/all.zig"),
        .target = in_target,
        .optimize = in_optimize,
    });

    separate_unit_tests.root_module.addImport("bspsuite", &lib_bspsuite.root_module);

    const run_separate_unit_tests = b.addRunArtifact(separate_unit_tests);

    // Similar to creating the run step earlier, this exposes a `test` step to
    // the `zig build --help` menu, providing a way for the user to request
    // running the unit tests.
    const test_step = b.step("test", "Build and run all unit tests");
    test_step.dependOn(&run_separate_unit_tests.step);
    test_step.dependOn(&run_lib_unit_tests.step);
    test_step.dependOn(&install_lib_unit_tests.step);

    const test_build_only = b.step("test-build-only", "Build unit tests without running them");
    test_build_only.dependOn(&lib_unit_tests.step);
    test_build_only.dependOn(&separate_unit_tests.step);
    test_build_only.dependOn(&install_lib_unit_tests.step);

    try addDemos(b);
}

// Reference for some of this behaviour:
// https://cookbook.ziglang.cc/01-03-file-modified-24h-ago.html
fn addDemos(b: *std.Build) !void {
    const demos_root = "src/demos";

    var demos_dir: std.fs.Dir = try std.fs.openDirAbsolute(b.path(demos_root).getPath(b), .{ .iterate = true });
    defer demos_dir.close();

    var iterator: std.fs.Dir.Iterator = demos_dir.iterate();

    while (try iterator.next()) |entry| {
        if (entry.kind != .directory) {
            continue;
        }

        const demo_exe = b.addExecutable(.{
            .name = entry.name,
            .root_source_file = b.path(b.pathJoin(&.{ demos_root, entry.name, "root.zig" })),
            .target = in_target,
            .optimize = in_optimize,
        });

        b.installArtifact(demo_exe);

        const run_cmd = b.addRunArtifact(demo_exe);
        run_cmd.step.dependOn(b.getInstallStep());

        if (b.args) |args| {
            run_cmd.addArgs(args);
        }
    }
}
