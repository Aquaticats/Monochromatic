//! Controls for subsumption:
//!  the synthetic-base record formats, and the per-path outcome of a real repository scenario
//!  with identical, one-sided, far-apart, delete-against-modify, and capture-ordered paths.

use super::*;
use crate::replay_shared_paths::list_shared_paths;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::path::PathBuf;

/// The commit or blob a revision names, trimmed.
fn rev(repo: &std::path::Path, spec: &str) -> String {
    return git_text(repo, &["rev-parse", spec]).trim().to_string();
}

/// The shared paths of a repository's scenario, listed once as replay's caller does.
fn shared_of(repo: &Path, base: &str, current: &str, prepared: &str) -> Vec<SharedPath> {
    return list_shared_paths(&git_context(), repo.join(".git").as_path(), repo, base, current, prepared)
        .expect("shared paths");
}

/// An entry with a mode and object of the given text.
fn entry(mode: &str, oid: &str) -> Option<TreeEntry> {
    return Some(TreeEntry {
        mode: String::from(mode),
        oid: String::from(oid),
    });
}

/// A shared path with its three entries.
fn shared(base: Option<TreeEntry>, landed: Option<TreeEntry>, prepared: Option<TreeEntry>) -> SharedPath {
    return SharedPath {
        path: String::from("p.txt"),
        base,
        landed,
        prepared,
    };
}

/// A landed deletion records a zero object of the base's width and mode `0`.
#[test]
fn a_landed_deletion_removes_the_path() {
    let deleted: SharedPath = shared(entry("100644", "aaaaaaa"), None, entry("100644", "bbbbbbb"));
    assert_eq!(subsumed_record(&deleted), b"0 0000000\tp.txt\0".to_vec());
}

/// A regular file keeps the base's mode in the subsumed record; other entries take the landed mode.
#[test]
fn subsumed_records_keep_the_base_mode_only_for_regular_files() {
    let regular: SharedPath = shared(entry("100644", "aaaa"), entry("100755", "bbbb"), entry("100644", "cccc"));
    assert_eq!(subsumed_record(&regular), b"100644 bbbb\tp.txt\0".to_vec());
    let added: SharedPath = shared(None, entry("100755", "bbbb"), entry("100755", "bbbb"));
    assert_eq!(subsumed_record(&added), b"100755 bbbb\tp.txt\0".to_vec());
    let link: SharedPath = shared(entry("120000", "aaaa"), entry("120000", "bbbb"), entry("120000", "cccc"));
    assert_eq!(subsumed_record(&link), b"120000 bbbb\tp.txt\0".to_vec());
}

/// An exact record names the entry's mode and object; an absent entry writes zeros of the width.
#[test]
fn exact_records_name_the_entry_or_zeros() {
    assert_eq!(exact_record("p.txt", &entry("100644", "abcd"), 4), b"100644 abcd\tp.txt\0".to_vec());
    assert_eq!(exact_record("p.txt", &None, 4), b"0 0000\tp.txt\0".to_vec());
}

/// Prepared-ordered paths take the landed entry and landed-ordered paths take the prepared entry.
#[test]
fn ordered_edits_take_the_side_they_keep() {
    let both: SharedPath = shared(entry("100644", "aaaa"), entry("100644", "bbbb"), entry("100755", "cccc"));
    let prepared_rule: BaseEdit = BaseEdit { shared: both.clone(), rule: BaseRule::Prepared };
    let landed_rule: BaseEdit = BaseEdit { shared: both, rule: BaseRule::Landed };
    assert_eq!(base_record(&prepared_rule), b"100644 bbbb\tp.txt\0".to_vec());
    assert_eq!(base_record(&landed_rule), b"100755 cccc\tp.txt\0".to_vec());
}

