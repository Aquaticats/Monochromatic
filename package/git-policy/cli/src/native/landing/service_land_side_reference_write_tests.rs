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
use crate::service_land_side_reference_write::{
    AKeptCommit, ASideRefWrite, keep_on_a_side_ref, keep_on_the_scratch_reference, keep_on_the_scratch_reference_awaiting, keep_unlanded, keep_unlanded_awaiting,
};
use crate::service_scratch_ref_of_a_dropped_batch::{ScratchRefRemoval, clear_after_dropped_batch, kept_ref, scratch_ref};
use crate::service_index_lock::{Announce, Announcer};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository, while_test_lives};
use crate::transaction_git::STEP_CEILING_FOR_THIS_THREAD;
use std::cell::RefCell;
use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, SystemTime};

/// What:
///  The lines a landing announces to its clients,
/// kept so a control can read them.
/// Why:
///  A line for the person is announced over the batch's streams,
///  and a control that calls the landing itself owns the place those lines go.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Announced implements Announcer { lines: string[] = []; announce(note) { if (note.kind === 'waiting') this.lines.push(note.text) } }
/// ```
struct Announced(RefCell<Vec<String>>);

impl Announcer for Announced {
    fn announce(&self, note: Announce) {
        if let Announce::Waiting(line) = note {
            self.0.borrow_mut().push(line);
        }
    }
}

/// The bound of a step in these controls:
/// a second,
/// so a wait that has to run out costs the control a moment.
/// The short extra wait after it stays the shipped one,
/// because that is the room the write made after a wait that ended has to finish in,
/// and a control that shortened it too would measure how loaded the host is instead
/// (`wait_ceilings.rs`,
/// `SHORT_EXTRA_WAIT`).
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
///  and the write after it puts the reference in place from the shell itself,
///  so the control does not measure how long the host takes to start a Git there,
///  and every other command is the real Git's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slow = script({ on: 'update-ref --no-deref refs/', first: () => waitForTheFixtureToGo(), then: writeTheLooseReferenceInTheShell });
/// ```
fn git_slow_to_write_a_side_ref(world: &World) -> PathBuf {
    let entered: PathBuf = world.root.join("side-ref-write-entered");
    let program: PathBuf = world.root.join("slow-side-ref-git");
    make_the_side_reference_directories(world);
    let script: String = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{invocations}'\ncase \" $* \" in\n  *\" update-ref --no-deref refs/\"*)\n    if [ ! -e '{entered}' ]; then\n      : > '{entered}'\n      while {alive}; do sleep 0.05; done\n    fi\n    shift 4\n    printf '%s\\n' \"$2\" > '{repository}/.git/'\"$1\" || exit 1\n    exit 0;;\nesac\nexec '{real}' \"$@\"\n",
        invocations = side_ref_git_invocations(world).display(),
        entered = entered.display(),
        alive = while_test_lives(world.root.as_path()),
        repository = world.repo.display(),
        real = crate::test_support::REAL_GIT
    );
    std::fs::write(&program, script.as_str()).expect("the program");
    std::fs::set_permissions(program.as_path(), std::fs::Permissions::from_mode(0o755)).expect("executable");
    return program;
}

/// What:
///  The directories of the side references these controls write from a shell,
/// made before the run.
/// Why:
///  A shell writes a loose reference with a redirection and no program of its own,
///  so the directory has to be there first;
///  Git creates it otherwise,
///  and starting Git is the wait these controls must not be about.
fn make_the_side_reference_directories(world: &World) {
    for directory in ["refs/cli-git/kept", "refs/worktree/cli-git-service"] {
        std::fs::create_dir_all(world.repo.join(".git").join(directory)).expect("the reference's directory");
    }
}

/// What:
///  Where a Git a control stands in for writes down every command line the wrapper asked it to run,
/// and those lines as text.
/// Why:
///  A control that fails says what was actually asked for,
///  so the failure is read rather than guessed at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const invocations = () => readText(join(world.root, 'side-ref-git-invocations')) ?? ''
/// ```
fn side_ref_git_invocations(world: &World) -> PathBuf {
    return world.root.join("side-ref-git-invocations");
}

fn invocations_of(world: &World) -> String {
    return std::fs::read_to_string(side_ref_git_invocations(world).as_path()).unwrap_or_default();
}

