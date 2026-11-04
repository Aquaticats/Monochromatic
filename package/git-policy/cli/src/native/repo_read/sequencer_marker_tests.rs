//! Differential controls for the sequencer marker proof:
//!  a marker reported absent must make `git rev-parse --verify --quiet` fail,
//!  and every kind of entry that Git could resolve is reported as maybe present.

use super::*;
use crate::capture_invocation::resolve_conclusion_kind;
use crate::recording_git::{recorded_starts, recording_context};
use crate::ref_files::tests::{Probe, probe};
use crate::test_support::{fixture, git, git_output, git_text, remove, repository};
use crate::transaction_git::{GitContext, GitFailure};
use crate::transaction_journal::Conclusion;

/// The three markers in the order the commit path asks.
const NAMES: [&str; 3] = ["MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD"];

/// The ref file places of a probed repository.
fn layout_of(found: &Probe) -> RefFilesLayout<'_> {
    return RefFilesLayout {
        git_dir: found.git_dir.as_path(),
        common_dir: found.common_dir.as_path(),
        ref_format: found.ref_format,
        object_format: found.object_format.as_str(),
    };
}

/// Whether real Git resolves `name` in the repository at `directory`.
fn git_resolves(directory: &Path, name: &str) -> bool {
    return git_output(directory, &["rev-parse", "--verify", "--quiet", name]).status.success();
}

/// The proof's answers for the three markers,
///  after requiring that every "absent" answer agrees with real Git.
fn proven(directory: &Path, found: &Probe) -> Vec<bool> {
    let maybe: Vec<bool> = markers_maybe_present(&layout_of(found), &NAMES);
    for (name, maybe_present) in NAMES.iter().zip(maybe.iter()) {
        if !maybe_present {
            assert!(!git_resolves(directory, name), "{name} is reported absent but Git resolves it");
        }
    }
    return maybe;
}

/// The conclusion the commit path decides for the repository at `directory`,
///  with the recorded Git starts.
fn conclusion_with_starts(
    directory: &Path,
    found: &Probe,
    root: &Path,
) -> (Result<Conclusion, GitFailure>, Vec<String>) {
    let (context, log): (GitContext, PathBuf) = recording_context(root);
    let result: Result<Conclusion, GitFailure> =
        resolve_conclusion_kind(&context, directory, false, &layout_of(found));
    return (result, recorded_starts(log.as_path()));
}

/// The starts that asked Git about a marker.
fn marker_starts(starts: &[String]) -> Vec<&String> {
    return starts.iter().filter(|start| { return start.starts_with("rev-parse --verify --quiet"); }).collect();
}

/// With no marker anywhere every marker is absent,
///  the conclusion is none, and no Git process is started for it.
#[test]
fn no_marker_is_absent_without_a_start() {
    let root: PathBuf = fixture("marker-none");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let found: Probe = probe(repo.as_path());
    assert_eq!(proven(repo.as_path(), &found), vec![false, false, false]);
    let (result, starts): (Result<Conclusion, GitFailure>, Vec<String>) =
        conclusion_with_starts(repo.as_path(), &found, root.as_path());
    assert_eq!(result, Ok(Conclusion::None));
    assert!(starts.is_empty(), "unexpected starts: {starts:?}");
    remove(root.as_path());
}

/// A valid marker is maybe present, Git resolves it, and only that marker's start happens.
#[test]
fn a_valid_marker_is_resolved_by_git() {
    let root: PathBuf = fixture("marker-valid");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let found: Probe = probe(repo.as_path());
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(found.git_dir.join("MERGE_HEAD"), format!("{head}\n")).expect("fixture");
    assert_eq!(proven(repo.as_path(), &found), vec![true, false, false]);
    assert!(git_resolves(repo.as_path(), "MERGE_HEAD"));
    let (result, starts): (Result<Conclusion, GitFailure>, Vec<String>) =
        conclusion_with_starts(repo.as_path(), &found, root.as_path());
    assert_eq!(result, Ok(Conclusion::Merge));
    assert_eq!(marker_starts(&starts).len(), 1, "{starts:?}");
    remove(root.as_path());
}

/// A marker naming a missing object is maybe present and Git accepts it,
///  so the conclusion stays a merge.
#[test]
fn a_dangling_marker_is_resolved_by_git() {
    let root: PathBuf = fixture("marker-dangling");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let found: Probe = probe(repo.as_path());
    std::fs::write(found.git_dir.join("MERGE_HEAD"), format!("{}\n", "a".repeat(40))).expect("fixture");
    assert_eq!(proven(repo.as_path(), &found), vec![true, false, false]);
    assert!(git_resolves(repo.as_path(), "MERGE_HEAD"));
    let (result, _starts): (Result<Conclusion, GitFailure>, Vec<String>) =
        conclusion_with_starts(repo.as_path(), &found, root.as_path());
    assert_eq!(result, Ok(Conclusion::Merge));
    remove(root.as_path());
}

