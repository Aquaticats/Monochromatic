//! Controls for the clone rung of a source with changes:
//! the paths their entries prove are cloned,
//! the paths a change reaches are written by Git,
//! the new index
//!  describes the worktree Git's own checkout would make,
//! and the result is proven before it is published.

use super::*;
use crate::fs_beneath::open_root;
use crate::fs_clone_exchange::clone_file;
use crate::test_support::{git, git_context, git_output, git_text, repository};
use crate::worktree_clean::head_commit;
use crate::worktree_copy::{GitStatus, GitStep};
use crate::worktree_ladder_target::{Target, apply_difference};
use crate::worktree_snapshot_source::{SnapshotSource, source_or_dirty};
use crate::worktree_ladder_test_support::{commit_all, main_identity, tracked};
use std::ffi::OsString;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// What:
///  The fixture root when a file on it can be reflinked,
/// or nothing after saying that this control did not run.
/// Why:
///   A clone of a file with bytes needs `FICLONE`,
/// which the gate image's fixture root (a tmpfs) does not answer,
/// and a control that passed there would claim coverage it has not got.
///A root that cannot clone says so once,
///  with the control's name,
///  as a control that needs btrfs does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cloneCapableFixture(name: string): string | undefined;
/// ```
fn clone_capable_fixture(name: &str) -> Option<PathBuf> {
    let root: PathBuf = crate::test_support::fixture(name);
    let from: PathBuf = root.join("probe-from");
    let to: PathBuf = root.join("probe-to");
    std::fs::write(from.as_path(), b"probe\n").expect("the probe's file");
    std::fs::write(to.as_path(), b"").expect("the probe's clone");
    let opened = (std::fs::File::open(from.as_path()), std::fs::OpenOptions::new().write(true).open(to.as_path()));
    let cloned: bool = match opened {
        (Ok(source), Ok(destination)) => clone_file(&source, &destination, to.as_path()).is_ok(),
        _unreadable => false,
    };
    let _removed = (std::fs::remove_file(from.as_path()), std::fs::remove_file(to.as_path()));
    if cloned {
        return Some(root);
    }
    eprintln!("CLONE CONTROL NOT RUN: {name} (the fixture root cannot reflink a file)");
    return None;
}

/// What:
///  Wait until a new whole second has begun and let Git record the index again,
/// so no staged entry is as new as the index
///  file.
/// Why:
///   Git treats an entry modified in the same second as the index as racy and hashes it,
/// and the partial clone writes such
///  an entry rather than cloning it,
/// so a control of the object-name guard needs an index that is later than its
///  entries.
/// The refresh reports a path it could not refresh with a failure status,
/// which this control has already caused on
///  purpose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function settleTheIndex(repo: string): void;
/// ```
fn settle_the_index(repo: &Path) {
    let now: std::time::Duration = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("a clock after 1970");
    std::thread::sleep(std::time::Duration::from_secs(1) - std::time::Duration::from_nanos(u64::from(now.subsec_nanos())) + std::time::Duration::from_millis(20));
    let _refreshed = git_output(repo, &["update-index", "--really-refresh"]);
}

/// What:
///  A main checkout with a tracked plain file,
/// an executable,
/// a link,
/// a file whose bytes a control changes,
/// a staged path,
///  a deleted path and an untracked path,
/// and a `--no-checkout` worktree beside it.
/// Why:
///   These are the shapes the design answers one by one,
/// and every one of them is a state the machine's own main
///  checkout is in on most days.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Dirty = { repo: string; destination: string; admin: string; source: SnapshotSource };
/// ```
struct Dirty {
    /// The source checkout.
    repo: PathBuf,
    /// The new worktree's directory,
    /// which Git made with no checkout.
    destination: PathBuf,
    /// The new worktree's administrative directory.
    admin: PathBuf,
    /// What the source check proved about the source.
    source: SnapshotSource,
}

