//! Controls for the index update a landed batch owes:
//! the record of it in the worktree's own Git directory,
//! what recovery does with an open record that cannot be read,
//! and what a commit does about it,
//! which is to record nothing about the paths it cannot prove.

use super::tests::{World, land_with_failed_index, request, world};
use super::*;
use crate::service_journal::{OPEN_EXTENSION, batch_path};
use crate::test_support::{git, git_text, remove};
use std::path::Path;

/// The path of the owed record of batch `seq` in the world's worktree.
fn owed_of(world: &World, seq: u64) -> PathBuf {
    return crate::owed_index_update::path_of(world.repo.join(".git").as_path(), seq);
}

/// The file the index of the world's worktree names.
fn real_index_of(world: &World) -> PathBuf {
    return world.repo.join(".git").join("index");
}

/// The one owed record the world's worktree holds.
fn record_of(world: &World) -> crate::owed_index_update::Owed {
    let records: Vec<crate::owed_index_update::Owed> = crate::owed_index_update::records_of(world.repo.join(".git").as_path());
    assert_eq!(records.len(), 1, "one landed batch owes one index update: {records:?}");
    return records[0].clone();
}

/// What an index file holds of `arguments`' answer,
/// run in the world's repository with that index.
fn index_text(world: &World, index: &Path, arguments: &[&str]) -> String {
    let output: std::process::Output = std::process::Command::new(crate::test_support::REAL_GIT)
        .current_dir(world.repo.as_path())
        .env("GIT_INDEX_FILE", index)
        .args(arguments)
        .output()
        .expect("Git runs");
    let text: String = String::from_utf8(output.stdout).expect("text");
    return String::from(text.trim_end());
}

/// A copy of the world's index,
/// as a commit's preparation makes before it reads the index.
fn commit_index_of(world: &World) -> PathBuf {
    let copy: PathBuf = world.root.join("commit.index");
    std::fs::copy(real_index_of(world), copy.as_path()).expect("a copy of the index");
    return copy;
}

/// What a commit does about the world's owed records,
/// with its own index given (nothing for an explicit-path commit),
/// and the index file as it is now,
/// since the copy the commit made is current and nothing moved after it.
fn exclude(world: &World, commit_index: Option<&Path>) -> Vec<String> {
    let captured: crate::service_land_start::IndexWitness =
        crate::service_land_start::witness_of(real_index_of(world).as_path()).unwrap_or(crate::service_land_start::IndexWitness::Absent);
    return crate::owed_index_update::exclude_what_a_landed_commit_left(&world.context.git, world.repo.join(".git").as_path(), real_index_of(world).as_path(), commit_index, &captured);
}

/// The same with the index file the commit copied named as gone,
/// which is what a writer that replaced the index leaves behind.
fn exclude_after_the_index_moved(world: &World, commit_index: Option<&Path>, of_index: &Path) -> Vec<String> {
    return crate::owed_index_update::exclude_what_a_landed_commit_left(
        &world.context.git,
        world.repo.join(".git").as_path(),
        of_index,
        commit_index,
        &crate::service_land_start::IndexWitness::Absent,
    );
}

/// Owner's rule "never discard user data":
/// an open record the journal holds that this build cannot read names the landed batch's branch values,
/// paths and index,
/// so removing it forgets the index update the landing owed and leaves the person's next commit recording the landed commit's own
///  paths against the index.
/// It is kept and named,
/// and what it owed survives in the worktree's own record of it.
#[test]
fn an_open_record_that_cannot_be_read_is_kept_and_its_owed_index_update_stays_in_the_worktree() {
    let world: World = world("service-land-unreadable-intent");
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    let intent: PathBuf = batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION);
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "D  one.txt\n?? one.txt", "the index describes the branch before the landing");
    assert!(owed_of(&world, 1).exists(), "the landing wrote what it owes in the worktree before it tried to install the index");
    assert_eq!(record_of(&world).paths, vec![String::from("one.txt")], "the record names the commit's paths");
    std::fs::write(&intent, "seq=1\nrepos").expect("the record is replaced by text this build cannot read");
    let mut reports: Vec<RecoveryReport> = Vec::new();
    let entries: Vec<crate::service_timing::capture::Captured> = crate::service_timing::capture::entries_of(|| { reports = recover(&world.context); });
    assert_eq!(reports, Vec::new(), "nothing is decided about a record that cannot be read");
    assert!(intent.exists(), "a record that cannot be read is not destroyed");
    let said: Vec<&crate::service_timing::capture::Captured> = entries.iter().filter(|entry| { return entry.priority == crate::journald_native::Priority::Error; }).collect();
    assert_eq!(said.len(), 1, "{entries:?}");
    assert!(said[0].message.contains("cannot be read") && said[0].message.contains("kept"), "{}", said[0].message);
    assert!(said[0].message.contains(intent.display().to_string().as_str()), "the record is named: {}", said[0].message);
    assert!(said[0].message.contains("batch 1"), "the batch is named: {}", said[0].message);
    assert!(owed_of(&world, 1).exists(), "what the batch owes stays where the person's next command reads it");
    remove(world.root.as_path());
}

