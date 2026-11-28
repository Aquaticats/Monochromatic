//! Controls for the clone rung of a source with changes,
//! reached through the conversion fence:
//! the paths their entries prove are cloned,
//! the paths a change reaches and the paths a conversion rule reaches are written by Git,
//! the new index describes the worktree Git's own checkout would make,
//! and it takes the worktree's name only after that write.

use super::*;
use crate::fs_beneath::open_root;
use crate::fs_clone_exchange::clone_file;
use crate::test_support::{
    git,
    git_context,
    git_output,
    git_text,
    repository,
};
use crate::worktree_clean::head_commit;
use crate::worktree_clone_partial_staged::a_staged_path_among_the_clones;
use crate::worktree_clone_slices::{
    A_HOST_THAT_BOUNDS_NO_WORKER,
    clone_worker_count,
};
use crate::worktree_copy::{
    GitStatus,
    GitStep,
};
use crate::worktree_ladder_target::{
    Target,
    apply_difference_with_index,
};
use crate::worktree_ladder_test_support::{
    commit_all,
    main_identity,
    tracked,
};
use crate::worktree_snapshot_source::{
    SnapshotSource,
    source_or_dirty,
};
use std::ffi::OsString;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The fixture root when a file on it can be reflinked,
/// or nothing after saying that this control did not run.
/// Why:
///   A clone of a file with bytes needs `FICLONE`,
/// which the gate image's fixture root (a tmpfs) does not answer,
/// and a control that passed there would claim coverage it has not got.
/// A root that cannot clone says so once,
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
    let opened = (
        std::fs::File::open(from.as_path()),
        std::fs::OpenOptions::new().write(true).open(to.as_path()),
    );
    let cloned: bool = match opened {
        (Ok(source), Ok(destination)) => clone_file(&source, &destination, to.as_path()).is_ok(),
        _unreadable => false,
    };
    let _removed = (
        std::fs::remove_file(from.as_path()),
        std::fs::remove_file(to.as_path()),
    );
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
    let now: std::time::Duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("a clock after 1970");
    std::thread::sleep(
        std::time::Duration::from_secs(1)
            - std::time::Duration::from_nanos(u64::from(now.subsec_nanos()))
            + std::time::Duration::from_millis(20),
    );
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
/// The unchanged tracked paths the fixture holds beside the paths its changes reach.
///
/// Why:
///  The changed-file cut is a quarter of the tracked paths,
///  and the partial clone takes the ladder's refusal path when the paths Git must write exceed it
///  (the integration pass's finding M1).
///  The paths of this fixture a change reaches are three or four,
///  so ten more unchanged paths are what keeps the cut above them and the rung reached.
const UNCHANGED_PATHS: usize = 10;

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
    std::fs::set_permissions(
        repo.join("bin/tool.sh"),
        std::fs::Permissions::from_mode(0o755),
    )
    .expect("executable");
    tracked(repo.as_path(), "edited.txt", COMMITTED_EDIT);
    tracked(repo.as_path(), "staged.txt", COMMITTED_STAGED);
    tracked(repo.as_path(), "deleted.txt", COMMITTED_DELETED);
    for index in 0..UNCHANGED_PATHS {
        tracked(
            repo.as_path(),
            format!("kept/unchanged-{index}.txt").as_str(),
            KEPT,
        );
    }
    std::os::unix::fs::symlink("kept.txt", repo.join("link")).expect("link");
    crate::test_support::age(repo.join("link").as_path());
    commit_all(repo.as_path(), "the commit the person edits");
    std::fs::write(repo.join("edited.txt"), PERSONAL_EDIT).expect("the person's change");
    std::fs::write(repo.join("staged.txt"), STAGED_BYTES).expect("the staged change");
    git(repo.as_path(), &["add", "--", "staged.txt"]);
    settle_the_index(repo.as_path());
    std::fs::remove_file(repo.join("deleted.txt")).expect("the person's deletion");
    std::fs::write(repo.join("untracked.txt"), b"nobody tracks this\n")
        .expect("the untracked file");
    git(
        repo.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--no-checkout",
            "-b",
            "topic",
            "../topic",
        ],
    );
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    let admin: PathBuf = repo.join(".git/worktrees/topic");
    let source: SnapshotSource = source_or_dirty(&git_context(), &main_identity(repo.as_path()))
        .expect("a source with changes");
    assert!(
        source.dirty.is_some(),
        "the fixture's source has uncommitted changes"
    );
    return Dirty {
        repo,
        destination,
        admin,
        source,
    };
}

/// The target of the add:
/// the administrative directory Git made,
/// Git's pointer,
/// and the commit the new `HEAD` names.
fn target_of(dirty: &Dirty) -> Target {
    let hash_length: usize = dirty.source.clean.hash_length;
    let format = if hash_length == 32 {
        crate::repo_read_layout::ObjectFormat::Sha256
    } else {
        crate::repo_read_layout::ObjectFormat::Sha1
    };
    let commit: String = head_commit(
        dirty.admin.as_path(),
        dirty.source.common_dir.as_path(),
        format,
    )
    .expect("the new HEAD");
    let pointer: Vec<u8> = std::fs::read(dirty.destination.join(".git")).expect("Git's pointer");
    return Target {
        admin: dirty.admin.clone(),
        index: None,
        pointer,
        commit,
        differs: false,
        line: String::new(),
    };
}