/// Builds the scenario repository: the base, the landed commit on `main`, and the prepared commit on a branch.
fn scenario(name: &str) -> (PathBuf, PathBuf, String, String, String) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let twelve: String = (1..=12).map(|number| { return format!("line {number}\n"); }).collect();
    std::fs::write(repo.join("a.txt"), "x\n").expect("a");
    std::fs::write(repo.join("b.txt"), "a\nb\nc\nd\n").expect("b");
    std::fs::write(repo.join("c.txt"), &twelve).expect("c");
    std::fs::write(repo.join("d.txt"), "keep\n").expect("d");
    std::fs::write(repo.join("e.txt"), "e base\n").expect("e");
    std::fs::write(repo.join("f.txt"), "f base\n").expect("f");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    git(repo.as_path(), &["checkout", "--quiet", "-b", "prepared"]);
    std::fs::write(repo.join("a.txt"), "landed a\n").expect("prepared a");
    std::fs::write(repo.join("b.txt"), "a\nB\nC\nd\n").expect("prepared b");
    std::fs::write(repo.join("c.txt"), twelve.replace("line 11\n", "OWN\n")).expect("prepared c");
    std::fs::write(repo.join("d.txt"), "d prepared\n").expect("prepared d");
    std::fs::write(repo.join("e.txt"), "e prepared\n").expect("prepared e");
    std::fs::write(repo.join("f.txt"), "f prepared\n").expect("prepared f");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=prepared"]);
    let prepared: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::write(repo.join("a.txt"), "landed a\n").expect("landed a");
    std::fs::write(repo.join("b.txt"), "a\nB\nc\nd\n").expect("landed b");
    std::fs::write(repo.join("c.txt"), twelve.replace("line 2\n", "LANDED\n")).expect("landed c");
    std::fs::remove_file(repo.join("d.txt")).expect("landed d");
    std::fs::write(repo.join("e.txt"), "e landed\n").expect("landed e");
    std::fs::write(repo.join("f.txt"), "f landed\n").expect("landed f");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=landed"]);
    let current: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    return (root, repo, base, current, prepared);
}

/// Per path: identity and the text check subsume; far-apart edits, delete against modify, and unlisted paths do not;
///  capture order settles e (prepared later) and f (landed later), and the synthetic base takes the entries it keeps.
#[test]
fn the_scenario_settles_each_path_by_its_rule() {
    let (root, repo, base, current, prepared) = scenario("replay-subsumption-scenario");
    let scratch: PathBuf = root.join("scratch");
    std::fs::create_dir(&scratch).expect("scratch");
    let order: BTreeMap<String, CaptureOrderDecision> = BTreeMap::from([
        (String::from("e.txt"), CaptureOrderDecision::Prepared),
        (String::from("f.txt"), CaptureOrderDecision::Landed),
    ]);
    let outcome: SubsumptionBase = subsume_landed_changes(
        &git_context(),
        repo.join(".git").as_path(),
        repo.join(".git").join("objects").as_path(),
        repo.as_path(),
        scratch.as_path(),
        1,
        base.as_str(),
        &shared_of(repo.as_path(), base.as_str(), current.as_str(), prepared.as_str()),
        &order,
    )
    .expect("subsumption");
    assert_eq!(outcome.subsumed_paths, vec!["a.txt", "b.txt", "e.txt"]);
    assert_eq!(outcome.kept_landed_paths, vec!["f.txt"]);
    assert_ne!(outcome.merge_base, base);
    let landed_b: String = rev(repo.as_path(), &format!("{current}:b.txt"));
    assert_eq!(rev(repo.as_path(), &format!("{}:b.txt", outcome.merge_base)), landed_b);
    let prepared_f: String = rev(repo.as_path(), &format!("{prepared}:f.txt"));
    assert_eq!(rev(repo.as_path(), &format!("{}:f.txt", outcome.merge_base)), prepared_f);
    let landed_e: String = rev(repo.as_path(), &format!("{current}:e.txt"));
    assert_eq!(rev(repo.as_path(), &format!("{}:e.txt", outcome.merge_base)), landed_e);
    remove(root.as_path());
}

