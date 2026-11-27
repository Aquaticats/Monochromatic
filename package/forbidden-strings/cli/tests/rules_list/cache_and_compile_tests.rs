//! What:
//!  The cache keyed over the joined text,
//!  and the two commands that fill it:
//!  a scan publishes on first use,
//!  `compile-rules` warms the artifact ahead of a scan and refuses a missing
//!  required entry,
//!  `--all` skips every listed rules file,
//!  and a changed file or order is a different key.
//! Why:
//!  The key is the SHA-256 of the joined text,
//!  so an unchanged list must hit and any edit must miss;
//!  a key that ignored a listed file would serve rules the user no longer
//!  wrote,
//!  and a skipped file that self-matched would fail every `--all` run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // cache-and-compile.test.ts: sha256(joined text) keys one artifact
//! ```

use std::fs;
use std::path::Path;
use std::process::Command;

use crate::{
    artifacts,
    fixture,
    scanner,
    stderr_of,
    two_file_fixture,
    write,
};

/// The compiled artifact is reused while the list's text is unchanged and missed when any listed file changes.
#[test]
fn the_cache_follows_the_joined_text() {
    let root = two_file_fixture("cache");
    let run = || {
        return scanner(&root)
            .args(["--rules", "shared.txt,local.txt?", "target.txt"])
            .output()
            .expect("run");
    };
    let first = stderr_of(&run());
    assert!(first.contains("\"reason\":\"missing\""), "{first}");
    assert_eq!(artifacts(&root), 1);

    let second = stderr_of(&run());
    assert!(
        !second.contains("cache-warning"),
        "unchanged list must hit: {second}"
    );
    assert_eq!(artifacts(&root), 1);

    write(
        &root,
        "local.txt",
        "==> qqq-local <==\nLOCAL_NEEDLE_LONG_ENOUGH\n# edited\n",
    );
    let edited = stderr_of(&run());
    assert!(
        edited.contains("\"reason\":\"missing\""),
        "an edited second file must miss: {edited}"
    );
    assert_eq!(artifacts(&root), 2);

    fs::remove_file(root.join("local.txt")).expect("remove optional file");
    let removed = run();
    let removed_stderr = stderr_of(&removed);
    assert!(
        removed_stderr.contains("optional rules file is absent; skipped"),
        "{removed_stderr}"
    );
    assert!(
        removed_stderr.contains("\"reason\":\"missing\""),
        "a removed optional file must miss: {removed_stderr}"
    );
    assert!(
        !removed_stderr.contains("rule=qqq-local"),
        "{removed_stderr}"
    );
    assert_eq!(artifacts(&root), 3);

    write(
        &root,
        "local.txt",
        "==> qqq-local <==\nLOCAL_NEEDLE_LONG_ENOUGH\n",
    );
    let restored = stderr_of(&run());
    assert!(
        !restored.contains("cache-warning"),
        "restored text must hit its earlier artifact: {restored}"
    );
    assert!(restored.contains("rule=qqq-local"), "{restored}");
    assert_eq!(artifacts(&root), 3);
    fs::remove_dir_all(root).expect("remove fixture");
}

/// A changed file order is a changed text and misses the cache.
#[test]
fn a_changed_order_is_a_cache_miss() {
    let root = two_file_fixture("order");
    let forward = scanner(&root)
        .args(["--rules", "shared.txt,local.txt", "target.txt"])
        .output()
        .expect("run");
    assert!(stderr_of(&forward).contains("\"reason\":\"missing\""));
    let reverse = scanner(&root)
        .args(["--rules", "local.txt,shared.txt", "target.txt"])
        .output()
        .expect("run");
    assert!(stderr_of(&reverse).contains("\"reason\":\"missing\""));
    assert_eq!(artifacts(&root), 2);
    fs::remove_dir_all(root).expect("remove fixture");
}

/// `compile-rules` takes the same list,
/// warms the artifact a scan then reuses,
/// and fails on a missing required entry.
#[test]
fn compile_rules_takes_a_list() {
    let root = two_file_fixture("compile");
    let compiled = scanner(&root)
        .args([
            "compile-rules",
            "--rules",
            "shared.txt,local.txt,absent.txt?",
        ])
        .output()
        .expect("run");
    let compile_stderr = stderr_of(&compiled);
    assert!(compiled.status.success(), "{compile_stderr}");
    assert!(compiled.stdout.is_empty());
    assert!(compile_stderr.contains("absent.txt"), "{compile_stderr}");
    assert_eq!(artifacts(&root), 1);

    let scan = scanner(&root)
        .args(["--rules", "shared.txt,local.txt", "target.txt"])
        .output()
        .expect("run");
    let scan_stderr = stderr_of(&scan);
    assert_eq!(scan.status.code(), Some(1), "{scan_stderr}");
    assert!(
        !scan_stderr.contains("cache-warning"),
        "the scan must reuse the compiled artifact: {scan_stderr}"
    );

    let missing = scanner(&root)
        .args(["compile-rules", "--rules", "shared.txt,gone.txt"])
        .output()
        .expect("run");
    assert_eq!(missing.status.code(), Some(2));
    assert!(
        stderr_of(&missing).contains("read rules gone.txt"),
        "{}",
        stderr_of(&missing)
    );

    let malformed = scanner(&root)
        .args(["compile-rules", "--rules", "shared.txt,,local.txt"])
        .output()
        .expect("run");
    assert_eq!(malformed.status.code(), Some(2));

    let none = scanner(&root)
        .args(["compile-rules", "--rules", "a.txt?,b.txt?"])
        .output()
        .expect("run");
    assert_eq!(none.status.code(), Some(2));
    assert!(
        stderr_of(&none).contains("no rules loaded"),
        "{}",
        stderr_of(&none)
    );
    fs::remove_dir_all(root).expect("remove fixture");
}

/// `--all` skips every listed rules file,
/// not only the first,
/// so a tracked second file does not self-match.
#[test]
fn all_mode_skips_every_listed_rules_file() {
    let root = two_file_fixture("all");
    fs::remove_file(root.join("target.txt")).expect("remove target");
    let git = if Path::new("/usr/bin/git").exists() {
        "/usr/bin/git"
    } else {
        "git"
    };
    let init = Command::new(git)
        .args(["init", "-q"])
        .current_dir(&root)
        .status()
        .expect("init");
    assert!(init.success());
    let add = Command::new(git)
        .args(["add", "shared.txt", "local.txt"])
        .current_dir(&root)
        .status()
        .expect("add");
    assert!(add.success());
    // The cache lives outside the walked tree, so only the rules files could self-match.
    let cache_root = fixture("all-cache");
    let output = scanner(&root)
        .env("FORBIDDEN_STRINGS_CACHE_DIR", &cache_root)
        .args(["--rules", "shared.txt,local.txt", "--all"])
        .output()
        .expect("run");
    let stderr = stderr_of(&output);
    assert_eq!(
        output.status.code(),
        Some(0),
        "both rules files must be skipped by --all: {stderr}"
    );
    fs::remove_dir_all(cache_root).expect("remove cache fixture");
    fs::remove_dir_all(root).expect("remove fixture");
}
