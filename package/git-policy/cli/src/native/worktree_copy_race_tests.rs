//! The races the settlement lock used to prevent, against real Git and real processes: two copies into one
//!  destination, a copy against a worktree removal, and a copy killed at each durable state it passes through.
//!  Nothing waits: every case ends because the code yields, settles or skips, never because a holder let go.

use super::*;
use crate::test_support::{fixture, git, git_context, remove, repository};
use crate::worktree_copy_claim::{Claim, claims_exist, publish_claim};
use crate::worktree_copy_journal::{Journal, ensure_journal_root, journal_root, read_pending_journals, write_journal};
use crate::worktree_copy_recovery::recover_pending_worktree_copies;
use crate::worktree_copy_registry::COPIED_MARKER_NAME;
use crate::worktree_copy_selection::STAGE_PREFIX;
use std::io::BufRead;
use std::os::unix::fs::PermissionsExt;
use std::process::{Child, Command, Stdio};

/// A main checkout with an ignored `cache` of four entries, a registered worktree `topic`, and the repository's
///  canonical common directory.
struct Scene {
    root: PathBuf,
    source: PathBuf,
    common: PathBuf,
    created: CreatedWorktree,
}

/// Builds the scene named `name`.
fn build_scene(name: &str) -> Scene {
    let root: PathBuf = fixture(name);
    let source: PathBuf = repository(root.as_path(), "source");
    std::fs::write(source.join(".gitignore"), b"cache/\n").expect("gitignore");
    std::fs::create_dir_all(source.join("cache/deep")).expect("cache");
    std::fs::write(source.join("cache/deep/data"), b"payload").expect("data");
    std::fs::write(source.join("cache/top"), b"top").expect("top");
    let topic: PathBuf = root.join("topic");
    git(
        source.as_path(),
        &["worktree", "add", "--quiet", "-b", "topic", topic.to_str().expect("utf8")],
    );
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let destination: PathBuf = std::fs::canonicalize(topic).expect("canonical destination");
    return Scene {
        root,
        source: std::fs::canonicalize(source).expect("canonical source"),
        common,
        created: CreatedWorktree {
            admin_id: String::from("topic"),
            root: destination,
        },
    };
}

impl Scene {
    /// One synchronization of the scene's destination, with its notices.
    fn synchronize(&self) -> (Result<Synchronized, WorktreeCopyError>, String) {
        return self.synchronize_at(&mut |_checkpoint| {});
    }

    /// One synchronization that calls `checkpoint` at each checkpoint.
    fn synchronize_at(
        &self,
        checkpoint: &mut dyn FnMut(Checkpoint),
    ) -> (Result<Synchronized, WorktreeCopyError>, String) {
        let mut stderr: Vec<u8> = Vec::new();
        let outcome = synchronize_with_checkpoints(
            &git_context(),
            self.common.as_path(),
            self.source.as_path(),
            &self.created,
            &[],
            &CopySelection::Ignored,
            &mut stderr,
            checkpoint,
        );
        return (outcome, String::from_utf8(stderr).expect("utf8"));
    }

    /// The claim a copy killed at the start would leave: published, container named, nothing else.
    fn claim_for_a_copy_that_dies(&self) -> (Claim, PathBuf) {
        let container: PathBuf = new_stage_container(self.created.root.as_path()).expect("container");
        let claim: Claim = publish_claim(
            ensure_journal_root(self.common.as_path()).expect("root").as_path(),
            self.created.admin_id.as_str(),
            self.created.root.as_path(),
            container.as_path(),
        )
        .expect("published")
        .expect("won the name");
        return (claim, container);
    }

    /// Whether anything of a copy stands: a claim, a journal, or a stage beside the destination.
    fn anything_left(&self) -> Vec<String> {
        let mut left: Vec<String> = Vec::new();
        if claims_exist(journal_root(self.common.as_path()).as_path()).expect("claims") {
            left.push(String::from("a claim"));
        }
        if !read_pending_journals(self.common.as_path()).expect("pending").is_empty() {
            left.push(String::from("a journal"));
        }
        for entry in std::fs::read_dir(self.root.as_path()).expect("listing").flatten() {
            if entry.file_name().to_string_lossy().starts_with(STAGE_PREFIX) {
                left.push(format!("the stage {}", entry.path().display()));
            }
        }
        return left;
    }

