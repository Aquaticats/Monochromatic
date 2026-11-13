//! What:
//!  What a landing does when the write of its batch's own scratch reference is not proven:
//!  the commit is made as real Git makes it,
//!  the person is told once that the extra protection was not in place,
//!  and a commit whose reference the write did prove is told nothing new.
//! Why:
//!  The project decided on 2026-11-12 that a commit is made and the person is told
//!  (`doc/handover/cli-git-rust-implementation.md`,
//!  the section of 2026-11-12T10:27:45-05:00,
//!  item 1.5:
//!  "Commit and say so"),
//!  because real Git needs no such reference and the commit it makes is the same commit.
//!  These controls show each way the write can fail to be proven,
//!  and that the commit survives a maintenance that drops every object no reference names
//!  whatever the answer was.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! plantTheRefusal(); expect(await landBatch(built)).toEqual([{ landed }]); expect(announced()).toContain('your commit was made'); expect(catFile(commit)).toBe('commit');
//! ```

use super::*;
use super::tests::{World, built_request, request, world};
use crate::service_index_lock::{Announce, Announcer};
use crate::service_scratch_ref_of_a_dropped_batch::{kept_ref, scratch_ref};
use crate::test_support::{git, git_output, git_text, remove, while_test_lives};
use crate::transaction_git::{EXTRA_WAIT_FOR_THIS_THREAD, STEP_CEILING_FOR_THIS_THREAD};
use std::cell::RefCell;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// What:
///  The lines a landing announces to its clients,
/// kept so a control can read them.
/// Why:
///  The line a person reads about a commit made without its reference is announced over the batch's streams,
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
const ONE_SECOND: Duration = Duration::from_secs(1);

/// The short extra wait a step that is left running gets in these controls:
/// a tenth of a second,
/// because the subject is that the wait ends before the write does,
/// never how much room the write then has.
const A_TENTH: Duration = Duration::from_millis(100);

/// How old the lock these controls plant is:
/// older than the margin a reference lock is judged at,
/// so the landing judges it at once and holds the batch instead of waiting on it,
/// as a lock a killed Git left is.
const LOCK_AGE: Duration = Duration::from_secs(120);

/// The words a line about a commit made without its own reference carries,
/// which is what a control that must see no such line looks for.
const THE_LINE: &str = "your commit was made";

/// Run `run` with the shortened bound of these controls,
/// and give the thread its shipped bound back afterwards,
/// whatever `run` did.
fn with_a_short_wait<T>(run: impl FnOnce() -> T) -> T {
    STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(Some(ONE_SECOND)); });
    EXTRA_WAIT_FOR_THIS_THREAD.with(|cell| { cell.set(Some(A_TENTH)); });
    let ended: T = run();
    EXTRA_WAIT_FOR_THIS_THREAD.with(|cell| { cell.set(None); });
    STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(None); });
    return ended;
}

/// Lands one batch of one commit Git built,
/// with every line it announces,
/// and answers what the landing said and the lines.
fn land_one_built(world: &World, built: &CommitRequest, short_wait: bool) -> (Result<Vec<Outcome>, FailureCode>, Vec<String>) {
    let announced: Announced = Announced(RefCell::new(Vec::new()));
    let landed: Result<Vec<Outcome>, FailureCode> = crate::landing_state_carried_by_its_task::in_a_landing(|| {
        return match short_wait {
            true => with_a_short_wait(|| { return land_batch_announcing(&world.context, 1, std::slice::from_ref(built), &announced); }),
            false => land_batch_announcing(&world.context, 1, std::slice::from_ref(built), &announced),
        };
    });
    let lines: Vec<String> = announced.0.borrow().clone();
    return (landed, lines);
}

/// The commit the batch built, as the single landed outcome names it,
/// and the lines said about it.
fn landed_commit(landed: &Result<Vec<Outcome>, FailureCode>, lines: &[String]) -> String {
    match landed {
        Ok(outcomes) => match outcomes.as_slice() {
            [Outcome::Landed { commit }] => return commit.clone(),
            other => panic!("the commit is made: {other:?}; the lines were {lines:?}"),
        },
        Err(code) => panic!("the commit is made, and the landing answered {code:?}; the lines were {lines:?}"),
    }
}

