// What:     Unit tests for `stderr_filter.rs`, pulled in by
//           `#[cfg(test)] #[path = "stderr_filter_tests.rs"] mod tests;` at
//           the bottom of `stderr_filter.rs`. Compiles only under
//           `cargo nextest run` / `cargo test`; reaches the module items
//           (including private ones) via `use super::*` because this file is
//           the `tests` CHILD of stderr_filter.
// Why:      Keep the tests beside the code without inflating
//           `stderr_filter.rs` or its max-lines budget (sibling
//           `*_tests.rs` files are exempt from the linter).

// What:     `use super::should_suppress_stderr_line;` imports the private predicate
//           from the parent module.
// Why:      Tests should check the rule directly.
use super::should_suppress_stderr_line;

#[test]
fn suppresses_ghostty_unimplemented_osc_callback() {
    assert!(should_suppress_stderr_line(
        b"debug(stream): unimplemented OSC callback: .{ .context_signal = .{} }\n",
    ));
}

#[test]
fn keeps_other_ghostty_debug_lines() {
    assert!(!should_suppress_stderr_line(
        b"debug(stream): some other Ghostty diagnostic\n",
    ));
}

#[test]
fn keeps_non_utf8_lines_without_marker() {
    assert!(!should_suppress_stderr_line(b"\xff\xfe\xfd\n"));
}
