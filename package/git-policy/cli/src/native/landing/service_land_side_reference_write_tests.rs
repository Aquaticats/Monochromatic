//! What:
//!  What a landing does when the wait for the write of one of its own side references ends
//!  before the write does.
//! Why:
//!  A side reference is the only thing that keeps a commit a reference lock held reachable
//!  (`held_commit_record.rs`),
//!  and the wrapper never ends such a write for being slow,
//!  because cutting a write in place risks the person's data
//!  (`transaction_git_step_bounds.rs`,
//!  `is_side_ref_write`):
//!  a wait that ends without its answer says the result is not known,
//!  never that the write failed.
//!  On 2026-11-11 the release gate of 0.6.1-alpha.11 read that ending as a failed write:
//!  the write of `refs/worktree/cli-git-service/<batch>` was still moving after its 127-second wait,
//!  in quiet gaps under a second,
//!  the landing answered `unavailable`,
//!  and a commit that was made was reported to the person as not made.
//!  The controls here make that write outlast its wait on purpose,
//!  and hold the branch behind a lock nothing proves gone,
//!  so the commit must be made,
//!  kept on the side reference the answer names,
//!  and survive a prune.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! slowGit({ on: 'update-ref --no-deref refs/' }); expect(await keepOnASideRef(...)).toBeUndefined();
//! plant('refs/heads/main.lock'); expect(await landBatch(...)).toEqual([{ held: ... }]);
//! ```

use super::*;
use super::tests::{World, built_request, world};
use crate::service_land_git_steps_and_import_stream::keep_on_a_side_ref;
use crate::test_support::{git, git_text, remove, while_test_lives};
use crate::transaction_git::{EXTRA_WAIT_FOR_THIS_THREAD, STEP_CEILING_FOR_THIS_THREAD};
use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, SystemTime};

/// The bound of a step in these controls,
/// and the extra wait after it:
/// a second each,
/// so a wait that has to run out costs the control a moment and not a minute.
const ONE_SECOND: Duration = Duration::from_secs(1);

/// How old the lock these controls plant is:
/// older than the margin a reference lock is judged at,
/// so the landing judges it at once and holds the batch instead of waiting on it,
/// as a lock a killed Git left is.
const LOCK_AGE: Duration = Duration::from_secs(120);

/// What:
///  A Git whose first write of a side reference is still running when the wait for it ends,
///  and whose every other run does what the real Git does.
/// Why:
///  The write is the step the wrapper never ends,
///  so the control has to make it outlast its wait without making it fail:
///  the first such write waits until the fixture goes,
///  a later one writes the reference at once,
///  and every other command is the real Git's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slow = script({ on: 'update-ref --no-deref refs/', first: () => waitForTheFixtureToGo(), then: execRealGit });
/// ```
fn git_slow_to_write_a_side_ref(world: &World) -> PathBuf {
    let entered: PathBuf = world.root.join("side-ref-write-entered");
    let program: PathBuf = world.root.join("slow-side-ref-git");
    let script: String = format!(
        "#!/bin/sh\ncase \" $* \" in\n  *\" update-ref --no-deref refs/\"*)\n    if [ ! -e '{entered}' ]; then\n      : > '{entered}'\n      while {alive}; do sleep 0.05; done\n    fi;;\nesac\nexec '{real}' \"$@\"\n",
        entered = entered.display(),
        alive = while_test_lives(world.root.as_path()),
        real = crate::test_support::REAL_GIT
    );
    std::fs::write(&program, script.as_str()).expect("the program");
    std::fs::set_permissions(program.as_path(), std::fs::Permissions::from_mode(0o755)).expect("executable");
    return program;
}