/// A garbage marker is maybe present, Git rejects it, and the conclusion is none.
#[test]
fn a_garbage_marker_is_resolved_by_git() {
    let root: PathBuf = fixture("marker-garbage");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let found: Probe = probe(repo.as_path());
    std::fs::write(found.git_dir.join("REVERT_HEAD"), "not-an-object\n").expect("fixture");
    assert_eq!(proven(repo.as_path(), &found), vec![false, false, true]);
    assert!(!git_resolves(repo.as_path(), "REVERT_HEAD"));
    let (result, starts): (Result<Conclusion, GitFailure>, Vec<String>) =
        conclusion_with_starts(repo.as_path(), &found, root.as_path());
    assert_eq!(result, Ok(Conclusion::None));
    assert_eq!(marker_starts(&starts).len(), 1, "{starts:?}");
    remove(root.as_path());
}

/// Branches and tags named like markers resolve through Git's lookup spellings,
///  so the proof reports them and the conclusion follows Git.
#[test]
fn a_branch_or_tag_named_like_a_marker_is_resolved_by_git() {
    let root: PathBuf = fixture("marker-named");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let found: Probe = probe(repo.as_path());
    git(repo.as_path(), &["branch", "MERGE_HEAD"]);
    git(repo.as_path(), &["tag", "CHERRY_PICK_HEAD"]);
    assert_eq!(proven(repo.as_path(), &found), vec![true, true, false]);
    assert!(git_resolves(repo.as_path(), "MERGE_HEAD"));
    assert!(git_resolves(repo.as_path(), "CHERRY_PICK_HEAD"));
    let (result, _starts): (Result<Conclusion, GitFailure>, Vec<String>) =
        conclusion_with_starts(repo.as_path(), &found, root.as_path());
    assert_eq!(result, Ok(Conclusion::Merge));
    git(repo.as_path(), &["branch", "--delete", "MERGE_HEAD"]);
    assert_eq!(proven(repo.as_path(), &found), vec![false, true, false]);
    let (second, _starts_again): (Result<Conclusion, GitFailure>, Vec<String>) =
        conclusion_with_starts(repo.as_path(), &found, root.as_path());
    assert_eq!(second, Ok(Conclusion::CherryPick));
    remove(root.as_path());
}

/// Every one of the six lookup spellings is a hit,
///  and the other markers stay quiet.
#[test]
fn each_lookup_spelling_is_a_hit() {
    let root: PathBuf = fixture("marker-spellings");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let found: Probe = probe(repo.as_path());
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let spellings: [&str; 6] = [
        "MERGE_HEAD",
        "refs/MERGE_HEAD",
        "refs/tags/MERGE_HEAD",
        "refs/heads/MERGE_HEAD",
        "refs/remotes/MERGE_HEAD",
        "refs/remotes/MERGE_HEAD/HEAD",
    ];
    for spelling in spellings {
        let base: PathBuf = if spelling == "MERGE_HEAD" { found.git_dir.clone() } else { found.common_dir.clone() };
        let path: PathBuf = base.join(spelling);
        std::fs::create_dir_all(path.parent().expect("fixture")).expect("fixture");
        std::fs::write(&path, format!("{head}\n")).expect("fixture");
        assert_eq!(proven(repo.as_path(), &found), vec![true, false, false], "{spelling}");
        assert!(git_resolves(repo.as_path(), "MERGE_HEAD"), "Git resolves {spelling}");
        std::fs::remove_file(&path).expect("fixture");
        if spelling.ends_with("/HEAD") {
            std::fs::remove_dir(found.common_dir.join("refs/remotes/MERGE_HEAD")).expect("fixture");
        }
    }
    assert_eq!(proven(repo.as_path(), &found), vec![false, false, false]);
    remove(root.as_path());
}

/// A directory, a link or a file of any content at a lookup place is a hit.
#[test]
fn any_kind_of_entry_is_a_hit() {
    let root: PathBuf = fixture("marker-kinds");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let found: Probe = probe(repo.as_path());
    std::fs::create_dir_all(found.common_dir.join("refs/heads/MERGE_HEAD/inner")).expect("fixture");
    assert_eq!(proven(repo.as_path(), &found), vec![true, false, false]);
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("nowhere", found.git_dir.join("CHERRY_PICK_HEAD")).expect("fixture");
        assert_eq!(proven(repo.as_path(), &found), vec![true, true, false]);
    }
    std::fs::write(found.git_dir.join("REVERT_HEAD"), b"").expect("fixture");
    assert_eq!(proven(repo.as_path(), &found).last().copied(), Some(true));
    remove(root.as_path());
}