/// Run `run` with the shortened bound of these controls,
/// and give the thread its shipped bound back afterwards,
/// whatever `run` did.
fn with_a_short_wait<T>(run: impl FnOnce() -> T) -> T {
    STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(Some(ONE_SECOND)); });
    let ended: T = run();
    STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(None); });
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
    let kept: Option<String> = with_a_short_wait(|| { return keep_on_a_side_ref(&git_context, world.repo.as_path(), kept_ref(1).as_str(), tip.as_str()); });
    let asked: String = invocations_of(&world);
    let named: String = git_text(world.repo.as_path(), &["rev-parse", "refs/cli-git/kept/1"]);
    remove(world.root.as_path());
    assert_eq!(kept, None, "the reference answers for the write whose wait ran out; the Git asked was:\n{asked}");
    assert_eq!(named, tip, "the side reference names the commit");
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

/// What:
///  A reference another command wrote between a write whose answer was not known and the write after it
///  is not replaced by the write after it.
/// Why:
///  The first write's wait ended before the write did,
///  so the write is made once more,
///  and by then the name holds a commit another command kept:
///  that commit is the only thing that names it,
///  so replacing it makes `git gc` take a commit the person had
///  (the owner's rule "never discard user data").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await keepOnASideRef(gitThatKeepsACompetingCommitFirst, repo, 1, requested)).not.toBeUndefined();
/// expect(revParse('refs/cli-git/kept/1')).toBe(competing);
/// expect(catFileExists(competing)).toBe(true);
/// ```
#[test]
fn a_competing_kept_commit_is_not_replaced_by_the_write_after_an_unknown_one() {
    let world: World = world("service-land-side-ref-competing-commit");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let competing_built: CommitRequest = built_request(&world, "competing", "competing.txt", &[base.as_str()]);
    let competing: String = competing_built.commit.clone().expect("the competing commit");
    let built: CommitRequest = built_request(&world, "kept", "kept.txt", &[base.as_str()]);
    let requested: String = built.commit.clone().expect("the requested commit");
    let entered: PathBuf = world.root.join("competing-write-entered");
    let program: PathBuf = world.root.join("git-that-keeps-a-competing-commit-first");
    let script: String = format!(
        "#!/bin/sh\ncase \" $* \" in\n  *\" update-ref --no-deref refs/\"*)\n    if [ ! -e '{entered}' ]; then\n      : > '{entered}'\n      '{real}' -c core.logAllRefUpdates=false update-ref --no-deref refs/cli-git/kept/1 '{competing}'\n      while {alive}; do sleep 0.05; done\n    fi;;\nesac\nexec '{real}' \"$@\"\n",
        entered = entered.display(),
        real = crate::test_support::REAL_GIT,
        competing = competing,
        alive = while_test_lives(world.root.as_path()),
    );
    std::fs::write(&program, script.as_str()).expect("the program");
    std::fs::set_permissions(program.as_path(), std::fs::Permissions::from_mode(0o755)).expect("executable");
    let git_context: GitContext = GitContext { real_git: program, ..world.context.git.clone() };
    let kept: Option<String> = with_a_short_wait(|| { return keep_on_a_side_ref(&git_context, world.repo.as_path(), kept_ref(1).as_str(), requested.as_str()); });
    let actual: String = git_text(world.repo.as_path(), &["rev-parse", "refs/cli-git/kept/1"]);
    git(world.repo.as_path(), &["gc", "--prune=now"]);
    let surviving: std::process::Output = crate::test_support::git_output(world.repo.as_path(), &["cat-file", "-e", competing.as_str()]);
    remove(world.root.as_path());
    assert_eq!(actual, competing, "the reference another command wrote keeps its commit, not the one this write asked for");
    assert!(surviving.status.success(), "the competing commit survives a maintenance that drops what no reference names");
    assert!(kept.is_some(), "the write the name refused is answered with Git's refusal: {kept:?}");
}