/// The fixture's file contents,
/// which every control asserts.
const KEPT: &[u8] = b"the commit's bytes for a path nobody changed\n";
/// The bytes the edited file's commit holds.
const COMMITTED_EDIT: &[u8] = b"the commit's bytes\n";
/// The bytes the person wrote into the edited file.
const PERSONAL_EDIT: &[u8] = b"a change nobody committed\n";
/// The bytes the staged path's commit holds.
const COMMITTED_STAGED: &[u8] = b"what the commit holds\n";
/// The bytes the person staged for the staged path,
/// as long as the commit's own,
/// so only the object name tells the two apart and no stat field can.
const STAGED_BYTES: &[u8] = b"what the person stold\n";
/// The bytes the deleted path's commit holds.
const COMMITTED_DELETED: &[u8] = b"a path the person deleted\n";

/// What:
///  A repository in the state the design calls dirty:
/// two paths the person changed,
/// one change staged,
/// one deletion,
/// and one untracked file.
/// Why:
///   A control of the partial clone needs every case the design answers in one source,
/// so the numbers it
///  asserts are the rung's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dirty = await dirtyRepository(root); // throws the reason Git refused a step
/// ```
fn dirty_repository(root: &Path) -> Dirty {
    let repo: PathBuf = std::fs::canonicalize(repository(root, "repo")).expect("canonical");
    tracked(repo.as_path(), "kept.txt", KEPT);
    tracked(repo.as_path(), "bin/tool.sh", KEPT);
    std::fs::set_permissions(repo.join("bin/tool.sh"), std::fs::Permissions::from_mode(0o755)).expect("executable");
    tracked(repo.as_path(), "edited.txt", COMMITTED_EDIT);
    tracked(repo.as_path(), "staged.txt", COMMITTED_STAGED);
    tracked(repo.as_path(), "deleted.txt", COMMITTED_DELETED);
    std::os::unix::fs::symlink("kept.txt", repo.join("link")).expect("link");
    crate::test_support::age(repo.join("link").as_path());
    commit_all(repo.as_path(), "the commit the person edits");
    std::fs::write(repo.join("edited.txt"), PERSONAL_EDIT).expect("the person's change");
    std::fs::write(repo.join("staged.txt"), STAGED_BYTES).expect("the staged change");
    git(repo.as_path(), &["add", "--", "staged.txt"]);
    settle_the_index(repo.as_path());
    std::fs::remove_file(repo.join("deleted.txt")).expect("the person's deletion");
    std::fs::write(repo.join("untracked.txt"), b"nobody tracks this\n").expect("the untracked file");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    let admin: PathBuf = repo.join(".git/worktrees/topic");
    let source: SnapshotSource = source_or_dirty(&git_context(), &main_identity(repo.as_path())).expect("a source with changes");
    assert!(source.dirty.is_some(), "the fixture's source has uncommitted changes");
    return Dirty { repo, destination, admin, source };
}

/// The target of the add:
/// the administrative directory Git made,
/// Git's pointer,
/// and the commit the new `HEAD` names.
fn target_of(dirty: &Dirty) -> Target {
    let hash_length: usize = dirty.source.clean.hash_length;
    let format = if hash_length == 32 { crate::repo_read_layout::ObjectFormat::Sha256 } else { crate::repo_read_layout::ObjectFormat::Sha1 };
    let commit: String = head_commit(dirty.admin.as_path(), dirty.source.common_dir.as_path(), format).expect("the new HEAD");
    let pointer: Vec<u8> = std::fs::read(dirty.destination.join(".git")).expect("Git's pointer");
    return Target { admin: dirty.admin.clone(), pointer, commit, differs: false, line: String::new() };
}

/// What:
///  Run the partial clone step,
/// prove the worktree it left,
/// and answer what it did.
/// Why:
///   Every control asks the same three things of the same fixture;
/// the step,
/// its proof,
/// and one line that
///  says what happened if either fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const part = await cloneAndProve(dirty); // throws the reason the worktree is not Git's
/// ```
/// The size rules that clone every settled path and decline nothing,
/// which is the rule this prototype applies unless a measurement sets one.
fn no_rule() -> SizeRules {
    return SizeRules { clone_min_bytes: 0, decline_below_total_bytes: 0 };
}

