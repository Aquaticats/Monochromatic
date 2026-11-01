//! What:
//!  Controls for configuration lookup,
//!  memoization and per-file planning.
//! Why:
//!  Which configuration governs a file,
//!  and what its patterns resolve against,
//!  decides every
//! rule that runs;
//!  these are checked on real disposable directory trees.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('ConfigStore', () => { /* nearest alone, --config base, ignored, unconfigured, errors */ });
//! ```

/// Import the store,
///  its outcomes and fixture helpers.
use super::{ConfigStore, FilePlan, Planned, RootRules};
use crate::diagnostic::Severity;
use crate::run_paths::Language;
use crate::run_test_support::{CONFIG, write};
use crate::test_fs::Fixture;
use std::path::{Path, PathBuf};

/// Plan a path and require a lint plan.
fn lint(store: &mut ConfigStore, path: &str) -> FilePlan {
    match store.plan(Path::new(path)).expect("planning succeeds") {
        Planned::Lint { plan } => return *plan,
        _ => panic!("{path} must be planned for linting"),
    }
}

/// The nearest configuration is used alone,
///  and its patterns resolve against its own directory.
#[test]
fn the_nearest_configuration_governs_alone() {
    let fixture: Fixture = Fixture::new();
    let root: &Path = fixture.path.as_path();
    write(
        root,
        CONFIG,
        r#"[{ "files": ["**/*.rs"], "rules": { "rust/require-rustdoc": { "severity": "error" }, "rust/max-lines": { "severity": "warn", "max": 7 } } }]"#,
    );
    write(
        root,
        &format!("nested/{CONFIG}"),
        r#"[{ "files": ["src/*.rs"], "rules": { "rust/tests-in-sidecars": { "severity": "warn" } } }]"#,
    );
    let mut store: ConfigStore = ConfigStore::new(root, None).expect("store");
    let outer: FilePlan = lint(&mut store, "top/src/a.rs");
    assert_eq!(outer.display, "top/src/a.rs");
    assert_eq!(outer.absolute, root.join("top/src/a.rs"));
    assert_eq!(outer.relative, PathBuf::from("top/src/a.rs"));
    assert_eq!(outer.language, Language::Rust);
    assert_eq!(outer.config.base, root);
    let RootRules::Rust {
        settings: outer_rules,
    } = outer.root
    else {
        panic!("Rust rules expected");
    };
    assert_eq!(outer_rules.rustdoc, Some(Severity::Error));
    assert_eq!(outer_rules.max_lines.expect("budget").max, 7);
    assert_eq!(outer_rules.tests_in_sidecars, None);
    // The nested configuration replaces the outer one for its subtree; nothing is merged across files.
    let inner: FilePlan = lint(&mut store, "nested/src/a.rs");
    assert_eq!(inner.display, "nested/src/a.rs");
    assert_eq!(inner.relative, PathBuf::from("src/a.rs"));
    assert_eq!(inner.config.base, root.join("nested"));
    let RootRules::Rust {
        settings: inner_rules,
    } = inner.root
    else {
        panic!("Rust rules expected");
    };
    assert_eq!(inner_rules.tests_in_sidecars, Some(Severity::Warn));
    assert_eq!(inner_rules.rustdoc, None);
    assert_eq!(inner_rules.max_lines, None);
    // A nested file its own configuration does not match is unconfigured, not handed to the outer one.
    let unmatched: FilePlan = lint(&mut store, "nested/other/a.rs");
    assert!(matches!(unmatched.root, RootRules::None));
    assert!(!unmatched.needs_semantic_engine());
}

/// Lookup results are remembered per directory,
///  including "none found".
#[test]
fn lookups_are_memoized_per_directory() {
    let fixture: Fixture = Fixture::new();
    let root: &Path = fixture.path.as_path();
    write(
        root,
        &format!("a/{CONFIG}"),
        r#"[{ "files": ["**/*.md"], "rules": {} }]"#,
    );
    let mut store: ConfigStore = ConfigStore::new(root, None).expect("store");
    let first: FilePlan = lint(&mut store, "a/b/c/one.md");
    // Removing the file after the first lookup does not change the store's answer for that subtree.
    std::fs::remove_file(root.join("a").join(CONFIG)).expect("remove configuration");
    let second: FilePlan = lint(&mut store, "a/b/two.md");
    assert!(std::sync::Arc::ptr_eq(&first.config, &second.config));
    let sibling: FilePlan = lint(&mut store, "a/b/c/three.md");
    assert!(std::sync::Arc::ptr_eq(&first.config, &sibling.config));
    // A fresh store sees the removal: no configuration governs the file any more.
    let mut fresh: ConfigStore = ConfigStore::new(root, None).expect("store");
    assert!(matches!(
        fresh.plan(Path::new("a/b/two.md")).expect("planning"),
        Planned::NoConfiguration
    ));
    // The remembered absence holds even after a configuration appears.
    write(root, &format!("a/{CONFIG}"), "[]");
    assert!(matches!(
        fresh.plan(Path::new("a/b/c/four.md")).expect("planning"),
        Planned::NoConfiguration
    ));
}

