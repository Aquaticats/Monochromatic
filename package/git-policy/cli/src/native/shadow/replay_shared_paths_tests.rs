//! Controls for shared-path listing:
//!  raw record parsing,
//! the regular-file predicates,
//! and the listing against real repositories.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::path::PathBuf;

/// An entry with a mode and object of the given text.
fn entry(mode: &str, oid: &str) -> Option<TreeEntry> {
    return Some(TreeEntry {
        mode: String::from(mode),
        oid: String::from(oid),
    });
}

/// Raw records keep the absent side as `None` and the path bytes as written.
#[test]
fn raw_records_parse_with_absent_sides() {
    let changes: Vec<RawChange> = parse_raw_changes(
        b":000000 100644 0000 1111 A\0added.txt\0:100644 000000 2222 0000 D\0gone.txt\0:100644 100755 3333 4444 M\0mode.txt\0",
    )
    .expect("records");
    assert_eq!(changes.len(), 3);
    assert_eq!(changes[0].from, None);
    assert_eq!(changes[0].to, entry("100644", "1111"));
    assert_eq!(changes[1].to, None);
    assert_eq!(changes[2].path, b"mode.txt".to_vec());
    assert_eq!(changes[2].from, entry("100644", "3333"));
    assert_eq!(changes[2].to, entry("100755", "4444"));
}

/// A name that is not UTF-8 reaches the record as the bytes Git listed,
/// so nothing rewrites it on the way to the synthetic base.
#[test]
fn raw_records_keep_path_bytes_that_are_not_text() {
    let output: &[u8] = b":100644 100644 3333 4444 M\0caf\xff.txt\0";
    let changes: Vec<RawChange> = parse_raw_changes(output).expect("records");
    assert_eq!(changes[0].path, b"caf\xff.txt".to_vec());
}

/// A header without its path,
/// or without five fields,
/// fails.
#[test]
fn malformed_raw_records_fail() {
    assert!(parse_raw_changes(b":100644 100644 a b M").is_err());
    assert!(parse_raw_changes(b":100644 100644 a M\0x\0").is_err());
    assert!(parse_raw_changes(b"").expect("empty").is_empty());
}

/// Only `100644` and `100755` are regular files;
/// a symbolic link or a gitlink never is.
#[test]
fn regular_file_modes_are_the_only_regular_files() {
    assert!(is_regular_mode("100644"));
    assert!(is_regular_mode("100755"));
    assert!(!is_regular_mode("120000"));
    assert!(!is_regular_mode("160000"));
}

/// Regular triples need all three present;
/// a text candidate needs landed and prepared present and regular,
///  and the base regular or absent.
#[test]
fn the_predicates_follow_the_entries() {
    let regular = |oid: &str| { return entry("100644", oid); };
    let all = SharedPath {
        path: b"p".to_vec(),
        base: regular("a"),
        landed: regular("b"),
        prepared: regular("c"),
    };
    assert!(is_regular_triple(&all));
    assert!(is_text_candidate(&all));
    let added_both: SharedPath = SharedPath {
        path: b"p".to_vec(),
        base: None,
        landed: regular("b"),
        prepared: regular("c"),
    };
    assert!(!is_regular_triple(&added_both));
    assert!(is_text_candidate(&added_both));
    let link: SharedPath = SharedPath {
        path: b"p".to_vec(),
        base: regular("a"),
        landed: entry("120000", "b"),
        prepared: regular("c"),
    };
    assert!(!is_text_candidate(&link));
    let deleted: SharedPath = SharedPath {
        path: b"p".to_vec(),
        base: regular("a"),
        landed: None,
        prepared: regular("c"),
    };
    assert!(!is_text_candidate(&deleted));
    assert!(same_entry(&None, &None));
    assert!(!same_entry(&None, &regular("a")));
}

