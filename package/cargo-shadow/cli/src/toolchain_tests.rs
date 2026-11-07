//! The unit tests of the toolchain choice,
//! ported from `toolchain.unit.test.ts`.

use crate::test_scratch::scratch;
use crate::toolchain::{cargo_toolchain, rust_toolchain_from_lock};

/// The Rust toolchain the repository's own `toolchain.lock.json` records,
/// read with `serde_json` alone (not the function
/// under test) so the tests follow the lock instead of a hand-kept date.
fn pinned() -> String {
    let lock_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../toolchain.lock.json");
    let lock = std::fs::read_to_string(&lock_path).expect("read the repository's toolchain.lock.json");
    let parsed: serde_json::Value = serde_json::from_str(&lock).expect("parse the repository's toolchain.lock.json");
    let toolchain = parsed.pointer("/tools/rust/version").and_then(serde_json::Value::as_str).expect("the lock records a Rust toolchain");
    return toolchain.to_string();
}

fn lock_text() -> String {
    return format!("{{\"tools\":{{\"rust\":{{\"version\":\"{}\"}}}}}}", pinned());
}

#[test]
fn the_rust_version_is_read_from_the_tools_table() {
    assert_eq!(rust_toolchain_from_lock(&lock_text()), Ok(pinned()));
}

#[test]
fn a_lock_without_a_rust_entry_pins_nothing() {
    assert_eq!(rust_toolchain_from_lock("{\"tools\":{}}"), Ok(String::new()));
    assert_eq!(rust_toolchain_from_lock("{}"), Ok(String::new()));
}

#[test]
fn text_that_is_not_json_is_an_error() {
    assert!(rust_toolchain_from_lock("{").is_err());
}

#[test]
fn the_repository_pin_wins_over_an_inherited_toolchain_and_says_so() {
    let root = scratch();
    std::fs::write(root.path().join("toolchain.lock.json"), lock_text()).expect("write the lock");

    let (toolchain, warning) = cargo_toolchain(root.path(), "1.98.1").expect("choose");

    assert_eq!(toolchain, pinned());
    assert!(warning.expect("a warning").contains("1.98.1"));
}

#[test]
fn an_explicit_toolchain_applies_only_when_nothing_is_pinned() {
    let root = scratch();

    let (toolchain, warning) = cargo_toolchain(root.path(), "1.98.1").expect("choose");
    assert_eq!(toolchain, "1.98.1");
    assert_eq!(warning, None);

    let (empty, empty_warning) = cargo_toolchain(root.path(), "").expect("choose");
    assert_eq!(empty, "");
    assert!(empty_warning.is_some());
}
