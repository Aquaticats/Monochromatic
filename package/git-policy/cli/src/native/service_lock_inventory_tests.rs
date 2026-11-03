//! What:
//!  Guard for the lock inventory of the service and its support modules: a lock primitive (a mutex, a read-write lock, a
//!  condition variable, a `flock`) may stand in these sources only on the inventory below, and only with its reason written in
//!  the comment block directly above it.
//! Why:
//!  The owner's rule is lock-free first: atomic rename, exclusive creation, one owner per item, and an unavoidable lock that
//!  covers the smallest unit with its reason beside it (`AGENTS.md`, "Concurrency is lock-free first"). A lock added to one of
//!  these modules later must therefore be a decision recorded here and explained in the code, never a reflex.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const file of sources) for (const line of lockLines(file)) expect(inventory.has(file) && reasonAbove(file, line)).toBe(true);
//! ```

use std::path::{Path, PathBuf};

/// The sources this guard covers, by the start of the file name: the service, the work queue, the discarded store, the index
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
];

/// Words that name a lock primitive in code.
const LOCK_WORDS: &[&str] = &["Mutex", "RwLock", "Condvar", "flock(", "LOCK_EX", "LOCK_SH"];

/// The words a comment block above a kept lock must contain, which is where its reason is written.
const REASON_MARKER: &str = "Lock kept:";

/// The locks that stay, by file: what each protects is in the comment block above it, and in `SPEC.md`.
const INVENTORY: &[&str] = &[];

/// The covered sources of this package: path and text.
fn covered_sources() -> Vec<(String, String)> {
    let directory: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/native");
    let mut found: Vec<(String, String)> = Vec::new();
    for entry in std::fs::read_dir(directory).expect("the source directory").flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let covered: bool = COVERED_PREFIXES.iter().any(|prefix| { return name.starts_with(prefix); });
        if covered && name.ends_with(".rs") && !name.ends_with("_tests.rs") {
            found.push((name, std::fs::read_to_string(entry.path()).expect("a readable source")));
        }
    }
    found.sort();
    return found;
}

/// The lines of `source` that use a lock primitive in code: not comments, not imports.
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

/// A lock stands only in a file on the inventory.
#[test]
fn a_lock_stands_only_in_a_file_on_the_inventory() {
    let strays: Vec<String> = covered_sources()
        .into_iter()
        .filter(|(name, source)| { return !lock_lines(source.as_str()).is_empty() && !INVENTORY.contains(&name.as_str()); })
        .map(|(name, _)| { return name; })
        .collect();
    assert!(strays.is_empty(), "locks outside the inventory (remove them, or keep them with the reason and list the file): {strays:?}");
}

/// Every lock on the inventory has its reason in the comment block above it, and no file stays on the inventory once it has no lock.
#[test]
fn every_kept_lock_has_its_reason_beside_it_and_the_inventory_has_no_stale_entry() {
    let sources: Vec<(String, String)> = covered_sources();
    for kept in INVENTORY {
        let (_, source) = sources.iter().find(|(name, _)| { return name == kept; }).unwrap_or_else(|| { panic!("{kept} is on the inventory but is not a covered source"); });
        let lines: Vec<usize> = lock_lines(source.as_str());
        assert!(!lines.is_empty(), "{kept} is on the inventory and holds no lock any more");
        for number in lines {
            assert!(reason_above(source.as_str(), number), "{kept}:{} holds a lock with no `{REASON_MARKER}` comment block above it", number + 1);
        }
    }
}

/// The guard sees locks: the positive control that a scan which finds nothing would fail.
#[test]
fn the_scan_finds_a_lock_in_text_that_has_one() {
    let sample: &str = "// Mutex in a comment\nuse std::sync::Mutex;\nstruct A {\n    held: Mutex<u8>,\n}\n";
    assert_eq!(lock_lines(sample), vec![3]);
    assert!(!reason_above(sample, 3));
    assert!(reason_above("/// Lock kept: because.\n#[derive(Debug)]\nheld: Mutex<u8>,\n", 2));
}
