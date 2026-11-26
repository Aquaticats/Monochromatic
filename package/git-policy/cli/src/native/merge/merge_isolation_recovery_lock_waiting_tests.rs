//! What:
//!  What a recovery does about `index.lock`:
//!  the wait for another program's lock and the line that says what it waits for,
//!  the three answers a held lock gets,
//!  the clearing of a lock a killed Git left,
//!  and a saved merge that a live holder defers at once.
//! Why:
//!  One held lock used to end every command in the worktree with status 2,
//!  and a recovery that wrote the index after giving up would put Git's own sentence
//!  in place of the record.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(mayInstall(place, { kind: 'owner-gone' })).toBe(true); expect(exists(lock)).toBe(false);
//! ```

use super::super::*;
use super::fixtures::{
    World,
    dead_owner,
    deferred_record_of,
    place,
    record_of,
    world,
};
use super::readings::{
    snapshot,
    the_action_for,
};
use crate::index_holder_wait::IndexWait;
use crate::merge_isolation_journal::parse_merge_intent;
use crate::merge_isolation_recovery_index_lock::may_install;
/// The three facts that decide what a command says about an index it cannot write.
use crate::merge_isolation_recovery_steps::IndexHeldAt;
use crate::test_support::{
    git,
    git_text,
    remove,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  A writer that lets another program's `index.lock` go the first time the wait says what it waits for,
///  and keeps everything the wait wrote.
/// Why:
///  The control must not sleep until a lock is released:
/// the notice of the wait is the instant the holder is known to be
///  waited for (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ReleasingSink { write(bytes) { this.said += bytes; if (exists(lock)) rm(lock); return bytes.length; } }
/// ```
struct ReleasingSink<'a> {
    /// The lock the other program holds.
    lock: &'a Path,
    /// Everything the wait wrote.
    said: Vec<u8>,
}

impl std::io::Write for ReleasingSink<'_> {
    fn write(
        &mut self,
        bytes: &[u8],
    ) -> std::io::Result<usize> {
        self.said.extend_from_slice(bytes);
        // A wait that has said what it waits for is waiting; the other program lets go at that instant.
        if self.lock.exists() {
            std::fs::remove_file(self.lock)?;
        }
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// What:
///  A recovery waits for another program's `index.lock`,
///  says what it waits for,
///  and finishes the merge when the lock is free.
/// Why:
///  `git update-index` dies on a held lock,
/// so one held lock used to end every command in the worktree with
///  status 2 until it was gone (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sink = new ReleasingSink(lock); await recoverInterruptedMerges(place, dir, sink); expect(outcome.action).toBe('completed');
/// ```
#[test]
fn recovery_waits_for_another_programs_index_lock() {
    let world: World = world("merge-recovery-waits-for-lock");
    git(
        world.repo.as_path(),
        &[
            "read-tree",
            "-m",
            "-u",
            world.old_oid.as_str(),
            world.new_oid.as_str(),
        ],
    );
    let _record: PathBuf = record_of(
        &world,
        dead_owner("12121212-0000-4000-8000-000000000012"),
        Some("refs/heads/main"),
    );
    let lock: PathBuf = world.git_dir.join("index.lock");
    std::fs::write(lock.as_path(), b"").expect("another program holds the index");
    let mut sink: ReleasingSink<'_> = ReleasingSink {
        lock: lock.as_path(),
        said: Vec::new(),
    };
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut sink)
        .expect("recovery");
    assert_eq!(
        outcomes[0].action,
        MergeRecoveryAction::Completed,
        "the merge is finished once the lock is free"
    );
    let said: String = String::from_utf8(sink.said).expect("the notices are UTF-8");
    assert!(
        said.contains(lock.to_string_lossy().as_ref()),
        "the lock is named: {said}"
    );
    assert!(
        said.contains("waits for the index lock"),
        "it says what it waits for: {said}"
    );
    assert!(
        !lock.exists(),
        "the lock was the other program's and nothing removed it on the wrapper's side"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        world.new_oid
    );
    remove(world.parent.as_path());
}

