// What:     Unit tests for `input.rs`, pulled in by
//           `#[cfg(test)] #[path = "input_tests.rs"] mod tests;` at
//           the bottom of `input.rs`. Compiles only under
//           `cargo nextest run` / `cargo test`; reaches the module items
//           (including private ones) via `use super::*` because this file is
//           the `tests` CHILD of input.
// Why:      Keep the tests beside the code without inflating
//           `input.rs` or its max-lines budget (sibling
//           `*_tests.rs` files are exempt from the linter).

// What:     `use super::encode_terminal_key;` imports the function under test from
//           the parent module.
// Why:      Test code can call it without repeating the module path.
use super::encode_terminal_key;

#[test]
fn encodes_printable_text() {
    assert_eq!(encode_terminal_key("a", false, false), Some(vec![b'a']));
}

#[test]
fn encodes_control_letter() {
    assert_eq!(encode_terminal_key("c", true, false), Some(vec![3]));
}

#[test]
fn encodes_alt_prefix() {
    assert_eq!(encode_terminal_key("x", false, true), Some(vec![0x1b, b'x']));
}

#[test]
fn encodes_arrows() {
    assert_eq!(
        encode_terminal_key("UpArrow", false, false),
        Some(b"\x1b[A".to_vec()),
    );
}

#[test]
fn encodes_function_keys() {
    assert_eq!(
        encode_terminal_key("F5", false, false),
        Some(b"\x1b[15~".to_vec()),
    );
}

#[test]
fn encodes_insert_and_backtab() {
    assert_eq!(
        encode_terminal_key("Insert", false, false),
        Some(b"\x1b[2~".to_vec()),
    );
    assert_eq!(
        encode_terminal_key("Backtab", false, false),
        Some(b"\x1b[Z".to_vec()),
    );
}

#[test]
fn ignores_empty_text() {
    assert_eq!(encode_terminal_key("", false, false), None);
}
