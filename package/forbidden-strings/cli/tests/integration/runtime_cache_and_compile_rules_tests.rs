//! What:
//!  The content-addressed runtime cache and the `compile-rules` subcommand
//!  that fills it ahead of a scan:
//!  a first scan compiles and publishes,
//!  later scans reload,
//!  a corrupt or stale artifact is repaired,
//!  and every failed write still scans from the authoritative text.
//! Why:
//!  The cache is derived data,
//!  so no cache failure may change a verdict or print a path,
//!  and a repair that silently reported clean would be the worst outcome a
//!  secret scanner can produce.
//!  `files_named` walks the artifact tree because the slot path carries the
//!  text digest and no test may compute it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // runtime-cache.test.ts: publish, reuse, repair, and fail-open-to-text
//! ```

use std::fs;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

use crate::{
    BIN,
    blocked_root,
    unique_tmp,
};

/// Lists files with requested name under bounded cache hierarchy.
fn files_named(
    root: &std::path::Path,
    wanted: &str,
) -> Vec<PathBuf> {
    let mut pending = vec![root.to_path_buf()];
    let mut matches = Vec::new();
    while let Some(directory) = pending.pop() {
        let Ok(entries) = fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.file_name().and_then(|name| return name.to_str()) == Some(wanted) {
                matches.push(path);
            }
        }
    }
    return matches;
}

/// Explicit compile operation creates private artifact and emits no success output.
#[test]
fn compile_rules_subcommand_creates_silent_artifact() {
    let dir = unique_tmp("compile-rules-command");
    let cache = dir.join("cache");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "ALPHA_LITERAL_LONG\n").expect("write rules");

    let output = ProcessCommand::new(BIN)
        .args(["compile-rules", "--rules"])
        .arg(&rules)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache)
        .output()
        .expect("spawn binary");
    assert!(
        output.status.success(),
        "compile-rules must succeed; stderr: {}",
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        output.stdout.is_empty(),
        "compile-rules success stdout must stay empty"
    );
    assert!(
        output.stderr.is_empty(),
        "compile-rules success stderr must stay empty"
    );
    assert_eq!(files_named(&cache, "rules.bin").len(), 1);

    let repeated = ProcessCommand::new(BIN)
        .args(["compile-rules", "--rules"])
        .arg(&rules)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache)
        .output()
        .expect("spawn binary");
    assert!(
        repeated.status.success(),
        "repeated compile-rules must reuse artifact"
    );
    assert!(repeated.stdout.is_empty());
    assert!(repeated.stderr.is_empty());
    let _ = fs::remove_dir_all(dir);
}

/// First scan warns and repairs;
///  repeated scan reuses cache with identical finding.
#[test]
fn scan_repairs_missing_cache_then_reuses_artifact() {
    let dir = unique_tmp("scan-cache-repair");
    let cache = dir.join("cache");
    let rules = dir.join("rules.txt");
    let target = dir.join("target.txt");
    fs::write(&rules, "ALPHA_LITERAL_LONG\n").expect("write rules");
    fs::write(&target, "ALPHA_LITERAL_LONG\n").expect("write target");

    let run = || {
        return ProcessCommand::new(BIN)
            .args(["--rules"])
            .arg(&rules)
            .arg(&target)
            .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache)
            .output()
            .expect("spawn binary");
    };
    let first = run();
    let first_stderr = String::from_utf8_lossy(&first.stderr);
    assert_eq!(first.status.code(), Some(1));
    assert!(first_stderr.contains("\"reason\":\"missing\""));
    assert!(first_stderr.contains("target.txt:1 rule=0"));
    assert_eq!(files_named(&cache, "rules.bin").len(), 1);

    let second = run();
    let second_stderr = String::from_utf8_lossy(&second.stderr);
    assert_eq!(second.status.code(), Some(1));
    assert!(!second_stderr.contains("cache-warning"));
    assert!(second_stderr.contains("target.txt:1 rule=0"));
    let _ = fs::remove_dir_all(dir);
}

/// Clean first scan still emits missing-cache warning and exits successfully.
#[test]
fn clean_first_scan_reports_cache_warning_on_stderr() {
    let dir = unique_tmp("clean-cache-warning");
    let cache = dir.join("cache");
    let rules = dir.join("rules.txt");
    let target = dir.join("target.txt");
    fs::write(&rules, "ALPHA_LITERAL_LONG\n").expect("write rules");
    fs::write(&target, "clean\n").expect("write target");

    let output = ProcessCommand::new(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache)
        .output()
        .expect("spawn binary");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("\"reason\":\"missing\""));
    let _ = fs::remove_dir_all(dir);
}

/// Relative explicit cache override is configuration error rather than cwd path.
#[test]
fn relative_cache_override_exits_with_config_error() {
    let dir = unique_tmp("relative-cache-root");
    let rules = dir.join("rules.txt");
    let target = dir.join("target.txt");
    fs::write(&rules, "ALPHA_LITERAL_LONG\n").expect("write rules");
    fs::write(&target, "clean\n").expect("write target");

    let output = ProcessCommand::new(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", "relative/cache")
        .output()
        .expect("spawn binary");
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("FORBIDDEN_STRINGS_CACHE_DIR must be an absolute path"),
    );
    let _ = fs::remove_dir_all(dir);
}

/// Compile operation rejects missing required rules argument as usage error.
#[test]
fn compile_rules_subcommand_requires_rules_path() {
    let output = ProcessCommand::new(BIN)
        .arg("compile-rules")
        .output()
        .expect("spawn binary");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--rules"));
}