/// What:
///  A wait that ends with the index still held by a program that lives writes nothing,
/// a free or released lock lets the merge go on,
///  and a lock a gone process left is removed and then lets it go on.
/// Why:
///  The decision is the whole of recovery's behaviour on a held lock,
/// and every half of it is checked:
///  a recovery that wrote the index after giving up would put Git's own sentence in place of the record,
/// while one that
///  treated a dead Git's lock as a live holder's left every command in the worktree exiting 2
///  (finding 4 of the merge-and-pull audit of 2026-11-09).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(mayInstall(place, { kind: 'not-held' })).toBe(true);
/// expect(mayInstall(place, { kind: 'gave-up' })).toBe(false);
/// expect(mayInstall(place, { kind: 'owner-gone' })).toBe(true); expect(exists(lock)).toBe(false);
/// ```
#[test]
fn only_a_lock_that_is_free_or_left_by_a_gone_process_lets_the_merge_be_written() {
    let second: std::time::Duration = std::time::Duration::from_secs(1);
    let world: World = world("merge-recovery-may-install");
    assert!(may_install(world.git_dir.as_path(), IndexWait::NotHeld));
    assert!(may_install(
        world.git_dir.as_path(),
        IndexWait::Released(second)
    ));
    assert!(!may_install(
        world.git_dir.as_path(),
        IndexWait::GaveUp(second)
    ));

    // A lock whose PID file names a process that no longer runs is nobody's to release, so it is removed.
    let dead: PathBuf = world.git_dir.join("index.lock");
    std::fs::write(dead.as_path(), b"").expect("a killed Git left its empty lock");
    std::fs::write(
        crate::recovery_files::lock_pid_path(world.git_dir.join("index").as_path()),
        b"pid 4294967\n",
    )
    .expect("the PID file of a process that is gone");
    assert!(
        may_install(world.git_dir.as_path(), IndexWait::OwnerGone(second)),
        "a dead Git's lock is cleared and the merge goes on"
    );
    assert!(!dead.exists(), "the lock of a Git nothing runs was removed");

    // A lock a program that lives holds is not the recovery's to remove.
    std::fs::write(dead.as_path(), b"").expect("a program holds the index");
    std::fs::write(
        crate::recovery_files::lock_pid_path(world.git_dir.join("index").as_path()),
        format!("pid {}\n", std::process::id()),
    )
    .expect("the PID file names this process, which runs");
    assert!(
        !may_install(world.git_dir.as_path(), IndexWait::OwnerGone(second)),
        "a live holder's lock stays"
    );
    assert!(dead.exists(), "the live holder's lock is not removed");
    remove(world.parent.as_path());
}