/// What:
///  The commit of a batch whose reference a lock keeps from being written is made,
///  and the person is told once what the missing reference exposed.
/// Why:
///  This is the state a repository on a host that is slow to write a reference leaves,
///  and real Git would have made the commit:
///  a lock standing on `refs/worktree/cli-git-service/<batch>` refuses the write,
///  the landing lands the batch as Git lands it,
///  and the line says the commit was made and what the window was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plant('refs/worktree/cli-git-service/1.lock'); expect(landed.message).toBe('the commit is made'); expect(lines.join('')).toContain('your commit was made')
/// ```
#[test]
fn a_commit_whose_reference_a_lock_keeps_from_being_written_is_made() {
    let world: World = world("service-land-without-its-reference-locked");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    std::fs::create_dir_all(world.repo.join(".git/refs/worktree/cli-git-service")).expect("the reference's folder");
    let lock: PathBuf = world.repo.join(".git/refs/worktree/cli-git-service/1.lock");
    std::fs::File::create(lock.as_path()).expect("the lock another writer holds");
    let (landed, lines) = land_one_built(&world, &built, false);
    let commit: String = landed_commit(&landed, lines.as_slice());
    let branch: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    git(world.repo.as_path(), &["gc", "--prune=now"]);
    let surviving: std::process::Output = git_output(world.repo.as_path(), &["cat-file", "-e", commit.as_str()]);
    let said: String = lines.join("");
    remove(world.root.as_path());
    assert_eq!(branch, commit, "the branch names the commit the batch made");
    assert!(surviving.status.success(), "the commit survives a maintenance that drops what no reference names");
    assert!(said.contains(THE_LINE), "the person is told the commit was made: {said}");
    assert!(said.contains("refs/worktree/cli-git-service/1"), "the line names the reference that was not written: {said}");
    assert!(!said.contains("was not made"), "nothing says the commit was not made: {said}");
}

/// What:
///  A name another command wrote is neither handed to Git to replace nor deleted by the branch write,
///  and the commit that command wrote stays named.
/// Why:
///  The reference holds a commit that nothing else names,
///  so a write that replaced it,
///  and a branch write that removed it,
///  would each leave that commit to a `git gc --prune=now`
///  (the owner's rule "never discard user data").
///  The batch still lands,
///  and its own commit is named by its branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeScratch('refs/worktree/cli-git-service/1', competing); expect(landed.message).toBe('the commit is made'); expect(revParse(scratch)).toBe(competing); expect(catFile(competing)).toBe('commit')
/// ```
#[test]
fn a_reference_another_command_wrote_is_neither_replaced_nor_removed() {
    let world: World = world("service-land-without-its-reference-taken");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let competing_built: CommitRequest = built_request(&world, "competing", "competing.txt", &[base.as_str()]);
    let competing: String = competing_built.commit.clone().expect("the competing commit");
    let scratch: String = scratch_ref(1);
    git(world.repo.as_path(), &["-c", "core.logAllRefUpdates=false", "update-ref", "--no-deref", scratch.as_str(), competing.as_str()]);
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    let (landed, lines) = land_one_built(&world, &built, false);
    let commit: String = landed_commit(&landed, lines.as_slice());
    let still: String = git_text(world.repo.as_path(), &["rev-parse", scratch.as_str()]);
    git(world.repo.as_path(), &["gc", "--prune=now"]);
    let surviving: std::process::Output = git_output(world.repo.as_path(), &["cat-file", "-e", competing.as_str()]);
    let said: String = lines.join("");
    remove(world.root.as_path());
    assert_ne!(commit, competing, "the batch made its own commit");
    assert_eq!(still, competing, "the reference another command wrote is untouched");
    assert!(surviving.status.success(), "the commit that reference names survives a maintenance that drops what no reference names");
    assert!(said.contains(THE_LINE), "the person is told the commit was made: {said}");
}

/// What:
///  A reference store that refuses the write for a storage reason leaves the commit made.
/// Why:
///  A folder this account may not write is a filesystem that answers every write with a refusal,
///  and real Git needs no reference there to make the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// chmod('refs/worktree', 0o500); expect(landed.message).toBe('the commit is made'); expect(announced()).toContain('your commit was made')
/// ```
#[test]
fn a_reference_store_that_refuses_the_write_leaves_the_commit_made() {
    let world: World = world("service-land-without-its-reference-folder-refused");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    let references: PathBuf = world.repo.join(".git/refs/worktree");
    std::fs::create_dir_all(references.as_path()).expect("the reference's folder");
    std::fs::set_permissions(references.as_path(), std::fs::Permissions::from_mode(0o500)).expect("a folder that refuses writes");
    let (landed, lines) = land_one_built(&world, &built, false);
    std::fs::set_permissions(references.as_path(), std::fs::Permissions::from_mode(0o755)).expect("the folder takes writes again");
    let commit: String = landed_commit(&landed, lines.as_slice());
    let branch: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let said: String = lines.join("");
    remove(world.root.as_path());
    assert_eq!(branch, commit, "the branch names the commit the batch made");
    assert!(said.contains(THE_LINE), "the person is told the commit was made: {said}");
}