/// A marker that exists only in `packed-refs` is a hit and Git resolves it;
///  an unrelated packed ref is not.
#[test]
fn a_marker_only_in_packed_refs_is_a_hit() {
    let root: PathBuf = fixture("marker-packed");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let found: Probe = probe(repo.as_path());
    git(repo.as_path(), &["pack-refs", "--all", "--prune"]);
    assert!(found.common_dir.join("packed-refs").is_file());
    assert_eq!(proven(repo.as_path(), &found), vec![false, false, false], "only main is packed");
    git(repo.as_path(), &["tag", "REVERT_HEAD"]);
    git(repo.as_path(), &["pack-refs", "--all", "--prune"]);
    assert!(!found.common_dir.join("refs/tags/REVERT_HEAD").exists());
    assert_eq!(proven(repo.as_path(), &found), vec![false, false, true]);
    assert!(git_resolves(repo.as_path(), "REVERT_HEAD"));
    let (result, _starts): (Result<Conclusion, GitFailure>, Vec<String>) =
        conclusion_with_starts(repo.as_path(), &found, root.as_path());
    assert_eq!(result, Ok(Conclusion::Revert));
    remove(root.as_path());
}

/// A linked worktree has its own markers in its own Git directory:
///  the main worktree's marker is invisible to it, and its marker is invisible to the main worktree.
#[test]
fn a_linked_worktree_uses_its_own_git_directory() {
    let root: PathBuf = fixture("marker-linked");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let linked: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "feature", linked.to_str().expect("fixture")]);
    let main_found: Probe = probe(repo.as_path());
    let linked_found: Probe = probe(linked.as_path());
    assert_ne!(main_found.git_dir, linked_found.git_dir);
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(main_found.git_dir.join("MERGE_HEAD"), format!("{head}\n")).expect("fixture");
    assert_eq!(proven(repo.as_path(), &main_found), vec![true, false, false]);
    assert_eq!(proven(linked.as_path(), &linked_found), vec![false, false, false]);
    assert!(!git_resolves(linked.as_path(), "MERGE_HEAD"));
    std::fs::write(linked_found.git_dir.join("CHERRY_PICK_HEAD"), format!("{head}\n")).expect("fixture");
    assert_eq!(proven(linked.as_path(), &linked_found), vec![false, true, false]);
    assert_eq!(proven(repo.as_path(), &main_found), vec![true, false, false]);
    assert!(git_resolves(linked.as_path(), "CHERRY_PICK_HEAD"));
    assert!(!git_resolves(repo.as_path(), "CHERRY_PICK_HEAD"));
    let (result, _starts): (Result<Conclusion, GitFailure>, Vec<String>) =
        conclusion_with_starts(linked.as_path(), &linked_found, root.as_path());
    assert_eq!(result, Ok(Conclusion::CherryPick));
    remove(root.as_path());
}

/// A reftable repository is never proven absent,
///  and Git decides, with the same conclusion as before.
#[test]
fn a_reftable_repository_is_left_to_git() {
    let root: PathBuf = fixture("marker-reftable");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("fixture");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main", "--ref-format=reftable"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    let found: Probe = probe(repo.as_path());
    assert_eq!(proven(repo.as_path(), &found), vec![true, true, true]);
    let (none, starts): (Result<Conclusion, GitFailure>, Vec<String>) =
        conclusion_with_starts(repo.as_path(), &found, root.as_path());
    assert_eq!(none, Ok(Conclusion::None));
    assert_eq!(marker_starts(&starts).len(), 3, "{starts:?}");
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(found.git_dir.join("MERGE_HEAD"), format!("{head}\n")).expect("fixture");
    assert!(git_resolves(repo.as_path(), "MERGE_HEAD"));
    let (merge, _starts_again): (Result<Conclusion, GitFailure>, Vec<String>) =
        conclusion_with_starts(repo.as_path(), &found, root.as_path());
    assert_eq!(merge, Ok(Conclusion::Merge));
    remove(root.as_path());
}

/// The backend check refuses on its own:
///  reftable repositories also trip over their placeholder `refs` files,
///  but a layout that says reftable is never proven absent even when every place looks empty.
#[test]
fn the_backend_check_refuses_when_every_place_is_empty() {
    let root: PathBuf = fixture("marker-backend");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let found: Probe = probe(repo.as_path());
    let layout: RefFilesLayout<'_> = RefFilesLayout { ref_format: RefFormat::Reftable, ..layout_of(&found) };
    assert_eq!(markers_maybe_present(&layout, &NAMES), vec![true, true, true]);
    assert_eq!(markers_maybe_present(&layout_of(&found), &NAMES), vec![false, false, false]);
    remove(root.as_path());
}

/// An unreadable `packed-refs` (a directory in its place) is not proof of absence.
#[test]
fn an_unreadable_packed_refs_is_a_hit() {
    let root: PathBuf = fixture("marker-packed-directory");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let found: Probe = probe(repo.as_path());
    std::fs::create_dir(found.common_dir.join("packed-refs")).expect("fixture");
    assert_eq!(markers_maybe_present(&layout_of(&found), &NAMES), vec![true, true, true]);
    remove(root.as_path());
}

/// The byte search finds a needle anywhere and treats an empty needle as found.
#[test]
fn the_byte_search_is_a_plain_substring_search() {
    assert!(contains_bytes(b"abc MERGE_HEAD\n", b"MERGE_HEAD"));
    assert!(!contains_bytes(b"abc MERGE_HEA\n", b"MERGE_HEAD"));
    assert!(!contains_bytes(b"", b"MERGE_HEAD"));
    assert!(contains_bytes(b"x", b""));
}