    /// The marker of a finished copy.
    fn marker(&self) -> PathBuf {
        return self.common.join("worktrees").join(self.created.admin_id.as_str()).join(COPIED_MARKER_NAME);
    }
}

/// A child process that holds a file open and says so, the way a running copy holds its claim.
fn holder_of(path: &Path) -> Child {
    let mut child: Child = Command::new("sh")
        .args(["-c", "exec 3<\"$1\"; echo ready; exec sleep 3600", "holder"])
        .arg(path)
        .stdout(Stdio::piped())
        .spawn()
        .expect("holder starts");
    let stdout = child.stdout.take().expect("stdout");
    let mut line: String = String::new();
    std::io::BufReader::new(stdout).read_line(&mut line).expect("ready line");
    assert_eq!(line, "ready\n");
    return child;
}

/// The files of the destination's `cache`, sorted.
fn cache_tree(destination: &Path) -> Vec<String> {
    let mut found: Vec<String> = vec![String::from("cache")];
    let mut pending: Vec<PathBuf> = vec![destination.join("cache")];
    while let Some(directory) = pending.pop() {
        for listed in std::fs::read_dir(directory.as_path()).expect("read").flatten() {
            let path: PathBuf = listed.path();
            found.push(path.strip_prefix(destination).expect("prefix").to_string_lossy().into_owned());
            if path.is_dir() {
                pending.push(path);
            }
        }
    }
    found.sort();
    return found;
}

/// Entries the copy creates in the scene: the `cache` directory, its `deep` directory and its two files.
const COPIED_ENTRIES: usize = 4;

//region Two copies into one destination

/// A copy that meets the claim of a copy still running yields at once: nothing is staged, nothing is installed, the
///  running copy's claim is untouched, and the same call succeeds once that copy has died.
#[test]
fn a_copy_yields_to_a_live_claim_on_its_destination() {
    let scene: Scene = build_scene("race-live-claim");
    let (live, container) = scene.claim_for_a_copy_that_dies();
    let before: String = std::fs::read_to_string(live.path()).expect("claim");
    let (yielded, notices) = scene.synchronize();
    match yielded.expect("yielded without failing") {
        Synchronized::Yielded(reason) => assert!(reason.contains("descriptor"), "{reason}"),
        other => panic!("expected the copy to yield, got {other:?}"),
    }
    assert!(notices.is_empty(), "{notices}");
    assert!(!scene.created.root.join("cache").exists(), "nothing is installed behind a live copy");
    assert!(!container.exists(), "nothing is staged behind a live copy");
    assert_eq!(std::fs::read_to_string(live.path()).expect("claim"), before, "the live claim is untouched");
    assert!(!scene.marker().exists());
    // The live copy dies; the next copy settles its claim and does the work itself, without anyone having let go.
    drop(live);
    let (copied, notices) = scene.synchronize();
    assert_eq!(copied.expect("copied"), Synchronized::Copied(COPIED_ENTRIES));
    assert!(notices.is_empty(), "{notices}");
    assert_eq!(cache_tree(scene.created.root.as_path()), vec!["cache", "cache/deep", "cache/deep/data", "cache/top"]);
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    assert!(scene.marker().exists());
    remove(scene.root.as_path());
}

/// A copy that arrives after a finished copy marked the registration does nothing: no stage, no claim, no install,
///  even when the user edited a copied file in between.
#[test]
fn a_late_copy_skips_a_destination_a_finished_copy_marked() {
    let scene: Scene = build_scene("race-late-copy");
    let (first, _notices) = scene.synchronize();
    assert_eq!(first.expect("copied"), Synchronized::Copied(COPIED_ENTRIES));
    std::fs::write(scene.created.root.join("cache/top"), b"edited by the user").expect("edit");
    let mut checkpoints: Vec<Checkpoint> = Vec::new();
    let (late, notices) = scene.synchronize_at(&mut |checkpoint| checkpoints.push(checkpoint));
    assert_eq!(late.expect("late copy"), Synchronized::AlreadyCopied);
    assert!(checkpoints.is_empty(), "the late copy never reached its claim: {checkpoints:?}");
    assert!(notices.is_empty(), "{notices}");
    assert_eq!(std::fs::read(scene.created.root.join("cache/top")).expect("top"), b"edited by the user");
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    remove(scene.root.as_path());
}

