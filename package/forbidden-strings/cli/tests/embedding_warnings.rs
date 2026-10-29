//! What:
//!  Structured cache warnings exercised through the exported loader in isolated child consumers.
//! Why:
//!  Constructor token tests alone do not prove real cache recovery reaches library callers without terminal output.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Give each consumer its own home/cache and inspect warning fields after real cache failures.
//! ```

/// Import the actual published boundary and native fixture/process APIs.
use forbidden_strings::{Scanner, ScanFinding};
use std::path::{Path, PathBuf};
use std::process::Command;

/// What:
///  `#[path = "..."] mod blocked_root;` compiles the named file as a private module of this test crate.
/// Why:
///   The CLI integration suite includes the same file,
///  so both assert one platform-specific reason
///       for a cache root blocked by a regular file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as blockedRoot from './support/blocked_root.ts';
/// ```
#[path = "support/blocked_root.rs"]
mod blocked_root;

/// Find only the artifact generated in this disposable fixture,
///  never a user's cache.
fn artifact(root: &Path) -> PathBuf {
    // A Vec work stack walks bounded fixture directories without recursive call depth.
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for outcome in std::fs::read_dir(directory).expect("fixture cache directory") {
            let entry = outcome.expect("fixture cache entry");
            let path: PathBuf = entry.path();
            if path.is_dir() { pending.push(path); }
            else if path.file_name() == Some(std::ffi::OsStr::new("rules.bin")) { return path; }
        }
    }
    panic!("fixture artifact was not published");
}

/// The exported loader returns structured recovery evidence while remaining silent itself.
#[test]
fn warning_consumer_probe() {
    let Ok(mode) = std::env::var("EMBEDDING_WARNING_MODE") else { return; };
    let root: PathBuf = PathBuf::from(std::env::var_os("EMBEDDING_WARNING_ROOT").expect("fixture root"));
    let rules: PathBuf = root.join("rules");
    if mode == "relative" {
        let error = match Scanner::load(rules.as_path(), false, true) {
            Ok(_) => panic!("relative cache configuration must be rejected"),
            Err(failure) => failure,
        };
        assert!(error.to_string().contains("FORBIDDEN_STRINGS_CACHE_DIR must be an absolute path"));
        return;
    }
    if mode == "corrupt" {
        let _first: Scanner = Scanner::load(rules.as_path(), false, true).expect("warm fixture cache");
        std::fs::write(artifact(root.join("cache").as_path()), b"invalid").expect("corrupt fixture artifact");
    }
    let scanner: Scanner = Scanner::load(rules.as_path(), false, true).expect("public cache recovery");
    // A root blocked by a regular file reads as `unreadable` on Unix and `missing` on Windows; every other reason is shared.
    let expected_reason = if mode == "blocked" { blocked_root::blocked_root_reason() }
        else if mode == "unavailable" { "cache-root-unavailable" } else { "invalid" };
    assert_eq!(scanner.cache_warnings()[0].reason(), expected_reason);
    assert_eq!(scanner.cache_warnings()[0].recovery(), "compile-from-text");
    if mode == "blocked" {
        assert_eq!(scanner.cache_warnings().len(), 2);
        assert_eq!(scanner.cache_warnings()[1].reason(), "write-failed");
        assert_eq!(scanner.cache_warnings()[1].recovery(), "continue-with-compiled-rules");
    } else { assert_eq!(scanner.cache_warnings().len(), 1); }
    assert_eq!(scanner.scan(9, Path::new("safe"), b"WARNING_FIXTURE_LONG").findings,
        vec![ScanFinding::Content { line: 1, rule: String::from("0") }]);
    if mode == "corrupt" {
        assert!(Scanner::load(rules.as_path(), false, true).expect("repaired public cache").cache_warnings().is_empty());
    }
}

/// Every warning mode runs with a separate cache/home and is proven to execute,
///  not merely enumerate.
#[test]
fn public_warning_paths_preserve_scan_results_without_emitting_terminal_json() {
    for mode in ["blocked", "unavailable", "relative", "corrupt"] {
        let root: PathBuf = std::env::temp_dir().join(format!("scanner-warning-{}-{mode}", std::process::id()));
        std::fs::create_dir(&root).expect("fresh fixture");
        std::fs::write(root.join("rules"), "WARNING_FIXTURE_LONG\n").expect("fixture rules");
        if mode == "blocked" { std::fs::write(root.join("cache"), "blocked").expect("cache blocker"); }
        let mut command = Command::new(std::env::current_exe().expect("consumer executable"));
        command.args(["--exact", "warning_consumer_probe", "--nocapture"])
            .env("EMBEDDING_WARNING_MODE", mode).env("EMBEDDING_WARNING_ROOT", &root)
            .env("HOME", &root).env("FORBIDDEN_STRINGS_CACHE_DIR", root.join("cache"));
        if mode == "relative" { command.env("FORBIDDEN_STRINGS_CACHE_DIR", "relative-cache"); }
        if mode == "unavailable" {
            for variable in ["HOME", "XDG_CACHE_HOME", "FORBIDDEN_STRINGS_CACHE_DIR", "LOCALAPPDATA", "USERPROFILE"] {
                command.env_remove(variable);
            }
        }
        let output = command.output().expect("isolated warning consumer");
        std::fs::remove_dir_all(&root).expect("remove only disposable fixture");
        assert!(output.status.success(), "{mode}: {}", String::from_utf8_lossy(&output.stderr));
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
        assert!(!String::from_utf8_lossy(&output.stderr).contains("forbidden-strings/cache-warning"));
    }
}
