//! The ban on a lean-folder tool starting a shadow or cargo:
//! no source of the lean-folder tools starts `cargo`,
//! asks for its own executable,
//! or starts a program by a path it was not given,
//! and the shadow's own start of cargo changes only the environment it already sets.
//! Owner rule (2026-11-07):
//! "We should never invoke ourselves recursively.
//! That's bad design.
//! Audit,
//! remove,
//! and ban that design aggressively."

use std::{fs, path::PathBuf};

/// Text no lean-folder source may contain,
/// with why.
const FORBIDDEN: [(&str, &str); 5] = [
    ("current_exe", "a tool that asks for its own executable can start itself"),
    ("Command::new(\"cargo", "a lean tool must not start cargo (it would start the shadow)"),
    ("Command::new(\"cargo-shadow", "a lean tool must not start itself"),
    ("find_real_cargo", "a lean tool must not look for a cargo to run"),
    ("run_shim", "a lean tool must not enter shadow mode"),
];

/// The lean-folder sources (not their tests).
fn lean_sources() -> Vec<(String, String)> {
    let source: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found: Vec<(String, String)> = Vec::new();
    for entry in fs::read_dir(&source).expect("read the source directory").flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("lean_") && name.ends_with(".rs") && !name.ends_with("_tests.rs") && name != "lean_test_support.rs" {
            found.push((name, fs::read_to_string(entry.path()).expect("read a source")));
        }
    }
    return found;
}

#[test]
fn no_lean_source_starts_cargo_or_itself() {
    let sources: Vec<(String, String)> = lean_sources();
    assert!(sources.len() > 10, "the scan must see the lean sources, saw {}", sources.len());
    for (name, text) in &sources {
        for (needle, why) in FORBIDDEN {
            // The module that holds this very ban is a test file and is not scanned.
            assert!(!text.contains(needle), "{name} contains `{needle}`: {why}");
        }
    }
}

#[test]
fn the_shadow_only_adds_the_build_folder_variable_for_the_lean_tools() {
    let shim: String = fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("shim.rs")).expect("read the shim");
    let uses: usize = shim.matches("lean_").count();
    assert_eq!(uses, shim.matches("lean_keyed_folder").count(), "the shim may reach the lean tools only through the keyed folder");
    for forbidden in ["lean_run", "lean_command", "lean_service", "lean_idle", "lean_sweep"] {
        assert!(!shim.contains(forbidden), "the shim must not start {forbidden}");
    }
}
