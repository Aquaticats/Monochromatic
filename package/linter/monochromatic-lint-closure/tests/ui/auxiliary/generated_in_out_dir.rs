// A file below the build script's output directory (`OUT_DIR` is `auxiliary` in this test), which is inside the
// package directory. It would be the package's own source by path, but it is generated, so the lint skips it.

/// Apply a closure that captures nothing and is not kept by the callee,
/// as generated code does.
pub fn generated_capture_free() -> bool {
    return "a1".bytes().all(|byte| byte.is_ascii_digit());
}