/// A copy that finished between another copy's first look and its claim leaves the marker, and the claim is released unused.
#[test]
fn a_copy_that_wins_its_claim_after_a_finished_copy_releases_it_unused() {
    let scene: Scene = build_scene("race-marker-after-claim");
    let marker: PathBuf = scene.marker();
    let mut checkpoints: Vec<Checkpoint> = Vec::new();
    // Another copy finishes between the first look and this copy's claim: its marker appears once the claim is held.
    let (late, notices) = scene.synchronize_at(&mut |checkpoint| {
        checkpoints.push(checkpoint);
        std::fs::write(marker.as_path(), b"").expect("marker");
    });
    assert_eq!(late.expect("late copy"), Synchronized::AlreadyCopied);
    assert_eq!(checkpoints, vec![Checkpoint::Claimed], "nothing was staged");
    assert!(notices.is_empty(), "{notices}");
    assert!(!scene.created.root.join("cache").exists());
    assert_eq!(scene.anything_left(), Vec::<String>::new(), "the claim was released");
    remove(scene.root.as_path());
}

/// Two copies started together into one destination: exactly one of them copies, the other yields or finds it done,
///  and the destination holds one complete copy with nothing of either left behind.
#[test]
fn two_copies_started_together_copy_the_destination_once() {
    let scene: Scene = build_scene("race-together");
    let barrier: std::sync::Barrier = std::sync::Barrier::new(2);
    let outcomes: Vec<Synchronized> = std::thread::scope(|threads| {
        let runs: Vec<_> = (0..2)
            .map(|_| {
                return threads.spawn(|| {
                    barrier.wait();
                    return scene.synchronize().0.expect("neither copy fails");
                });
            })
            .collect();
        return runs.into_iter().map(|run| return run.join().expect("thread")).collect();
    });
    let copies: usize = outcomes
        .iter()
        .filter(|outcome| return matches!(outcome, Synchronized::Copied(_)))
        .count();
    assert_eq!(copies, 1, "{outcomes:?}");
    assert!(
        outcomes.iter().all(|outcome| return matches!(
            outcome,
            Synchronized::Copied(COPIED_ENTRIES) | Synchronized::Yielded(_) | Synchronized::AlreadyCopied
        )),
        "{outcomes:?}"
    );
    assert_eq!(cache_tree(scene.created.root.as_path()), vec!["cache", "cache/deep", "cache/deep/data", "cache/top"]);
    assert_eq!(std::fs::read(scene.created.root.join("cache/top")).expect("top"), b"top");
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    assert!(scene.marker().exists());
    remove(scene.root.as_path());
}

/// A worktree Git is still creating belongs to its creator: it is not found as created, so no copy starts in it.
#[test]
fn a_destination_still_being_created_is_left_to_its_creator() {
    let scene: Scene = build_scene("race-initializing");
    let locked: PathBuf = scene.common.join("worktrees/topic/locked");
    // The text Git writes while it creates a worktree.
    std::fs::write(locked.as_path(), b"initializing").expect("locked");
    let observation = crate::worktree_copy_registry::WorktreeCopyObservation {
        admin_root: scene.common.join("worktrees"),
        before_admin_ids: std::collections::BTreeSet::new(),
        common_dir: scene.common.clone(),
        effective_cwd: scene.source.clone(),
        source_root: Some(scene.source.clone()),
        selection: CopySelection::Ignored,
    };
    let found = crate::worktree_copy_registry::find_created_worktrees(&git_context(), &observation).expect("found");
    assert!(found.created.is_empty(), "{:?}", found.created);
    // Git finishes: the lock goes, and the same worktree is now found.
    std::fs::remove_file(locked.as_path()).expect("unlocked");
    let found = crate::worktree_copy_registry::find_created_worktrees(&git_context(), &observation).expect("found");
    assert_eq!(found.created, vec![scene.created.clone()]);
    // A user's own lock reason does not stop the copy.
    std::fs::write(locked.as_path(), b"added with --lock").expect("locked by the user");
    let found = crate::worktree_copy_registry::find_created_worktrees(&git_context(), &observation).expect("found");
    assert_eq!(found.created, vec![scene.created.clone()]);
    remove(scene.root.as_path());
}

