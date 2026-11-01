//! What:
//!  Invocation-level lazy workspace and selected-rule dispatch controls.
//! Why:
//!  Syntax-only and disabled checks must work without a physical file or installed project context.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // No semantic selection means no Cargo initialization; enabled semantics shares its manifest cache.
//! ```

/// Import the actual engine and its selected-rule inputs.
use super::RustFileEngine;
use crate::diagnostic::{Diagnostic, Severity};
use crate::rust_rule_settings::{LineBudget, RustRuleSettings};
use crate::rust_semantic_test_support::{prepare_lockfile, progress};
use crate::rust_workspace::WorkspacePreparation;
/// Import native paths and owned fixtures for real workspace verification.
use crate::test_fs::Fixture;
use std::path::{Path, PathBuf};

/// Syntax-only checks cannot need project discovery,
///  because this path deliberately has no physical source.
#[test]
fn no_semantic_selection_avoids_workspace_initialization() {
    let mut engine: RustFileEngine =
        RustFileEngine::new(WorkspacePreparation::SourceOnly, progress);
    let selected: RustRuleSettings = RustRuleSettings {
        max_lines: Some(LineBudget {
            severity: Severity::Error,
            max: 0,
        }),
        ..RustRuleSettings::default()
    };
    let findings: Vec<Diagnostic> = engine
        .check(
            Path::new("/intentionally-unavailable/input.rs"),
            String::from("fn main() {}"),
            String::from("host.md/fence.rs"),
            selected,
        )
        .expect("syntax checks do not read the path");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].code, "rust/max-lines");
    assert_eq!(findings[0].filename, "host.md/fence.rs");
    assert_eq!(engine.workspace_count(), 0);
    assert!(
        engine
            .check(
                Path::new("/intentionally-unavailable/input.rs"),
                String::from("fn main() {}"),
                String::from("input.rs"),
                RustRuleSettings::default()
            )
            .expect("all rules off")
            .is_empty()
    );
    assert_eq!(engine.workspace_count(), 0);
}

/// Real workspace reuse still applies every supplied source snapshot and selected severity.
#[test]
fn selected_semantics_reuses_the_manifest_session() {
    let fixture: Fixture = Fixture::new();
    let directory: PathBuf = fixture.path.join("src");
    std::fs::create_dir(&directory).expect("fixture source directory");
    std::fs::write(fixture.path.join("Cargo.toml"), "[package]\nname = \"engine-fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[workspace]\nmembers = [\".\"]\n").expect("fixture manifest");
    let path: PathBuf = directory.join("main.rs");
    std::fs::write(&path, "fn main() {}\n").expect("physical source");
    prepare_lockfile(&fixture.path);
    let mut engine: RustFileEngine =
        RustFileEngine::new(WorkspacePreparation::SourceOnly, progress);
    let first: RustRuleSettings = RustRuleSettings {
        explicit_types: Some(Severity::Warn),
        ..RustRuleSettings::default()
    };
    let findings: Vec<Diagnostic> = engine
        .check(
            &path,
            String::from("fn main() { let value = 1_u16; }"),
            String::from("display.rs"),
            first,
        )
        .expect("semantic check");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].severity, Severity::Warn);
    assert_eq!(findings[0].filename, "display.rs");
    assert_eq!(engine.workspace_count(), 1);
    let second: RustRuleSettings = RustRuleSettings {
        explicit_types: Some(Severity::Error),
        ..RustRuleSettings::default()
    };
    assert!(
        engine
            .check(
                &path,
                String::from("fn main() { let value: u16 = 1_u16; }"),
                String::from("display.rs"),
                second.clone()
            )
            .expect("clean next snapshot")
            .is_empty()
    );
    assert_eq!(engine.workspace_count(), 1);
    let missing: PathBuf = directory.join("missing.rs");
    assert!(
        engine
            .check(
                &missing,
                String::from("fn main() {}"),
                String::from("missing.rs"),
                second
            )
            .is_err()
    );
    assert_eq!(engine.workspace_count(), 1);
    assert_eq!(
        std::fs::read_to_string(&path).expect("read physical source"),
        "fn main() {}\n"
    );
}
