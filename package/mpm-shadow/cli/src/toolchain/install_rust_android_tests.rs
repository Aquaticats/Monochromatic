//! Unit tests of the Rust component and target reconciler,
//! against the fake runner and a disposable scope:
//! the tests assert that the listing decides what is missing,
//! that a listed component carrying its host triple counts as installed,
//! and that a tool declaring no parts asks `rustup` nothing.

use super::fakes::Rig;
use super::install_rust_android::{RustPart, RustPartKind, missing_rust_parts, reconcile_rust_parts};
use super::model::LockedTool;

/// The sample lock's `rust` tool,
/// whose options declare the components `clippy` and `rust-src`
/// and the target `aarch64-linux-android`.
fn rust_tool(rig: &Rig) -> LockedTool {
    return rig.tool("rust");
}

/// What `rustup toolchain list` prints:
/// a toolchain another program installed at the locked version.
const LOCKED_TOOLCHAIN_LINE: &str = "nightly-2026-10-30-x86_64-unknown-linux-gnu\n";

#[test]
fn a_component_listed_with_its_host_triple_counts_as_installed() {
    let rig = Rig::new();
    rig.runner.answer("rustup toolchain list", LOCKED_TOOLCHAIN_LINE, 0);
    rig.runner.answer(
        "rustup component list --installed --toolchain nightly-2026-10-30",
        "clippy-x86_64-unknown-linux-gnu\nrust-src\n",
        0,
    );
    rig.runner.answer("rustup target list --installed --toolchain nightly-2026-10-30", "x86_64-unknown-linux-gnu\n", 0);
    let missing: Vec<RustPart> = missing_rust_parts(&rig.context(), &rust_tool(&rig)).expect("listed");
    assert_eq!(missing, vec![RustPart { kind: RustPartKind::Target, name: "aarch64-linux-android".to_string() }]);
}

#[test]
fn every_declared_part_counts_as_missing_when_rustup_lists_none() {
    let rig = Rig::new();
    let missing: Vec<RustPart> = missing_rust_parts(&rig.context(), &rust_tool(&rig)).expect("an empty listing is a listing");
    let names: Vec<String> = missing.iter().map(|part| return part.name.clone()).collect();
    assert_eq!(names, vec!["clippy".to_string(), "rust-src".to_string(), "aarch64-linux-android".to_string()]);
    assert_eq!(missing[0].kind, RustPartKind::Component);
    assert_eq!(missing[2].kind, RustPartKind::Target);
}

#[test]
fn a_failed_listing_fails_the_tool_before_any_add_runs() {
    let rig = Rig::new();
    rig.runner.fail("rustup component list --installed", "error: no installed toolchains");
    let error = reconcile_rust_parts(&rig.context(), &rust_tool(&rig)).expect_err("the listing failed");
    assert!(error.message.contains("rustup component list --installed failed for rust: error: no installed toolchains"), "{error}");
    assert_eq!(rig.runner.lines(), vec!["rustup component list --installed --toolchain nightly-2026-10-30".to_string()]);
}

#[test]
fn a_tool_that_declares_no_parts_asks_rustup_nothing() {
    let rig = Rig::new();
    let mut tool: LockedTool = rust_tool(&rig);
    tool.declaration.options.rust_components = None;
    tool.declaration.options.rust_targets = None;
    let added: Vec<String> = reconcile_rust_parts(&rig.context(), &tool).expect("nothing to do");
    assert!(added.is_empty());
    assert!(rig.runner.calls().is_empty(), "no listing is needed when the lock declares no part");
}

#[test]
fn the_added_sentence_names_the_part_and_the_listing_that_missed_it() {
    let rig = Rig::new();
    rig.runner.answer("rustup component list --installed --toolchain nightly-2026-10-30", "rust-src\n", 0);
    rig.runner.answer("rustup target list --installed --toolchain nightly-2026-10-30", "aarch64-linux-android\n", 0);
    let added: Vec<String> = reconcile_rust_parts(&rig.context(), &rust_tool(&rig)).expect("one component added");
    assert_eq!(
        added,
        vec![
            "rust: nightly-2026-10-30 was already installed; added the declared component clippy, which \
             `rustup component list --installed` did not show"
                .to_string()
        ]
    );
    assert_eq!(
        rig.runner.lines(),
        vec![
            "rustup component list --installed --toolchain nightly-2026-10-30".to_string(),
            "rustup target list --installed --toolchain nightly-2026-10-30".to_string(),
            "rustup component add clippy --toolchain nightly-2026-10-30".to_string(),
        ]
    );
}