/// What:
///  A runner that runs the ladder's own steps with real Git in an isolated environment.
/// Why:
///   The tree difference is one of the ladder's Git runs,
/// and a control that stood in for it would not show the worktree
///  Git's own `read-tree` leaves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runner = recordingRunner(); // one real Git run per step
/// ```
fn a_real_runner(
    program: &Path
) -> impl FnMut(&GitStep<'_>) -> Result<GitStatus, crate::worktree_copy_error::WorktreeCopyError> + '_
{
    return move |step: &GitStep<'_>| {
        let GitStep::Other {
            arguments,
            environment,
        } = step
        else {
            return Err(crate::worktree_copy_error::WorktreeCopyError(String::from(
                "the tree difference runs its own step",
            )));
        };
        let answer = crate::test_support::real_git_within(
            program,
            None,
            environment,
            arguments,
            None,
            crate::test_support::FIXTURE_GIT_BOUND,
        );
        return Ok(GitStatus {
            code: answer.output.status.code(),
        });
    };
}

/// What:
///  Run the partial clone through the fence's own order:
/// the copies with the fence beside them,
/// the index under its waiting name,
/// the tree difference,
/// Git's write of the paths the fence leaves it,
/// and the index taking the worktree's name.
/// Why:
///   This is the serving step's order,
/// and the waiting name is part of what the control proves:
/// the worktree holds no index
///  while Git still has paths to write,
/// so nothing can vouch for an entry Git has not written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const part = cloneThroughTheFence(dirty); // the worktree is proven before this returns
/// ```
fn clone_through_the_fence(dirty: &Dirty) -> PartCloned {
    let context: crate::transaction_git::GitContext = git_context();
    let target: Target = target_of(dirty);
    let held_fence: AttributeFence =
        crate::worktree_clone_conversion_fence::fence_the_attributes(&dirty.source, &context)
            .expect("the attributes");
    let destination: std::fs::File =
        open_root(dirty.destination.as_path()).expect("the destination");
    let admin: std::fs::File =
        open_root(dirty.admin.as_path()).expect("the administrative directory");
    let cloned = clone_the_unchanged(PartialClone {
        context: &context,
        source: &dirty.source,
        entries: None,
        destination: &destination,
        admin: &admin,
        admin_path: dirty.admin.as_path(),
        fence: held_fence,
        host: A_HOST_THAT_BOUNDS_NO_WORKER,
    });
    let (part, decided, pending): (PartCloned, Fence, PendingIndex) =
        cloned.expect("the partial clone");
    assert!(
        !dirty.admin.join("index").exists(),
        "the worktree has no index while Git still has paths to write"
    );
    assert!(
        pending.path_in().exists(),
        "the index waits under the fence's own name"
    );
    let program: PathBuf = context.real_git.clone();
    let mut runner = a_real_runner(program.as_path());
    apply_difference_with_index(
        (
            &dirty.source,
            &target,
            dirty.destination.as_path(),
            &mut runner,
        ),
        Some(pending.path_in()),
    )
    .expect("the tree difference applied");
    crate::worktree_clone_conversion_fence::checkout_index_through_the_index_at(
        (dirty.admin.as_path(), dirty.destination.as_path(), &context),
        pending.path_in(),
        decided.git_writes.as_slice(),
        dirty.source.clean.hash_length,
    )
    .expect("Git writes the paths the fence leaves it");
    assert!(
        !dirty.admin.join("index").exists(),
        "the write records its stat data in the waiting index, not in the worktree's"
    );
    pending
        .publish(&admin)
        .expect("the index takes the worktree's name");
    prove_the_worktree(
        &context,
        dirty.destination.as_path(),
        dirty.admin.as_path(),
        target.commit.as_str(),
    )
    .expect("the worktree Git's checkout makes");
    return part;
}