/// A landing killed at the instant its branch moves owes an index update,
/// and the record of it is written before the branch moves,
/// so the kill leaves what the person's next commit reads:
/// the index says the landed file is deleted,
/// and the commit records nothing about it.
#[test]
fn a_landing_killed_at_its_branch_move_leaves_what_it_owes() {
    let world: World = world("service-land-owed-killed-at-ref");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    crate::service_fault::hooks::set("after-ref", std::rc::Rc::new(|| panic!("the service was killed at its branch move")));
    let killed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { return land_batch(&world.context, 1, std::slice::from_ref(&landed)); }));
    crate::service_fault::hooks::clear();
    assert!(killed.is_err(), "the landing of the killed service never answered");
    assert_ne!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), base, "the branch moved before the kill");
    assert!(owed_of(&world, 1).exists(), "the record is written before the branch moves, so no kill can leave the obligation unrecorded");
    // Another program writes the index, so recovery cannot replay the update; the record it keeps is what the next commit reads.
    std::fs::write(world.repo.join("other.txt"), "other\n").expect("write");
    git(world.repo.as_path(), &["add", "other.txt"]);
    let reports: Vec<RecoveryReport> = recover(&world.context);
    assert_eq!(reports, vec![RecoveryReport { seq: 1, decision: Recovery::Finish, handled: Handled::Finished }], "the batch is finished");
    assert!(owed_of(&world, 1).exists(), "the record outlives the finished batch");
    let commit_index: PathBuf = commit_index_of(&world);
    assert_eq!(exclude(&world, Some(commit_index.as_path())), vec![String::from("one.txt")], "the commit leaves the landed path out");
    assert_eq!(index_text(&world, commit_index.as_path(), &["diff-index", "--cached", "--name-only", "HEAD"]), "other.txt", "the commit records nothing about the landed file, and the program's own staging stands");
    remove(world.root.as_path());
}

