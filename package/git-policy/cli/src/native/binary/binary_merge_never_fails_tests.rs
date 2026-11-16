//! What:
//!  A `git merge` through the built executable while something else uses the checkout:
//! another program holds
//!  `index.lock`,
//! or the branch moves between the merge's computation and its application.
//! Why:
//!  The owner's rules (`SPEC.md`,
//! "Merges and commits never fail"):
//! "Auto merging should never fail."
//! and "Manual merge
//!  should never fail too."
//! Native Git dies on a held lock (`stash failed`,
//! `Unable to create ... index.lock`) and,
//! when
//!  its ref update loses a race,
//! leaves the checkout half merged with `MERGE_HEAD` behind.
//! The wrapper waits for the lock,
//!  and lands the merge on the head that moved,
//! leaving the checkout either as it was or merged,
//! never between.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const merge = spawn(['merge', '--no-edit', 'feature']); await sawNotice(merge); removeLock(); expect(await merge).toExit(0);
//! ```

use super::merge_isolation::{diverged, records_in, snapshot, stage_unrelated_work};
use super::commit::{git_text, remove_settled, wrapped_commit};
use super::held_lock_support::{PATIENCE, run_acting_on_line, Ended};
use super::support::{Fixture, fixture, git};
use git_policy_cli::transaction_phase::PHASE_SIGNAL_VARIABLE;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The parents of one commit,
/// as object names.
fn parents_of(fixture: &Fixture, repo: &Path, commit: &str) -> Vec<String> {
    return git_text(fixture, repo, &["rev-list", "--parents", "-n", "1", commit]).split_whitespace().skip(1).map(String::from).collect();
}

/// The parents of `HEAD`,
/// as object names.
fn parents(fixture: &Fixture, repo: &Path) -> Vec<String> {
    return git_text(fixture, repo, &["rev-list", "--parents", "-n", "1", "HEAD"]).split_whitespace().skip(1).map(String::from).collect();
}

/// What:
///  A merge that meets an `index.lock` nothing proves the owner of saves its result at once,
///  exits 0,
///  names the lock and the ref that keeps it,
///  and the next command in that worktree finishes the checkout once the lock is gone.
/// Why:
///  This is the owner's answer for a foreign `index.lock`,
///  and the paragraph `SPEC.md` states it in:
///  the merge never waits for another program's command
///  (a cross-vendor audit of alpha.12,
///  "A merge waits for another agent's live index holder",
///  found the minute of waiting this replaces).
///  The fixture is that rule at its hardest:
///  an empty lock with no PID file,
///  which is what a killed Git leaves and what a person's own `git commit` holds through its editor
///  (`service_index_unheld_removal.rs`,
///  the measurement of 2026-11-07),
///  so nothing can tell that owner from another program's live command,
///  and a merge that waited for it would wait a minute for the person at the machine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write('.git/index.lock', '')
/// const saved = await wrapped(['merge', '--no-edit', 'feature'])
/// expect(saved.code).toBe(0)
/// expect(saved.stderr).toContain('this merge does not wait for it')
/// const side = sideRefOf(saved.stderr)
/// expect(parents(await revParse(side))).toHaveLength(2)
/// rm('.git/index.lock')
/// await wrapped(['status'])
/// expect(await revParse('HEAD')).toEqual(await revParse(side))
/// ```
#[test]
fn a_merge_that_meets_a_lock_no_process_proves_saves_itself_and_the_next_command_lands_it() {
    for staged in [false, true] {
        let fixture: Fixture = fixture(format!("merge-unproven-lock-{staged}").as_str());
        let repo: PathBuf = diverged(&fixture, "repo");
        if staged {
            stage_unrelated_work(&fixture, repo.as_path());
        }
        let before = snapshot(&fixture, repo.as_path());
        let lock: PathBuf = repo.join(".git/index.lock");
        std::fs::write(&lock, b"").expect("a lock nothing proves the owner of");
        let saved = super::commit::run_commit(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
        let said: String = String::from_utf8_lossy(&saved.stderr).into_owned();
        assert_eq!(saved.code, Some(0), "staged={staged}: {said}");
        assert!(said.contains("an owner nothing proves alive"), "the words say why the merge does not wait: {said}");
        assert!(said.contains("this merge does not wait for it"), "the line says the merge did not wait: {said}");
        assert!(said.contains("index.lock"), "the lock is named: {said}");
        assert!(!said.contains("waits for the index lock"), "nothing promises a wait the merge does not make: {said}");
        let side: String = side_ref_of(said.as_str());
        let merged: String = git_text(&fixture, repo.as_path(), &["rev-parse", side.as_str()]).trim().to_owned();
        assert_eq!(parents_of(&fixture, repo.as_path(), merged.as_str()).len(), 2, "the side ref names a merge commit");
        assert_eq!(snapshot_head(&fixture, repo.as_path()), before.head, "nothing of the merge is in the branch yet");
        assert!(!repo.join(".git/MERGE_HEAD").exists(), "the checkout is not half merged");
        assert_eq!(records_in(repo.join(".git/monochromatic-git-policy-cli-merges").as_path()).len(), 2, "the merge and its save are both recorded");
        // The next command in the worktree finishes the checkout once the lock is gone.
        std::fs::remove_file(lock.as_path()).expect("the other program lets go");
        let finished = super::commit::run_commit(&fixture, repo.as_path(), &["status", "--porcelain=v1"]);
        let result: String = String::from_utf8_lossy(&finished.stderr).into_owned();
        assert_eq!(finished.code, Some(0), "staged={staged}: {result}");
        assert!(result.contains("finished an interrupted merge"), "the finish is stated: {result}");
        assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim(), merged, "the branch holds the saved merge");
        assert_eq!(parents(&fixture, repo.as_path()).len(), 2, "staged={staged}: a merge commit");
        assert_eq!(records_in(repo.join(".git/monochromatic-git-policy-cli-merges").as_path()), Vec::<PathBuf>::new(), "no record stays");
        assert_eq!(kept_refs(&fixture, repo.as_path()), Vec::<String>::new(), "the side ref goes away when the merge lands");
        git(&fixture, repo.as_path(), &["fsck", "--strict", "--no-dangling"]);
        remove_settled(&fixture);
    }
}