/// What:
///  A write whose wait ended before the write did leaves the commit made.
/// Why:
///  The wrapper never ends such a write for being slow,
///  so the wait can end first and the reference then shows nothing:
///  the two attempts of the write cost the person nothing,
///  and the commit is made and told about as Git would have made it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// slowGit({ on: 'update-ref --no-deref refs/worktree/cli-git-service/' }); expect(landed.message).toBe('the commit is made'); expect(announced()).toContain('your commit was made')
/// ```
#[test]
fn a_write_whose_wait_ended_before_the_write_did_leaves_the_commit_made() {
    let mut world: World = world("service-land-without-its-reference-left-running");
    // The repository names a shared-file mode, even the one that shares nothing, which is one of the settings whose write the
    //  landing leaves to Git (`side_reference_written_here.rs`): this control is about the wait for Git's write of the batch's
    //  own reference, so that write must be Git's and not the landing's own.
    git(world.repo.as_path(), &["config", "core.sharedRepository", "false"]);
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    let commit: String = built.commit.clone().expect("the commit Git built");
    let program: PathBuf = world.root.join("git-stuck-on-the-batch-reference");
    let script: String = format!(
        "#!/bin/sh\ncase \" $* \" in\n  *\" update-ref --no-deref refs/worktree/cli-git-service/\"*) while {alive}; do sleep 0.05; done;;\nesac\nexec '{real}' \"$@\"\n",
        alive = while_test_lives(world.root.as_path()),
        real = crate::test_support::REAL_GIT,
    );
    std::fs::write(program.as_path(), script.as_str()).expect("the program");
    std::fs::set_permissions(program.as_path(), std::fs::Permissions::from_mode(0o755)).expect("executable");
    world.context.git.real_git = program;
    let (landed, lines) = land_one_built(&world, &built, true);
    assert_eq!(landed_commit(&landed, lines.as_slice()), commit, "the commit the wait left standing is the one the batch made");
    let branch: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    git(world.repo.as_path(), &["gc", "--prune=now"]);
    let surviving: std::process::Output = git_output(world.repo.as_path(), &["cat-file", "-e", commit.as_str()]);
    let said: String = lines.join("");
    remove(world.root.as_path());
    assert_eq!(branch, commit, "the branch names the commit the batch made");
    assert!(surviving.status.success(), "the commit survives a maintenance that drops what no reference names");
    assert!(said.contains(THE_LINE), "the person is told the commit was made: {said}");
    assert!(said.contains("stopped waiting for the write"), "the line says which wait ended: {said}");
}

/// What:
///  A batch whose import could not write its own reference lands the commits it made.
/// Why:
///  `fast-import` writes the commits and updates the batch's reference last,
///  so a marks file naming every commit is a batch whose objects the repository holds and whose reference it
///  refused.
///  Before 2026-11-12 such a batch was answered as not made;
///  now it is made as real Git makes it,
///  and the person is told once what the missing reference exposed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// chmod('refs/worktree', 0o500); expect(await landBatch(changes)).toEqual([{ landed }]); expect(revParse('HEAD')).toBe(landed.commit)
/// ```
#[test]
fn a_batch_whose_import_could_not_write_its_reference_lands_its_commits() {
    let world: World = world("service-land-without-its-reference-import-refused");
    let change: CommitRequest = request(&world, "typed", "typed.txt", "the change a person made\n");
    let references: PathBuf = world.repo.join(".git/refs/worktree");
    std::fs::create_dir_all(references.as_path()).expect("the reference's folder");
    std::fs::set_permissions(references.as_path(), std::fs::Permissions::from_mode(0o500)).expect("a folder that refuses writes");
    let announced: Announced = Announced(RefCell::new(Vec::new()));
    let landed: Result<Vec<Outcome>, FailureCode> = crate::landing_state_carried_by_its_task::in_a_landing(|| {
        return land_batch_announcing(&world.context, 1, std::slice::from_ref(&change), &announced);
    });
    std::fs::set_permissions(references.as_path(), std::fs::Permissions::from_mode(0o755)).expect("the folder takes writes again");
    let commit: String = landed_commit(&landed, announced.0.borrow().as_slice());
    let branch: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    git(world.repo.as_path(), &["gc", "--prune=now"]);
    let surviving: std::process::Output = git_output(world.repo.as_path(), &["cat-file", "-e", commit.as_str()]);
    let said: String = announced.0.borrow().join("");
    remove(world.root.as_path());
    assert_eq!(branch, commit, "the branch names the commit the import made");
    assert!(surviving.status.success(), "the commit survives a maintenance that drops what no reference names");
    assert!(said.contains(THE_LINE), "the person is told the commit was made: {said}");
}