/// A commit of an index a landing left behind records nothing about the landed paths,
/// and the worktree's own index is not written:
/// what its entries hold may be a program's staging,
/// and only the person can say which.
#[test]
fn a_commit_of_a_stale_index_records_nothing_about_the_landed_paths() {
    let world: World = world("service-land-owed-exclusion");
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    assert_eq!(git_text(world.repo.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "D\tone.txt", "a commit of this index would record the landed file's deletion");
    let commit_index: PathBuf = commit_index_of(&world);
    let left: Vec<String> = exclude(&world, Some(commit_index.as_path()));
    assert_eq!(left, vec![String::from("one.txt")], "the commit leaves the landed path out");
    assert_eq!(index_text(&world, commit_index.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "", "the commit records nothing about the landed file");
    assert_eq!(git_text(world.repo.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "D\tone.txt", "the worktree's own index is not touched");
    assert!(owed_of(&world, 1).exists(), "the record stays until the worktree's index agrees with the branch");
    remove(world.root.as_path());
}

/// A person who restores the landed batch's old content and stages it keeps that staging:
/// the commit records nothing about the path,
/// and the live index still holds what they staged.
#[test]
fn a_staged_reversal_stands_in_the_index_and_the_commit_leaves_it_out() {
    let world: World = world("service-land-owed-reversal");
    std::fs::write(world.repo.join("tracked.txt"), "A\n").expect("write");
    git(world.repo.as_path(), &["add", "tracked.txt"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let landed: CommitRequest = request(&world, "landed", "tracked.txt", "B\n");
    land_with_failed_index(&world, 1, &[landed]);
    // The person puts the old content back and stages it: the entry is the batch's old one, and it is theirs.
    std::fs::write(world.repo.join("tracked.txt"), "A\n").expect("write");
    git(world.repo.as_path(), &["add", "tracked.txt"]);
    assert_eq!(git_text(world.repo.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "M\ttracked.txt");
    let commit_index: PathBuf = commit_index_of(&world);
    assert_eq!(exclude(&world, Some(commit_index.as_path())), vec![String::from("tracked.txt")], "the commit leaves the path out");
    assert_eq!(index_text(&world, commit_index.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "", "the commit records nothing about it");
    assert_eq!(git_text(world.repo.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "M\ttracked.txt", "the person's staged reversal stands");
    remove(world.root.as_path());
}

/// A merge in progress is the person's own act:
/// an unmerged path the batch's record names is left out of nothing,
/// and its conflict stages stand.
#[test]
fn a_path_with_conflict_stages_is_left_alone() {
    let world: World = world("service-land-owed-conflict");
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    // A merge in progress leaves the path unmerged: its three stages stand where one entry was.
    let stage = |content: &str| -> String {
        std::fs::write(world.root.join("stage-content"), content).expect("write");
        let file: String = world.root.join("stage-content").display().to_string();
        return git_text(world.repo.as_path(), &["hash-object", "-w", "--", file.as_str()]);
    };
    let (ours, theirs, base_blob) = (stage("ours\n"), stage("theirs\n"), stage("base\n"));
    let listing: String = format!(
        "0 {zero}\tone.txt\n100644 {base_blob} 1\tone.txt\n100644 {ours} 2\tone.txt\n100644 {theirs} 3\tone.txt\n",
        zero = "0".repeat(40)
    );
    let mut child = std::process::Command::new(crate::test_support::REAL_GIT)
        .current_dir(world.repo.as_path())
        .args(["update-index", "--index-info"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("git starts");
    {
        use std::io::Write as _;
        child.stdin.as_mut().expect("stdin").write_all(listing.as_bytes()).expect("the stages are fed");
    }
    assert!(child.wait().expect("git ends").success(), "the unmerged stages are made");
    assert!(!git_text(world.repo.as_path(), &["ls-files", "--unmerged"]).is_empty(), "the path is unmerged");
    let commit_index: PathBuf = commit_index_of(&world);
    assert_eq!(exclude(&world, Some(commit_index.as_path())), Vec::<String>::new(), "an unmerged path is the person's own and is not left out");
    assert!(!git_text(world.repo.as_path(), &["ls-files", "--unmerged"]).is_empty(), "the conflict stages stand");
    remove(world.root.as_path());
}

/// A record of a batch whose branch this worktree has left is forgotten,
/// and nothing of another branch's content is applied to this one's commits:
/// a file of the other branch that holds the landed batch's old content is that branch's own.
#[test]
fn a_record_of_another_branch_is_forgotten() {
    let world: World = world("service-land-owed-other-branch");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(world.repo.join("tracked.txt"), "A\n").expect("write");
    git(world.repo.as_path(), &["add", "tracked.txt"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let started: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let landed: CommitRequest = request(&world, "landed", "tracked.txt", "B\n");
    land_with_failed_index(&world, 1, &[landed]);
    // The other branch stands at the commit before the landing, where the file holds the batch's old content, and this
    //  worktree stages that file there.
    git(world.repo.as_path(), &["update-ref", "refs/heads/other", started.as_str()]);
    git(world.repo.as_path(), &["symbolic-ref", "HEAD", "refs/heads/other"]);
    git(world.repo.as_path(), &["update-index", "--add", "--", "tracked.txt"]);
    let commit_index: PathBuf = commit_index_of(&world);
    // The staged entry is the branch's own content, not what the landing replaced: nothing is left out, and no commit removes
    //  a record another command's snapshot may still need.
    assert_eq!(exclude(&world, Some(commit_index.as_path())), Vec::<String>::new(), "nothing of the other branch's landing is applied here");
    assert!(owed_of(&world, 1).exists(), "the record stays for the commands that still hold a stale snapshot");
    let _ = base;
    remove(world.root.as_path());
}

/// A batch whose index update could not be replayed because another program wrote the index since its landing is finished,
/// and what it owes is kept:
/// the next commit in the worktree records nothing about the landed paths,
/// and the entry the other program staged stands.
#[test]
fn a_batch_finished_with_its_index_written_meanwhile_keeps_what_it_owes() {
    let world: World = world("service-land-owed-written-meanwhile");
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    // The program the audit names for this state: a person's `git add`, after the landing and before any recovery.
    std::fs::write(world.repo.join("other.txt"), "other\n").expect("write");
    git(world.repo.as_path(), &["add", "other.txt"]);
    let reports: Vec<RecoveryReport> = recover(&world.context);
    assert_eq!(reports, vec![RecoveryReport { seq: 1, decision: Recovery::Finish, handled: Handled::Finished }], "the batch is finished");
    assert_eq!(git_text(world.repo.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "D\tone.txt\nA\tother.txt", "the index still describes the branch before the landing");
    assert!(owed_of(&world, 1).exists(), "what the batch owes outlives the batch, where the next commit reads it");
    let commit_index: PathBuf = commit_index_of(&world);
    assert_eq!(exclude(&world, Some(commit_index.as_path())), vec![String::from("one.txt")], "the commit leaves the landed path out");
    assert_eq!(index_text(&world, commit_index.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "A\tother.txt", "the commit records nothing about the landed file, and the program's staging stands");
    assert_eq!(git_text(world.repo.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "D\tone.txt\nA\tother.txt", "the worktree's index is not written");
    remove(world.root.as_path());
}

/// A stale entry that matches neither the head nor the worktree is the ordinary shape of a failed index update,
/// and the commit still leaves it out:
/// a removal that refused it would record the landed file's old content.
#[test]
fn a_stale_entry_that_matches_neither_the_head_nor_the_worktree_is_left_out() {
    let world: World = world("service-land-owed-neither");
    std::fs::write(world.repo.join("tracked.txt"), "A\n").expect("write");
    git(world.repo.as_path(), &["add", "tracked.txt"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let landed: CommitRequest = request(&world, "landed", "tracked.txt", "B\n");
    land_with_failed_index(&world, 1, &[landed]);
    // The index holds A, the head and the worktree hold B: neither is what the stale entry says.
    assert_eq!(git_text(world.repo.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "M\ttracked.txt");
    assert_eq!(std::fs::read_to_string(world.repo.join("tracked.txt")).expect("read"), "B\n", "the worktree holds the landed content");
    let commit_index: PathBuf = commit_index_of(&world);
    assert_eq!(exclude(&world, Some(commit_index.as_path())), vec![String::from("tracked.txt")], "the commit leaves the path out");
    assert_eq!(index_text(&world, commit_index.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "", "the commit records nothing about it, so the landed change is not reverted");
    assert_eq!(git_text(world.repo.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "M\ttracked.txt", "the worktree's own index stands");
    remove(world.root.as_path());
}

/// A program that catches the live index up between the copy and the comparison removes the record,
/// and the commit's own snapshot – which still holds the stale entry – is what the commit leaves out.
#[test]
fn a_stale_snapshot_is_left_out_even_when_the_live_index_is_caught_up() {
    let world: World = world("service-land-owed-snapshot");
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    let commit_index: PathBuf = commit_index_of(&world);
    // Another program adds the path, so the live index agrees with the head before this commit compares.
    git(world.repo.as_path(), &["add", "one.txt"]);
    assert_eq!(git_text(world.repo.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "");
    assert_eq!(exclude(&world, Some(commit_index.as_path())), vec![String::from("one.txt")], "the snapshot's stale entry is left out");
    assert_eq!(index_text(&world, commit_index.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "", "the commit cannot land the deletion");
    remove(world.root.as_path());
}

/// An amend rewrites the ancestry of the same branch and takes no index obligation away:
/// the landed commit is no longer below the head,
/// and its owed entry is still left out.
#[test]
fn an_amended_landing_still_leaves_its_owed_entry_out() {
    let world: World = world("service-land-owed-amend");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    // An amended commit with the landed commit's tree and its parent: the landed commit is not an ancestor any more.
    let tree: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    let amended: String = git_text(world.repo.as_path(), &["commit-tree", tree.as_str(), "-p", base.as_str(), "-m", "amended"]);
    git(world.repo.as_path(), &["update-ref", "refs/heads/main", amended.as_str()]);
    let commit_index: PathBuf = commit_index_of(&world);
    assert_eq!(exclude(&world, Some(commit_index.as_path())), vec![String::from("one.txt")], "the obligatation of the same branch survives the amend");
    assert_eq!(index_text(&world, commit_index.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "", "the commit records nothing about the landed path");
    assert!(owed_of(&world, 1).exists(), "the worktree's index still holds the stale entry, so the record stays");
    remove(world.root.as_path());
}

/// A record whose obligation the landing finished survives a kill,
/// and it is forgotten before a person's later staging of one of its paths is taken for an owed entry.
#[test]
fn a_settled_record_is_forgotten_before_a_later_staging() {
    let world: World = world("service-land-owed-settled");
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    // The installation the landing made is the last of its obligations: the index catches up, and the kill came before the record
    //  went. The person then stages a change of that path of their own.
    crate::owed_index_update::settle(world.repo.join(".git").as_path(), 1);
    git(world.repo.as_path(), &["add", "one.txt"]);
    std::fs::write(world.repo.join("one.txt"), "mine\n").expect("write");
    git(world.repo.as_path(), &["add", "one.txt"]);
    assert!(!owed_of(&world, 1).exists(), "the settled record is gone");
    let commit_index: PathBuf = commit_index_of(&world);
    assert_eq!(exclude(&world, Some(commit_index.as_path())), Vec::<String>::new(), "their staging is not taken for an owed entry");
    assert_eq!(index_text(&world, commit_index.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "M\tone.txt", "their change is committed");
    remove(world.root.as_path());
}

/// A record made on a detached head does not apply after the worktree attaches to a branch:
/// that branch's index is not the detached head's.
#[test]
fn a_record_of_a_detached_head_does_not_apply_after_attaching_to_a_branch() {
    let world: World = world("service-land-owed-detached");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    git(world.repo.as_path(), &["checkout", "--quiet", "--detach", base.as_str()]);
    let mut landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    landed.reference = String::from("HEAD");
    land_with_failed_index(&world, 1, &[landed]);
    assert_eq!(record_of(&world).reference, "HEAD", "the record names the detached head it landed on");
    // The worktree attaches to a branch at the landed commit, and stages that path there.
    git(world.repo.as_path(), &["symbolic-ref", "HEAD", "refs/heads/main"]);
    git(world.repo.as_path(), &["update-ref", "refs/heads/main", "HEAD"]);
    std::fs::write(world.repo.join("one.txt"), "mine\n").expect("write");
    git(world.repo.as_path(), &["add", "one.txt"]);
    let commit_index: PathBuf = commit_index_of(&world);
    assert_eq!(exclude(&world, Some(commit_index.as_path())), Vec::<String>::new(), "the detached head's record does not apply to the branch");
    assert_eq!(index_text(&world, commit_index.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "A\tone.txt", "the branch's staging is committed");
    remove(world.root.as_path());
}

/// A record another command removed does not leave a snapshot already copied unprotected:
/// a path whose live entry moved after the copy is left out of the commit,
/// because what the snapshot would record there is not the person's current intent.
#[test]
fn a_snapshot_older_than_the_live_index_is_left_out_after_another_command_removed_the_record() {
    let world: World = world("service-land-owed-older-snapshot");
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    // A commit copies the stale index here, and another command catches the live index up and settles the record.
    let commit_index: PathBuf = commit_index_of(&world);
    git(world.repo.as_path(), &["add", "one.txt"]);
    crate::owed_index_update::settle(world.repo.join(".git").as_path(), 1);
    assert_eq!(git_text(world.repo.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "");
    // Nothing in the record covers it now; the commit's own index is still older than the index, so its entry is left out.
    // The record is gone, and the commit's copy is older than the index: that is all the guard needs.
    let left: Vec<String> = exclude_after_the_index_moved(&world, Some(commit_index.as_path()), real_index_of(&world).as_path());
    assert_eq!(left, vec![String::from("one.txt")], "the entry the copy would record is left out and said");
    assert_eq!(
        index_text(&world, commit_index.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]),
        "",
        "the more a stale snapshot would record is not the person's current intent, so the commit records nothing about it"
    );
    remove(world.root.as_path());
}

/// A history reset that replaced the index leaves no content the landing replaced,
/// so the person's staging on that path is committed and not left out.
#[test]
fn a_record_of_a_landing_a_reset_replaced_does_not_leave_a_person_staging_out() {
    let world: World = world("service-land-owed-reset-hard");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    // Real Git's reset --hard to before the landing replaces the index with that commit's.
    git(world.repo.as_path(), &["reset", "--hard", base.as_str()]);
    std::fs::write(world.repo.join("one.txt"), "mine\n").expect("write");
    git(world.repo.as_path(), &["add", "one.txt"]);
    let commit_index: PathBuf = commit_index_of(&world);
    assert_eq!(exclude(&world, Some(commit_index.as_path())), Vec::<String>::new(), "nothing the landing replaced is in the index any more");
    assert_eq!(index_text(&world, commit_index.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "A\tone.txt", "their staging is committed");
    remove(world.root.as_path());
}

/// An obligation of an alternate index applies to a commit of that index even after the worktree attaches to a branch:
/// the record asks what the index holds,
/// not what the head is called.
#[test]
fn an_alternate_indexes_obligation_survives_attaching_a_detached_head() {
    let world: World = world("service-land-owed-alternate-attach");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    git(world.repo.as_path(), &["checkout", "--quiet", "--detach", base.as_str()]);
    let mut landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    landed.reference = String::from("HEAD");
    landed.index = world.root.join("alternate.index");
    std::fs::copy(real_index_of(&world), landed.index.as_path()).expect("the alternate index starts as the worktree's");
    land_with_failed_index(&world, 1, &[landed]);
    // The worktree attaches to a branch at the landed commit, and the alternate index keeps its stale entry.
    git(world.repo.as_path(), &["symbolic-ref", "HEAD", "refs/heads/main"]);
    git(world.repo.as_path(), &["update-ref", "refs/heads/main", "HEAD"]);
    let alternate: PathBuf = world.root.join("alternate.index");
    let commit_index: PathBuf = world.root.join("commit.index");
    std::fs::copy(alternate.as_path(), commit_index.as_path()).expect("a commit of that index copies it");
    let left: Vec<String> = exclude_after_the_index_moved(&world, Some(commit_index.as_path()), alternate.as_path());
    assert_eq!(left, vec![String::from("one.txt")], "the alternate index's stale entry is left out");
    assert_eq!(
        index_text(&world, commit_index.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]),
        "",
        "the commit cannot record the landed file as deleted"
    );
    remove(world.root.as_path());
}

/// Recovery settles an obligation whose installation it finds recorded:
/// a kill between the installation and the landing's own settling leaves nothing that would withhold a later staging.
#[test]
fn recovery_settles_the_record_of_an_installation_it_finds_recorded() {
    let world: World = world("service-land-owed-installed");
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    // The installation was made and recorded, and the kill came before the landing settled the record.
    git(world.repo.as_path(), &["read-tree", "HEAD"]);
    let installed: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(batch_path(world.context.journal.as_path(), 1, crate::service_journal::INDEXED_EXTENSION), format!("{installed}\n")).expect("the completion record");
    assert!(owed_of(&world, 1).exists(), "the kill left the record");
    let reports: Vec<RecoveryReport> = recover(&world.context);
    assert_eq!(reports, vec![RecoveryReport { seq: 1, decision: Recovery::Finish, handled: Handled::Finished }], "the batch is finished");
    assert!(!owed_of(&world, 1).exists(), "nothing is owed, so the record is settled");
    // A later staging of that path is committed, not left out.
    std::fs::write(world.repo.join("one.txt"), "mine\n").expect("write");
    git(world.repo.as_path(), &["add", "one.txt"]);
    let commit_index: PathBuf = commit_index_of(&world);
    assert_eq!(exclude(&world, Some(commit_index.as_path())), Vec::<String>::new(), "their staging is not taken for an owed entry");
    assert_eq!(index_text(&world, commit_index.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "M\tone.txt", "their change is committed");
    remove(world.root.as_path());
}

/// One Git command in the world's repository with `index` as its index file.
fn index_run(world: &World, index: &Path, arguments: &[&str]) {
    let output: std::process::Output = std::process::Command::new(crate::test_support::REAL_GIT)
        .current_dir(world.repo.as_path())
        .env("GIT_INDEX_FILE", index)
        .args(arguments)
        .output()
        .expect("Git runs");
    assert!(output.status.success(), "git {arguments:?} failed: {}", String::from_utf8_lossy(output.stderr.as_slice()));
}

/// A Git whose named subcommands refuse,
/// and whose every other command is the real one's.
fn git_refusing(world: &World, refusing: &[&str]) -> PathBuf {
    let program: PathBuf = world.root.join("refusing-git");
    let mut script: String = String::from("#!/bin/sh\ncase \" $* \" in\n");
    for subcommand in refusing {
        script.push_str(format!("  *\" {subcommand} \"*) exit 1;;\n").as_str());
    }
    script.push_str(format!("esac\nexec '{}' \"$@\"\n", crate::test_support::REAL_GIT).as_str());
    std::fs::write(program.as_path(), script).expect("script");
    std::fs::set_permissions(program.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("executable");
    return program;
}

/// A tab inside a file name is the name's own character,
/// never the tab that ends an entry's words:
/// a record read from the back would take the file `b` for a conflict of another path,
/// skip it,
/// and commit its stale content.
#[test]
fn a_tab_in_a_file_name_is_not_taken_for_a_conflict() {
    let world: World = world("service-land-owed-tab-name");
    for name in ["b", "z\tb"] {
        std::fs::write(world.repo.join(name), "A\n").expect("write");
        git(world.repo.as_path(), &["add", "--", name]);
    }
    git(world.repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let first: CommitRequest = request(&world, "one", "b", "B\n");
    let second: CommitRequest = request(&world, "two", "z\tb", "B\n");
    land_with_failed_index(&world, 1, &[first, second]);
    let commit_index: PathBuf = commit_index_of(&world);
    let left: Vec<String> = exclude(&world, Some(commit_index.as_path()));
    assert_eq!(left.len(), 2, "both stale entries are left out: {left:?}");
    assert_eq!(
        index_text(&world, commit_index.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]),
        "",
        "neither file's stale content is recorded"
    );
    remove(world.root.as_path());
}

/// What the command staged itself (`--all`) is the command's own,
/// not an entry the live index moved:
/// the worktree's content is committed.
#[test]
fn a_commit_all_keeps_the_content_it_staged() {
    let world: World = world("service-land-owed-commit-all");
    std::fs::write(world.repo.join("tracked.txt"), "A\n").expect("write");
    git(world.repo.as_path(), &["add", "tracked.txt"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let landed: CommitRequest = request(&world, "landed", "tracked.txt", "B\n");
    land_with_failed_index(&world, 1, &[landed]);
    let commit_index: PathBuf = commit_index_of(&world);
    // The person edits the worktree and asks for `--all`: the command stages that content into its own index, and the live index is
    //  exactly the file it copied.
    std::fs::write(world.repo.join("tracked.txt"), "C\n").expect("write");
    index_run(&world, commit_index.as_path(), &["add", "--", "tracked.txt"]);
    assert_eq!(exclude(&world, Some(commit_index.as_path())), Vec::<String>::new(), "the command's own staging is not left out");
    let blob: String = git_text(world.repo.as_path(), &["hash-object", "-w", "--", "tracked.txt"]);
    assert_eq!(
        index_text(&world, commit_index.as_path(), &["ls-files", "-s", "--", "tracked.txt"]),
        format!("100644 {blob} 0\ttracked.txt"),
        "the worktree's content is what the commit records"
    );
    remove(world.root.as_path());
}

/// The notice names a command that works for a stale addition:
/// `git commit -- <paths>` is refused there,
/// and `git add -- <paths>` then `git commit` is not.
#[test]
fn the_notice_names_a_command_that_works_for_a_stale_addition() {
    let notice: String = crate::owed_index_update::excluded_notice(&[String::from("p")]).expect("a notice");
    assert!(notice.contains("git add -- <paths>"), "{notice}");
    assert!(!notice.contains("git commit -- <paths>"), "the refused command is not advertised: {notice}");
    assert!(notice.contains("git reset -- <paths>"), "{notice}");
}

/// A `reset` the wrapper refuses is not the end of the way to the head:
/// the entry is written where the head's tree has it,
/// so the commit still records nothing about the landed path.
#[test]
fn a_refused_reset_still_sets_the_entry_to_the_head() {
    let world: World = world("service-land-owed-refused-reset");
    std::fs::write(world.repo.join("tracked.txt"), "A\n").expect("write");
    git(world.repo.as_path(), &["add", "tracked.txt"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let landed: CommitRequest = request(&world, "landed", "tracked.txt", "B\n");
    land_with_failed_index(&world, 1, &[landed]);
    let commit_index: PathBuf = commit_index_of(&world);
    let mut context: crate::transaction_git::GitContext = world.context.git.clone();
    context.real_git = git_refusing(&world, &["reset"]);
    let left: Vec<String> = crate::owed_index_update::exclude_what_a_landed_commit_left(
        &context,
        world.repo.join(".git").as_path(),
        real_index_of(&world).as_path(),
        Some(commit_index.as_path()),
        &crate::service_land_start::IndexWitness::Absent,
    );
    assert_eq!(left, vec![String::from("tracked.txt")], "the path is left out even when the reset refuses");
    assert_eq!(index_text(&world, commit_index.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "", "the commit records nothing about the landed path");
    remove(world.root.as_path());
}

/// When every way to set the entry to the head refuses,
/// the commit's index is left exactly as it was:
/// a successful removal never outlives a refused write.
#[test]
fn a_refused_reset_and_write_leaves_the_index_as_it_was() {
    let world: World = world("service-land-owed-refused-both");
    std::fs::write(world.repo.join("tracked.txt"), "A\n").expect("write");
    git(world.repo.as_path(), &["add", "tracked.txt"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let landed: CommitRequest = request(&world, "landed", "tracked.txt", "B\n");
    land_with_failed_index(&world, 1, &[landed]);
    let commit_index: PathBuf = commit_index_of(&world);
    let before: String = index_text(&world, commit_index.as_path(), &["ls-files", "-s"]);
    let mut context: crate::transaction_git::GitContext = world.context.git.clone();
    context.real_git = git_refusing(&world, &["reset", "update-index"]);
    let left: Vec<String> = crate::owed_index_update::exclude_what_a_landed_commit_left(
        &context,
        world.repo.join(".git").as_path(),
        real_index_of(&world).as_path(),
        Some(commit_index.as_path()),
        &crate::service_land_start::IndexWitness::Absent,
    );
    assert!(left.is_empty(), "nothing is claimed to be left out when nothing was: {left:?}");
    assert_eq!(index_text(&world, commit_index.as_path(), &["ls-files", "-s"]), before, "the index is exactly as it was");
    remove(world.root.as_path());
}

/// A check that cannot be made is never read as "nothing moved":
/// an index that a tree cannot be written from (a path in conflict elsewhere) still leaves the moved paths out.
#[test]
fn a_check_that_cannot_be_made_still_leaves_the_moved_paths_out() {
    let world: World = world("service-land-owed-check-fallback");
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    let commit_index: PathBuf = commit_index_of(&world);
    git(world.repo.as_path(), &["add", "one.txt"]);
    // The record is settled, so the record's own comparison cannot be what leaves the path out: only the check that the copied index
    //  is older than the live one can.
    crate::owed_index_update::settle(world.repo.join(".git").as_path(), 1);
    // A path in conflict in the live index: `write-tree` refuses it, so the check falls back to the listings.
    let stage = |content: &str| -> String {
        std::fs::write(world.root.join("stage-content"), content).expect("write");
        let file: String = world.root.join("stage-content").display().to_string();
        return git_text(world.repo.as_path(), &["hash-object", "-w", "--", file.as_str()]);
    };
    let listing: String = format!(
        "0 {zero}\tq\n100644 {base_blob} 1\tq\n100644 {ours} 2\tq\n100644 {theirs} 3\tq\n",
        zero = "0".repeat(40),
        base_blob = stage("base\n"),
        ours = stage("ours\n"),
        theirs = stage("theirs\n")
    );
    let mut child = std::process::Command::new(crate::test_support::REAL_GIT)
        .current_dir(world.repo.as_path())
        .args(["update-index", "--index-info"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("git starts");
    {
        use std::io::Write as _;
        child.stdin.as_mut().expect("stdin").write_all(listing.as_bytes()).expect("the stages are fed");
    }
    assert!(child.wait().expect("git ends").success(), "the conflict stages are made");
    let left: Vec<String> = exclude_after_the_index_moved(&world, Some(commit_index.as_path()), real_index_of(&world).as_path());
    assert!(left.contains(&String::from("one.txt")), "the moved path is left out and said: {left:?}");
    assert_eq!(
        index_text(&world, commit_index.as_path(), &["diff-index", "--cached", "--name-status", "HEAD", "--", "one.txt"]),
        "",
        "the commit cannot record the landed file as deleted"
    );
    remove(world.root.as_path());
}
