//! What:
//!  Guard for the lock inventory of the service and its support modules:
//! a lock primitive (a mutex,
//! a read-write lock,
//! a
//!  condition variable,
//! a `flock`) may stand in these sources only on the inventory below,
//! and only with its reason written in
//!  the comment block directly above its declaration (the first line that uses it).
//! Why:
//!  The owner's rule is lock-free first:
//! atomic rename,
//! exclusive creation,
//! one owner per item,
//! and an unavoidable lock that
//!  covers the smallest unit with its reason beside it (`AGENTS.md`,
//! "Concurrency is lock-free first").
//! A lock added to one of
//!  these modules later must therefore be a decision recorded here and explained in the code,
//! never a reflex.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const file of sources) for (const line of lockLines(file)) expect(inventory.has(file) && reasonAbove(file, line)).toBe(true);
//! ```

use std::path::{Path, PathBuf};

/// The sources this guard covers,
/// by the start of the file name:
/// the service,
/// the work queue,
/// the discarded store,
/// the index
/// lock handling and the support modules they use.
const COVERED_PREFIXES: &[&str] = &[
    "service_",
    "claim_time",
    "discarded_",
    "index_lock_",
    "journald_native",
    "git_version",
    "config_",
    "process_identity",
    "private_storage",
    "recovery_files",
    "work_item",
    "candidate_",
    "policy_",
    "markdown_",
    "standards_review_",
    "commit_",
];

/// The source files that ship nothing:
///  a mock server and a fixture,
///  and the service's hooks at its fault points,
///  declared under `#[cfg(test)]` in their folder's module tree.
/// Why:
///   Their locks are a test's own,
///  and an inventory entry would say one of them ships;
///  keeping the
///       file out of the scan instead would hide the production locks beside it.
/// `service/service_fault_hooks_test_support.rs` holds the hold a control sets on whichever thread reaches
///  a fault point,
/// and `service/service_fault.rs` declares it under `#[cfg(test)]`.
///  It arrived with `7663284896`,
///  which ran a filtered unit selection that did not include this scan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TEST_ONLY_SOURCES = ['standards_review/standards_review_mock.rs', 'service/service_fault_hooks_test_support.rs'];
/// ```
const TEST_ONLY_SOURCES: &[&str] = &["standards_review/standards_review_mock.rs", "service/service_fault_hooks_test_support.rs"];

/// Words that name a lock primitive in code.
const LOCK_WORDS: &[&str] = &["Mutex", "RwLock", "Condvar", "flock(", "LOCK_EX", "LOCK_SH"];

/// The words a comment block above a kept lock must contain,
/// which is where its reason is written.
const REASON_MARKER: &str = "Lock kept:";

/// The locks that stay,
/// by file:
/// what each protects is in the comment block above it,
/// and in `SPEC.md`.
const INVENTORY: &[&str] = &[
    "service/journald_native.rs",
    "commit/commit_build_part_durations.rs",
    "commit/commit_index_builds.rs",
    "commit/commit_prepare_part_durations.rs",
    "landing/service_index_prestage.rs",
];

/// Every `.rs` file below `root`,
///  as `/`-separated paths relative to it,
///  found with a work stack,
///  so that the sources of the subsystem folders are covered as well as the files beside `lib.rs`.
fn rust_files_below(root: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("a readable source directory").flatten() {
            let path: PathBuf = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| { return extension == "rs"; }) {
                let relative: &Path = path.strip_prefix(root).expect("a path below the root");
                found.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    found.sort();
    return found;
}

/// Whether the file name of `path` starts with one of the covered prefixes.
fn is_covered(path: &str) -> bool {
    let name: &str = path.rsplit('/').next().unwrap_or(path);
    return COVERED_PREFIXES.iter().any(|prefix| { return name.starts_with(prefix); });
}

/// The covered sources of this package:
///  path relative to `src/native` and text,
///  from every subsystem folder.
fn covered_sources() -> Vec<(String, String)> {
    let directory: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/native");
    let mut found: Vec<(String, String)> = Vec::new();
    for relative in rust_files_below(directory.as_path()) {
        if is_covered(relative.as_str()) && !relative.ends_with("_tests.rs") {
            let text: String = std::fs::read_to_string(directory.join(relative.as_str())).expect("a readable source");
            found.push((relative, text));
        }
    }
    found.sort();
    return found;
}

/// The lines of `source` that use a lock primitive in code:
/// not comments,
/// not imports.
fn lock_lines(source: &str) -> Vec<usize> {
    return source
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let code: &str = line.trim_start();
            return !code.starts_with("//") && !code.starts_with("use ") && !code.starts_with("pub use ") && LOCK_WORDS.iter().any(|word| { return code.contains(word); });
        })
        .map(|(number, _)| { return number; })
        .collect();
}

/// The lines of `source` that declare a shipped lock:
///  every lock line that is not under `#[cfg(test)]`.
/// Why:
///   A test's own lock protects nothing a person runs,
///  and the reason an inventory entry
///       demands is about what ships.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shipped = lockLines(source).filter((line) => !isTestOnlyLock(source, line));
/// ```
fn shipped_lock_lines(source: &str) -> Vec<usize> {
    return lock_lines(source)
        .into_iter()
        .filter(|number| return !is_test_only_lock(source, *number))
        .collect();
}