fn clone_and_prove(dirty: &Dirty) -> PartCloned {
    let target: Target = target_of(dirty);
    let held: std::fs::File = open_root(dirty.destination.as_path()).expect("the destination");
    let admin: std::fs::File = open_root(dirty.admin.as_path()).expect("the administrative directory");
    let part: PartCloned = clone_the_unchanged(&git_context(), &dirty.source, &target, dirty.destination.as_path(), &held, &admin, &no_rule()).expect("the partial clone");
    prove_the_worktree(&git_context(), &target, dirty.destination.as_path()).expect("the worktree Git's checkout makes");
    return part;
}

/// The partial clone clones the paths their entries prove,
/// writes the paths a change reaches,
/// leaves the untracked file behind,
/// and the worktree it leaves is clean at once.
#[test]
fn the_unchanged_paths_are_cloned_and_the_changed_ones_written() {
    let Some(root) = clone_capable_fixture("clone-partial") else { return };
    let dirty: Dirty = dirty_repository(root.as_path());
    let part: PartCloned = clone_and_prove(&dirty);
    eprintln!("DEBUG cloned={} written={} staged=[{}] edited=[{}]", part.cloned, part.written, String::from_utf8_lossy(&std::fs::read(dirty.destination.join("staged.txt")).unwrap_or_default()), String::from_utf8_lossy(&std::fs::read(dirty.destination.join("edited.txt")).unwrap_or_default()));
    // The two paths nothing changed, the executable and the link are clones; the three a change touches are Git's.
    assert_eq!(part.cloned, 3, "kept.txt, bin/tool.sh and link are cloned; edited.txt, staged.txt and deleted.txt are not");
    assert_eq!(part.written, 3);
    assert_eq!(git_text(dirty.destination.as_path(), &["status", "--porcelain=v1"]), "", "the first status of the worktree is clean");
    assert_eq!(std::fs::read(dirty.destination.join("edited.txt")).expect("the commit's bytes"), COMMITTED_EDIT);
    assert_eq!(std::fs::read(dirty.destination.join("staged.txt")).expect("the commit's bytes"), COMMITTED_STAGED);
    assert_eq!(std::fs::read(dirty.destination.join("deleted.txt")).expect("the commit's bytes"), COMMITTED_DELETED);
    assert!(!dirty.destination.join("untracked.txt").exists(), "an untracked file of the source reaches no worktree");
    assert_eq!(std::fs::read_link(dirty.destination.join("link")).expect("the link"), PathBuf::from("kept.txt"));
    let tool: std::fs::Metadata = std::fs::metadata(dirty.destination.join("bin/tool.sh")).expect("the executable");
    assert_eq!(tool.mode() & 0o7777, 0o755, "the cloned file keeps the mode Git's checkout writes");
    crate::test_support::remove(root.as_path());
}

/// The index the partial clone installs records the files the clones made,
/// so Git's first status of the new worktree costs no hashing of them.
#[test]
fn the_index_records_the_files_the_clones_made() {
    let Some(root) = clone_capable_fixture("clone-partial-index") else { return };
    let dirty: Dirty = dirty_repository(root.as_path());
    let _part: PartCloned = clone_and_prove(&dirty);
    let index: Vec<u8> = std::fs::read(dirty.admin.join("index")).expect("the new index");
    let entries = crate::worktree_clean::stored_stats(index.as_slice(), dirty.source.clean.hash_length).expect("the index's stat data");
    for name in ["kept.txt", "bin/tool.sh"] {
        let entry = entries.iter().find(|entry| { return entry.path.as_slice() == name.as_bytes(); }).expect("the entry of a cloned path");
        let metadata: std::fs::Metadata = std::fs::symlink_metadata(dirty.destination.join(name)).expect("the cloned file");
        assert_eq!(entry.stored.size, metadata.size() as u32, "{name}");
        assert_eq!(entry.stored.mtime_seconds, metadata.mtime() as u32, "{name}");
        assert_eq!(entry.stored.ino, metadata.ino() as u32, "{name}");
    }
    // A link's entry keeps the fields the source recorded, so no name another process may have replaced is certified:
    //  Git compares the target, which is what the source's own index install leaves in a whole clone.
    let link_entry = entries.iter().find(|entry| { return entry.path.as_slice() == b"link"; }).expect("the link's entry");
    let source_entries = crate::worktree_clean::stored_stats(dirty.source.clean.index_bytes.as_slice(), dirty.source.clean.hash_length).expect("the source's stat data");
    let source_link = source_entries.iter().find(|entry| { return entry.path.as_slice() == b"link"; }).expect("the source's link entry");
    assert_eq!(link_entry.stored.ino, source_link.stored.ino, "a link's entry keeps the source's inode");
    crate::test_support::remove(root.as_path());
}