/// Compile operation fails closed on invalid text and publishes no artifact.
#[test]
fn compile_rules_subcommand_rejects_invalid_rules() {
    let dir = unique_tmp("compile-rules-invalid");
    let cache = dir.join("cache");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "/ALPHA*/\n").expect("write invalid rules");
    let output = ProcessCommand::new(BIN)
        .args(["compile-rules", "--rules"])
        .arg(&rules)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache)
        .output()
        .expect("spawn binary");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("rule 0"));
    assert!(files_named(&cache, "rules.bin").is_empty());
    let _ = fs::remove_dir_all(dir);
}

/// Corrupt artifact warns,
///  recompiles,
///  replaces cache,
///  and preserves finding.
#[test]
fn corrupt_artifact_is_repaired_without_false_clean() {
    let dir = unique_tmp("corrupt-cache-repair");
    let cache = dir.join("cache");
    let rules = dir.join("rules.txt");
    let target = dir.join("target.txt");
    fs::write(&rules, "ALPHA_LITERAL_LONG\n").expect("write rules");
    fs::write(&target, "ALPHA_LITERAL_LONG\n").expect("write target");

    let compiled = ProcessCommand::new(BIN)
        .args(["compile-rules", "--rules"])
        .arg(&rules)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache)
        .output()
        .expect("spawn compiler");
    assert!(compiled.status.success());
    let artifacts = files_named(&cache, "rules.bin");
    assert_eq!(artifacts.len(), 1);
    fs::write(&artifacts[0], b"corrupt").expect("corrupt artifact");

    let repaired = ProcessCommand::new(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache)
        .output()
        .expect("spawn scanner");
    let stderr = String::from_utf8_lossy(&repaired.stderr);
    assert_eq!(repaired.status.code(), Some(1));
    assert!(stderr.contains("\"reason\":\"invalid\""));
    assert!(stderr.contains("target.txt:1 rule=0"));

    let hit = ProcessCommand::new(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache)
        .output()
        .expect("spawn scanner");
    assert!(!String::from_utf8_lossy(&hit.stderr).contains("cache-warning"));
    let _ = fs::remove_dir_all(dir);
}

/// Changed source content selects a second immutable content-addressed slot.
#[test]
fn changed_rules_content_selects_new_cache_slot() {
    let dir = unique_tmp("cache-content-change");
    let cache = dir.join("cache");
    let rules = dir.join("rules.txt");
    let target = dir.join("target.txt");
    fs::write(&rules, "ALPHA_LITERAL_LONG\n").expect("write first rules");
    fs::write(&target, "BETA_LITERAL_LONG\n").expect("write target");
    assert!(
        ProcessCommand::new(BIN)
            .args(["compile-rules", "--rules"])
            .arg(&rules)
            .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache)
            .status()
            .expect("spawn compiler")
            .success(),
    );

    fs::write(&rules, "BETA_LITERAL_LONG\n").expect("write changed rules");
    let output = ProcessCommand::new(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache)
        .output()
        .expect("spawn scanner");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr.contains("\"reason\":\"missing\""));
    assert!(stderr.contains("target.txt:1 rule=0"));
    assert_eq!(files_named(&cache, "rules.bin").len(), 2);
    let _ = fs::remove_dir_all(dir);
}

/// Failed publication warns but retains correct compiled in-memory finding.
#[test]
fn cache_write_failure_keeps_scan_correct() {
    let dir = unique_tmp("cache-write-failure");
    let cache_root_file = dir.join("cache-root-file");
    let rules = dir.join("rules.txt");
    let target = dir.join("target.txt");
    fs::write(&cache_root_file, "not a directory").expect("write root blocker");
    fs::write(&rules, "ALPHA_LITERAL_LONG\n").expect("write rules");
    fs::write(&target, "ALPHA_LITERAL_LONG\n").expect("write target");

    let output = ProcessCommand::new(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache_root_file)
        .output()
        .expect("spawn scanner");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1));
    // The read-side reason for a root blocked by a regular file is the platform's:
    // `unreadable` on Unix and `missing` on Windows. `format!` builds the owned JSON
    // fragment once; `.as_str()` lends it to `contains` without copying.
    let blocked_reason: String = format!("\"reason\":\"{}\"", blocked_root::blocked_root_reason());
    assert!(
        stderr.contains(blocked_reason.as_str()),
        "blocked-root reason missing; stderr: {stderr}"
    );
    assert!(stderr.contains("\"reason\":\"write-failed\""));
    assert!(stderr.contains("target.txt:1 rule=0"));
    let _ = fs::remove_dir_all(dir);
}

/// Missing native cache environment warns and scans from authoritative text.
#[test]
fn unavailable_native_cache_root_keeps_scan_correct() {
    let dir = unique_tmp("cache-root-unavailable");
    let rules = dir.join("rules.txt");
    let target = dir.join("target.txt");
    fs::write(&rules, "ALPHA_LITERAL_LONG\n").expect("write rules");
    fs::write(&target, "ALPHA_LITERAL_LONG\n").expect("write target");

    let output = ProcessCommand::new(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .env_remove("FORBIDDEN_STRINGS_CACHE_DIR")
        .env_remove("XDG_CACHE_HOME")
        .env_remove("HOME")
        .env_remove("LOCALAPPDATA")
        .output()
        .expect("spawn scanner");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr.contains("\"reason\":\"cache-root-unavailable\""));
    assert!(stderr.contains("target.txt:1 rule=0"));
    let _ = fs::remove_dir_all(dir);
}
