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

/// The parents of `HEAD`,
/// as object names.
fn parents(fixture: &Fixture, repo: &Path) -> Vec<String> {
    return git_text(fixture, repo, &["rev-list", "--parents", "-n", "1", "HEAD"]).split_whitespace().skip(1).map(String::from).collect();
}

/// A merge started while another program holds `index.lock` says what it waits for,
/// and lands when the lock is gone;
///  native Git dies on it (`stash failed`) after the work it did before.
#[test]
fn a_merge_waits_for_a_held_index_lock_and_then_lands() {
    for staged in [false, true] {
        let fixture: Fixture = fixture(format!("merge-waits-lock-{staged}").as_str());
        let repo: PathBuf = diverged(&fixture, "repo");
        if staged {
            stage_unrelated_work(&fixture, repo.as_path());
        }
        let before = snapshot(&fixture, repo.as_path());
        let lock: PathBuf = repo.join(".git/index.lock");
        std::fs::write(&lock, b"").expect("another program holds the lock");
        let mut command: Command = wrapped_commit(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
        command.env("CLI_GIT_WAIT_NOTICE_SECONDS", "1");
        let ended: Ended = run_acting_on_line(&mut command, "waits for the index lock", &mut || {
            assert_eq!(snapshot_head(&fixture, repo.as_path()), before.head, "nothing was merged while the lock was held");
            std::fs::remove_file(&lock).expect("the other program lets go");
        });
        assert_eq!(ended.code, Some(0), "staged={staged}: {}", ended.stderr);
        assert!(ended.stderr.contains("git merge has changed nothing yet and waits for the index lock"), "{}", ended.stderr);
        assert!(ended.stderr.contains("index.lock"), "the lock is named: {}", ended.stderr);
        assert_eq!(parents(&fixture, repo.as_path()).len(), 2, "staged={staged}: a merge commit");
        assert!(!repo.join(".git/MERGE_HEAD").exists());
        assert_eq!(records_in(repo.join(".git/monochromatic-git-policy-cli-merges").as_path()), Vec::<PathBuf>::new());
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
        repository_root: repo.to_string_lossy().into_owned(),
        target_ref: Some(String::from("refs/heads/main")),
        old_oid: String::from(old_oid),
        new_oid: String::from(merged),
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