/// A path written after its entry was recorded is written by Git:
/// the entry's stat data names bytes the file no longer holds.
#[test]
fn a_path_written_after_its_entry_was_recorded_is_written() {
    let Some(root) = clone_capable_fixture("clone-partial-changed") else { return };
    let dirty: Dirty = dirty_repository(root.as_path());
    std::fs::write(dirty.repo.join("kept.txt"), b"bytes the entry never recorded\n").expect("the write");
    let part: PartCloned = clone_and_prove(&dirty);
    assert_eq!(part.written, 4, "the written path joins the three the fixture's changes reach");
    assert_eq!(std::fs::read(dirty.destination.join("kept.txt")).expect("the commit's bytes"), KEPT);
    crate::test_support::remove(root.as_path());
}

/// An unchanged file below the size rule's cutoff is written by Git and not cloned,
/// and the worktree it leaves is still Git's own result.
#[test]
fn an_unchanged_file_below_the_cutoff_is_written() {
    let Some(root) = clone_capable_fixture("clone-partial-cutoff") else { return };
    let dirty: Dirty = dirty_repository(root.as_path());
    let target: Target = target_of(&dirty);
    let held: std::fs::File = open_root(dirty.destination.as_path()).expect("the destination");
    let admin: std::fs::File = open_root(dirty.admin.as_path()).expect("the administrative directory");
    // Every unchanged path of this fixture is shorter than a kilobyte, so the rule moves all four to Git.
    let rules: SizeRules = SizeRules { clone_min_bytes: 1024, decline_below_total_bytes: 0 };
    let part: PartCloned = clone_the_unchanged(&git_context(), &dirty.source, &target, dirty.destination.as_path(), &held, &admin, &rules).expect("the partial clone");
    assert_eq!((part.cloned, part.written), (0, 6), "every path is Git's when the rule takes no clone");
    prove_the_worktree(&git_context(), &target, dirty.destination.as_path()).expect("the worktree Git's checkout makes");
    assert_eq!(std::fs::read(dirty.destination.join("kept.txt")).expect("the commit's bytes"), KEPT);
    crate::test_support::remove(root.as_path());
}

/// A source whose tracked bytes are below the whole-tree rule's threshold is declined,
/// so the ladder falls back to Git's own checkout.
#[test]
fn a_tree_below_the_whole_tree_threshold_declines_the_rung() {
    let Some(root) = clone_capable_fixture("clone-partial-declined") else { return };
    let dirty: Dirty = dirty_repository(root.as_path());
    let target: Target = target_of(&dirty);
    let held: std::fs::File = open_root(dirty.destination.as_path()).expect("the destination");
    let admin: std::fs::File = open_root(dirty.admin.as_path()).expect("the administrative directory");
    let rules: SizeRules = SizeRules { clone_min_bytes: 0, decline_below_total_bytes: 1 << 40 };
    let reason: String = clone_the_unchanged(&git_context(), &dirty.source, &target, dirty.destination.as_path(), &held, &admin, &rules).expect_err("a declined rung");
    assert!(reason.contains("declines the rung"), "{reason}");
    assert!(!dirty.destination.join("kept.txt").exists(), "nothing was made before the rule was applied");
    crate::test_support::remove(root.as_path());
}