/// What:
///  A write Git refused is not answered by what a symbolic reference at the same name leads to.
/// Why:
///  Every write of a side reference is `--no-deref`,
///  so a symbolic reference at such a name is not one of those writes.
///  A read that followed the symbolic reference answered a refused write as written,
///  and a caller that trusted that answer left the commit held by nothing this command wrote:
///  moving the reference the target names takes the commit with it
///  (the owner's rule "never discard user data").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await keepOnASideRef(repo, 1, commit)).not.toBeUndefined(); expect(symbolicRef('refs/cli-git/kept/1')).toBe('refs/temporary/target')
/// ```
#[test]
fn a_refused_write_is_not_answered_by_what_a_symbolic_reference_leads_to() {
    let world: World = world("service-land-side-ref-symbolic");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    let commit: String = built.commit.clone().expect("the commit Git built");
    git(world.repo.as_path(), &["-c", "core.logAllRefUpdates=false", "update-ref", "refs/temporary/target", commit.as_str()]);
    git(world.repo.as_path(), &["symbolic-ref", "refs/cli-git/kept/1", "refs/temporary/target"]);
    let lock: PathBuf = world.repo.join(".git/refs/cli-git/kept/1.lock");
    std::fs::File::create(lock.as_path()).expect("the lock that refuses the direct write");
    let kept: Option<String> = keep_on_a_side_ref(&world.context.git, world.repo.as_path(), kept_ref(1).as_str(), commit.as_str());
    let still: String = git_text(world.repo.as_path(), &["symbolic-ref", "refs/cli-git/kept/1"]);
    std::fs::remove_file(lock.as_path()).expect("the holder lets go");
    remove(world.root.as_path());
    assert_eq!(still, "refs/temporary/target", "the symbolic reference is unchanged, so the write did not happen");
    assert!(kept.is_some(), "the write was refused, so the answer is Git's refusal: {kept:?}");
}

/// What:
///  A symbolic reference at the kept name does not let the batch's scratch reference go.
/// Why:
///  The commit is held by `refs/worktree/cli-git-service/<seq>`,
///  and a symbolic `refs/cli-git/kept/<seq>` that leads to that same commit is not a reference this wrapper wrote:
///  the command that moves the reference it names takes the commit with it,
///  so the removal keeps the scratch reference and the record beside it
///  (the owner's rule "never discard user data").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await clearAfterDroppedBatch(repo, 1)).toEqual('left'); expect(revParse(scratchRef(1))).toBe(commit)
/// ```
#[test]
fn a_symbolic_kept_reference_does_not_let_the_scratch_reference_go() {
    let world: World = world("service-land-side-ref-symbolic-clearing");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    let commit: String = built.commit.clone().expect("the commit Git built");
    let scratch: String = scratch_ref(1);
    git(world.repo.as_path(), &["-c", "core.logAllRefUpdates=false", "update-ref", "--no-deref", scratch.as_str(), commit.as_str()]);
    git(world.repo.as_path(), &["-c", "core.logAllRefUpdates=false", "update-ref", "refs/temporary/target", commit.as_str()]);
    git(world.repo.as_path(), &["symbolic-ref", "refs/cli-git/kept/1", "refs/temporary/target"]);
    let removal: ScratchRefRemoval = clear_after_dropped_batch(&world.context.git, world.repo.as_path(), 1);
    let scratch_read: std::process::Output = crate::test_support::git_output(world.repo.as_path(), &["rev-parse", "--verify", "--quiet", scratch.as_str()]);
    let still: String = String::from_utf8_lossy(scratch_read.stdout.as_slice()).trim().to_string();
    remove(world.root.as_path());
    assert_eq!(removal, ScratchRefRemoval::Left, "a symbolic reference is not a reference that keeps the commit");
    assert_eq!(still, commit, "the scratch reference still holds the batch's commit");
}

