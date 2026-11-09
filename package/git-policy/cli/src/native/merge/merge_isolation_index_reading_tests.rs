//! Controls for the index reading a merge's recovery needs:
//!  the work each changed path's standing costs over the index records.

use super::*;
/// The paths a fixture repository is built under.
use std::path::PathBuf;
/// The place a reading runs in.
use crate::merge_isolation_recovery_steps::RecoveryPlace;
/// Fixtures,
/// and the counters that weigh a pass (`foundation/collected_elements_looked_at.rs`).
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};

/// What:
///  Deciding how the index stands at each changed path asks the staged records once per path,
///  not every one of them per path.
/// Why:
///  `AGENTS.md` `RG2`:
/// both the changed paths and the staged records grow with the repository,
///  so looking each path up in the staged list one at a time walks that list once per path and is
///  quadratic.
///  The seam counts the collected elements a question looks at
///  (`foundation/collected_elements_looked_at_tests.rs` proves the count separates a set from a walk):
///  a map built in one pass and asked once per path counts one look per record and one per question,
///  while a walk over the records counts every record up to the one that matched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(lookedAt(() => indexStandings(place, root, oldOid, staged))).toBe(2 * staged.length);
/// ```
#[test]
fn a_standing_for_each_changed_path_reads_the_staged_records_once() {
    let root: PathBuf = fixture("merge-index-standings-once");
    let repo: PathBuf = repository(root.as_path(), "repo");
    // The width is the number of staged paths and of changed paths at once, so a walk costs its square.
    let width: usize = 64;
    for number in 0..width {
        std::fs::write(repo.join(format!("path-{number:04}.txt")), format!("base {number}\n")).expect("base file");
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let old_oid: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    for number in 0..width {
        std::fs::write(repo.join(format!("path-{number:04}.txt")), format!("staged {number}\n")).expect("staged file");
    }
    git(repo.as_path(), &["add", "--all"]);
    let git_dir: PathBuf = repo.join(".git");
    let place: RecoveryPlace = RecoveryPlace { git: git_context(), cwd: repo.clone(), common_dir: git_dir.clone(), git_dir };
    let staged: Vec<Change> = index_entries(&place, repo.as_path(), old_oid.as_str()).expect("the staged records");
    assert_eq!(staged.len(), width, "every path is staged against the commit");
    // Measuring the records first is what makes the count below the standings' own work: `index_standings` reads them again.
    let before: u64 = crate::collected_elements_looked_at::looked_at();
    let standings: Vec<Standing> = index_standings(&place, repo.as_path(), old_oid.as_str(), staged.as_slice()).expect("the standings");
    let counted: u64 = crate::collected_elements_looked_at::looked_at() - before;
    assert_eq!(standings.len(), width, "every changed path got its standing");
    assert!(
        standings.iter().all(|standing| return *standing == Standing::New),
        "every staged entry is the change's own end state: {standings:?}"
    );
    assert_eq!(
        counted,
        2 * width as u64,
        "{counted} staged records looked at for {width} changed paths; one pass to map them and one question per path is \
linear, a walk over them per path is {} looks",
        width as u64 * width as u64
    );
    remove(root.as_path());
}