/// The flow runs where no file can be reflinked:
/// a clone of a file with no bytes needs none,
/// and the paths Git writes are the same.
#[test]
fn the_flow_runs_where_no_file_can_be_cloned() {
    let root: PathBuf = crate::test_support::fixture("clone-partial-empty");
    let repository: PathBuf = std::fs::canonicalize(crate::test_support::repository(root.as_path(), "repo")).expect("canonical");
    tracked(repository.as_path(), "empty.txt", b"");
    std::os::unix::fs::symlink("empty.txt", repository.join("link")).expect("link");
    crate::test_support::age(repository.join("link").as_path());
    tracked(repository.as_path(), "edited.txt", COMMITTED_EDIT);
    commit_all(repository.as_path(), "the commit the person edits");
    std::fs::write(repository.join("edited.txt"), PERSONAL_EDIT).expect("the person's change");
    git(repository.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    let admin: PathBuf = repository.join(".git/worktrees/topic");
    let source: SnapshotSource = source_or_dirty(&git_context(), &main_identity(repository.as_path())).expect("a source with changes");
    let dirty: Dirty = Dirty { repo: repository, destination, admin, source };
    let part: PartCloned = clone_and_prove(&dirty);
    assert_eq!((part.cloned, part.written), (2, 1), "the empty file and the link are clones and the changed file is Git's");
    assert_eq!(git_text(dirty.destination.as_path(), &["status", "--porcelain=v1"]), "");
    crate::test_support::remove(root.as_path());
}

/// What:
///  A commit above the source's that adds `later.txt`,
/// made from the source's own `HEAD` tree through a private index,
///  so the new worktree's `HEAD` differs from the tree the rung builds its index from.
/// Why:
///   The routing and the differing-`HEAD` path were only exercised by the timing harness,
/// and a control that builds the
///  index from the source's tree must show that the ladder's `read-tree -m -u` brings the new commit in.
fn later_commit(repo: &Path, index: &Path) -> String {
    let program: PathBuf = crate::test_support::git_context().real_git;
    let environment: Vec<(OsString, OsString)> = vec![
        (OsString::from("PATH"), OsString::from("/usr/bin:/bin")),
        (OsString::from("GIT_CONFIG_NOSYSTEM"), OsString::from("1")),
        (OsString::from("GIT_AUTHOR_NAME"), OsString::from("Fixture")),
        (OsString::from("GIT_AUTHOR_EMAIL"), OsString::from("fixture@example.invalid")),
        (OsString::from("GIT_COMMITTER_NAME"), OsString::from("Fixture")),
        (OsString::from("GIT_COMMITTER_EMAIL"), OsString::from("fixture@example.invalid")),
        (OsString::from("GIT_INDEX_FILE"), index.as_os_str().to_os_string()),
    ];
    let run = |words: &[&str], input: Option<&[u8]>| -> String {
        let arguments: Vec<OsString> = words.iter().map(|word: &&str| { return OsString::from(*word); }).collect();
        let answer = crate::test_support::real_git_within(program.as_path(), Some(repo), environment.as_slice(), arguments.as_slice(), input, crate::test_support::FIXTURE_GIT_BOUND);
        assert!(answer.output.status.success(), "git {words:?} failed: {}", String::from_utf8_lossy(answer.output.stderr.as_slice()));
        return String::from_utf8(answer.output.stdout).expect("UTF-8 Git output").trim().to_string();
    };
    let blob: String = run(&["hash-object", "-w", "--stdin"], Some(b"the later commit's bytes\n"));
    run(&["read-tree", "HEAD"], None);
    run(&["update-index", "--add", "--cacheinfo", format!("100644,{blob},later.txt").as_str()], None);
    let tree: String = run(&["write-tree"], None);
    return run(&["commit-tree", tree.as_str(), "-p", "HEAD", "-m", "later"], None);
}

/// A staged path among the clones is named and refused,
/// a staged path the clones do not take is not,
/// and a listing that names nothing refuses nothing.
#[test]
fn a_staged_path_among_the_clones_is_refused() {
    let entries: Vec<crate::worktree_clean::StatEntry> = ["kept.txt", "staged.txt"]
        .iter()
        .map(|path: &&str| {
            return crate::worktree_clean::StatEntry { path: path.as_bytes().to_vec(), flags: 0, mode: 0o100_644, stored: crate::subvolume_index::StoredStat { ctime_seconds: 0, ctime_nanos: 0, mtime_seconds: 0, mtime_nanos: 0, ino: 0, uid: 0, gid: 0, size: 0 } };
        })
        .collect();
    let staged: Vec<Vec<u8>> = vec![b"staged.txt".to_vec()];
    let refusal: String = a_staged_path_among_the_clones(staged.as_slice(), entries.as_slice(), &[0, 1]).expect("the staged path among the clones");
    assert!(refusal.contains("staged.txt"), "{refusal}");
    assert!(a_staged_path_among_the_clones(staged.as_slice(), entries.as_slice(), &[0]).is_none(), "a staged path the clones do not take is no refusal");
    assert!(a_staged_path_among_the_clones(&[b"other.txt".to_vec()], entries.as_slice(), &[0, 1]).is_none(), "a staged path the tree does not name is no refusal");
    assert!(a_staged_path_among_the_clones(&[], entries.as_slice(), &[0, 1]).is_none(), "a listing with nothing in it refuses nothing");
}

/// A source with changes whose new `HEAD` holds a later commit is cloned from the source's own tree,
/// brought to the new commit by the ladder's `read-tree -m -u`,
/// and proven against the new `HEAD`.
#[test]
fn a_new_head_that_differs_is_brought_in_after_the_clones() {
    let Some(root) = clone_capable_fixture("clone-partial-differs") else { return };
    let dirty: Dirty = dirty_repository(root.as_path());
    let commit: String = later_commit(dirty.repo.as_path(), root.join("later-index").as_path());
    git(dirty.repo.as_path(), &["update-ref", "refs/heads/topic", commit.as_str()]);
    let mut target: Target = target_of(&dirty);
    target.commit = commit;
    target.differs = true;
    let held: std::fs::File = open_root(dirty.destination.as_path()).expect("the destination");
    let admin: std::fs::File = open_root(dirty.admin.as_path()).expect("the administrative directory");
    let part: PartCloned = clone_the_unchanged(&git_context(), &dirty.source, &target, dirty.destination.as_path(), &held, &admin, &no_rule()).expect("the partial clone");
    assert_eq!(part.cloned, 3, "the paths the source's changes did not touch are clones");
    let program: PathBuf = crate::test_support::git_context().real_git;
    let mut runner = |step: &GitStep<'_>| -> Result<GitStatus, crate::worktree_copy_error::WorktreeCopyError> {
        let GitStep::Other { arguments, environment } = step else { return Err(crate::worktree_copy_error::WorktreeCopyError(String::from("the tree difference runs its own step"))); };
        let answer = crate::test_support::real_git_within(program.as_path(), None, environment, arguments, None, crate::test_support::FIXTURE_GIT_BOUND);
        return Ok(GitStatus { code: answer.output.status.code() });
    };
    apply_difference(&dirty.source, &target, dirty.destination.as_path(), &mut runner).expect("the tree difference applied");
    prove_the_worktree(&git_context(), &target, dirty.destination.as_path()).expect("the worktree Git's checkout of the new commit makes");
    assert_eq!(std::fs::read(dirty.destination.join("later.txt")).expect("the new commit's file"), b"the later commit's bytes\n");
    assert_eq!(std::fs::read(dirty.destination.join("kept.txt")).expect("the commit's bytes"), KEPT, "a cloned path the new commit does not change keeps its bytes");
    assert_eq!(git_text(dirty.destination.as_path(), &["status", "--porcelain=v1"]), "");
    crate::test_support::remove(root.as_path());
}