/// What:
///  A kept reference another command wrote between the read and the write after it is not replaced.
/// Why:
///  The removal reads `refs/cli-git/kept/<seq>` as absent and then writes it;
///  a command that kept its own commit there in between made a reference that is the only thing naming that commit,
///  so a write that replaces it makes `git gc` take a commit the person had
///  (the owner's rule "never discard user data").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeKeptAfterTheRead({ commit: competing }); expect(await clearAfterDroppedBatch(repo, 1)).toEqual('left'); expect(revParse(keptRef(1))).toBe(competing)
/// ```
#[test]
fn a_kept_reference_another_command_wrote_after_the_read_is_not_replaced() {
    let world: World = world("service-land-side-ref-kept-written-in-between");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let competing_built: CommitRequest = built_request(&world, "competing", "competing.txt", &[base.as_str()]);
    let competing: String = competing_built.commit.clone().expect("the competing commit");
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    let commit: String = built.commit.clone().expect("the commit Git built");
    let scratch: String = scratch_ref(1);
    git(world.repo.as_path(), &["-c", "core.logAllRefUpdates=false", "update-ref", "--no-deref", scratch.as_str(), commit.as_str()]);
    let entered: PathBuf = world.root.join("kept-read-answered");
    let program: PathBuf = world.root.join("git-that-writes-the-kept-name-after-the-read");
    let script: String = format!(
        "#!/bin/sh\ncase \" $* \" in\n  *\" rev-parse --verify --quiet refs/cli-git/kept/1 \"*)\n    if [ ! -e '{entered}' ]; then\n      : > '{entered}'\n      '{real}' \"$@\"\n      answered=$?\n      '{real}' -c core.logAllRefUpdates=false update-ref refs/cli-git/kept/1 '{competing}'\n      exit $answered\n    fi;;\nesac\nexec '{real}' \"$@\"\n",
        entered = entered.display(),
        real = crate::test_support::REAL_GIT,
        competing = competing,
    );
    std::fs::write(&program, script.as_str()).expect("the program");
    std::fs::set_permissions(program.as_path(), std::fs::Permissions::from_mode(0o755)).expect("executable");
    let git_context: GitContext = GitContext { real_git: program, ..world.context.git.clone() };
    let removal: ScratchRefRemoval = clear_after_dropped_batch(&git_context, world.repo.as_path(), 1);
    let kept_still: String = git_text(world.repo.as_path(), &["rev-parse", kept_ref(1).as_str()]);
    git(world.repo.as_path(), &["gc", "--prune=now"]);
    let surviving: std::process::Output = crate::test_support::git_output(world.repo.as_path(), &["cat-file", "-e", competing.as_str()]);
    remove(world.root.as_path());
    assert_eq!(removal, ScratchRefRemoval::Left, "the kept name is taken, so nothing proven holds the commit and the scratch reference stays");
    assert_eq!(kept_still, competing, "the kept reference another command wrote is not replaced");
    assert!(surviving.status.success(), "the commit that other command kept survives a maintenance that drops what no reference names");
}

/// What:
///  A commit Git built survives a maintenance that drops what no reference names,
///  even though the write of the batch's own side reference was still running when this command stopped waiting for it.
/// Why:
///  The landing answers such a batch with a failure,
///  and nothing else names the commit it built:
///  a commit that is on no branch and that no reference names is one `git gc --prune=now` takes
///  (the owner's rule "never discard user data").
///  The commit is kept on the reference a commit that did not land is kept on,
///  and the person is told where it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await landBatchWithAGitStuckOnTheScratchReference(...)).toEqual({ error: 'unavailable' });
/// expect(revParse(keptRef(1))).toBe(commit); expect(catFileExists(commit)).toBe(true); expect(announced).toContain('refs/cli-git/kept/1')
/// ```
#[test]
fn a_built_commit_survives_a_prune_when_the_write_of_its_scratch_reference_is_left_running() {
    let mut world: World = world("service-land-side-ref-scratch-left-running");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    let commit: String = built.commit.clone().expect("the commit Git built");
    let program: PathBuf = world.root.join("git-stuck-on-the-scratch-reference");
    make_the_side_reference_directories(&world);
    let script: String = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{invocations}'\ncase \" $* \" in\n  *\" update-ref --no-deref refs/worktree/cli-git-service/\"*)\n    while {alive}; do sleep 0.05; done;;\n  *\" update-ref --no-deref refs/cli-git/kept/\"*)\n    shift 4\n    printf '%s\\n' \"$2\" > '{repository}/.git/'\"$1\" || exit 1\n    exit 0;;\nesac\nexec '{real}' \"$@\"\n",
        invocations = side_ref_git_invocations(&world).display(),
        alive = while_test_lives(world.root.as_path()),
        repository = world.repo.display(),
        real = crate::test_support::REAL_GIT,
    );
    std::fs::write(&program, script.as_str()).expect("the program");
    std::fs::set_permissions(program.as_path(), std::fs::Permissions::from_mode(0o755)).expect("executable");
    world.context.git.real_git = program;
    let announced: Announced = Announced(RefCell::new(Vec::new()));
    let landed: Result<Vec<Outcome>, FailureCode> = crate::landing_state_carried_by_its_task::in_a_landing(|| {
        return with_a_short_wait(|| { return land_batch_announcing(&world.context, 1, std::slice::from_ref(&built), &announced); });
    });
    let kept: String = kept_ref(1);
    let kept_read: std::process::Output = crate::test_support::git_output(world.repo.as_path(), &["rev-parse", "--verify", "--quiet", kept.as_str()]);
    let kept_names: String = String::from_utf8_lossy(kept_read.stdout.as_slice()).trim().to_string();
    git(world.repo.as_path(), &["gc", "--prune=now"]);
    let surviving: std::process::Output = crate::test_support::git_output(world.repo.as_path(), &["cat-file", "-e", commit.as_str()]);
    let lines: Vec<String> = announced.0.borrow().clone();
    let asked: String = invocations_of(&world);
    remove(world.root.as_path());
    assert_eq!(landed.err(), Some(FailureCode::Unavailable), "the batch is not made, and it says so; the Git asked was:\n{asked}");
    assert_eq!(kept_names, commit, "the commit the batch built is kept where a commit that did not land is kept");
    assert!(surviving.status.success(), "the commit survives a maintenance that drops what no reference names");
    assert!(lines.iter().any(|line| { return line.contains(kept.as_str()); }), "the person is told where the commit is kept: {lines:?}");
}