/// The partial clone clones the paths their entries prove,
/// writes the paths a change reaches,
/// leaves the untracked file behind,
/// and the worktree it leaves is clean at once.
#[test]
fn the_unchanged_paths_are_cloned_and_the_changed_ones_written() {
    let Some(root) = clone_capable_fixture("clone-partial") else {
        return;
    };
    let dirty: Dirty = dirty_repository(root.as_path());
    let part: PartCloned = clone_through_the_fence(&dirty);
    // The two paths nothing changed, the executable and the link are clones; the three a change touches are Git's.
    assert_eq!(
        part.cloned,
        3 + UNCHANGED_PATHS,
        "kept.txt, bin/tool.sh, link and the unchanged paths are cloned; edited.txt, staged.txt and deleted.txt are not"
    );
    assert_eq!(part.written, 3);
    assert_eq!(
        git_text(dirty.destination.as_path(), &["status", "--porcelain=v1"]),
        "",
        "the first status of the worktree is clean"
    );
    assert_eq!(
        std::fs::read(dirty.destination.join("edited.txt")).expect("the commit's bytes"),
        COMMITTED_EDIT
    );
    assert_eq!(
        std::fs::read(dirty.destination.join("staged.txt")).expect("the commit's bytes"),
        COMMITTED_STAGED
    );
    assert_eq!(
        std::fs::read(dirty.destination.join("deleted.txt")).expect("the commit's bytes"),
        COMMITTED_DELETED
    );
    assert!(
        !dirty.destination.join("untracked.txt").exists(),
        "an untracked file of the source reaches no worktree"
    );
    assert_eq!(
        std::fs::read_link(dirty.destination.join("link")).expect("the link"),
        PathBuf::from("kept.txt")
    );
    let tool: std::fs::Metadata =
        std::fs::metadata(dirty.destination.join("bin/tool.sh")).expect("the executable");
    assert_eq!(
        tool.mode() & 0o7777,
        0o755,
        "the cloned file keeps the mode Git's checkout writes"
    );
    crate::test_support::remove(root.as_path());
}

/// The index the partial clone installs records the files the clones made,
/// so Git's first status of the new worktree costs no hashing of them.
#[test]
fn the_index_records_the_files_the_clones_made() {
    let Some(root) = clone_capable_fixture("clone-partial-index") else {
        return;
    };
    let dirty: Dirty = dirty_repository(root.as_path());
    let _part: PartCloned = clone_through_the_fence(&dirty);
    let index: Vec<u8> = std::fs::read(dirty.admin.join("index")).expect("the new index");
    let entries =
        crate::worktree_clean::stored_stats(index.as_slice(), dirty.source.clean.hash_length)
            .expect("the index's stat data");
    for name in ["kept.txt", "bin/tool.sh"] {
        let entry = entries
            .iter()
            .find(|entry| {
                return entry.path.as_slice() == name.as_bytes();
            })
            .expect("the entry of a cloned path");
        let metadata: std::fs::Metadata =
            std::fs::symlink_metadata(dirty.destination.join(name)).expect("the cloned file");
        assert_eq!(entry.stored.size, metadata.size() as u32, "{name}");
        assert_eq!(
            entry.stored.mtime_seconds,
            metadata.mtime() as u32,
            "{name}"
        );
        assert_eq!(entry.stored.ino, metadata.ino() as u32, "{name}");
    }
    // A link's entry keeps the fields the source recorded, so no name another process may have replaced is certified:
    //  Git compares the target, which is what the source's own index install leaves in a whole clone.
    let link_entry = entries
        .iter()
        .find(|entry| {
            return entry.path.as_slice() == b"link";
        })
        .expect("the link's entry");
    let source_entries = crate::worktree_clean::stored_stats(
        dirty.source.clean.index_bytes.as_slice(),
        dirty.source.clean.hash_length,
    )
    .expect("the source's stat data");
    let source_link = source_entries
        .iter()
        .find(|entry| {
            return entry.path.as_slice() == b"link";
        })
        .expect("the source's link entry");
    assert_eq!(
        link_entry.stored.ino, source_link.stored.ino,
        "a link's entry keeps the source's inode"
    );
    crate::test_support::remove(root.as_path());
}

/// A path a conversion rule of its own reaches is written by Git,
/// even though its file still equals its entry,
/// so the worktree holds the bytes Git's checkout writes and not the source's.
#[test]
fn a_path_a_conversion_rule_reaches_is_written_by_git() {
    let Some(root) = clone_capable_fixture("clone-partial-converted") else {
        return;
    };
    let repo: PathBuf =
        std::fs::canonicalize(repository(root.as_path(), "repo")).expect("canonical");
    tracked(repo.as_path(), ".gitattributes", b"*.txt text eol=crlf\n");
    tracked(repo.as_path(), "notes.txt", b"line one\nline two\n");
    tracked(repo.as_path(), "kept.md", KEPT);
    // Unchanged paths, so the changed-file cut stays above the path this control changes. Their names no conversion rule
    // of the fixture matches, so the fence hands Git only the paths the classes name.
    for index in 0..6 {
        tracked(
            repo.as_path(),
            format!("kept/unchanged-{index}.dat").as_str(),
            KEPT,
        );
    }
    commit_all(repo.as_path(), "the commit the person edits");
    std::fs::write(repo.join("kept.md"), PERSONAL_EDIT).expect("the person's change");
    git(
        repo.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--no-checkout",
            "-b",
            "topic",
            "../topic",
        ],
    );
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    let admin: PathBuf = repo.join(".git/worktrees/topic");
    let source: SnapshotSource = source_or_dirty(&git_context(), &main_identity(repo.as_path()))
        .expect("a source with changes");
    let dirty: Dirty = Dirty {
        repo,
        destination,
        admin,
        source,
    };
    let part: PartCloned = clone_through_the_fence(&dirty);
    // The copies take every path the split settles, the rule's own path among them, and Git then overwrites it, which
    // is the whole rung's shape: the fence decides beside the copies and cannot unmake one of them.
    assert_eq!(
        (part.cloned, part.written),
        (2 + 6, 2),
        "the .gitattributes, notes.txt and the unchanged paths are copied, kept.md is only Git's, and Git writes notes.txt over its copy"
    );
    let served: Vec<u8> =
        std::fs::read(dirty.destination.join("notes.txt")).expect("the served path");
    assert_eq!(
        served, b"line one\r\nline two\r\n",
        "the served path holds the carriage returns Git's checkout writes"
    );
    assert_ne!(
        served,
        std::fs::read(dirty.repo.join("notes.txt")).expect("the source's file"),
        "the source's file holds the blob, so a clone of it would fail here"
    );
    crate::test_support::remove(root.as_path());
}