/// Listing a repository:
/// the landed listing's order,
/// the paths both sides changed,
///  and each side's entry,
/// with deletions and absent base entries.
#[test]
fn shared_paths_list_the_paths_both_sides_changed() {
    let root: PathBuf = fixture("replay-shared-paths");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("d.txt"), "d\n").expect("d");
    std::fs::write(repo.join("f.txt"), "f\n").expect("f");
    std::fs::write(repo.join("c.txt"), "c\n").expect("c");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    git(repo.as_path(), &["checkout", "--quiet", "-b", "prepared"]);
    std::fs::write(repo.join("d.txt"), "prepared d\n").expect("prepared d");
    std::fs::write(repo.join("f.txt"), "prepared f\n").expect("prepared f");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=prepared"]);
    let prepared: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::write(repo.join("d.txt"), "landed d\n").expect("landed d");
    std::fs::remove_file(repo.join("f.txt")).expect("remove f");
    std::fs::write(repo.join("e.txt"), "e\n").expect("e");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=landed"]);
    let current: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    let shared: Vec<SharedPath> = list_shared_paths(
        &git_context(),
        repo.join(".git").as_path(),
        repo.as_path(),
        base.as_str(),
        current.as_str(),
        prepared.as_str(),
    )
    .expect("shared paths");
    let paths: Vec<&[u8]> = shared.iter().map(|entry| { return entry.path.as_slice(); }).collect();
    assert_eq!(paths, vec![b"d.txt".as_slice(), b"f.txt".as_slice()]);
    assert!(shared[0].base.is_some() && shared[0].landed.is_some() && shared[0].prepared.is_some());
    assert_eq!(shared[1].landed, None);
    assert!(shared[1].prepared.is_some());
    remove(root.as_path());
}

/// Two sides that changed different paths share none,
/// so the listing is empty and the replay has no path to decide.
#[test]
fn sides_that_changed_different_paths_share_none() {
    let root: PathBuf = fixture("replay-shared-paths-none");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("base.txt"), "base\n").expect("base");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    git(repo.as_path(), &["checkout", "--quiet", "-b", "prepared"]);
    std::fs::write(repo.join("mine.txt"), "mine\n").expect("mine");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=prepared"]);
    let prepared: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::write(repo.join("theirs.txt"), "theirs\n").expect("theirs");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=landed"]);
    let current: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    let shared: Vec<SharedPath> = list_shared_paths(
        &git_context(),
        repo.join(".git").as_path(),
        repo.as_path(),
        base.as_str(),
        current.as_str(),
        prepared.as_str(),
    )
    .expect("shared paths");
    assert!(shared.is_empty(), "no path was changed on both sides: {shared:?}");
    remove(root.as_path());
}

/// What:
///  Listing the shared paths reads each side's changes once.
/// Why:
///  `AGENTS.md` `RG2`:
/// both listings grow with the history since the base and with the
///  prepared commit,
/// and finding each landed change by walking the whole prepared listing
///  is quadratic in the two change counts.
///  The seam counts the collected changes a question looks at,
///  which is one per change when a map answers and every prepared change when a walk does
///  (`foundation/collected_elements_looked_at_tests.rs` proves the count separates the two).
#[test]
fn listing_shared_paths_reads_each_change_once() {
    let root: PathBuf = fixture("replay-shared-paths-once");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let width: usize = 512;
    for number in 0..width {
        std::fs::write(repo.join(format!("shared-{number:04}.txt")), format!("{number}\n")).expect("file");
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["checkout", "--quiet", "-b", "prepared"]);
    for number in 0..width {
        std::fs::write(repo.join(format!("shared-{number:04}.txt")), format!("prepared {number}\n")).expect("prepared file");
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=prepared"]);
    let prepared: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    for number in 0..width {
        std::fs::write(repo.join(format!("shared-{number:04}.txt")), format!("landed {number}\n")).expect("landed file");
    }
    // A path only the landed side changed shares nothing and must not join the listing.
    std::fs::write(repo.join("landed-only.txt"), "landed\n").expect("landed only");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=landed"]);
    let current: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let before: u64 = crate::collected_elements_looked_at::looked_at();
    let shared: Vec<SharedPath> = list_shared_paths(
        &git_context(),
        repo.join(".git").as_path(),
        repo.as_path(),
        base.as_str(),
        current.as_str(),
        prepared.as_str(),
    )
    .expect("shared paths");
    let looked: u64 = crate::collected_elements_looked_at::looked_at() - before;
    assert_eq!(shared.len(), width, "the paths both sides changed, in the landed listing's order");
    assert!(!shared.iter().any(|entry| return entry.path == b"landed-only.txt".to_vec()));
    assert_eq!(
        looked,
        2 * width as u64 + 1,
        "{looked} collected changes looked at for {width} prepared changes and {} landed ones; one pass over each listing is linear", width + 1
    );
    remove(root.as_path());
}