/// With no path settled, the effective merge base is the preparation base itself and no synthetic commit is written.
#[test]
fn no_settled_path_keeps_the_preparation_base() {
    let root: PathBuf = fixture("replay-subsumption-none");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("x.txt"), "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n").expect("x");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    git(repo.as_path(), &["checkout", "--quiet", "-b", "prepared"]);
    std::fs::write(repo.join("x.txt"), "PREPARED\n2\n3\n4\n5\n6\n7\n8\n9\n10\n").expect("prepared");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=prepared"]);
    let prepared: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::write(repo.join("x.txt"), "1\n2\n3\n4\n5\n6\n7\n8\n9\nLANDED\n").expect("landed");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=landed"]);
    let current: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    let scratch: PathBuf = root.join("scratch");
    std::fs::create_dir(&scratch).expect("scratch");
    let outcome: SubsumptionBase = subsume_landed_changes(
        &git_context(),
        repo.join(".git").as_path(),
        repo.join(".git").join("objects").as_path(),
        repo.as_path(),
        scratch.as_path(),
        1,
        base.as_str(),
        &shared_of(repo.as_path(), base.as_str(), current.as_str(), prepared.as_str()),
        &BTreeMap::new(),
    )
    .expect("subsumption");
    assert_eq!(outcome.merge_base, base);
    assert!(outcome.subsumed_paths.is_empty() && outcome.kept_landed_paths.is_empty());
    remove(root.as_path());
}

/// Identical gitlinks and identical binary blobs are subsumed; a different gitlink and a different binary blob are not.
#[test]
fn identical_gitlinks_and_binary_blobs_are_subsumed_only_when_identical() {
    let root: PathBuf = fixture("replay-subsumption-gitlink-binary");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let gitlink = |path: &str, sha: &str| {
        let cacheinfo: String = format!("160000,{sha},{path}");
        git(repo.as_path(), &["update-index", "--add", "--cacheinfo", cacheinfo.as_str()]);
    };
    std::fs::write(repo.join("bin1.dat"), b"a\0b\n").expect("bin1");
    std::fs::write(repo.join("bin2.dat"), b"a\0b\n").expect("bin2");
    gitlink("sub1", "3333333333333333333333333333333333333333");
    gitlink("sub2", "3333333333333333333333333333333333333333");
    git(repo.as_path(), &["add", "bin1.dat", "bin2.dat"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = rev(repo.as_path(), "HEAD");
    git(repo.as_path(), &["checkout", "--quiet", "-b", "prepared"]);
    std::fs::write(repo.join("bin1.dat"), b"x\0L\n").expect("prepared bin1");
    std::fs::write(repo.join("bin2.dat"), b"x\0P\n").expect("prepared bin2");
    gitlink("sub1", "1111111111111111111111111111111111111111");
    gitlink("sub2", "2222222222222222222222222222222222222222");
    git(repo.as_path(), &["add", "bin1.dat", "bin2.dat"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=prepared"]);
    let prepared: String = rev(repo.as_path(), "HEAD");
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::write(repo.join("bin1.dat"), b"x\0L\n").expect("landed bin1");
    std::fs::write(repo.join("bin2.dat"), b"x\0L\n").expect("landed bin2");
    gitlink("sub1", "1111111111111111111111111111111111111111");
    gitlink("sub2", "1111111111111111111111111111111111111111");
    git(repo.as_path(), &["add", "bin1.dat", "bin2.dat"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=landed"]);
    let current: String = rev(repo.as_path(), "HEAD");
    let scratch: PathBuf = root.join("scratch");
    std::fs::create_dir(&scratch).expect("scratch");
    let outcome: SubsumptionBase = subsume_landed_changes(
        &git_context(),
        repo.join(".git").as_path(),
        repo.join(".git").join("objects").as_path(),
        repo.as_path(),
        scratch.as_path(),
        1,
        base.as_str(),
        &shared_of(repo.as_path(), base.as_str(), current.as_str(), prepared.as_str()),
        &BTreeMap::new(),
    )
    .expect("subsumption");
    assert_eq!(outcome.subsumed_paths, vec!["bin1.dat", "sub1"]);
    remove(root.as_path());
}

/// A pair that is not all regular files takes the landed mode, even when the base has another mode.
#[test]
fn a_non_regular_triple_takes_the_landed_mode() {
    let link: SharedPath = shared(entry("100644", "aa"), entry("120000", "bb"), entry("120000", "cc"));
    assert_eq!(subsumed_record(&link), b"120000 bb\tp.txt\0".to_vec());
}

/// An absent prepared entry is written as zeros of the width of the present entries.
#[test]
fn the_exact_record_width_follows_the_present_entries() {
    let partial: SharedPath = shared(entry("100644", "aaaa"), entry("100644", "bbbb"), None);
    let edit: BaseEdit = BaseEdit {
        shared: partial,
        rule: BaseRule::Landed,
    };
    assert_eq!(base_record(&edit), b"0 0000\tp.txt\0".to_vec());
}