/// What:
///  A held index stops a merge that is already installed and leaves a merge that installed nothing for the next
///  command.
/// Why:
///  The control proves both ends of the rule:
/// a merge whose files and entries are installed must not be read
///  as another session's staged work,
/// while one that installed nothing is left for the next command.
///  The holder lives here,
/// because a lock a gone process left is cleared and the merge finished
///  (`recovery_clears_the_index_lock_of_a_git_that_was_killed_and_finishes_the_merge`),
/// and the ceiling is shortened so the
///  control ends in a moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// shortCeiling(ms(100)); await expect(settleFailedMerge(installed)).rejects; await expect(settleFailedMerge(saved)).resolves.toBe('discarded');
/// ```
#[test]
fn a_held_index_defers_a_merge_that_installed_nothing_and_stops_an_installed_one() {
    // A merge that was already installed is not deferred: a command that went on would read it as staged work, so it stops.
    let installed: World = world("merge-recovery-dead-holder-lock-installed");
    git(
        installed.repo.as_path(),
        &[
            "read-tree",
            "-m",
            "-u",
            installed.old_oid.as_str(),
            installed.new_oid.as_str(),
        ],
    );
    let installed_record: PathBuf = record_of(
        &installed,
        dead_owner("13131313-0000-4000-8000-000000000013"),
        Some("refs/heads/main"),
    );
    let installed_intent: MergeIntent = parse_merge_intent(
        std::fs::read(installed_record.as_path())
            .expect("read")
            .as_slice(),
    )
    .expect("intent");
    std::fs::write(installed.git_dir.join("index.lock"), b"").expect("a Git left the index lock");
    // The holder is this process, which runs, so nothing may remove the lock and the wait ends at its ceiling.
    std::fs::write(
        crate::recovery_files::lock_pid_path(installed.git_dir.join("index").as_path()),
        format!("pid {}\n", std::process::id()),
    )
    .expect("the PID file of a process that runs");
    let mut installed_said: Vec<u8> = Vec::new();
    let settled =
        crate::index_holder_wait::RECOVERY_LOCK_CEILING_FOR_THIS_THREAD.with(|shortened| {
            shortened.set(Some(std::time::Duration::from_millis(100)));
            return settle_failed_merge(
                &place(&installed),
                installed_record.as_path(),
                &installed_intent,
                &mut installed_said,
            );
        });
    let refused: MergeRefusal =
        settled.expect_err("an installed merge behind a held index stops the command");
    assert!(refused.began, "it reports that the merge is installed");
    assert!(installed_record.exists(), "the record stays");
    remove(installed.parent.as_path());

    // A merge that was saved and installed nothing is deferred: the next command finishes it.
    let world: World = world("merge-recovery-dead-holder-lock");
    let before = snapshot(&world);
    let record: PathBuf = record_of(
        &world,
        dead_owner("13131313-0000-4000-8000-000000000014"),
        Some("refs/heads/main"),
    );
    let intent: MergeIntent =
        parse_merge_intent(std::fs::read(record.as_path()).expect("read").as_slice())
            .expect("intent");
    let lock: PathBuf = world.git_dir.join("index.lock");
    std::fs::write(lock.as_path(), b"").expect("a Git left the index lock");
    std::fs::write(
        crate::recovery_files::lock_pid_path(world.git_dir.join("index").as_path()),
        b"pid 4294967\n",
    )
    .expect("the PID file of a process that is gone");
    let mut said: Vec<u8> = Vec::new();
    let action =
        settle_failed_merge(&place(&world), record.as_path(), &intent, &mut said).expect("settled");
    assert_eq!(
        action,
        MergeRecoveryAction::Discarded,
        "nothing of it was installed, so the record only goes"
    );
    assert!(!record.exists(), "the record goes");
    assert!(lock.exists(), "the lock is not the recovery's to remove");
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        world.old_oid,
        "the branch did not move"
    );
    assert_eq!(
        snapshot(&world),
        before,
        "the index and the worktree are exactly as they were"
    );
    remove(world.parent.as_path());
}

/// What:
///  A lock a Git that no longer runs left over a merge it had partly installed is cleared on the same proof the
///  merge's own path acts on,
///  and the command finishes the merge instead of exiting 2.
/// Why:
///  An out-of-memory kill of the cgroup,
///  or `kill -9` of the process group,
///  leaves the merge's record,
///  its installed checkout and the dead Git's `index.lock`:
/// `git status`,
///  `git log`,
/// `git rev-parse HEAD`,
/// `git merge` and `git commit` all exited 2 until the person deleted the lock by hand,
///  where real Git answers `git status` with 0 in the same state
///  (finding 4 of the merge-and-pull audit).
/// The merge's own path already removes such a lock (`merge_isolation_apply.rs`),
/// and recovery is the same
///  decision from the same evidence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeLock(index, { pid: gonePid }); await expect(recoverInterruptedMerges(place, dir)).resolves.toMatchObject([{ action: 'completed' }]); expect(exists(lock)).toBe(false);
/// ```
#[test]
fn recovery_clears_the_index_lock_of_a_git_that_was_killed_and_finishes_the_merge() {
    let world: World = world("merge-recovery-dead-git-lock");
    // Git had updated the checkout and had not moved the branch when it was killed with the lock.
    git(
        world.repo.as_path(),
        &[
            "read-tree",
            "-m",
            "-u",
            world.old_oid.as_str(),
            world.new_oid.as_str(),
        ],
    );
    let record: PathBuf = record_of(
        &world,
        dead_owner("15151515-0000-4000-8000-000000000015"),
        Some("refs/heads/main"),
    );
    let lock: PathBuf = world.git_dir.join("index.lock");
    std::fs::write(lock.as_path(), b"").expect("a killed Git left its empty lock");
    std::fs::write(
        crate::recovery_files::lock_pid_path(world.git_dir.join("index").as_path()),
        b"pid 4294967\n",
    )
    .expect("the PID file of a process that is gone");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("the merge is finished, not refused");
    assert_eq!(
        the_action_for(&outcomes, record.as_path()),
        Some(MergeRecoveryAction::Completed)
    );
    assert!(!lock.exists(), "the lock of a Git nothing runs was removed");
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        world.new_oid,
        "the branch moved onto the merge"
    );
    remove(world.parent.as_path());
}