/// What:
///  The scratch reference another command wrote between the read and the removal is not removed.
/// Why:
///  The removal read the reference as naming the batch's commit and proved that commit kept elsewhere;
///  a command that wrote its own commit there in between made a reference that is the only thing naming it,
///  so a removal that deletes the name anyway makes `git gc` take that commit
///  (the owner's rule "never discard user data").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeScratchAfterTheKeptRead({ commit: competing }); expect(await clearAfterDroppedBatch(repo, 1)).toEqual('left'); expect(revParse(scratchRef(1))).toBe(competing)
/// ```
#[test]
fn a_scratch_reference_another_command_wrote_after_the_read_is_not_removed() {
    let world: World = world("service-land-side-ref-scratch-written-in-between");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let competing_built: CommitRequest = built_request(&world, "competing", "competing.txt", &[base.as_str()]);
    let competing: String = competing_built.commit.clone().expect("the competing commit");
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    let commit: String = built.commit.clone().expect("the commit Git built");
    let scratch: String = scratch_ref(1);
    let kept: String = kept_ref(1);
    git(world.repo.as_path(), &["-c", "core.logAllRefUpdates=false", "update-ref", "--no-deref", scratch.as_str(), commit.as_str()]);
    git(world.repo.as_path(), &["-c", "core.logAllRefUpdates=false", "update-ref", "--no-deref", kept.as_str(), commit.as_str()]);
    let entered: PathBuf = world.root.join("kept-read-answered");
    let program: PathBuf = world.root.join("git-that-writes-the-scratch-name-after-the-read");
    let script: String = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{invocations}'\ncase \" $* \" in\n  *\" rev-parse --verify --quiet refs/cli-git/kept/1 \"*)\n    if [ ! -e '{entered}' ]; then\n      : > '{entered}'\n      '{real}' \"$@\"\n      answered=$?\n      '{real}' -c core.logAllRefUpdates=false update-ref --no-deref refs/worktree/cli-git-service/1 '{competing}'\n      exit $answered\n    fi;;\nesac\nexec '{real}' \"$@\"\n",
        invocations = side_ref_git_invocations(&world).display(),
        entered = entered.display(),
        real = crate::test_support::REAL_GIT,
        competing = competing,
    );
    std::fs::write(&program, script.as_str()).expect("the program");
    std::fs::set_permissions(program.as_path(), std::fs::Permissions::from_mode(0o755)).expect("executable");
    let git_context: GitContext = GitContext { real_git: program, ..world.context.git.clone() };
    let removal: ScratchRefRemoval = clear_after_dropped_batch(&git_context, world.repo.as_path(), 1);
    let scratch_read: std::process::Output = crate::test_support::git_output(world.repo.as_path(), &["rev-parse", "--verify", "--quiet", scratch.as_str()]);
    let scratch_still: String = String::from_utf8_lossy(scratch_read.stdout.as_slice()).trim().to_string();
    git(world.repo.as_path(), &["gc", "--prune=now"]);
    let surviving: std::process::Output = crate::test_support::git_output(world.repo.as_path(), &["cat-file", "-e", competing.as_str()]);
    remove(world.root.as_path());
    assert_eq!(removal, ScratchRefRemoval::Left, "the scratch name is taken, so it is not removed");
    assert_eq!(scratch_still, competing, "the scratch reference another command wrote is not removed");
    assert!(surviving.status.success(), "the commit that other command wrote survives a maintenance that drops what no reference names");
}
/// What:
///  The side reference a landing keeps its batch on is written by both faces the same way,
///  and the
///  write runs inside the runtime.
/// Why:
///  The reference is what keeps a commit reachable while its branch write meets a lock,
/// so a face that
///  failed to write it would leave a commit no reference holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.equal(awaited, synchronous); assert.equal(git(repo, ['rev-parse', name]).trim(), tip);
/// ```
#[test]
fn the_awaited_side_reference_write_keeps_what_the_synchronous_one_keeps() {
    let root: PathBuf = fixture("land-steps-awaiting-side-ref");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let tip: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let scratch: String = crate::service_scratch_ref_of_a_dropped_batch::scratch_ref(1);
    let synchronous: ASideRefWrite = keep_on_the_scratch_reference(&context, repo.as_path(), scratch.as_str(), tip.as_str());
    assert_eq!(synchronous, ASideRefWrite::Written, "the synchronous face writes the batch's side reference");
    assert_eq!(
        git_text(repo.as_path(), &["rev-parse", "--verify", "--quiet", scratch.as_str()]),
        tip,
        "the synchronous reference names the batch's commit"
    );
    // The reference goes, so the awaited face writes the name nothing holds rather than reading one the synchronous face made.
    git(repo.as_path(), &["update-ref", "-d", scratch.as_str()]);
    let awaited: ASideRefWrite =
        crate::lazy_runtime::run_on_runtime(keep_on_the_scratch_reference_awaiting(&context, repo.as_path(), scratch.as_str(), tip.as_str())).expect("the runtime");
    assert_eq!(awaited, synchronous, "the two faces answer the write of a side reference the same way");
    assert_eq!(
        git_text(repo.as_path(), &["rev-parse", "--verify", "--quiet", scratch.as_str()]),
        tip,
        "the reference the awaited face wrote names the batch's commit"
    );
    remove(root.as_path());
}

