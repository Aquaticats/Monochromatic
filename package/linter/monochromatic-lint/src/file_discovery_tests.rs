//! What:
//!  Real disposable filesystem controls for source discovery.
//! Why:
//!  Hidden paths,
//!  multiple excludes,
//!  explicit files and failed ignore inputs must not silently change the candidate set.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Create an owned tree and inspect the exact discovered native paths.
//! ```

/// Import the production discovery entry and its explicit options.
use super::{DiscoveryOptions, FileDiscoveryError, discover_literal_path, supported_source};
/// Import owned temporary directories.
use crate::test_fs::Fixture;
/// Import native paths.
use std::path::{Path, PathBuf};

/// Build default discovery settings relative to the owned fixture.
fn options(fixture: &Fixture) -> DiscoveryOptions {
    return DiscoveryOptions {
        cwd: fixture.path.clone(),
        no_ignore: false,
        ignore_patterns: Vec::<String>::new(),
        ignore_paths: Vec::<PathBuf>::new(),
    };
}

/// Create one fixture file and its parent directories.
fn file(fixture: &Fixture, name: &str) -> PathBuf {
    let path: PathBuf = fixture.path.join(name);
    std::fs::create_dir_all(path.parent().expect("fixture file parent"))
        .expect("create fixture parent");
    std::fs::write(&path, "").expect("create fixture file");
    return path;
}

/// Supported extensions and hidden directories are independent of Git metadata and dependency exclusions.
#[test]
fn hidden_sources_and_builtin_exclusions_are_distinct() {
    let fixture: Fixture = Fixture::new();
    let rust: PathBuf = file(&fixture, "src/main.rs");
    let markdown: PathBuf = file(&fixture, ".hidden/notes.md");
    let mdx: PathBuf = file(&fixture, "pages/view.mdx");
    file(&fixture, ".git/internal.rs");
    let dependency: PathBuf = file(&fixture, "node_modules/dependency/code.rs");
    file(&fixture, "ordinary.txt");
    let mut expected: Vec<PathBuf> = vec![rust, markdown, mdx];
    expected.sort();
    assert_eq!(
        discover_literal_path(&fixture.path, &options(&fixture)).expect("walk"),
        expected
    );
    let mut unignored: DiscoveryOptions = options(&fixture);
    unignored.no_ignore = true;
    let files: Vec<PathBuf> =
        discover_literal_path(&fixture.path, &unignored).expect("ignore sources disabled");
    assert!(files.contains(&dependency));
    assert_eq!(files.len(), 4, ".git remains excluded");
}

/// All command-line exclusions must participate,
///  rather than only the last supplied pattern.
#[test]
fn multiple_exclusions_and_explicit_files_keep_their_contracts() {
    let fixture: Fixture = Fixture::new();
    let first: PathBuf = file(&fixture, "first.rs");
    file(&fixture, "second.rs");
    let retained: PathBuf = file(&fixture, "retained.md");
    let mut selected: DiscoveryOptions = options(&fixture);
    selected.ignore_patterns = vec![String::from("first.rs"), String::from("second.rs")];
    assert_eq!(
        discover_literal_path(&fixture.path, &selected).expect("all exclusions"),
        vec![retained]
    );
    assert_eq!(
        discover_literal_path(&first, &selected).expect("explicit file"),
        vec![first]
    );
    let other: PathBuf = file(&fixture, "other.ts");
    assert!(
        discover_literal_path(&other, &selected)
            .expect("unsupported explicit extension")
            .is_empty()
    );
}

/// An explicit ignore file is applied,
///  and an unreadable requested ignore file is a failure.
#[test]
fn explicit_ignore_files_are_applied_or_reported() {
    let fixture: Fixture = Fixture::new();
    file(&fixture, "omitted.rs");
    let retained: PathBuf = file(&fixture, "retained.rs");
    let ignore: PathBuf = fixture.path.join("custom.ignore");
    std::fs::write(&ignore, "omitted.rs\n").expect("ignore fixture");
    let mut selected: DiscoveryOptions = options(&fixture);
    selected.ignore_paths.push(ignore);
    assert_eq!(
        discover_literal_path(&fixture.path, &selected).expect("custom ignores"),
        vec![retained]
    );
    selected
        .ignore_paths
        .push(fixture.path.join("missing.ignore"));
    assert!(discover_literal_path(&fixture.path, &selected).is_err());
    selected.no_ignore = true;
    assert_eq!(
        discover_literal_path(&fixture.path, &selected)
            .expect("no-ignore bypasses requested ignores")
            .len(),
        2
    );
}

/// Bad roots and patterns cannot become apparently clean empty walks.
#[test]
fn discovery_errors_are_not_empty_successes() {
    let fixture: Fixture = Fixture::new();
    assert!(discover_literal_path(&fixture.path.join("missing"), &options(&fixture)).is_err());
    assert!(discover_literal_path(Path::new("."), &options(&fixture)).is_err());
    let mut invalid: DiscoveryOptions = options(&fixture);
    invalid.ignore_patterns.push(String::from("["));
    let error: FileDiscoveryError =
        discover_literal_path(&fixture.path, &invalid).expect_err("invalid exclusion");
    assert!(error.message.contains("exclusion"));
    assert_eq!(error.to_string(), error.message);
    assert!(supported_source(Path::new("a.rs")));
    assert!(!supported_source(Path::new("a.RS")));
    assert!(supported_source(Path::new(".github/workflows/ci.yml")));
    assert!(!supported_source(Path::new("config/other.yml")));
}