/// What:
///  A merge saved on its side ref,
///  with nothing of it in the checkout and a live program holding `index.lock`,
///  is left for the next command at once,
///  which says where the merge is and what it waits for.
/// Why:
///  The merge itself saves its result and exits 0 (`MERGE_LOCK_CEILING`,
///  one minute);
/// after that every command,
/// `git rev-parse HEAD` included,
/// waited the whole ceiling again for as long as
///  the lock stood,
/// where real Git answers each of them in a few milliseconds
///  (finding 3 of the merge-and-pull audit of 2026-11-09).
/// Nothing of the merge is in the checkout,
///  so the next command has nothing to wait for:
/// the merge is on its record and its side ref,
/// and a later
///  command finishes it once the lock is free.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const began = now(); const outcomes = await recoverInterruptedMerges(place, dir, said);
/// expect(now() - began).toBeLessThan(seconds(2)); expect(outcomes[0].action).toBe('index-held');
/// ```
#[test]
fn a_saved_merge_behind_a_live_lock_is_left_for_the_next_command_at_once() {
    let world: World = world("merge-recovery-saved-live-lock");
    let side_ref: &str = "refs/cli-git/kept/1";
    let record: PathBuf =
        deferred_record_of(&world, "16161616-0000-4000-8000-000000000016", side_ref);
    let lock: PathBuf = world.git_dir.join("index.lock");
    std::fs::write(lock.as_path(), b"").expect("another program holds the index");
    // The holder is this process, which runs: nothing may remove the lock, and waiting gains nothing.
    std::fs::write(
        crate::recovery_files::lock_pid_path(world.git_dir.join("index").as_path()),
        format!("pid {}\n", std::process::id()),
    )
    .expect("the PID file of a process that runs");
    let before = snapshot(&world);
    let mut said: Vec<u8> = Vec::new();
    let began: std::time::Instant = std::time::Instant::now();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("the command goes on");
    let took: std::time::Duration = began.elapsed();
    assert_eq!(
        the_action_for(&outcomes, record.as_path()),
        Some(MergeRecoveryAction::IndexHeld(IndexHeldAt {
            the_merges_entries_in_the_checkout: false,
            landed: false,
            branch_moved: false
        }))
    );
    assert!(
        took < std::time::Duration::from_secs(2),
        "the command does not wait for a lock that stands over a merge nothing of which is installed: {took:?}"
    );
    assert!(
        record.exists(),
        "the record stays for the command that finishes the merge"
    );
    assert_eq!(snapshot(&world), before, "nothing was written");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(
        text.contains(side_ref),
        "the line says where the merge is saved: {text}"
    );
    assert!(
        text.contains(lock.to_string_lossy().as_ref()),
        "the line names what it waits for: {text}"
    );
    assert!(
        text.contains("goes on"),
        "the line says this command goes on: {text}"
    );
    remove(world.parent.as_path());
}