/// A path written after its entry was recorded is written by Git:
/// the entry's stat data names bytes the file no longer holds.
#[test]
fn a_path_written_after_its_entry_was_recorded_is_written() {
    let Some(root) = clone_capable_fixture("clone-partial-changed") else {
        return;
    };
    let dirty: Dirty = dirty_repository(root.as_path());
    std::fs::write(
        dirty.repo.join("kept.txt"),
        b"bytes the entry never recorded\n",
    )
    .expect("the write");
    let part: PartCloned = clone_through_the_fence(&dirty);
    assert_eq!(
        part.written, 4,
        "the written path joins the three the fixture's changes reach"
    );
    assert_eq!(
        std::fs::read(dirty.destination.join("kept.txt")).expect("the commit's bytes"),
        KEPT
    );
    crate::test_support::remove(root.as_path());
}

/// The flow runs where no file can be reflinked:
/// a clone of a file with no bytes needs none,
/// and the paths Git writes are the same.
#[test]
fn the_flow_runs_where_no_file_can_be_cloned() {
    let root: PathBuf = crate::test_support::fixture("clone-partial-empty");
    let repository: PathBuf =
        std::fs::canonicalize(crate::test_support::repository(root.as_path(), "repo"))
            .expect("canonical");
    tracked(repository.as_path(), "empty.txt", b"");
    std::os::unix::fs::symlink("empty.txt", repository.join("link")).expect("link");
    crate::test_support::age(repository.join("link").as_path());
    tracked(repository.as_path(), "edited.txt", COMMITTED_EDIT);
    // Unchanged paths, so the changed-file cut stays above the path this control changes. They are empty, like the
    // fixture's own file, because the point of this control is a clone that needs no FICLONE at all: a root that cannot
    // reflink a file with bytes must still serve it.
    for index in 0..6 {
        tracked(
            repository.as_path(),
            format!("kept/unchanged-{index}.txt").as_str(),
            b"",
        );
    }
    commit_all(repository.as_path(), "the commit the person edits");
    std::fs::write(repository.join("edited.txt"), PERSONAL_EDIT).expect("the person's change");
    git(
        repository.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--no-checkout",
            "-b",
            "topic",
            "../topic",
        ],
    );
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    let admin: PathBuf = repository.join(".git/worktrees/topic");
    let source: SnapshotSource =
        source_or_dirty(&git_context(), &main_identity(repository.as_path()))
            .expect("a source with changes");
    let dirty: Dirty = Dirty {
        repo: repository,
        destination,
        admin,
        source,
    };
    let part: PartCloned = clone_through_the_fence(&dirty);
    assert_eq!(
        (part.cloned, part.written),
        (2 + 6, 1),
        "the empty file, the link and the unchanged paths are clones and the changed file is Git's"
    );
    assert_eq!(
        git_text(dirty.destination.as_path(), &["status", "--porcelain=v1"]),
        ""
    );
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
fn later_commit(
    repo: &Path,
    index: &Path,
) -> String {
    let program: PathBuf = crate::test_support::git_context().real_git;
    let environment: Vec<(OsString, OsString)> = vec![
        (OsString::from("PATH"), OsString::from("/usr/bin:/bin")),
        (OsString::from("GIT_CONFIG_NOSYSTEM"), OsString::from("1")),
        (OsString::from("GIT_AUTHOR_NAME"), OsString::from("Fixture")),
        (
            OsString::from("GIT_AUTHOR_EMAIL"),
            OsString::from("fixture@example.invalid"),
        ),
        (
            OsString::from("GIT_COMMITTER_NAME"),
            OsString::from("Fixture"),
        ),
        (
            OsString::from("GIT_COMMITTER_EMAIL"),
            OsString::from("fixture@example.invalid"),
        ),
        (
            OsString::from("GIT_INDEX_FILE"),
            index.as_os_str().to_os_string(),
        ),
    ];
    let run = |words: &[&str], input: Option<&[u8]>| -> String {
        let arguments: Vec<OsString> = words
            .iter()
            .map(|word: &&str| {
                return OsString::from(*word);
            })
            .collect();
        let answer = crate::test_support::real_git_within(
            program.as_path(),
            Some(repo),
            environment.as_slice(),
            arguments.as_slice(),
            input,
            crate::test_support::FIXTURE_GIT_BOUND,
        );
        assert!(
            answer.output.status.success(),
            "git {words:?} failed: {}",
            String::from_utf8_lossy(answer.output.stderr.as_slice())
        );
        return String::from_utf8(answer.output.stdout)
            .expect("UTF-8 Git output")
            .trim()
            .to_string();
    };
    let blob: String = run(
        &["hash-object", "-w", "--stdin"],
        Some(b"the later commit's bytes\n"),
    );
    run(&["read-tree", "HEAD"], None);
    run(
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            format!("100644,{blob},later.txt").as_str(),
        ],
        None,
    );
    let tree: String = run(&["write-tree"], None);
    return run(
        &["commit-tree", tree.as_str(), "-p", "HEAD", "-m", "later"],
        None,
    );
}