/// What:
///  The line that tells a person where their commit was kept is the one both faces write,
///  and the reference
///  it names holds the batch's commit.
/// Why:
///  A landing that ended before its branch write leaves a commit no branch names,
/// and this line is what hands it to
///  the person;
/// a face that kept the commit at another name,
/// or kept nothing and said it did,
/// loses the commit the
///  wrapper promised to keep (the owner's rule is "never discard user data").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = keepUnlanded(git, repo, seq, tip); git(repo, ['update-ref', '-d', refs/kept/seq]); assert.equal(await keepUnlandedAwaiting(git, repo, seq, tip), line);
/// ```
#[test]
fn the_awaited_kept_commit_is_the_one_the_synchronous_line_names() {
    const BATCH: u64 = 7;
    let root: PathBuf = fixture("land-steps-awaiting-kept");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let tip: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let kept: String = crate::service_scratch_ref_of_a_dropped_batch::kept_ref(BATCH);
    let scratch: String = crate::service_scratch_ref_of_a_dropped_batch::scratch_ref(BATCH);
    let synchronous: AKeptCommit = keep_unlanded(&context, repo.as_path(), BATCH, scratch.as_str(), tip.as_str());
    assert!(synchronous.named, "the synchronous face proves another reference names the commit");
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "--verify", "--quiet", kept.as_str()]), tip, "the synchronous line names a reference the commit is kept on");
    // The reference goes, so the awaited face keeps the commit on a name nothing holds yet.
    git(repo.as_path(), &["update-ref", "-d", kept.as_str()]);
    let awaited: AKeptCommit = crate::lazy_runtime::run_on_runtime(keep_unlanded_awaiting(&context, repo.as_path(), BATCH, scratch.as_str(), tip.as_str())).expect("the runtime");
    assert_eq!(awaited.line, synchronous.line, "both faces tell the person one thing about where their commit is kept");
    assert_eq!(awaited.named, synchronous.named, "and both prove the keep the same way");
    assert_eq!(
        git_text(repo.as_path(), &["rev-parse", "--verify", "--quiet", kept.as_str()]),
        tip,
        "the reference the awaited face wrote names the batch's commit"
    );
    remove(root.as_path());
}