/// Run `run` with the shortened bounds of these controls,
/// and give the thread its shipped bounds back afterwards,
/// whatever `run` did.
fn with_a_short_wait<T>(run: impl FnOnce() -> T) -> T {
    STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(Some(ONE_SECOND)); });
    EXTRA_WAIT_FOR_THIS_THREAD.with(|cell| { cell.set(Some(ONE_SECOND)); });
    let ended: T = run();
    STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(None); });
    EXTRA_WAIT_FOR_THIS_THREAD.with(|cell| { cell.set(None); });
    return ended;
}

/// What:
///  A write of a side reference whose wait ran out is answered by the reference itself.
/// Why:
///  The write names one value for one name,
///  so whether it finished is a question the reference answers,
///  and the answer is never guessed at from a wait that ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await withShortWait(() => keepOnASideRef(slowGit, repo, 1, tip))).toEqual('written'); expect(revParse('refs/cli-git/kept/1')).toBe(tip);
/// ```
#[test]
fn a_side_ref_write_whose_wait_ran_out_is_answered_by_the_reference() {
    let world: World = world("service-land-side-ref-slow");
    let tip: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let git_context: GitContext = GitContext { real_git: git_slow_to_write_a_side_ref(&world), ..world.context.git.clone() };
    let kept: Option<String> = with_a_short_wait(|| { return keep_on_a_side_ref(&git_context, world.repo.as_path(), 1, tip.as_str()); });
    assert_eq!(kept, None, "the reference answers for the write whose wait ran out");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "refs/cli-git/kept/1"]), tip, "the side reference names the commit");
    remove(world.root.as_path());
}

/// What:
///  A commit Git built,
///  held behind a lock nothing proves gone,
///  is made and kept even though the wait for its side-reference write ran out.
/// Why:
///  This is the release gate's shape:
///  a healthy write that the host made slow,
///  a refused branch write,
///  and a commit the person is told was made.
///  The answer is the held answer,
///  the side reference names the commit,
///  and a maintenance that drops what no reference names keeps it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plantOld('refs/heads/main.lock'); expect(await landBatch(...)).toEqual([{ held }]); expect(catFile(commit)).toBe(0);
/// ```
#[test]
fn a_held_commit_git_built_is_kept_when_its_side_ref_write_outlasts_the_wait_for_it() {
    let mut world: World = world("service-land-side-ref-held-slow");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    // The repository names a shared-file mode, even the one that shares nothing, which is one of the settings whose write the
    //  landing leaves to Git (`side_reference_written_here.rs`): this control is about the wait for Git's write, so it must
    //  be Git's. A mode that shares nothing leaves every other write of the fixture as Git makes it here.
    git(world.repo.as_path(), &["config", "core.sharedRepository", "false"]);
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    let commit: String = built.commit.clone().expect("the commit Git built");
    let lock: PathBuf = world.repo.join(".git/refs/heads/main.lock");
    let planted: std::fs::File = std::fs::File::create(lock.as_path()).expect("the lock a killed Git leaves");
    planted.set_modified(SystemTime::now() - LOCK_AGE).expect("the lock's age");
    world.context.git.real_git = git_slow_to_write_a_side_ref(&world);
    let landed: Result<Vec<Outcome>, FailureCode> = with_a_short_wait(|| { return land_batch(&world.context, 1, &[built]); });
    let outcomes: Vec<Outcome> = landed.expect("the commit is made and held");
    assert!(matches!(outcomes.as_slice(), [Outcome::Held { .. }]), "{outcomes:?}");
    let scratch: String = crate::service_scratch_ref_of_a_dropped_batch::scratch_ref(1);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", scratch.as_str()]), commit, "the side reference the answer names holds the held commit");
    git(world.repo.as_path(), &["gc", "--prune=now"]);
    assert_eq!(git_text(world.repo.as_path(), &["cat-file", "-t", commit.as_str()]), "commit", "the held commit survives a maintenance that drops what no reference names");
    std::fs::remove_file(lock.as_path()).expect("the holder lets go");
    crate::service_durable::drain();
    remove(world.root.as_path());
}