/// A staged path among the clones is named and refused,
/// a staged path the clones do not take is not,
/// and a listing that names nothing refuses nothing.
#[test]
fn a_staged_path_among_the_clones_is_refused() {
    let entries: Vec<crate::worktree_clean::StatEntry> = ["kept.txt", "staged.txt"]
        .iter()
        .map(|path: &&str| {
            return crate::worktree_clean::StatEntry {
                path: path.as_bytes().to_vec(),
                flags: 0,
                mode: 0o100_644,
                stored: crate::subvolume_index::StoredStat {
                    ctime_seconds: 0,
                    ctime_nanos: 0,
                    mtime_seconds: 0,
                    mtime_nanos: 0,
                    ino: 0,
                    uid: 0,
                    gid: 0,
                    size: 0,
                },
            };
        })
        .collect();
    let staged: Vec<Vec<u8>> = vec![b"staged.txt".to_vec()];
    let refusal: String =
        a_staged_path_among_the_clones(staged.as_slice(), entries.as_slice(), &[0, 1])
            .expect("the staged path among the clones");
    assert!(refusal.contains("staged.txt"), "{refusal}");
    assert!(
        a_staged_path_among_the_clones(staged.as_slice(), entries.as_slice(), &[0]).is_none(),
        "a staged path the clones do not take is no refusal"
    );
    assert!(
        a_staged_path_among_the_clones(&[b"other.txt".to_vec()], entries.as_slice(), &[0, 1])
            .is_none(),
        "a staged path the tree does not name is no refusal"
    );
    assert!(
        a_staged_path_among_the_clones(&[], entries.as_slice(), &[0, 1]).is_none(),
        "a listing with nothing in it refuses nothing"
    );
}

/// What:
///  A path the source staged after the source check is refused,
/// although the index snapshot the split reads still proves it.
/// Why:
///   The split judges the index the source check copied,
/// and the source's index can move before the copies,
/// as another session's `git add` moves this machine's main checkout;
/// the live listing of the paths staged against the source's `HEAD` is the only step that sees it.
/// The file itself is untouched,
/// so only that listing can refuse it,
/// and this control goes red when the listing's answer is computed and ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const source = checkSource(repo); stageAnotherBlob(repo, 'staged-later.txt');
/// expect(() => cloneTheUnchanged({ source, ... })).toThrow(/staged in the source and would have been cloned/);
/// ```
#[test]
fn a_path_staged_after_the_source_check_is_refused() {
    let Some(root) = clone_capable_fixture("clone-partial-staged-after-the-check") else {
        return;
    };
    let repo: PathBuf =
        std::fs::canonicalize(repository(root.as_path(), "repo")).expect("canonical");
    tracked(repo.as_path(), "kept.txt", KEPT);
    tracked(repo.as_path(), "staged-later.txt", COMMITTED_STAGED);
    // Unchanged paths, so the changed-file cut stays above the one path this control changes and the staged listing is
    // the refusal this control is about.
    for index in 0..6 {
        tracked(
            repo.as_path(),
            format!("kept/unchanged-{index}.txt").as_str(),
            KEPT,
        );
    }
    commit_all(repo.as_path(), "the commit the source check reads");
    std::fs::write(repo.join("kept.txt"), PERSONAL_EDIT).expect("the person's change");
    git(
        repo.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--no-checkout",
            "-b",
            "topic",
            "../topic",
        ],
    );
    settle_the_index(repo.as_path());
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    let admin: PathBuf = repo.join(".git/worktrees/topic");
    let context: crate::transaction_git::GitContext = git_context();
    let source: SnapshotSource =
        source_or_dirty(&context, &main_identity(repo.as_path())).expect("a source with changes");
    assert!(
        source.dirty.is_some(),
        "the fixture's source has uncommitted changes"
    );
    // Another blob goes into the index while the worktree file keeps the commit's bytes and its own times, so the
    // snapshot's entry still proves the file and only Git's live listing knows the path is staged.
    let bytes: PathBuf = root.join("staged-after-the-check");
    std::fs::write(bytes.as_path(), b"what the person stages after the check\n")
        .expect("the staged bytes");
    let blob: String = git_text(
        repo.as_path(),
        &["hash-object", "-w", bytes.to_str().expect("a UTF-8 path")],
    );
    let cacheinfo: String = format!("100644,{blob},staged-later.txt");
    git_text(
        repo.as_path(),
        &["update-index", "--add", "--cacheinfo", cacheinfo.as_str()],
    );
    let held_fence: AttributeFence =
        crate::worktree_clone_conversion_fence::fence_the_attributes(&source, &context)
            .expect("the attributes");
    let destination_file: std::fs::File =
        open_root(destination.as_path()).expect("the destination");
    let admin_file: std::fs::File =
        open_root(admin.as_path()).expect("the administrative directory");
    let cloned = clone_the_unchanged(PartialClone {
        context: &context,
        source: &source,
        entries: None,
        destination: &destination_file,
        admin: &admin_file,
        admin_path: admin.as_path(),
        fence: held_fence,
        host: A_HOST_THAT_BOUNDS_NO_WORKER,
    });
    let refusal: String = match cloned {
        Err(reason) => reason,
        Ok(_) => panic!("a path staged after the source check is refused rather than cloned"),
    };
    assert!(refusal.contains("staged-later.txt"), "{refusal}");
    assert!(
        refusal.contains("is staged in the source and would have been cloned"),
        "{refusal}"
    );
    crate::test_support::remove(root.as_path());
}