/// Whether the comment block directly above line `number` of `source` carries the reason marker.
fn reason_above(source: &str, number: usize) -> bool {
    let lines: Vec<&str> = source.lines().collect();
    let mut cursor: usize = number;
    while cursor > 0 {
        cursor -= 1;
        let above: &str = lines[cursor].trim_start();
        if above.starts_with("//") && above.contains(REASON_MARKER) {
            return true;
        }
        if !above.starts_with("//") && !above.starts_with("#[") {
            return false;
        }
    }
    return false;
}

/// The folders of the commit-check route the guard covers:
///  every policy pass,
///  candidate listing,
///  Markdown check,
///  standards review and commit preparation.
/// Why:
///   These are the modules a lock is most tempting in (the checks run before every
///       commit),
///  and a lock added there must be a decision recorded here like any other.
const CHECK_ROUTE_FOLDERS: &[&str] = &[
    "candidate/",
    "policy/",
    "markdown/",
    "standards_review/",
    "commit/",
];

/// What:
///  Every folder of the commit-check route reaches a covered source.
/// Why:
///   The guard enforced the owner's "use as few locks as possible" for the service and
///       missed the whole route that runs before a commit,
///  so a lock added to a candidate or policy file shipped
///       without a reason and nothing reported it.
///  A source counted here can be flagged by the inventory control.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const folder of CHECK_ROUTE_FOLDERS) expect(coveredSources().some((s) => s.startsWith(folder))).toBe(true);
/// ```
#[test]
fn the_check_route_is_covered() {
    let sources: Vec<(String, String)> = covered_sources();
    for folder in CHECK_ROUTE_FOLDERS {
        let reached: bool = sources.iter().any(|(path, _)| return path.starts_with(folder));
        assert!(reached, "no covered source in {folder}");
    }
}

/// What:
///  A lock declared under `#[cfg(test)]` ships nothing and needs no inventory entry.
/// Why:
///  The service and the check route hold test-only locks (a handoff gate,
/// a mock server);
///  an
///       inventory entry for one would say it ships,
///  which is wrong,
///  and leaving the file out of the scan would
///       hide the production locks beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(isTestOnly('#[cfg(test)]\n    gate: Mutex<()>,', 1)).toBe(true);
/// ```
#[test]
fn the_scan_tells_a_test_only_lock_from_a_shipped_one() {
    assert!(is_test_only_lock("#[cfg(test)]\n    gate: Mutex<()>,\n", 1));
    assert!(!is_test_only_lock("static HELD: Mutex<u8> = Mutex::new(0);\n", 0));
}

/// Whether the lock at `number` of `source` is declared under `#[cfg(test)]`,
///  so it ships nothing.
fn is_test_only_lock(source: &str, number: usize) -> bool {
    let lines: Vec<&str> = source.lines().collect();
    let mut cursor: usize = number;
    while cursor > 0 {
        cursor -= 1;
        let above: &str = lines[cursor].trim_start();
        if above.starts_with("#[cfg(test)]") {
            return true;
        }
        // Only the attribute lines directly above the declaration decide; a doc comment or code ends the block.
        if !above.starts_with("#[") {
            return false;
        }
    }
    return false;
}

/// A lock stands only in a file on the inventory.
#[test]
fn a_lock_stands_only_in_a_file_on_the_inventory() {
    let strays: Vec<String> = covered_sources()
        .into_iter()
        .filter(|(name, source)| return !TEST_ONLY_SOURCES.contains(&name.as_str()) && !shipped_lock_lines(source.as_str()).is_empty() && !INVENTORY.contains(&name.as_str()))
        .map(|(name, _)| { return name; })
        .collect();
    assert!(strays.is_empty(), "locks outside the inventory (remove them, or keep them with the reason and list the file): {strays:?}");
}

/// Every lock on the inventory has its reason in the comment block above its declaration,
/// and no file stays on the inventory once it has no lock.
#[test]
fn every_kept_lock_has_its_reason_beside_it_and_the_inventory_has_no_stale_entry() {
    let sources: Vec<(String, String)> = covered_sources();
    for kept in INVENTORY {
        let (_, source) = sources.iter().find(|(name, _)| { return name == kept; }).unwrap_or_else(|| { panic!("{kept} is on the inventory but is not a covered source"); });
        let lines: Vec<usize> = shipped_lock_lines(source.as_str());
        let first: usize = *lines.first().unwrap_or_else(|| { panic!("{kept} is on the inventory and holds no shipped lock any more"); });
        assert!(reason_above(source.as_str(), first), "{kept}:{} declares a lock with no `{REASON_MARKER}` comment block above it", first + 1);
    }
}

/// Every covered prefix reaches at least one source,
///  so a move of files into folders cannot leave the scan with fewer files than it was written for.
#[test]
fn every_covered_prefix_reaches_a_source_in_some_folder() {
    let sources: Vec<(String, String)> = covered_sources();
    for prefix in COVERED_PREFIXES {
        let reached: bool = sources.iter().any(|(path, _)| {
            return path.rsplit('/').next().is_some_and(|name| { return name.starts_with(prefix); });
        });
        assert!(reached, "no covered source starts with {prefix}");
    }
}

/// The guard sees locks:
/// the positive control that a scan which finds nothing would fail.
#[test]
fn the_scan_finds_a_lock_in_text_that_has_one() {
    let sample: &str = "// Mutex in a comment\nuse std::sync::Mutex;\nstruct A {\n    held: Mutex<u8>,\n}\n";
    assert_eq!(lock_lines(sample), vec![3]);
    assert!(!reason_above(sample, 3));
    assert!(reason_above("/// Lock kept: because.\n#[derive(Debug)]\nheld: Mutex<u8>,\n", 2));
}