//endregion

//region A copy against a worktree removal

/// A worktree removed while its stage is being built is not filled: the stage and journal are discarded, nothing is
///  created where the worktree was, and the copy ends with the removal's notice instead of a failure.
#[test]
fn a_worktree_removed_while_staging_is_not_filled() {
    let scene: Scene = build_scene("race-removed-staging");
    let root: PathBuf = scene.created.root.clone();
    let source: PathBuf = scene.source.clone();
    let (outcome, _notices) = scene.synchronize_at(&mut |checkpoint| {
        if checkpoint == Checkpoint::Staged {
            git(source.as_path(), &["worktree", "remove", "--force", root.to_str().expect("utf8")]);
        }
    });
    assert_eq!(outcome.expect("no failure"), Synchronized::Unregistered);
    assert!(!root.exists(), "the removed worktree was not brought back");
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    remove(scene.root.as_path());
}

/// A worktree removed right after the claim is won is never staged for.
#[test]
fn a_worktree_removed_before_staging_is_never_staged() {
    let scene: Scene = build_scene("race-removed-claimed");
    let root: PathBuf = scene.created.root.clone();
    let source: PathBuf = scene.source.clone();
    let mut staged: bool = false;
    let (outcome, _notices) = scene.synchronize_at(&mut |checkpoint| {
        match checkpoint {
            Checkpoint::Claimed => {
                git(source.as_path(), &["worktree", "remove", "--force", root.to_str().expect("utf8")]);
            }
            Checkpoint::Staged => staged = true,
        }
    });
    assert_eq!(outcome.expect("no failure"), Synchronized::Unregistered);
    assert!(!staged, "no stage was built for a removed worktree");
    assert!(!root.exists());
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    remove(scene.root.as_path());
}

//endregion

//region A copy killed at each durable state

/// A copy killed right after publishing its claim leaves only the claim; the next pass settles it quietly.
#[test]
fn a_copy_killed_after_its_claim_is_settled_quietly() {
    let scene: Scene = build_scene("race-killed-claim");
    let (claim, _container) = scene.claim_for_a_copy_that_dies();
    drop(claim);
    let mut notices: Vec<u8> = Vec::new();
    recover_pending_worktree_copies(scene.common.as_path(), &mut notices).expect("settled");
    assert!(notices.is_empty(), "{}", String::from_utf8_lossy(notices.as_slice()));
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    assert!(!scene.created.root.join("cache").exists());
    remove(scene.root.as_path());
}

/// A copy killed while staging leaves a claim and a half-filled stage; the next pass removes the stage with a notice.
#[test]
fn a_copy_killed_while_staging_loses_its_stage_with_a_notice() {
    let scene: Scene = build_scene("race-killed-staging");
    let (claim, container) = scene.claim_for_a_copy_that_dies();
    std::fs::create_dir(container.as_path()).expect("stage");
    std::fs::set_permissions(container.as_path(), std::fs::Permissions::from_mode(0o700)).expect("private");
    std::fs::write(container.join("half"), b"copied so far").expect("partial copy");
    drop(claim);
    let mut notices: Vec<u8> = Vec::new();
    recover_pending_worktree_copies(scene.common.as_path(), &mut notices).expect("settled");
    assert!(String::from_utf8_lossy(notices.as_slice()).contains("removed the stage"));
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    remove(scene.root.as_path());
}