/// What:
///  A split that leaves Git more paths to write than the changed-file cut ends the rung before any copy.
/// Why:
///   The ladder's cut counts `git status` entries,
/// and a tracked file whose recorded times are stale but whose bytes are unchanged is no status entry,
/// although the split hands it to Git and Git pays a write for it:
/// the integration pass measured 20,000 such paths at 1.39 times `main` (its finding M1).
/// The refusal is the ladder's own path,
/// so the add falls to Git's own checkout at about `main`'s cost.
/// This control goes red when the guard is removed,
/// because the rung then copies the proved paths and writes the stale ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// touchMostTrackedFiles(repo); // the index is not refreshed
/// expect(() => cloneTheUnchanged({ source, ... })).toThrow(/file times recorded for/);
/// ```
#[test]
fn a_split_that_leaves_git_more_paths_than_the_cut_is_refused_before_any_copy() {
    let Some(root) = clone_capable_fixture("clone-partial-stale-past-the-cut") else {
        return;
    };
    let repo: PathBuf =
        std::fs::canonicalize(repository(root.as_path(), "repo")).expect("canonical");
    let files: usize = 40;
    for index in 0..files {
        tracked(repo.as_path(), format!("plain-{index}.txt").as_str(), KEPT);
    }
    commit_all(repo.as_path(), "the commit the person's times fall behind");
    // A file nobody tracks makes the source one with changes, so the partial clone is the rung the ladder chose.
    std::fs::write(repo.join("untracked.txt"), b"nobody tracks this\n")
        .expect("the untracked file");
    let cut: usize = crate::worktree_clone_rung_limits::the_changed_file_cut(files);
    // Every file but a few gets new times and the same bytes; the index is not refreshed, so Git's status names none of
    // them and only the split's own comparison does.
    let stale: usize = cut * 3 + 6;
    let now: std::time::SystemTime = std::time::SystemTime::now();
    for index in 0..stale {
        let path: PathBuf = repo.join(format!("plain-{index}.txt").as_str());
        std::fs::File::options()
            .append(true)
            .open(path.as_path())
            .expect("the file")
            .set_modified(now)
            .expect("new times");
    }
    git(
        repo.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--no-checkout",
            "-b",
            "topic",
            "../topic",
        ],
    );
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    let admin: PathBuf = repo.join(".git/worktrees/topic");
    let context: crate::transaction_git::GitContext = git_context();
    let source: SnapshotSource =
        source_or_dirty(&context, &main_identity(repo.as_path())).expect("a source with changes");
    assert!(
        source.dirty.is_some(),
        "the untracked file makes the source one with changes"
    );
    assert_eq!(
        source.dirty.as_ref().expect("the changes").tracked,
        0,
        "a stale entry is no status change, which is the shape"
    );
    assert!(
        stale > cut,
        "the stale paths are more than the cut: {stale} of {files}, cut {cut}"
    );
    let held_fence: AttributeFence =
        crate::worktree_clone_conversion_fence::fence_the_attributes(&source, &context)
            .expect("the attributes");
    let destination_file: std::fs::File =
        open_root(destination.as_path()).expect("the destination");
    let admin_file: std::fs::File =
        open_root(admin.as_path()).expect("the administrative directory");
    let cloned = clone_the_unchanged(PartialClone {
        context: &context,
        source: &source,
        entries: None,
        destination: &destination_file,
        admin: &admin_file,
        admin_path: admin.as_path(),
        fence: held_fence,
        host: A_HOST_THAT_BOUNDS_NO_WORKER,
    });
    let refusal: String = match cloned {
        Err(reason) => reason,
        Ok(_) => {
            panic!("a split that leaves Git more paths than the cut is refused rather than cloned")
        }
    };
    assert!(refusal.contains("no longer match their files"), "{refusal}");
    assert!(
        refusal.contains("Run `git status` once in"),
        "the sentence gives the remedy: {refusal}"
    );
    assert!(
        refusal.contains(format!("the file times recorded for {stale} of them").as_str()),
        "the sentence names how many times are stale: {refusal}"
    );
    assert!(
        refusal.contains("and 0 of them changed or were staged"),
        "no path of this source changed: {refusal}"
    );
    assert!(
        refusal.contains(cut.to_string().as_str()),
        "and the cut: {refusal}"
    );
    let copied: Vec<PathBuf> = std::fs::read_dir(destination.as_path())
        .expect("the destination")
        .map(|entry| {
            return entry.expect("an entry").path();
        })
        .filter(|path: &PathBuf| {
            return path
                .file_name()
                .is_some_and(|name: &std::ffi::OsStr| return name != ".git");
        })
        .collect();
    assert!(
        copied.is_empty(),
        "nothing was copied into the destination: {copied:?}"
    );
    crate::test_support::remove(root.as_path());
}