/// `--config` skips lookup and resolves its patterns against the working directory.
#[test]
fn an_explicit_configuration_resolves_against_the_working_directory() {
    let fixture: Fixture = Fixture::new();
    let work: PathBuf = fixture.path.join("work");
    let elsewhere: PathBuf = fixture.path.join("elsewhere");
    std::fs::create_dir_all(&work).expect("working directory");
    write(
        &elsewhere,
        "rules.jsonc",
        r#"[{ "files": ["doc/*.md"], "rules": { "markdown/single-h1": { "severity": "warn" } } }]"#,
    );
    // A nearer discovered configuration must be skipped entirely.
    write(&work, &format!("doc/{CONFIG}"), "not even JSONC");
    let explicit: PathBuf = elsewhere.join("rules.jsonc");
    let mut store: ConfigStore = ConfigStore::new(&work, Some(&explicit)).expect("store");
    let plan: FilePlan = lint(&mut store, "doc/a.md");
    assert_eq!(plan.config.path, explicit);
    assert_eq!(plan.config.base, work);
    assert_eq!(plan.relative, PathBuf::from("doc/a.md"));
    let RootRules::Markdown { settings: rules } = plan.root else {
        panic!("Markdown rules expected");
    };
    assert_eq!(rules.single_h1, Some(Severity::Warn));
    // A file outside the working directory keeps an absolute display name and a climbing logical path.
    let outside: FilePlan = lint(&mut store, "../outside/doc/a.md");
    assert_eq!(outside.relative, PathBuf::from("../outside/doc/a.md"));
    assert_eq!(
        outside.display,
        fixture.path.join("outside/doc/a.md").to_string_lossy()
    );
    assert!(matches!(outside.root, RootRules::None));
    // A relative --config path resolves against the working directory.
    let mut relative: ConfigStore =
        ConfigStore::new(&work, Some(Path::new("../elsewhere/rules.jsonc"))).expect("store");
    assert!(matches!(
        relative.plan(Path::new("doc/a.md")).expect("planning"),
        Planned::Lint { .. }
    ));
}

/// Ignored,
///  unsupported and unconfigured inputs are distinguished from lint plans.
#[test]
fn skipped_inputs_are_classified() {
    let fixture: Fixture = Fixture::new();
    let root: &Path = fixture.path.as_path();
    let mut empty: ConfigStore = ConfigStore::new(root, None).expect("store");
    assert!(matches!(
        empty.plan(Path::new("a.md")).expect("planning"),
        Planned::NoConfiguration
    ));
    assert!(matches!(
        empty.plan(Path::new("a.txt")).expect("planning"),
        Planned::Unsupported
    ));
    write(
        root,
        CONFIG,
        r#"[{ "ignores": ["vendor/"] }, { "files": ["**/*.mdx"], "rules": { "markdown/single-h1": { "severity": "error" } } }, { "files": ["**/*.rs"], "rules": { "rust/require-explicit-types": { "severity": "error" } } }]"#,
    );
    let mut store: ConfigStore = ConfigStore::new(root, None).expect("store");
    assert!(matches!(
        store.plan(Path::new("vendor/a.mdx")).expect("planning"),
        Planned::Ignored
    ));
    assert!(matches!(
        store.plan(Path::new("vendor/a.txt")).expect("planning"),
        Planned::Unsupported
    ));
    let mdx: FilePlan = lint(&mut store, "doc/a.mdx");
    assert_eq!(mdx.language, Language::Mdx);
    assert!(matches!(mdx.root, RootRules::Markdown { .. }));
    assert!(!mdx.needs_semantic_engine());
    let markdown: FilePlan = lint(&mut store, "doc/a.md");
    assert_eq!(markdown.language, Language::Markdown);
    assert!(matches!(markdown.root, RootRules::None));
    let rust: FilePlan = lint(&mut store, "src/a.rs");
    assert!(rust.needs_semantic_engine());
}

/// Configuration failures are setup errors that name the configuration file.
#[test]
fn configuration_failures_are_errors() {
    let fixture: Fixture = Fixture::new();
    let root: &Path = fixture.path.as_path();
    let missing = ConfigStore::new(root, Some(Path::new("absent.jsonc")));
    assert!(missing.is_err());
    write(root, "broken.jsonc", "[{");
    assert!(ConfigStore::new(root, Some(Path::new("broken.jsonc"))).is_err());
    write(
        root,
        "pattern.jsonc",
        r#"[{ "files": ["[z-a]"], "rules": {} }]"#,
    );
    assert!(ConfigStore::new(root, Some(Path::new("pattern.jsonc"))).is_err());
    write(
        root,
        &format!("bad/{CONFIG}"),
        r#"[{ "files": ["**/*.md"], "rules": { "markdown/unknown": { "severity": "error" } } }]"#,
    );
    let mut store: ConfigStore = ConfigStore::new(root, None).expect("store");
    let error = match store.plan(Path::new("bad/a.md")) {
        Err(error) => error,
        Ok(_) => panic!("an unknown rule must be a configuration error"),
    };
    assert!(error.message.contains(CONFIG), "{}", error.message);
    // A rule selected without a severity fails when a file resolves to it, not silently later.
    write(
        root,
        &format!("partial/{CONFIG}"),
        r#"[{ "files": ["**/*.md"], "rules": { "markdown/single-h1": {} } }]"#,
    );
    assert!(store.plan(Path::new("partial/a.md")).is_err());
    // A relative working directory is refused before any lookup.
    assert!(ConfigStore::new(Path::new("relative"), Some(Path::new("x.jsonc"))).is_err());
}