/// What:
///  A commit whose reference the write did prove is told nothing new.
/// Why:
///  The line exists for the batch whose protection was missing;
///  a batch that has it lands as it did before,
///  with no line about a window that never opened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await landBatch(built)).toEqual([{ landed }]); expect(announced().join('')).not.toContain('your commit was made')
/// ```
#[test]
fn a_commit_whose_reference_the_write_proved_is_told_nothing_new() {
    let world: World = world("service-land-without-its-reference-proven");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    let (landed, lines) = land_one_built(&world, &built, false);
    let commit: String = landed_commit(&landed, lines.as_slice());
    let branch: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let said: String = lines.join("");
    remove(world.root.as_path());
    assert_eq!(branch, commit, "the branch names the commit the batch made");
    assert!(!said.contains(THE_LINE), "nothing is said about a protection that was in place: {said}");
}

/// What:
///  A commit held behind a reference lock,
///  whose own reference was never written,
///  is kept on the name a commit that did not land is kept on.
/// Why:
///  The branch has not moved yet,
///  so the commit is named by nothing the batch holds:
///  a `git gc --prune=now` before the lock goes would take it,
///  and the settlement that moves the branch would then have no commit to move.
///  The kept reference names it,
///  and the person is told both that the commit is held and that the extra protection was missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plantOld('refs/heads/main.lock'); plant('refs/worktree/cli-git-service/1.lock'); expect(outcomes).toEqual([{ held }]); expect(revParse(keptRef(1))).toBe(commit); expect(catFile(commit)).toBe('commit')
/// ```
#[test]
fn a_held_commit_whose_reference_was_never_written_is_kept_on_its_kept_name() {
    let world: World = world("service-land-without-its-reference-held");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    let commit: String = built.commit.clone().expect("the commit Git built");
    std::fs::create_dir_all(world.repo.join(".git/refs/worktree/cli-git-service")).expect("the reference's folder");
    std::fs::File::create(world.repo.join(".git/refs/worktree/cli-git-service/1.lock").as_path()).expect("the lock another writer holds");
    let lock: PathBuf = world.repo.join(".git/refs/heads/main.lock");
    let planted: std::fs::File = std::fs::File::create(lock.as_path()).expect("the lock a killed Git leaves");
    planted.set_modified(SystemTime::now() - LOCK_AGE).expect("the lock's age");
    let (landed, lines) = land_one_built(&world, &built, false);
    let outcomes: Vec<Outcome> = landed.expect("the commit is made and held");
    assert!(matches!(outcomes.as_slice(), [Outcome::Held { .. }]), "{outcomes:?}");
    let kept: String = git_text(world.repo.as_path(), &["rev-parse", kept_ref(1).as_str()]);
    git(world.repo.as_path(), &["gc", "--prune=now"]);
    let surviving: std::process::Output = git_output(world.repo.as_path(), &["cat-file", "-e", commit.as_str()]);
    let said: String = lines.join("");
    std::fs::remove_file(lock.as_path()).expect("the holder lets go");
    crate::service_durable::drain();
    remove(world.root.as_path());
    assert_eq!(kept, commit, "the commit held behind the lock is kept on its kept reference");
    assert!(surviving.status.success(), "the held commit survives a maintenance that drops what no reference names");
    assert!(said.contains(THE_LINE), "the person is told the commit was made: {said}");
}

/// What:
///  A batch that will not land says nothing about a commit it did not make.
/// Why:
///  The line about a missing reference belongs to a commit that was made;
///  a branch write no reference store accepts leaves the commit unlanded,
///  and the ending that answers the batch names the commit where a commit that did not land is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refuseEveryReferenceWrite(); expect(landed.error).toBe('reference-locked'); expect(announced().join('')).not.toContain('your commit was made')
/// ```
#[test]
fn a_batch_that_will_not_land_says_nothing_about_a_commit_it_did_not_make() {
    let mut world: World = world("service-land-without-its-reference-not-landing");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    let program: PathBuf = world.root.join("git-that-refuses-reference-writes");
    let script: String = format!(
        "#!/bin/sh\ncase \" $* \" in\n  *\" update-ref \"*) printf '%s\\n' 'fatal: cannot lock ref: the stand-in refuses every reference write' >&2; exit 1;;\nesac\nexec '{real}' \"$@\"\n",
        real = crate::test_support::REAL_GIT
    );
    std::fs::write(program.as_path(), script.as_str()).expect("the program");
    std::fs::set_permissions(program.as_path(), std::fs::Permissions::from_mode(0o755)).expect("executable");
    world.context.git.real_git = program;
    let (landed, lines) = land_one_built(&world, &built, false);
    let said: String = lines.join("");
    remove(world.root.as_path());
    assert_eq!(landed.err(), Some(FailureCode::ReferenceLocked), "the branch write Git refused answers the batch; the lines were {said}");
    assert!(!said.contains(THE_LINE), "no line says a commit that did not land was made: {said}");
}