/// A source with changes whose new `HEAD` holds a later commit is cloned from the source's own tree,
/// brought to the new commit by the ladder's `read-tree -m -u`,
/// and proven against the new `HEAD`.
#[test]
fn a_new_head_that_differs_is_brought_in_after_the_clones() {
    let Some(root) = clone_capable_fixture("clone-partial-differs") else {
        return;
    };
    let dirty: Dirty = dirty_repository(root.as_path());
    let commit: String = later_commit(dirty.repo.as_path(), root.join("later-index").as_path());
    git(
        dirty.repo.as_path(),
        &["update-ref", "refs/heads/topic", commit.as_str()],
    );
    let context: crate::transaction_git::GitContext = git_context();
    let mut target: Target = target_of(&dirty);
    target.commit = commit;
    target.differs = true;
    let held_fence: AttributeFence =
        crate::worktree_clone_conversion_fence::fence_the_attributes(&dirty.source, &context)
            .expect("the attributes");
    let destination: std::fs::File =
        open_root(dirty.destination.as_path()).expect("the destination");
    let admin: std::fs::File =
        open_root(dirty.admin.as_path()).expect("the administrative directory");
    let cloned = clone_the_unchanged(PartialClone {
        context: &context,
        source: &dirty.source,
        entries: None,
        destination: &destination,
        admin: &admin,
        admin_path: dirty.admin.as_path(),
        fence: held_fence,
        host: A_HOST_THAT_BOUNDS_NO_WORKER,
    });
    let (part, decided, pending): (PartCloned, Fence, PendingIndex) =
        cloned.expect("the partial clone");
    assert_eq!(
        part.cloned,
        3 + UNCHANGED_PATHS,
        "the paths the source's changes did not touch are clones"
    );
    let program: PathBuf = context.real_git.clone();
    let mut runner = a_real_runner(program.as_path());
    apply_difference_with_index(
        (
            &dirty.source,
            &target,
            dirty.destination.as_path(),
            &mut runner,
        ),
        Some(pending.path_in()),
    )
    .expect("the tree difference applied");
    crate::worktree_clone_conversion_fence::checkout_index_through_the_index_at(
        (dirty.admin.as_path(), dirty.destination.as_path(), &context),
        pending.path_in(),
        decided.git_writes.as_slice(),
        dirty.source.clean.hash_length,
    )
    .expect("Git writes the paths the fence leaves it");
    pending
        .publish(&admin)
        .expect("the index takes the worktree's name");
    prove_the_worktree(
        &context,
        dirty.destination.as_path(),
        dirty.admin.as_path(),
        target.commit.as_str(),
    )
    .expect("the worktree Git's checkout of the new commit makes");
    assert_eq!(
        std::fs::read(dirty.destination.join("later.txt")).expect("the new commit's file"),
        b"the later commit's bytes\n"
    );
    assert_eq!(
        std::fs::read(dirty.destination.join("kept.txt")).expect("the commit's bytes"),
        KEPT,
        "a cloned path the new commit does not change keeps its bytes"
    );
    assert_eq!(
        git_text(dirty.destination.as_path(), &["status", "--porcelain=v1"]),
        ""
    );
    crate::test_support::remove(root.as_path());
}

