// A file beside the package directory, as a path module reached with `#[path = "../outside/generated.rs"]` is.
// Every closure here would be reported if the file were the package's own source; none is reported
// because the lint skips a file outside the package directory.

/// Apply a closure that captures nothing and is not kept by the callee,
/// as generated code does.
pub fn outside_capture_free() -> bool {
    return "a1".bytes().all(|byte| byte.is_ascii_digit());
}

/// Apply a closure that captures a variable and is not kept by the callee.
pub fn outside_borrowing(limit: usize) -> Vec<usize> {
    return (0..10).filter(|value| *value < limit).collect();
}