/// The branch tip,
/// read without taking any lock.
fn snapshot_head(fixture: &Fixture, repo: &Path) -> String {
    return git_text(fixture, repo, &["rev-parse", "HEAD"]);
}

/// The branch moves while the merge is between its computation and its application (a commit by another session,
/// or the
///  auto-push's own reconcile merge):
/// the merge lands on the head that moved,
/// the checkout holds both,
/// and there is no
///  `MERGE_HEAD` and no record.
/// Native Git's update of the branch loses the same race after it changed the checkout.
#[test]
fn a_merge_whose_branch_moved_underneath_it_lands_on_the_new_head() {
    for staged in [true, false] {
        let fixture: Fixture = fixture(format!("merge-moved-{staged}").as_str());
        let repo: PathBuf = diverged(&fixture, "repo");
        if staged {
            stage_unrelated_work(&fixture, repo.as_path());
        }
        let markers: PathBuf = fixture.root.join("markers");
        std::fs::create_dir(&markers).expect("marker directory");
        let mut command: Command = wrapped_commit(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
        command.env(PHASE_SIGNAL_VARIABLE, format!("merge-intent:pause:{}", markers.display()));
        let child = command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("the merge starts");
        let began = std::time::Instant::now();
        while !markers.join("merge-intent.reached").exists() {
            assert!(began.elapsed() < PATIENCE, "staged={staged}: the merge never reached the point between computing and applying");
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        // Another session commits to the branch meanwhile: the merge's first parent is no longer the branch tip.
        std::fs::write(repo.join("z.txt"), b"committed meanwhile\n").expect("file");
        git(&fixture, repo.as_path(), &["add", "--", "z.txt"]);
        git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=meanwhile", "--", "z.txt"]);
        let moved: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
        std::fs::write(markers.join("merge-intent.release"), b"").expect("release the merge");
        let output = child.wait_with_output().expect("the merge ends");
        let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
        assert_eq!(output.status.code(), Some(0), "staged={staged}: {stderr}");
        let parents: Vec<String> = parents(&fixture, repo.as_path());
        assert_eq!(parents.len(), 2, "staged={staged}: a merge commit");
        assert_eq!(parents[0], moved, "staged={staged}: the merge sits on the head that moved");
        assert_eq!(parents[1], git_text(&fixture, repo.as_path(), &["rev-parse", "feature"]).trim());
        assert!(repo.join("z.txt").exists() && repo.join("dir/new.txt").exists(), "staged={staged}: both sides are in the checkout");
        assert!(!repo.join(".git/MERGE_HEAD").exists());
        assert_eq!(records_in(repo.join(".git/monochromatic-git-policy-cli-merges").as_path()), Vec::<PathBuf>::new());
        let status: String = git_text(&fixture, repo.as_path(), &["--no-optional-locks", "status", "--porcelain=v1"]);
        assert_eq!(status.lines().count(), if staged { 3 } else { 0 }, "staged={staged}: only the other session's work is staged: {status}");
        git(&fixture, repo.as_path(), &["fsck", "--strict", "--no-dangling"]);
        remove_settled(&fixture);
    }
}

/// What:
///  A command run only to look at the worktree finishes the merge a killed command had started,
///  and says so on standard error alone.
/// Why:
///  The startup step settles a dead merge before any command reads the checkout,
/// so `git status` can move the branch onto the
///  merge commit;
/// silence there was the audit's finding 1
///  (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`).
/// Standard output stays byte for byte what a script reads today,
/// because scripts parse `git status --porcelain`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeKilledMergeRecord(repo); await readTree(repo, old, merged); const looked = await run(['status', '--porcelain=v1']);
/// expect(looked.code).toBe(0); expect(looked.stdout).toEqual(''); expect(looked.stderr).toContain(merged);
/// ```
#[test]
fn a_read_only_command_that_finishes_a_killed_merge_says_so() {
    let fixture: Fixture = fixture("merge-recovery-reported");
    let repo: PathBuf = diverged(&fixture, "repo");
    let old_oid: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    let feature: String = git_text(&fixture, repo.as_path(), &["rev-parse", "feature"]).trim().to_owned();
    let tree: String = git_text(&fixture, repo.as_path(), &["merge-tree", "--write-tree", old_oid.as_str(), feature.as_str()]).trim().to_owned();
    let merged: String = git_text(
        &fixture,
        repo.as_path(),
        &["commit-tree", tree.as_str(), "-p", old_oid.as_str(), "-p", feature.as_str(), "-m", "merge"],
    )
    .trim()
    .to_owned();
    let record: PathBuf = killed_after_the_checkout(&fixture, repo.as_path(), old_oid.as_str(), merged.as_str());
    let looked = super::commit::run_commit(&fixture, repo.as_path(), &["status", "--porcelain=v1"]);
    assert_eq!(looked.code, Some(0), "{}", String::from_utf8_lossy(&looked.stderr));
    assert_eq!(String::from_utf8_lossy(&looked.stdout), "", "the command's standard output is what it always was");
    let said: String = String::from_utf8_lossy(&looked.stderr).into_owned();
    assert!(said.contains("finished an interrupted merge"), "the recovery is stated: {said}");
    assert!(said.contains("refs/heads/main"), "the ref it landed on is named: {said}");
    assert!(said.contains(merged.as_str()), "the merge commit is named: {said}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim(), merged, "the branch is on the merge");
    assert!(!record.exists(), "the record is gone");
    remove_settled(&fixture);
}

/// What:
///  Put `repo` in the state a merge killed after Git moved the checkout and before it moved the branch leaves,
///  with the record such a merge leaves.
/// Why:
///  Two controls start from that state,
/// one where only a read-only command meets it and one where another program holds
///  `index.lock` while that command runs,
/// so the state is made in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = writeMergeRecord({ repo, oldOid, merged, deadOwner }); await readTree(repo, oldOid, merged); return record;
/// ```
fn killed_after_the_checkout(fixture: &Fixture, repo: &Path, old_oid: &str, merged: &str) -> PathBuf {
    use git_policy_cli::merge_isolation_journal::{MergeIntent, write_merge_intent};
    use git_policy_cli::transaction_owner::{TransactionOwner, current_transaction_owner};
    // The owner is one whose process-birth identity no process has, so recovery judges it dead.
    let mut owner: TransactionOwner = current_transaction_owner("dddddddd-0000-4000-8000-00000000000d", "2026-10-29T00:00:00.000Z").expect("owner");
    owner.owner_identity = String::from("linux:1");
    let intent: MergeIntent = MergeIntent {
        owner,
        repository_root: repo.to_path_buf(),
        target_ref: Some(String::from("refs/heads/main")),
        old_oid: String::from(old_oid),
        new_oid: String::from(merged),
        side_ref: None,
        carried: None,
    };
    let record: PathBuf = write_merge_intent(repo.join(".git").as_path(), &intent).expect("the record");
    git(fixture, repo, &["read-tree", "-m", "-u", old_oid, merged]);
    assert_eq!(git_text(fixture, repo, &["rev-parse", "HEAD"]).trim(), old_oid, "the checkout holds the merge and the branch does not");
    return record;
}

/// What:
///  A killed merge is finished even though another program holds `index.lock` while the finishing command runs,
///  and the command says what it waits for.
/// Why:
///  Recovery writes the index to finish the merge,
/// and `git update-index` dies on a held lock;
/// before this control every
///  command in that worktree exited 2 for as long as any other program held the index,
/// with Git's own sentence in place of
///  what is unfinished (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 2).
/// The lock goes when the wait is proven to be waiting,
/// so the recovery finishes and the command exits 0.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lock = write('.git/index.lock'); const ended = await runActingOnLine(['status'], 'waits for the index lock', () => rm(lock)); expect(ended.code).toBe(0);
/// ```
#[test]
fn a_held_index_lock_does_not_end_every_command_in_the_worktree() {
    let fixture: Fixture = fixture("merge-recovery-held-index");
    let repo: PathBuf = diverged(&fixture, "repo");
    let old_oid: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    let feature: String = git_text(&fixture, repo.as_path(), &["rev-parse", "feature"]).trim().to_owned();
    let tree: String = git_text(&fixture, repo.as_path(), &["merge-tree", "--write-tree", old_oid.as_str(), feature.as_str()]).trim().to_owned();
    let merged: String = git_text(
        &fixture,
        repo.as_path(),
        &["commit-tree", tree.as_str(), "-p", old_oid.as_str(), "-p", feature.as_str(), "-m", "merge"],
    )
    .trim()
    .to_owned();
    let record: PathBuf = killed_after_the_checkout(&fixture, repo.as_path(), old_oid.as_str(), merged.as_str());
    let lock: PathBuf = repo.join(".git/index.lock");
    std::fs::write(&lock, b"").expect("another program holds the index");
    let mut command: Command = wrapped_commit(&fixture, repo.as_path(), &["status", "--porcelain=v1"]);
    command.env("CLI_GIT_WAIT_NOTICE_SECONDS", "1");
    let ended: Ended = run_acting_on_line(&mut command, "waits for the index lock", &mut || {
        assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim(), old_oid, "nothing was finished while the lock was held");
        std::fs::remove_file(&lock).expect("the other program lets go");
    });
    assert_eq!(ended.code, Some(0), "{}", ended.stderr);
    assert!(ended.stderr.contains("index.lock"), "the lock is named: {}", ended.stderr);
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim(), merged, "the merge is finished");
    assert!(!record.exists(), "the record is gone");
    remove_settled(&fixture);
}

/// What:
///  A merge whose repository holds an `index.lock` that a gone Git left lands and exits 0,
///  the way a commit does,
///  and says what became of the lock.
/// Why:
///  A Git killed under `core.lockfilePid` leaves the empty lock and a PID file naming a process that ended;
/// the merge used
///  to wait on it twenty times and then exit 2 with advice that could not work,
/// while the commit service removed
///  exactly such a lock and landed
/// (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 3).
/// The removal acts on the
///  commit service's proof of disuse,
/// so this control is the merge's half of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write('.git/index.lock', ''); write('.git/index~pid.lock', 'pid 4294967\n'); expect((await wrapped(['merge', '--no-edit', 'feature'])).code).toBe(0);
/// ```
#[test]
fn a_lock_a_gone_git_left_does_not_end_a_merge() {
    for staged in [false, true] {
        let fixture: Fixture = fixture(format!("merge-dead-lock-{staged}").as_str());
        let repo: PathBuf = diverged(&fixture, "repo");
        if staged {
            stage_unrelated_work(&fixture, repo.as_path());
        }
        let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
        let lock: PathBuf = repo.join(".git/index.lock");
        std::fs::write(lock.as_path(), b"").expect("a Git left the lock");
        // The PID file `core.lockfilePid` leaves beside it, naming a process that has ended.
        std::fs::write(repo.join(".git/index~pid.lock"), b"pid 4294967\n").expect("the PID file of a process that is gone");
        let mut command: Command = wrapped_commit(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
        command.env("CLI_GIT_WAIT_NOTICE_SECONDS", "1");
        let ended: Ended = run_acting_on_line(&mut command, "no longer runs", &mut || {});
        assert_eq!(ended.code, Some(0), "staged={staged}: {}", ended.stderr);
        assert!(ended.stderr.contains("the merge clears that lock"), "the line says what a merge does with it: {}", ended.stderr);
        assert!(!lock.exists(), "staged={staged}: the lock a gone process left is gone");
        assert_eq!(parents(&fixture, repo.as_path()).len(), 2, "staged={staged}: a merge commit");
        assert_ne!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim(), head, "staged={staged}: the branch moved");
        assert_eq!(records_in(repo.join(".git/monochromatic-git-policy-cli-merges").as_path()), Vec::<PathBuf>::new());
        git(&fixture, repo.as_path(), &["fsck", "--strict", "--no-dangling"]);
        remove_settled(&fixture);
    }
}

/// What:
/// What:
///  Put a repository in the state a kill between dropping a merge record's capture ref and removing the record leaves,
///  and answer the record's path.
/// Why:
///  The state is the wrapper's own record bytes,
/// a landed merge,
/// a capture ref that is gone,
/// and (for the second round) the sibling marker that says the carry finished.
/// It is built from a real saved merge,
/// so the record names a real capture of the person's bytes,
/// and the ref that kept them is deleted the way the wrapper deletes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { record, bytes } = saveMergeAndFinish(fixture, repo);
/// git(["update-ref", "--no-deref", "-d", carried]); write(record, bytes); if (settled) write(record.replace(/\.json$/, ".settled"), "");
/// return record;
/// ```
fn killed_between_the_drop_and_the_removal(fixture: &Fixture, repo: &Path, settled: bool) -> PathBuf {
    std::fs::write(repo.join("f.txt"), b"my own edit of a file the merge changes\n").expect("the person's edit");
    let lock: PathBuf = repo.join(".git/index.lock");
    std::fs::write(lock.as_path(), b"").expect("a running program's lock");
    std::fs::write(repo.join(".git/index~pid.lock"), format!("pid {}\n", std::process::id())).expect("the PID file of a process that runs");
    let saved = super::commit::run_commit(fixture, repo, &["merge", "--no-edit", "feature"]);
    assert_eq!(saved.code, Some(0), "the merge is saved for a later finish: {}", String::from_utf8_lossy(&saved.stderr));
    // The merge's own record is the one whose `sideRef` is null; its bytes are what the kill leaves behind.
    let directory: PathBuf = repo.join(".git/monochromatic-git-policy-cli-merges");
    let mut own: Option<(PathBuf, Vec<u8>)> = None;
    for record in records_in(directory.as_path()) {
        let bytes: Vec<u8> = std::fs::read(record.as_path()).expect("the record");
        let names_a_saved_merge: bool = String::from_utf8_lossy(bytes.as_slice()).contains("\"sideRef\":\"refs/cli-git/kept/");
        if !names_a_saved_merge {
            own = Some((record, bytes));
        }
    }
    let (record, bytes): (PathBuf, Vec<u8>) = own.expect("the merge's own record");
    std::fs::remove_file(lock.as_path()).expect("the holder lets go");
    let _pid = std::fs::remove_file(repo.join(".git/index~pid.lock"));
    let finished = super::commit::run_commit(fixture, repo, &["status", "--porcelain=v1"]);
    assert_eq!(finished.code, Some(0), "{}", String::from_utf8_lossy(&finished.stderr));
    assert_eq!(records_in(directory.as_path()), Vec::<PathBuf>::new(), "the merge is finished and its records are gone");
    // The kill state: the merge landed and the carry is in the files, the record is back, and the ref it names is gone.
    for name in carried_refs(fixture, repo) {
        git(fixture, repo, &["update-ref", "--no-deref", "-d", name.as_str()]);
    }
    assert_eq!(carried_refs(fixture, repo), Vec::<String>::new(), "no ref keeps the capture any more");
    std::fs::write(record.as_path(), bytes.as_slice()).expect("the record a kill left");
    if settled {
        std::fs::write(record.with_extension("settled"), b"").expect("the marker of a finished carry");
    }
    return record;
}

/// What:
///  A kill between dropping a merge record's capture ref and removing the record is finished by the next command,
///  and a commit is made afterwards,
///  even once `git gc --prune=now` has collected the capture's objects.
/// Why:
///  Git refuses a compare-and-delete of a ref that is already gone,
/// so the record stayed for ever,
/// every command printed that a ref its record keeps could not be removed,
/// and after a gc the carry could not read the objects the record names and refused every command,
/// commit among them
/// (the second review of this branch,
/// its first finding,
/// against "Commits should never fail").
/// A deletion of a name that is already gone is what the deletion wanted,
/// and a record whose sibling says its carry finished needs nothing from the objects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = killedBetween(false); const after = await runCommit(["status"]);
/// expect(after.code).toBe(0); expect(existsSync(record)).toBe(false);
/// git(["gc", "--prune=now"]); expect((await runCommit(["commit", "--allow-empty", "-m", "after"])).code).toBe(0);
/// ```
#[test]
fn a_kill_after_the_capture_ref_is_dropped_is_finished_by_the_next_command() {
    for settled in [false, true] {
        let fixture: Fixture = fixture(format!("merge-kill-after-drop-{settled}").as_str());
        let repo: PathBuf = diverged(&fixture, "repo");
        let record: PathBuf = killed_between_the_drop_and_the_removal(&fixture, repo.as_path(), settled);
        let after = super::commit::run_commit(&fixture, repo.as_path(), &["status", "--porcelain=v1"]);
        let said: String = String::from_utf8_lossy(&after.stderr).into_owned();
        assert_eq!(after.code, Some(0), "settled={settled}: the next command reads the worktree: {said}");
        assert!(!record.exists(), "settled={settled}: the record whose ref was already gone is removed: {said}");
        assert!(!said.contains("could not be removed yet"), "settled={settled}: no notice about a ref that is gone: {said}");
        // A collection of everything no ref names is what a person runs next, and the objects the record named go with it.
        git(&fixture, repo.as_path(), &["gc", "--prune=now", "--quiet"]);
        let committed = super::commit::run_commit(&fixture, repo.as_path(), &["commit", "--allow-empty", "--message=after the kill"]);
        let commit_said: String = String::from_utf8_lossy(&committed.stderr).into_owned();
        assert_eq!(committed.code, Some(0), "settled={settled}: a commit is made after the gc: {commit_said}");
        assert_eq!(
            git_text(&fixture, repo.as_path(), &["log", "--format=%s", "-1"]).trim(),
            "after the kill",
            "settled={settled}: the commit is in the branch"
        );
        assert_eq!(records_in(repo.join(".git/monochromatic-git-policy-cli-merges").as_path()), Vec::<PathBuf>::new(), "settled={settled}");
        super::commit::remove_settled(&fixture);
    }
}

/// What:
///  A saved merge whose kept ref was pruned away is finished by the next command,
///  and a commit is made after it.
/// Why:
///  The record names the ref its merge commit was kept on,
/// and a wrapper that prunes kept refs by age removes that name while the record still waits
/// (`refs/cli-git/kept/`,
/// the answers of 2026-11-12T11:58:45-05:00).
/// The deletion of a name already gone is refused by Git,
/// and the record then stayed for ever with the notice that a ref it keeps could not be removed
/// (the second review of this branch,
/// its first finding,
/// whose shape the side ref shares with the capture's ref).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const saved = await wrapped(["merge", "--no-edit", "feature"]); git(["update-ref", "--no-deref", "-d", sideRefOf(saved.stderr)]);
/// rm(lock); const after = await wrapped(["status"]); expect(after.code).toBe(0); expect(records()).toEqual([]);
/// ```
#[test]
fn a_saved_merge_whose_kept_ref_was_pruned_is_finished_by_the_next_command() {
    let fixture: Fixture = fixture("merge-kept-ref-pruned");
    let repo: PathBuf = diverged(&fixture, "repo");
    std::fs::write(repo.join("f.txt"), b"my own edit of a file the merge changes\n").expect("the person's edit");
    let lock: PathBuf = repo.join(".git/index.lock");
    std::fs::write(lock.as_path(), b"").expect("a running program's lock");
    std::fs::write(repo.join(".git/index~pid.lock"), format!("pid {}\n", std::process::id())).expect("the PID file of a process that runs");
    let saved = super::commit::run_commit(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    let said: String = String::from_utf8_lossy(&saved.stderr).into_owned();
    assert_eq!(saved.code, Some(0), "the merge is saved for a later finish: {said}");
    let side: String = side_ref_of(said.as_str());
    let merged: String = git_text(&fixture, repo.as_path(), &["rev-parse", side.as_str()]).trim().to_owned();
    // The prune of kept refs by age, which is another command's work, removes the name the record keeps.
    git(&fixture, repo.as_path(), &["update-ref", "--no-deref", "-d", side.as_str()]);
    assert_eq!(kept_refs(&fixture, repo.as_path()), Vec::<String>::new(), "the kept ref is pruned away");
    std::fs::remove_file(lock.as_path()).expect("the holder lets go");
    let _pid = std::fs::remove_file(repo.join(".git/index~pid.lock"));
    let finished = super::commit::run_commit(&fixture, repo.as_path(), &["status", "--porcelain=v1"]);
    let out: String = String::from_utf8_lossy(&finished.stderr).into_owned();
    assert_eq!(finished.code, Some(0), "the next command goes on: {out}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim(), merged, "the saved merge is in the branch: {out}");
    assert_eq!(records_in(repo.join(".git/monochromatic-git-policy-cli-merges").as_path()), Vec::<PathBuf>::new(), "the record goes: {out}");
    assert!(!out.contains("could not be removed yet"), "no notice about a ref that is gone: {out}");
    let committed = super::commit::run_commit(&fixture, repo.as_path(), &["commit", "--allow-empty", "--message=after the prune"]);
    assert_eq!(committed.code, Some(0), "a commit is made: {}", String::from_utf8_lossy(&committed.stderr));
    super::commit::remove_settled(&fixture);
}

///  The side ref a deferred merge's line names.
/// Why:
///  The control reads the merge commit from the very name the person is given,
/// so the line and the ref cannot disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const side = /saved on (\S+)/.exec(stderr)?.[1];
/// ```
/// The carried refs the wrapper keeps in `repo`.
fn carried_refs(fixture: &Fixture, repo: &Path) -> Vec<String> {
    let listed: String = git_text(fixture, repo, &["for-each-ref", "--format=%(refname)", "refs/cli-git/carried"]);
    return listed.lines().map(|line| { return line.trim().to_owned(); }).filter(|name| { return !name.is_empty(); }).collect();
}

/// What:
///  A merge saved behind a live foreign `index.lock` while the worktree holds an edit to a path the merge changes
///  finishes once,
///  and then leaves no ref and no record behind.
/// Why:
///  The save writes two records that name one capture of the person's bytes:
/// the merge's own,
/// and the one that says the checkout is still to be finished.
/// Each record's finish must be able to let the capture go,
/// or the person is left with refs and records nothing will ever remove,
/// and a notice on every later command
/// (the alpha.14 audit's finding 3).
/// The edit is to `f.txt`,
/// a path the merge changes,
/// which is what makes the merge capture anything at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write('f.txt', 'mine\n'); await mergeBehindLiveLock(); rm(lock); await wrapped(['status']);
/// expect(carriedRefs()).toEqual([]); expect(keptRefs()).toEqual([]); expect(records()).toEqual([]);
/// const later = await wrapped(['diff', '--stat']); expect(later.stderr).not.toContain('finished');
/// ```
#[test]
fn a_saved_merge_that_carries_an_edit_leaves_no_ref_and_no_record_behind() {
    let fixture: Fixture = fixture("merge-saved-carry-cleanup");
    let repo: PathBuf = diverged(&fixture, "repo");
    std::fs::write(repo.join("f.txt"), b"my own edit of a file the merge changes\n").expect("the person's edit");
    let lock: PathBuf = repo.join(".git/index.lock");
    std::fs::write(lock.as_path(), b"").expect("a running program's lock");
    std::fs::write(repo.join(".git/index~pid.lock"), format!("pid {}\n", std::process::id())).expect("the PID file of this process, which runs");
    let saved = super::commit::run_commit(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    let said: String = String::from_utf8_lossy(&saved.stderr).into_owned();
    assert_eq!(saved.code, Some(0), "the merge is saved, not failed: {said}");
    assert!(said.contains("saved on refs/cli-git/kept/"), "the side ref is named: {said}");
    let merged: String = git_text(&fixture, repo.as_path(), &["rev-parse", side_ref_of(said.as_str()).as_str()]).trim().to_owned();
    assert_eq!(records_in(repo.join(".git/monochromatic-git-policy-cli-merges").as_path()).len(), 2, "the merge and its save are both recorded");
    assert_eq!(carried_refs(&fixture, repo.as_path()).len(), 1, "the edit is kept on a carried ref");
    std::fs::remove_file(lock.as_path()).expect("the holder lets go");
    let _pid = std::fs::remove_file(repo.join(".git/index~pid.lock"));
    // The next command finishes the merge, once.
    let finished = super::commit::run_commit(&fixture, repo.as_path(), &["status", "--porcelain=v1"]);
    let result: String = String::from_utf8_lossy(&finished.stderr).into_owned();
    assert_eq!(finished.code, Some(0), "{result}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim(), merged, "the branch holds the saved merge");
    let file: Vec<u8> = std::fs::read(repo.join("f.txt")).expect("the person's file");
    let carried: String = String::from_utf8_lossy(file.as_slice()).into_owned();
    assert!(carried.contains("my own edit of a file the merge changes"), "the edit is carried: {carried}");
    assert_eq!(records_in(repo.join(".git/monochromatic-git-policy-cli-merges").as_path()), Vec::<PathBuf>::new(), "no record stays: {result}");
    assert_eq!(kept_refs(&fixture, repo.as_path()), Vec::<String>::new(), "the side ref goes: {result}");
    assert_eq!(carried_refs(&fixture, repo.as_path()), Vec::<String>::new(), "the carried ref goes: {result}");
    // A read-only command afterwards says nothing about a merge: nothing is left to finish and nothing to retry.
    let later = super::commit::run_commit(&fixture, repo.as_path(), &["diff", "--stat"]);
    let quiet: String = String::from_utf8_lossy(&later.stderr).into_owned();
    assert!(!quiet.contains("finished an interrupted merge") && !quiet.contains("could not be removed"), "no merge is spoken of any more: {quiet}");
    git(&fixture, repo.as_path(), &["fsck", "--strict", "--no-dangling"]);
    remove_settled(&fixture);
}

fn side_ref_of(said: &str) -> String {
    // Every wait notice can say the merge is saved on a ref of its own, so the line is read at the ref it names.
    let (_, rest) = said.split_once("saved on refs/").expect("the line names the side ref");
    return format!("refs/{}", rest.split_whitespace().next().expect("a ref name"));
}

/// The side refs the wrapper keeps in `repo`.
fn kept_refs(fixture: &Fixture, repo: &Path) -> Vec<String> {
    let listed: String = git_text(fixture, repo, &["for-each-ref", "--format=%(refname)", "refs/cli-git/kept"]);
    return listed.lines().map(|line| { return line.trim().to_owned(); }).filter(|name| { return !name.is_empty(); }).collect();
}

/// What:
///  A merge that meets an index lock it may not clear saves its result on a side ref,
///  exits 0,
///  says so,
///  and the next command in that worktree finishes the checkout once the lock is free,
///  keeping an edit made meanwhile to a file the merge does not change.
/// Why:
///  That is the owner's answer for a live foreign `index.lock` (`SPEC.md:2187`):
/// "a merge saves its result on a
///  side ref,
/// exits 0,
/// and finishes the checkout when the lock frees,
/// carrying edits made meanwhile forward",
/// and "Manual
///  merge should never fail too.".
/// The lock holds the whole index beside a PID file of a process that is gone,
/// which is what a
///  person's editor-phase `git commit --all` leaves and what the removal proof refuses,
/// so the wait ends at once
///  and the control needs no minute of patience.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const saved = await wrapped(['merge', '--no-edit', 'feature']); expect(saved.code).toBe(0);
/// const side = sideRefOf(saved.stderr); expect(parents(await revParse(side))).toHaveLength(2);
/// write('u.txt', edit); rm(lock); await wrapped(['status']); expect(await revParse('HEAD')).toEqual(await revParse(side)); expect(read('u.txt')).toEqual(edit);
/// ```
#[test]
fn a_merge_that_may_not_clear_the_lock_saves_itself_and_the_next_command_finishes_it() {
    let fixture: Fixture = fixture("merge-deferred-save");
    let repo: PathBuf = diverged(&fixture, "repo");
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    let lock: PathBuf = repo.join(".git/index.lock");
    std::fs::write(lock.as_path(), b"an index a person's editor-phase commit wrote into the lock").expect("a lock with content");
    std::fs::write(repo.join(".git/index~pid.lock"), b"pid 4294967\n").expect("the PID file of a process that is gone");
    let saved = super::commit::run_commit(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    let said: String = String::from_utf8_lossy(&saved.stderr).into_owned();
    assert_eq!(saved.code, Some(0), "a merge that may not clear the lock still exits 0: {said}");
    assert!(said.contains("saved on refs/cli-git/kept/"), "the side ref is named: {said}");
    assert!(said.contains("The next git command in this worktree finishes the checkout"), "{said}");
    assert!(said.contains("This merge wrote nothing to your files"), "a merge that captured nothing claims no path it changed: {said}");
    let side: String = side_ref_of(said.as_str());
    let merged: String = git_text(&fixture, repo.as_path(), &["rev-parse", side.as_str()]).trim().to_owned();
    assert_eq!(parents_of(&fixture, repo.as_path(), merged.as_str()).len(), 2, "the side ref names a merge commit");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim(), head, "the branch did not move yet");
    assert_eq!(records_in(repo.join(".git/monochromatic-git-policy-cli-merges").as_path()).len(), 2, "the merge and its save are both recorded");
    // The person works on while the lock is held, in a file the merge does not change.
    std::fs::write(repo.join("u.txt"), b"an edit made while the lock was held\n").expect("the person's edit");
    std::fs::remove_file(lock.as_path()).expect("the other program lets go");
    let _pid = std::fs::remove_file(repo.join(".git/index~pid.lock"));
    // The next command finishes the checkout.
    let finished = super::commit::run_commit(&fixture, repo.as_path(), &["status", "--porcelain=v1"]);
    let result: String = String::from_utf8_lossy(&finished.stderr).into_owned();
    assert_eq!(finished.code, Some(0), "{result}");
    assert!(result.contains("finished an interrupted merge"), "the finish is stated: {result}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim(), merged, "the branch holds the saved merge");
    assert_eq!(std::fs::read(repo.join("u.txt")).expect("the person's file"), b"an edit made while the lock was held\n", "the edit made meanwhile is kept");
    assert!(repo.join("dir/new.txt").exists(), "the merge's own files are in the checkout");
    assert_eq!(records_in(repo.join(".git/monochromatic-git-policy-cli-merges").as_path()), Vec::<PathBuf>::new(), "no record stays");
    assert_eq!(kept_refs(&fixture, repo.as_path()), Vec::<String>::new(), "the side ref goes away when the merge lands");
    git(&fixture, repo.as_path(), &["fsck", "--strict", "--no-dangling"]);
    remove_settled(&fixture);
}

/// What:
///  A merge that meets a lock a running process holds open saves its result at once,
///  exits 0,
///  names that process and the proof the scan made,
///  and the next command in that worktree finishes the checkout once the lock is gone.
/// Why:
///  A holder that runs is not this command's to remove,
///  so the merge must not fail and its result must not be collectable before it lands,
///  and the merge does not wait for another program's command either
///  (`SPEC.md`,
///  "Merges and commits never fail";
///  the audit of alpha.12 named above).
///  The holder is a process of this account holding the file open,
///  which is the proof the scan makes;
///  a PID file naming this test process proves nothing,
///  because this binary's command name is not Git's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holder = holdOpenUntilTheScanSeesIt(lock)
/// const saved = await wrapped(['merge', '--no-edit', 'feature'])
/// expect(saved.code).toBe(0)
/// expect(saved.stderr).toContain('proven by an open descriptor')
/// const side = sideRefOf(saved.stderr)
/// expect(parents(await revParse(side))).toHaveLength(2)
/// rm(lock)
/// holder.kill()
/// await wrapped(['status'])
/// expect(await revParse('HEAD')).toEqual(await revParse(side))
/// ```
#[test]
fn a_merge_that_meets_a_holder_proven_alive_saves_itself_at_once() {
    let fixture: Fixture = fixture("merge-live-holder-saved");
    let repo: PathBuf = diverged(&fixture, "repo");
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    let lock: PathBuf = repo.join(".git/index.lock");
    std::fs::write(lock.as_path(), b"").expect("a running program's lock");
    let mut holder: std::process::Child = hold_the_index_open_until_the_scan_sees_it(lock.as_path());
    let saved = super::commit::run_commit(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    let said: String = String::from_utf8_lossy(&saved.stderr).into_owned();
    assert_eq!(saved.code, Some(0), "a merge behind a running holder still exits 0: {said}");
    assert!(said.contains(format!("PID {}", holder.id()).as_str()), "the holder is named: {said}");
    assert!(said.contains("proven by an open descriptor"), "the proof is named: {said}");
    assert!(said.contains("this merge does not wait for it"), "the line says the merge did not wait: {said}");
    assert!(!said.contains("waits for the index lock"), "nothing promises a wait the merge does not make: {said}");
    assert!(said.contains("saved on refs/cli-git/kept/"), "{said}");
    // The line says what this merge does, not what Git does after a wait it never makes.
    assert!(!said.contains("reports what Git says"), "no line promises another command's report: {said}");
    assert!(!said.contains("meets Git's own refusal"), "no line promises a refusal the merge does not make: {said}");
    let side: String = side_ref_of(said.as_str());
    let merged: String = git_text(&fixture, repo.as_path(), &["rev-parse", side.as_str()]).trim().to_owned();
    assert_eq!(parents_of(&fixture, repo.as_path(), merged.as_str()).len(), 2, "the side ref names a merge commit");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim(), head, "the branch has not moved");
    std::fs::remove_file(lock.as_path()).expect("the name of the lock goes");
    let _ended = holder.kill();
    let _collected = holder.wait();
    let finished = super::commit::run_commit(&fixture, repo.as_path(), &["status", "--porcelain=v1"]);
    assert_eq!(finished.code, Some(0), "{}", String::from_utf8_lossy(&finished.stderr));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim(), merged, "the branch holds the saved merge");
    assert_eq!(kept_refs(&fixture, repo.as_path()), Vec::<String>::new(), "the side ref goes away when the merge lands");
    remove_settled(&fixture);
}

/// How long a holder this control starts may take to appear in the scan,
/// so a broken start fails the control instead of holding it.
const HOW_LONG_A_HOLDER_MAY_TAKE_TO_APPEAR: std::time::Duration = std::time::Duration::from_secs(20);

/// What:
///  Start a process of this account that holds `lock` open on descriptor 3,
///  and wait for the scan the wrapper makes to see it,
///  bounded,
///  so the merge it tests meets a holder that is there.
/// Why:
///  A control about a holder proven alive needs the proof in place before the merge runs,
///  and a shell that has not reached its redirect yet is a process holding nothing.
///  The scan is the one the wrapper reads,
///  so the control waits on the same fact.
fn hold_the_index_open_until_the_scan_sees_it(lock: &Path) -> std::process::Child {
    let holder: std::process::Child = std::process::Command::new("sh")
        .args(["-c", "exec 3<\"$0\"; exec sleep 30", lock.to_str().expect("a UTF-8 lock path")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("a holder starts");
    let began: std::time::Instant = std::time::Instant::now();
    while !the_scan_sees(lock, holder.id()) {
        assert!(began.elapsed() < HOW_LONG_A_HOLDER_MAY_TAKE_TO_APPEAR, "the scan never saw the holder {}", holder.id());
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    return holder;
}

/// Whether the host's bounded scan for open holders sees `pid` holding `lock` open.
fn the_scan_sees(lock: &Path, pid: u32) -> bool {
    let Ok(Some(metadata)) = git_policy_cli::index_lock_evidence::read_lock_metadata(lock) else {
        return false;
    };
    let seen = git_policy_cli::index_lock_holders::scan_platform_holders_bounded(lock, metadata.device, metadata.inode);
    return seen.holders.iter().any(|holder| { return holder.pid == i64::from(pid); });
}