/// What:
///  A staged addition among the entries of a partial clone reaches no destination,
/// in the arm where the slices run and in the one-turn arm a refused runtime takes.
/// Why:
///   The one-turn arm clones what `clone_in_slices` was handed,
///  which is the entry set the partial clone's proof
///   settled and not the source's whole index;
///   an arm that re-read the index would clone a staged addition,
///  which is in no tree,
///  into the new worktree and say
///   nothing about it (the release pass over alpha.45's finding H2).
/// The runtime's refusal cannot be asked for by an argument,
///  so a control asks for it through the module's own flag;
/// both arms assert the same absence,
///  which is what makes the flag's arm the one under test.
/// The fixture holds more unchanged paths than the clone's break-even,
///  so the slices are what runs and the one-turn arm is
/// their fallback rather than the shape a small repository takes.
/// The host the copies' budget is asked about is this control's own,
/// so the fixture and the flag decide which arm runs,
/// and never the processors and the memory the run happens to have (issue 6537).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const forced of [false, true]) { forcedOneTurn = forced; const part = cloneTheUnchanged(...); expect(exists(destination, 'added.txt')).toBe(false); }
/// ```
#[test]
fn a_staged_addition_reaches_no_destination_in_the_one_turn_arm_the_runtime_refusal_takes() {
    for forced in [false, true] {
        let Some(root) = clone_capable_fixture(if forced {
            "clone-partial-one-turn-forced"
        } else {
            "clone-partial-one-turn-sliced"
        }) else {
            return;
        };
        let dirty: Dirty = repository_for_the_one_turn_arm(root.as_path());
        let context: crate::transaction_git::GitContext = git_context();
        let source: SnapshotSource =
            source_or_dirty(&context, &main_identity(dirty.repo.as_path()))
                .expect("a source with changes");
        let entries: Vec<crate::worktree_clean::StatEntry> =
            crate::worktree_clone_rung_limits::tracked_entries(&source).expect("the entries");
        let changed: usize = source.dirty.as_ref().expect("the changes").tracked;
        let clones: usize = entries.len() - changed;
        assert!(
            clone_worker_count(clones, &A_HOST_THAT_BOUNDS_NO_WORKER) > 1,
            "the fixture has {clones} cloned paths of {} and the host this control names leaves them in slices, so the one-turn arm is the fallback under test",
            entries.len()
        );
        let held_fence: AttributeFence =
            crate::worktree_clone_conversion_fence::fence_the_attributes(&source, &context)
                .expect("the attributes");
        let destination: std::fs::File =
            open_root(dirty.destination.as_path()).expect("the destination");
        let admin: std::fs::File =
            open_root(dirty.admin.as_path()).expect("the administrative directory");
        crate::worktree_clone_slices::FORCED_ONE_TURN_ARM
            .store(forced, std::sync::atomic::Ordering::SeqCst);
        let cloned = clone_the_unchanged(PartialClone {
            context: &context,
            source: &source,
            entries: None,
            destination: &destination,
            admin: &admin,
            admin_path: dirty.admin.as_path(),
            fence: held_fence,
            host: A_HOST_THAT_BOUNDS_NO_WORKER,
        });
        crate::worktree_clone_slices::FORCED_ONE_TURN_ARM
            .store(false, std::sync::atomic::Ordering::SeqCst);
        let (part, _decided, _pending): (PartCloned, Fence, PendingIndex) =
            cloned.expect("the partial clone");
        assert!(
            !dirty.destination.join("added.txt").exists(),
            "forced={forced}: a staged addition is in no tree, so no arm clones it into the destination"
        );
        assert_eq!(
            part.cloned, clones,
            "forced={forced}: the proof's own paths are the clones, as many as the source's index holds beyond its changes"
        );
        assert!(
            dirty.destination.join("kept.txt").exists(),
            "forced={forced}: the paths the proof settled are cloned in both arms"
        );
        assert_eq!(
            part.written, changed,
            "forced={forced}: every path the source's own status names is Git's, and the staged addition is one of them"
        );
        crate::test_support::remove(root.as_path());
    }
}

/// What:
///  The fixture of the one-turn arm's control:
/// a source with a tracked edit,
///  a staged change,
///  a deletion and a staged addition,
///  and more unchanged paths than
/// the clone's break-even,
///  so the copies are split into slices.
/// Why:
///   The arm under test is the slices' fallback,
/// and a repository below the break-even never takes it:
/// a fixture of ten unchanged paths would test the one-turn shape twice and the flag would prove nothing
/// (measured while this control was written).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dirty = await repositoryForTheOneTurnArm(root); // 30 unchanged paths and the three changes
/// ```
fn repository_for_the_one_turn_arm(root: &Path) -> Dirty {
    let repo: PathBuf = std::fs::canonicalize(repository(root, "repo")).expect("canonical");
    tracked(repo.as_path(), "kept.txt", KEPT);
    for index in 0..30 {
        tracked(
            repo.as_path(),
            format!("kept/unchanged-{index}.txt").as_str(),
            KEPT,
        );
    }
    commit_all(repo.as_path(), "the commit the person adds to");
    std::fs::write(
        repo.join("added.txt"),
        b"a file the person staged and never committed\n",
    )
    .expect("the staged addition");
    git(repo.as_path(), &["add", "--", "added.txt"]);
    settle_the_index(repo.as_path());
    git(
        repo.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--no-checkout",
            "-b",
            "topic",
            "../topic",
        ],
    );
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    let admin: PathBuf = repo.join(".git/worktrees/topic");
    let source: SnapshotSource = source_or_dirty(&git_context(), &main_identity(repo.as_path()))
        .expect("a source with changes");
    assert!(
        source.dirty.is_some(),
        "the fixture's source has uncommitted changes"
    );
    return Dirty {
        repo,
        destination,
        admin,
        source,
    };
}
