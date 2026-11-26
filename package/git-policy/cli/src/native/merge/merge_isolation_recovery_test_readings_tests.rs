//! What:
//!  What a control reads back:
//!  the staged entries and the worktree as another session sees them,
//!  what recovery decided for one record,
//!  the judgement of every changed path of a merge,
//!  the mode Git checks a path out with,
//!  and the bytes a merge holds at one path.
//! Why:
//!  A control states its expectation through one of these,
//!  and a reading written twice reads two ways.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theActionFor(outcomes, record)).toBe('completed'); expect(snapshot(world)).toEqual(before);
//! ```

use super::super::*;
use super::fixtures::{
    World,
    dead_owner,
    place,
};
use crate::merge_isolation_changes::{
    WorktreeState,
    file_identity,
    worktree_path,
    worktree_state,
};
/// The judgement of one path,
/// for the controls that build a changed path of their own.
use crate::merge_isolation_recovery_steps::standing;
use crate::test_support::{
    git,
    git_text,
};
use std::path::{
    Path,
    PathBuf,
};

/// Everything about the staged entries and the worktree that another session would notice.
pub(super) fn snapshot(world: &World) -> (String, String) {
    return (
        git_text(world.repo.as_path(), &["ls-files", "--stage"]),
        git_text(
            world.repo.as_path(),
            &["status", "--porcelain=v1", "--untracked-files=all"],
        ),
    );
}

/// What:
///  What recovery did for the record at one path,
///  or nothing when it settled no record there.
/// Why:
///  A worktree can hold more than one record,
/// and recovery reads them in the directory's order,
/// so a control
///  names the record it asks about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theActionFor(outcomes, record): MergeRecoveryAction | undefined; // outcomes.find((outcome) => outcome.record === record)?.action
/// ```
pub(super) fn the_action_for(
    outcomes: &[MergeRecoveryOutcome],
    record: &Path,
) -> Option<MergeRecoveryAction> {
    for outcome in outcomes {
        if outcome.record.as_path() == record {
            return Some(outcome.action);
        }
    }
    return None;
}

/// What:
///  Judge every changed path of the world's merge the way recovery does,
///  and return the changes with their judgements.
/// Why:
///  A control about a file that changes after it was judged needs the judgement and the write separated,
/// and that
///  separation is the very instant the finding is about;
/// no sleep and no second process is needed,
/// because the judgement is
///  the value the control hands on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function judge(world) { return mergeChanges(place, root, intent).map((change) => ({ change, judged: { standing: standing(change, stateOf(change)), identity: identityOf(change) } })); }
/// ```
pub(super) fn judged_changes(world: &World) -> (Vec<Change>, Vec<Judged>) {
    let intent: MergeIntent = MergeIntent {
        owner: dead_owner("14141414-0000-4000-8000-000000000014"),
        repository_root: world.repo.clone(),
        target_ref: Some(String::from("refs/heads/main")),
        old_oid: world.old_oid.clone(),
        new_oid: world.new_oid.clone(),
        side_ref: None,
        carried: None,
    };
    let place: RecoveryPlace = place(world);
    let changes: Vec<Change> =
        merge_changes(&place, world.repo.as_path(), &intent).expect("the merge's changes");
    let records_file_modes: bool =
        crate::merge_isolation_changes::the_repository_records_file_modes(
            &place.git,
            world.repo.as_path(),
        );
    let mut judgements: Vec<Judged> = Vec::with_capacity(changes.len());
    for change in &changes {
        // The identity first, as recovery reads it, then the content: exactly the order of `settle_failed_merge`.
        let identity =
            file_identity(worktree_path(world.repo.as_path(), change.path.as_slice()).as_path());
        let state: WorktreeState =
            worktree_state(&place.git, world.repo.as_path(), change.path.as_slice())
                .expect("the state");
        let content: Option<String> = match &state {
            WorktreeState::Content {
                object,
                ..
            } => Some(object.clone()),
            WorktreeState::Missing | WorktreeState::Other => None,
        };
        judgements.push(Judged {
            standing: standing(change, &state, records_file_modes),
            identity,
            content,
        });
    }
    return (changes, judgements);
}

/// What:
///  The permission bits Git's own checkout gives one path of a world's index.
/// Why:
///  The install gives each file it wrote the bits the take read and keeps the executable bits Git's entry decided,
///  and the mode Git's entry implies is what `checkout-index` writes under this run's umask,
///  never a number a control names:
/// a repository that records file modes writes `100644` as `0644` only where the umask allows it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theModeGitChecksOut = (world, path) => mode(join(world.parent, 'git-wrote', path)); // after `git checkout-index --prefix=...`
/// ```
#[cfg(unix)]
pub(super) fn the_mode_git_checks_out(
    world: &World,
    path: &str,
) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    let elsewhere: PathBuf = world.parent.join("git-wrote");
    std::fs::create_dir(elsewhere.as_path()).expect("a directory Git checks the entry out into");
    git(
        world.repo.as_path(),
        &[
            "checkout-index",
            format!("--prefix={}/", elsewhere.display()).as_str(),
            "--",
            path,
        ],
    );
    return std::fs::symlink_metadata(elsewhere.join(path))
        .expect("Git's checkout of the entry")
        .permissions()
        .mode()
        & 0o7777;
}

/// The bytes the file the merge holds at one path has,
/// from the tree of a commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git(['show', `${oid}:${path}`])
/// ```
pub(super) fn the_bytes_of_a_commit(
    world: &World,
    oid: &str,
    path: &str,
) -> Vec<u8> {
    let text: String = git_text(
        world.repo.as_path(),
        &["show", format!("{oid}:{path}").as_str()],
    );
    return format!("{text}\n").into_bytes();
}