/// A copy killed after its journal was written is finished by the next pass: the destination ends up complete.
#[test]
fn a_copy_killed_after_its_journal_is_finished_by_the_next_pass() {
    let scene: Scene = build_scene("race-killed-journal");
    let (claim, container) = scene.claim_for_a_copy_that_dies();
    let snapshot: StagedSnapshot = stage_ignored_snapshot(
        &git_context(),
        scene.source.as_path(),
        container.as_path(),
        &[],
        &CopySelection::Ignored,
    )
    .expect("staged");
    create_journal(scene.common.as_path(), scene.created.root.as_path(), &snapshot).expect("journal");
    drop(claim);
    let mut notices: Vec<u8> = Vec::new();
    recover_pending_worktree_copies(scene.common.as_path(), &mut notices).expect("settled");
    assert_eq!(
        String::from_utf8_lossy(notices.as_slice()),
        "cli-git: recovered ignored-state copies for 1 worktree transaction.\n"
    );
    assert_eq!(cache_tree(scene.created.root.as_path()), vec!["cache", "cache/deep", "cache/deep/data", "cache/top"]);
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    remove(scene.root.as_path());
}

/// A copy killed after it began installing is finished by the next pass from the phase its journal recorded.
#[test]
fn a_copy_killed_while_installing_is_finished_by_the_next_pass() {
    let scene: Scene = build_scene("race-killed-installing");
    let (claim, container) = scene.claim_for_a_copy_that_dies();
    let snapshot: StagedSnapshot = stage_ignored_snapshot(
        &git_context(),
        scene.source.as_path(),
        container.as_path(),
        &[],
        &CopySelection::Ignored,
    )
    .expect("staged");
    let pending: PendingJournal =
        create_journal(scene.common.as_path(), scene.created.root.as_path(), &snapshot).expect("journal");
    write_journal(
        pending.path.as_path(),
        &Journal {
            phase: Phase::Installing,
            ..pending.journal.clone()
        },
    )
    .expect("installing");
    drop(claim);
    let mut notices: Vec<u8> = Vec::new();
    recover_pending_worktree_copies(scene.common.as_path(), &mut notices).expect("settled");
    assert_eq!(cache_tree(scene.created.root.as_path()), vec!["cache", "cache/deep", "cache/deep/data", "cache/top"]);
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    remove(scene.root.as_path());
}

/// A publisher killed between creating its temporary and linking it leaves a temporary only; the next pass removes it.
#[test]
fn a_copy_killed_while_publishing_leaves_a_temporary_the_next_pass_removes() {
    let scene: Scene = build_scene("race-killed-publishing");
    let journals: PathBuf = ensure_journal_root(scene.common.as_path()).expect("root");
    let temporary: PathBuf = journals.join("topic.claim.0123.tmp");
    std::fs::write(temporary.as_path(), b"cut off").expect("temporary");
    let mut notices: Vec<u8> = Vec::new();
    recover_pending_worktree_copies(scene.common.as_path(), &mut notices).expect("settled");
    assert!(!temporary.exists());
    assert!(notices.is_empty());
    remove(scene.root.as_path());
}

/// A pass never touches a copy a live process holds, however long it takes, and settles it the moment the process is
///  killed: the same files, the same call, and nothing in between but the death.
#[test]
fn a_pass_leaves_a_live_copy_alone_and_settles_it_after_the_kill() {
    let scene: Scene = build_scene("race-live-process");
    let (claim, container) = scene.claim_for_a_copy_that_dies();
    let claim_path: PathBuf = claim.path().to_path_buf();
    drop(claim);
    std::fs::create_dir(container.as_path()).expect("stage");
    std::fs::set_permissions(container.as_path(), std::fs::Permissions::from_mode(0o700)).expect("private");
    std::fs::write(container.join("half"), b"copied so far").expect("partial copy");
    let mut holder: Child = holder_of(claim_path.as_path());
    let mut notices: Vec<u8> = Vec::new();
    recover_pending_worktree_copies(scene.common.as_path(), &mut notices).expect("skipped");
    assert!(notices.is_empty(), "{}", String::from_utf8_lossy(notices.as_slice()));
    assert!(container.exists(), "a live copy keeps its stage");
    assert!(claim_path.exists());
    holder.kill().expect("killed");
    holder.wait().expect("reaped");
    recover_pending_worktree_copies(scene.common.as_path(), &mut notices).expect("settled");
    assert!(String::from_utf8_lossy(notices.as_slice()).contains("removed the stage"));
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    remove(scene.root.as_path());
}

//endregion
