//! Controls for the index update a landed batch owes:
//! the record of it in the worktree's own Git directory,
//! what recovery does with an open record that cannot be read,
//! and what a commit does about it,
//! which is to record nothing about the paths it cannot prove.

use super::tests::{World, land_with_failed_index, request, world};
use super::*;
/// The trait that gives a child's input and this process's standard error their `write_all`.
use std::io::Write as _;
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
    let copy: PathBuf = commit_index.map_or_else(|| { return real_index_of(world); }, |index| { return index.to_path_buf(); });
    return crate::owed_index_update::exclude_what_a_landed_commit_left(&world.context.git, world.repo.as_path(), world.repo.join(".git").as_path(), real_index_of(world).as_path(), commit_index, Some(copy.as_path()), &captured).expect("the repair that has nothing to move");
}

/// The same with the index file the commit copied named as gone,
/// which is what a writer that replaced the index leaves behind.
fn exclude_after_the_index_moved(world: &World, commit_index: Option<&Path>, of_index: &Path) -> Vec<String> {
    return crate::owed_index_update::exclude_what_a_landed_commit_left(
        &world.context.git,
        world.repo.as_path(),
        world.repo.join(".git").as_path(),
        of_index,
        commit_index,
        commit_index,
        &crate::service_land_start::IndexWitness::Absent,
    )
    .expect("the repair that has nothing to move");
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
    assert_eq!(record_of(&world).paths, vec![b"one.txt".to_vec()], "the record names the commit's paths");
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
    assert_eq!(reports, vec![RecoveryReport { seq: 1, repository: world.repo.clone(), decision: Recovery::Finish, handled: Handled::Finished }], "the batch is finished");
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

/// A person who restores the landed batch's old content and stages it keeps that staging,
/// and their commit records it:
/// the worktree holds what they staged,
/// so the entry is their statement and not the landing's leftover.
#[test]
fn a_staged_reversal_stands_in_the_index_and_is_committed() {
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
    assert_eq!(exclude(&world, Some(commit_index.as_path())), Vec::<String>::new(), "their own statement is not left out");
    assert_eq!(index_text(&world, commit_index.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "M\ttracked.txt", "the commit records their reversal");
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
    assert_eq!(reports, vec![RecoveryReport { seq: 1, repository: world.repo.clone(), decision: Recovery::Finish, handled: Handled::Finished }], "the batch is finished");
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
    landed.reference = crate::ref_name::RefName::from_text("HEAD");
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
    keep_one_record(&world);
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
    landed.reference = crate::ref_name::RefName::from_text("HEAD");
    landed.index = world.root.join("alternate.index");
    std::fs::copy(real_index_of(&world), landed.index.as_path()).expect("the alternate index starts as the worktree's");
    land_with_failed_index(&world, 1, &[landed]);
    // The worktree attaches to a branch at the landed commit, and the alternate index keeps its stale entry.
    let landed_tip: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    git(world.repo.as_path(), &["update-ref", "refs/heads/main", landed_tip.as_str()]);
    git(world.repo.as_path(), &["symbolic-ref", "HEAD", "refs/heads/main"]);
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
    assert_eq!(reports, vec![RecoveryReport { seq: 1, repository: world.repo.clone(), decision: Recovery::Finish, handled: Handled::Finished }], "the batch is finished");
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
        world.repo.as_path(),
        world.repo.join(".git").as_path(),
        real_index_of(&world).as_path(),
        Some(commit_index.as_path()),
        Some(commit_index.as_path()),
        &crate::service_land_start::IndexWitness::Absent,
    )
    .expect("the repair that has nothing to move");
    assert_eq!(left, vec![String::from("tracked.txt")], "the path is left out even when the reset refuses");
    assert_eq!(index_text(&world, commit_index.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "", "the commit records nothing about the landed path");
    remove(world.root.as_path());
}

/// The repair looks its entries up by repository-relative path,
/// while `git ls-files` answers relative to the directory Git runs in:
/// a commit from a nested directory must not read the person's staged entry as absent and reset it to `HEAD`.
#[test]
fn a_repair_keeps_the_staged_entry_of_a_nested_path_under_a_caller_prefix() {
    let world: World = world("service-land-owed-nested-c");
    let landed: CommitRequest = request(&world, "landed", "sub/f", "landed\n");
    land_with_failed_index(&world, 1, &[landed]);
    // The person writes other contents and stages them with real Git, so the worktree matches their staging.
    std::fs::write(world.repo.join("sub/f"), "staged\n").expect("stage other contents");
    git(world.repo.as_path(), &["add", "--", "sub/f"]);
    let commit_index: PathBuf = commit_index_of(&world);
    assert_eq!(index_text(&world, commit_index.as_path(), &["show", ":sub/f"]), "staged", "the commit's copy holds the staged entry");
    // The caller wrote `-C <repository>/sub`, so every Git of this commit runs one directory down.
    let mut context: crate::transaction_git::GitContext = world.context.git.clone();
    context.global_prefix = vec![std::ffi::OsString::from("-C"), world.repo.join("sub").into_os_string()];
    let captured: crate::service_land_start::IndexWitness =
        crate::service_land_start::witness_of(real_index_of(&world).as_path()).unwrap_or(crate::service_land_start::IndexWitness::Absent);
    let left: Vec<String> = crate::owed_index_update::exclude_what_a_landed_commit_left(
        &context,
        world.repo.as_path(),
        world.repo.join(".git").as_path(),
        real_index_of(&world).as_path(),
        Some(commit_index.as_path()),
        Some(commit_index.as_path()),
        &captured,
    )
    .expect("the repair of the landed path");
    assert_eq!(left, Vec::<String>::new(), "the person's own staging is left alone");
    assert_eq!(index_text(&world, commit_index.as_path(), &["show", ":sub/f"]), "staged", "the staged contents stay in the commit's index");
    remove(world.root.as_path());
}

/// A `--literal-pathspecs` the caller wrote as a global option makes every read answer empty,
/// so the repair would exclude nothing and the next ordinary commit would record the contents the landing replaced.
#[test]
fn a_repair_excludes_the_landed_path_under_the_callers_literal_option() {
    let world: World = world("service-land-owed-literal-option");
    std::fs::write(world.repo.join("f"), "A\n").expect("write");
    git(world.repo.as_path(), &["add", "f"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let landed: CommitRequest = request(&world, "landed", "f", "B\n");
    land_with_failed_index(&world, 1, &[landed]);
    let commit_index: PathBuf = commit_index_of(&world);
    let mut context: crate::transaction_git::GitContext = world.context.git.clone();
    context.global_prefix.push(std::ffi::OsString::from("--literal-pathspecs"));
    let captured: crate::service_land_start::IndexWitness =
        crate::service_land_start::witness_of(real_index_of(&world).as_path()).unwrap_or(crate::service_land_start::IndexWitness::Absent);
    let left: Vec<String> = crate::owed_index_update::exclude_what_a_landed_commit_left(
        &context,
        world.repo.as_path(),
        world.repo.join(".git").as_path(),
        real_index_of(&world).as_path(),
        Some(commit_index.as_path()),
        Some(commit_index.as_path()),
        &captured,
    )
    .expect("the repair of the landed path");
    assert_eq!(left, vec![String::from("f")], "the path the landing replaced is left out");
    assert_eq!(index_text(&world, commit_index.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "", "the commit records nothing about the landed path");
    remove(world.root.as_path());
}

/// The repair changes the paths it decided on and nothing else:
/// with `GIT_ICASE_PATHSPECS=1` in the service's environment,
/// `:(top,literal)a.txt` matches `A.txt` as well,
/// so the removal before the write takes the staged entry of `A.txt` out of the commit's index,
/// and the commit records that path as a deletion.
#[test]
fn a_repair_removes_no_staged_entry_of_a_case_twin() {
    let world: World = world("service-land-owed-icase-prune");
    std::fs::write(world.repo.join("A.txt"), "upper\n").expect("write A.txt");
    std::fs::write(world.repo.join("a.txt"), "lower\n").expect("write a.txt");
    git(world.repo.as_path(), &["add", "--", "A.txt", "a.txt"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=base", "--", "A.txt", "a.txt"]);
    let landed: CommitRequest = request(&world, "landed", "a.txt", "lower changed\n");
    land_with_failed_index(&world, 1, &[landed]);
    let commit_index: PathBuf = commit_index_of(&world);
    // The commit stages its own edit of `A.txt` into its copy, and the worktree copy goes back,
    // so those contents exist in the index and the object store alone.
    std::fs::write(world.repo.join("A.txt"), "staged only\n").expect("stage other contents");
    index_run(&world, commit_index.as_path(), &["add", "--", "A.txt"]);
    std::fs::write(world.repo.join("A.txt"), "worktree again\n").expect("overwrite the worktree copy");
    assert_eq!(index_text(&world, commit_index.as_path(), &["show", ":A.txt"]), "staged only", "the commit's copy holds the staged entry");
    let mut context: crate::transaction_git::GitContext = world.context.git.clone();
    context.overlay.push((std::ffi::OsString::from("GIT_ICASE_PATHSPECS"), std::ffi::OsString::from("1")));
    let captured: crate::service_land_start::IndexWitness =
        crate::service_land_start::witness_of(real_index_of(&world).as_path()).unwrap_or(crate::service_land_start::IndexWitness::Absent);
    let left: Vec<String> = crate::owed_index_update::exclude_what_a_landed_commit_left(
        &context,
        world.repo.as_path(),
        world.repo.join(".git").as_path(),
        real_index_of(&world).as_path(),
        Some(commit_index.as_path()),
        Some(commit_index.as_path()),
        &captured,
    )
    .expect("the repair of the landed path");
    assert_eq!(left, vec![String::from("a.txt")], "the landed path is left out");
    assert_eq!(
        index_text(&world, commit_index.as_path(), &["show", ":A.txt"]),
        "staged only",
        "the staged entry the batch never selected stays in the commit's index"
    );
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
    let answered: Result<Vec<String>, String> = crate::owed_index_update::exclude_what_a_landed_commit_left(
        &context,
        world.repo.as_path(),
        world.repo.join(".git").as_path(),
        real_index_of(&world).as_path(),
        Some(commit_index.as_path()),
        Some(commit_index.as_path()),
        &crate::service_land_start::IndexWitness::Absent,
    );
    assert!(answered.is_err(), "a repair that cannot be made stops the commit: {answered:?}");
    assert_eq!(index_text(&world, commit_index.as_path(), &["ls-files", "-s"]), before, "and the index is exactly as it was");
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
    keep_one_record(&world);
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


/// A second owed record of an unrelated path,
/// so the repair has work to do at all:
/// with no record it compares nothing.
fn keep_one_record(world: &World) {
    let commit: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let owed: crate::owed_index_update::Owed = crate::owed_index_update::Owed {
        seq: 99,
        repository: world.repo.clone(),
        git_dir: world.repo.join(".git"),
        index: real_index_of(world),
        reference: String::from("refs/heads/main"),
        old: commit.clone(),
        new: commit,
        paths: vec![b"one.txt".to_vec()],
        derived: false,
    };
    crate::owed_index_update::write(&owed).expect("an unrelated record");
}

/// A concurrent command that writes the index does not remove what `git commit --all` staged:
/// the live index is compared with the copy the commit made,
/// never with the index the commit has already staged into.
#[test]
fn a_commit_all_keeps_its_edit_when_another_command_writes_the_index() {
    let world: World = world("service-land-owed-all-concurrent");
    std::fs::write(world.repo.join("f"), "A\n").expect("write");
    git(world.repo.as_path(), &["add", "f"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let capture: PathBuf = world.root.join("capture.index");
    std::fs::copy(real_index_of(&world), capture.as_path()).expect("the copy a commit makes before staging");
    let commit_index: PathBuf = commit_index_of(&world);
    // `git commit --all`: the person's edit is staged into the command's own index.
    std::fs::write(world.repo.join("f"), "C\n").expect("write");
    index_run(&world, commit_index.as_path(), &["add", "--", "f"]);
    // Another command stages an unrelated file, replacing the live index.
    std::fs::write(world.repo.join("r"), "r\n").expect("write");
    git(world.repo.as_path(), &["add", "r"]);
    let left: Vec<String> = crate::owed_index_update::exclude_what_a_landed_commit_left(
        &world.context.git,
        world.repo.as_path(),
        world.repo.join(".git").as_path(),
        real_index_of(&world).as_path(),
        Some(commit_index.as_path()),
        Some(capture.as_path()),
        &crate::service_land_start::IndexWitness::Absent,
    )
    .expect("the repair that has nothing to move");
    assert!(!left.contains(&String::from("f")), "the command's own staging is not reported as moved: {left:?}");
    let blob: String = git_text(world.repo.as_path(), &["hash-object", "-w", "--", "f"]);
    assert_eq!(index_text(&world, commit_index.as_path(), &["ls-files", "-s", "--", "f"]), format!("100644 {blob} 0\tf"), "the person's edit is what the commit records");
    remove(world.root.as_path());
}

/// A change the repair decided on and cannot make stops the commit,
/// because continuing would record the content the landing replaced.
#[test]
fn a_repair_that_cannot_be_made_stops_the_commit() {
    let world: World = world("service-land-owed-stops");
    std::fs::write(world.repo.join("f"), "A\n").expect("write");
    git(world.repo.as_path(), &["add", "f"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let landed: CommitRequest = request(&world, "landed", "f", "B\n");
    land_with_failed_index(&world, 1, &[landed]);
    let commit_index: PathBuf = commit_index_of(&world);
    let mut context: crate::transaction_git::GitContext = world.context.git.clone();
    context.real_git = git_refusing(&world, &["reset", "update-index"]);
    let answered: Result<Vec<String>, String> = crate::owed_index_update::exclude_what_a_landed_commit_left(
        &context,
        world.repo.as_path(),
        world.repo.join(".git").as_path(),
        real_index_of(&world).as_path(),
        Some(commit_index.as_path()),
        Some(commit_index.as_path()),
        &crate::service_land_start::IndexWitness::Absent,
    );
    let reason: String = answered.expect_err("a repair that cannot be made stops the commit");
    assert!(reason.contains("f") || reason.contains("batch 1"), "{reason}");
    remove(world.root.as_path());
}

/// A worktree whose Git directory stands outside it is repaired where the worktree is:
/// the Git questions are asked in the repository preparation already knows.
#[test]
fn a_separate_git_directory_is_repaired_in_the_worktree() {
    let root: PathBuf = crate::test_support::fixture("service-land-owed-separate-git-dir");
    let work: PathBuf = root.join("work");
    std::fs::create_dir_all(work.as_path()).expect("worktree");
    std::fs::create_dir_all(root.join("metadata").as_path()).expect("metadata");
    let metadata: PathBuf = root.join("metadata").join("repo.git");
    git(work.as_path(), &["init", "--quiet", "--separate-git-dir", metadata.display().to_string().as_str()]);
    std::fs::write(work.join("f"), "A\n").expect("write");
    git(work.as_path(), &["add", "f"]);
    git(work.as_path(), &["commit", "--quiet", "--message=base"]);
    let old: String = git_text(work.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(work.join("f"), "B\n").expect("write");
    git(work.as_path(), &["add", "f"]);
    git(work.as_path(), &["commit", "--quiet", "--message=landed"]);
    let landed: String = git_text(work.as_path(), &["rev-parse", "HEAD"]);
    let index: PathBuf = metadata.join("index");
    // The index still says A, as a failed index update leaves it, and the record names the worktree and its Git directory.
    let old_blob: String = git_text(work.as_path(), &["rev-parse", format!("{old}:f").as_str()]);
    index_run_in(work.as_path(), index.as_path(), &["update-index", "--add", "--cacheinfo", "100644", old_blob.as_str(), "f"]);
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(journal.as_path()).expect("journal");
    let owed: crate::owed_index_update::Owed = crate::owed_index_update::Owed {
        seq: 1,
        repository: work.clone(),
        git_dir: metadata.clone(),
        index: index.clone(),
        reference: String::from("refs/heads/main"),
        old,
        new: landed,
        paths: vec![b"f".to_vec()],
        derived: false,
    };
    crate::owed_index_update::write(&owed).expect("the record");
    let commit_index: PathBuf = root.join("commit.index");
    std::fs::copy(index.as_path(), commit_index.as_path()).expect("a commit of that index copies it");
    let left: Vec<String> = crate::owed_index_update::exclude_what_a_landed_commit_left(
        &crate::test_support::git_context(),
        work.as_path(),
        metadata.as_path(),
        index.as_path(),
        Some(commit_index.as_path()),
        Some(commit_index.as_path()),
        &crate::service_land_start::IndexWitness::Absent,
    )
    .expect("the repair asks in the worktree");
    assert_eq!(left, vec![String::from("f")], "the stale entry is left out");
    assert_eq!(index_run_text(work.as_path(), commit_index.as_path(), &["diff-index", "--cached", "--name-status", "HEAD"]), "", "the commit records nothing about the landed path");
    remove(root.as_path());
}

/// One Git command in `repository` with `index` as its index file.
fn index_run_in(repository: &Path, index: &Path, arguments: &[&str]) {
    let output: std::process::Output = std::process::Command::new(crate::test_support::REAL_GIT)
        .current_dir(repository)
        .env("GIT_INDEX_FILE", index)
        .args(arguments)
        .output()
        .expect("Git runs");
    assert!(output.status.success(), "git {arguments:?} failed: {}", String::from_utf8_lossy(output.stderr.as_slice()));
}

/// What one Git command prints in `repository` with `index` as its index file.
fn index_run_text(repository: &Path, index: &Path, arguments: &[&str]) -> String {
    let output: std::process::Output = std::process::Command::new(crate::test_support::REAL_GIT)
        .current_dir(repository)
        .env("GIT_INDEX_FILE", index)
        .args(arguments)
        .output()
        .expect("Git runs");
    return String::from_utf8_lossy(output.stdout.as_slice()).trim_end().to_string();
}

/// A deliberate restoration of the content the landing replaced is the person's own statement:
/// the worktree matches the index,
/// so the path is committed and not left out.
#[test]
fn a_restoration_the_worktree_matches_is_committed() {
    let world: World = world("service-land-owed-restoration");
    std::fs::write(world.repo.join("f"), "A\n").expect("write");
    git(world.repo.as_path(), &["add", "f"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let landed: CommitRequest = request(&world, "landed", "f", "B\n");
    land_with_failed_index(&world, 1, &[landed]);
    // The person deliberately puts the old content back and stages it; the worktree matches what they staged.
    std::fs::write(world.repo.join("f"), "A\n").expect("write");
    git(world.repo.as_path(), &["add", "f"]);
    let commit_index: PathBuf = commit_index_of(&world);
    let left: Vec<String> = exclude(&world, Some(commit_index.as_path()));
    assert_eq!(left, Vec::<String>::new(), "their restoration is not left out");
    let blob: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD^:f"]);
    assert_eq!(index_text(&world, commit_index.as_path(), &["ls-files", "-s", "--", "f"]), format!("100644 {blob} 0\tf"), "the commit records their restoration");
    remove(world.root.as_path());
}

/// A commit that owes nothing is not compared at all:
/// a concurrent write of the index never drops an edit `git commit --all` selected,
/// because no record names a path here.
#[test]
fn a_commit_with_no_owed_record_is_never_compared() {
    let world: World = world("service-land-owed-nothing");
    std::fs::write(world.repo.join("f"), "A\n").expect("write");
    git(world.repo.as_path(), &["add", "f"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let capture: PathBuf = world.root.join("capture.index");
    std::fs::copy(real_index_of(&world), capture.as_path()).expect("the copy a commit makes");
    let commit_index: PathBuf = commit_index_of(&world);
    std::fs::write(world.repo.join("f"), "B\n").expect("write");
    index_run(&world, commit_index.as_path(), &["add", "--", "f"]);
    // Another command puts that same content into the live index, and the copy is older than it.
    git(world.repo.as_path(), &["add", "f"]);
    let left: Vec<String> = crate::owed_index_update::exclude_what_a_landed_commit_left(
        &world.context.git,
        world.repo.as_path(),
        world.repo.join(".git").as_path(),
        real_index_of(&world).as_path(),
        Some(commit_index.as_path()),
        Some(capture.as_path()),
        &crate::service_land_start::IndexWitness::Absent,
    )
    .expect("nothing is compared, so nothing fails");
    assert_eq!(left, Vec::<String>::new(), "no record names a path, so none is moved");
    let expected: String = git_text(world.repo.as_path(), &["hash-object", "-w", "--", "f"]);
    assert_eq!(index_text(&world, commit_index.as_path(), &["ls-files", "-s", "--", "f"]), format!("100644 {expected} 0\tf"), "the command's own staging stands");
    remove(world.root.as_path());
}

/// A landed executable-bit change is not taken for the person's staging:
/// the mode is part of the entry,
/// so the stale entry is left out and the landed mode is committed.
#[test]
fn a_landed_mode_change_is_left_out() {
    let world: World = world("service-land-owed-mode");
    std::fs::write(world.repo.join("f"), "X\n").expect("write");
    git(world.repo.as_path(), &["add", "f"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::set_permissions(world.repo.join("f"), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("mode");
    let mut landed: CommitRequest = request(&world, "landed", "f", "X\n");
    landed.changes = vec![crate::service_protocol::Change { path: std::path::PathBuf::from("f"), mode: 0o100755, blob: Some(git_text(world.repo.as_path(), &["hash-object", "-w", "--", "f"])) }];
    land_with_failed_index(&world, 1, &[landed]);
    let commit_index: PathBuf = commit_index_of(&world);
    assert_eq!(exclude(&world, Some(commit_index.as_path())), vec![String::from("f")], "the stale mode is left out");
    let object: String = git_text(world.repo.as_path(), &["hash-object", "-w", "--", "f"]);
    assert_eq!(index_text(&world, commit_index.as_path(), &["ls-files", "-s", "--", "f"]), format!("100755 {object} 0\tf"), "the commit records the landed mode");
    remove(world.root.as_path());
}

/// A worktree file that cannot be read is no evidence:
/// the landed content stays,
/// and the path is said.
#[test]
fn an_unreadable_worktree_file_is_not_an_absent_one() {
    let world: World = world("service-land-owed-unreadable");
    let landed: CommitRequest = request(&world, "landed", "p", "p\n");
    land_with_failed_index(&world, 1, &[landed]);
    // The landing added p; the index still lacks it, and the file is there but cannot be read.
    std::fs::set_permissions(world.repo.join("p"), std::os::unix::fs::PermissionsExt::from_mode(0o000)).expect("no read");
    let commit_index: PathBuf = commit_index_of(&world);
    let left: Vec<String> = exclude(&world, Some(commit_index.as_path()));
    std::fs::set_permissions(world.repo.join("p"), std::os::unix::fs::PermissionsExt::from_mode(0o644)).expect("readable again");
    assert_eq!(left, vec![String::from("p")], "the path is left out and said");
    assert_eq!(index_text(&world, commit_index.as_path(), &["ls-files", "-s", "--", "p"]), format!("100644 {} 0\tp", git_text(world.repo.as_path(), &["rev-parse", "HEAD:p"])), "the commit cannot record the landed file as deleted");
    remove(world.root.as_path());
}

/// A linked worktree that was moved is still repaired:
/// the Git questions are asked in the repository preparation has,
/// not in the one an old record remembers.
#[test]
fn a_moved_worktree_is_repaired_where_it_now_stands() {
    let world: World = world("service-land-owed-moved-worktree");
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    // The record remembers the repository as it was; the worktree is asked about where it stands now.
    let moved: PathBuf = world.root.join("moved");
    std::fs::create_dir_all(moved.as_path()).expect("moved");
    std::fs::copy(world.repo.join("one.txt"), moved.join("one.txt")).expect("the file");
    let mut records: Vec<crate::owed_index_update::Owed> = crate::owed_index_update::records_of(world.repo.join(".git").as_path());
    assert_eq!(records.len(), 1);
    records[0].repository = world.root.join("gone-away");
    crate::owed_index_update::write(&records[0]).expect("the record");
    let commit_index: PathBuf = commit_index_of(&world);
    let left: Vec<String> = exclude(&world, Some(commit_index.as_path()));
    assert_eq!(left, vec![String::from("one.txt")], "the record's stale path is left out even though its old repository is gone");
    remove(world.root.as_path());
}

/// A repair that cannot be made stops the command with its own message:
/// the wiring between the repair and the preparation is asked here,
/// end to end through `service_prepare`'s own step.
#[test]
fn a_refused_repair_stops_the_preparation_with_its_reason() {
    let world: World = world("service-land-owed-prepare-stops");
    std::fs::write(world.repo.join("f"), "A\n").expect("write");
    git(world.repo.as_path(), &["add", "f"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let landed: CommitRequest = request(&world, "landed", "f", "B\n");
    land_with_failed_index(&world, 1, &[landed]);
    let commit_index: PathBuf = commit_index_of(&world);
    let mut context: crate::transaction_git::GitContext = world.context.git.clone();
    context.real_git = git_refusing(&world, &["reset", "update-index"]);
    let capture: crate::capture_invocation::InvocationCapture = crate::capture_invocation::capture_invocation_base(
        &world.context.git,
        world.repo.as_path(),
        crate::capture_invocation::capture_invocation_layout(&world.context.git, world.repo.as_path(), false).expect("layout"),
    )
    .expect("capture");
    let indexes: crate::private_index_request::CaptureIndexes = crate::private_index_request::CaptureIndexes {
        captured_index: commit_index.clone(),
        commit_index: commit_index.clone(),
        object_directory: world.root.join("objects"),
    };
    let stopped: crate::commit_transaction::TransactionStop = crate::service_prepare_exclude_or_stop_for_test::exclude_or_stop_for_test(
        &context,
        &capture,
        &indexes,
        crate::transaction_journal::TransactionMode::Index,
        &crate::service_land_start::IndexWitness::Absent,
    )
    .expect_err("a refused repair stops the command");
    let text: String = match stopped {
        crate::commit_transaction::TransactionStop::Failed { message, .. } => message,
        other => panic!("{other:?}"),
    };
    assert!(text.contains("batch 1"), "{text}");
    remove(world.root.as_path());
}

/// A staged symlink is compared by what Git stores for a link,
/// the hash of its target text:
/// a restoration the person staged is theirs,
/// and a target that does not exist does not make it unreadable.
#[test]
fn a_staged_symlink_restoration_is_compared_by_its_target_text() {
    let world: World = world("service-land-owed-symlink");
    let link: PathBuf = world.repo.join("link");
    std::os::unix::fs::symlink("missing-old", link.as_path()).expect("the first link");
    git(world.repo.as_path(), &["add", "link"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::remove_file(link.as_path()).expect("the link goes");
    std::os::unix::fs::symlink("missing-new", link.as_path()).expect("the landed link");
    let mut landed: CommitRequest = request(&world, "landed", "other.txt", "other\n");
    let target: String = git_text(world.repo.as_path(), &["hash-object", "-w", "--stdin"]);
    let _ = target;
    landed.changes.push(crate::service_protocol::Change {
        path: std::path::PathBuf::from("link"),
        mode: 0o120000,
        blob: Some(crate::test_support::git_text(world.repo.as_path(), &["hash-object", "-w", "--stdin"]).clone()),
    });
    let _ = &landed;
    // Land the link's target text as a blob of its own: the change is the link, not its referent.
    let blob: String = {
        
        let mut child = std::process::Command::new(crate::test_support::REAL_GIT)
            .current_dir(world.repo.as_path())
            .args(["hash-object", "-w", "--stdin"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("git starts");
        child.stdin.as_mut().expect("stdin").write_all(b"missing-new").expect("the target text");
        let output: std::process::Output = child.wait_with_output().expect("git ends");
        String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string()
    };
    landed.changes = vec![crate::service_protocol::Change { path: std::path::PathBuf::from("link"), mode: 0o120000, blob: Some(blob) }];
    land_with_failed_index(&world, 1, &[landed]);
    // The person deliberately restores the old target text and stages it.
    std::fs::remove_file(link.as_path()).expect("the link goes");
    std::os::unix::fs::symlink("missing-old", link.as_path()).expect("their link");
    git(world.repo.as_path(), &["add", "link"]);
    let commit_index: PathBuf = commit_index_of(&world);
    let left: Vec<String> = exclude(&world, Some(commit_index.as_path()));
    assert_eq!(left, Vec::<String>::new(), "their staged restoration is theirs: {left:?}");
    let old_blob: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD^:link"]);
    assert_eq!(index_text(&world, commit_index.as_path(), &["ls-files", "-s", "--", "link"]), format!("120000 {old_blob} 0\tlink"), "the commit records their restoration");
    remove(world.root.as_path());
}

/// An absent worktree file with no entry in the copy is no proof of the person's staging:
/// it is what a failed index update leaves,
/// so the landed entry stays and nothing is deleted.
#[test]
fn an_absent_worktree_file_with_no_entry_keeps_the_landed_entry() {
    let world: World = world("service-land-owed-absent");
    let landed: CommitRequest = request(&world, "landed", "f", "f\n");
    land_with_failed_index(&world, 1, &[landed]);
    // Nobody staged the deletion: the file is simply gone from the worktree, and the index lacks it as the failed update left it.
    std::fs::remove_file(world.repo.join("f")).expect("the file goes");
    std::fs::write(world.repo.join("other.txt"), "other\n").expect("write");
    git(world.repo.as_path(), &["add", "other.txt"]);
    let commit_index: PathBuf = commit_index_of(&world);
    let left: Vec<String> = exclude(&world, Some(commit_index.as_path()));
    assert_eq!(left, vec![String::from("f")], "the landed entry is kept and the path is said");
    let landed_blob: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD:f"]);
    assert_eq!(index_text(&world, commit_index.as_path(), &["ls-files", "-s", "--", "f"]), format!("100644 {landed_blob} 0\tf"), "nothing is deleted");
    remove(world.root.as_path());
}

/// A derived record is read in the repository the worktree stands in now,
/// not in the one an old record remembers,
/// so a moved worktree commits.
#[test]
fn a_derived_record_of_a_moved_worktree_does_not_fail_the_commit() {
    let world: World = world("service-land-owed-derived-moved");
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    let mut records: Vec<crate::owed_index_update::Owed> = crate::owed_index_update::records_of(world.repo.join(".git").as_path());
    assert_eq!(records.len(), 1);
    records[0].derived = true;
    records[0].paths = Vec::new();
    records[0].repository = world.root.join("gone-away");
    crate::owed_index_update::write(&records[0]).expect("the record");
    let commit_index: PathBuf = commit_index_of(&world);
    let left: Vec<String> = exclude(&world, Some(commit_index.as_path()));
    assert_eq!(left, vec![String::from("one.txt")], "the derived path is left out, in the repository that is there");
    remove(world.root.as_path());
}

/// A record round-trips a path whose bytes are not text:
/// the repository,
/// the Git directory and the index are read back as the bytes they were written from,
/// and so is a path of the batch.
#[test]
fn a_record_keeps_the_bytes_of_every_path_of_a_non_utf8_worktree() {
    let root: PathBuf = crate::test_support::fixture("service-land-owed-non-utf8-record");
    // A name whose bytes are not UTF-8: `0xFF` is no start byte of any text.
    let odd: std::ffi::OsString = <std::ffi::OsString as std::os::unix::ffi::OsStringExt>::from_vec(b"work\xfftree".to_vec());
    let worktree: PathBuf = root.join(odd.as_os_str());
    let git_dir: PathBuf = worktree.join(".git");
    let index: PathBuf = git_dir.join("index");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(&index).expect("the index's directory");
    std::fs::create_dir_all(journal.as_path()).expect("journal");
    let owed: crate::owed_index_update::Owed = crate::owed_index_update::Owed {
        seq: 3,
        repository: worktree.clone(),
        git_dir: git_dir.clone(),
        index: index.clone(),
        reference: String::from("refs/heads/main"),
        old: String::from("a".repeat(40)),
        new: String::from("b".repeat(40)),
        paths: vec![b"caf\xe9.txt".to_vec()],
        derived: false,
    };
    crate::owed_index_update::write(&owed).expect("the record");
    let records: Vec<crate::owed_index_update::Owed> = crate::owed_index_update::records_of(git_dir.as_path());
    assert_eq!(records.len(), 1, "the record is read back");
    assert_eq!(records[0].repository.as_os_str().as_encoded_bytes(), worktree.as_os_str().as_encoded_bytes(), "the repository keeps its bytes");
    assert_eq!(records[0].git_dir.as_os_str().as_encoded_bytes(), git_dir.as_os_str().as_encoded_bytes(), "the Git directory keeps its bytes");
    assert_eq!(records[0].index.as_os_str().as_encoded_bytes(), index.as_os_str().as_encoded_bytes(), "the index keeps its bytes");
    assert_eq!(records[0].paths, vec![b"caf\xe9.txt".to_vec()], "the batch's path keeps its bytes");
    remove(root.as_path());
}

/// An unrelated owed record does not let a concurrent staging change what `--all` selected:
/// the repair changes only a path a record names.
#[test]
fn an_unrelated_record_never_changes_what_all_selected() {
    let world: World = world("service-land-owed-unrelated-all");
    std::fs::write(world.repo.join("f"), "A\n").expect("write");
    std::fs::write(world.repo.join("g"), "A\n").expect("write");
    git(world.repo.as_path(), &["add", "f", "g"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=base"]);
    // A record for a path of no interest, so the repair has work to do at all.
    let commit: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let owed: crate::owed_index_update::Owed = crate::owed_index_update::Owed {
        seq: 7,
        repository: world.repo.clone(),
        git_dir: world.repo.join(".git"),
        index: real_index_of(&world),
        reference: String::from("refs/heads/main"),
        old: commit.clone(),
        new: commit,
        paths: vec![b"h".to_vec()],
        derived: false,
    };
    crate::owed_index_update::write(&owed).expect("an unrelated record");
    // `git commit --all`: the copy is taken first, then both edits are staged into the command's index.
    let capture: PathBuf = world.root.join("capture.index");
    std::fs::copy(real_index_of(&world), capture.as_path()).expect("the copy a commit makes");
    let commit_index: PathBuf = commit_index_of(&world);
    std::fs::write(world.repo.join("f"), "C\n").expect("write");
    std::fs::write(world.repo.join("g"), "D\n").expect("write");
    index_run(&world, commit_index.as_path(), &["add", "--", "f", "g"]);
    // Another command stages the same content of f into the live index.
    git(world.repo.as_path(), &["add", "f"]);
    let left: Vec<String> = crate::owed_index_update::exclude_what_a_landed_commit_left(
        &world.context.git,
        world.repo.as_path(),
        world.repo.join(".git").as_path(),
        real_index_of(&world).as_path(),
        Some(commit_index.as_path()),
        Some(capture.as_path()),
        &crate::service_land_start::IndexWitness::Absent,
    )
    .expect("the repair has nothing to say about a path no record names");
    assert!(!left.contains(&String::from("f")), "f is not reported as moved: {left:?}");
    let blob: String = git_text(world.repo.as_path(), &["hash-object", "-w", "--", "f"]);
    assert_eq!(index_text(&world, commit_index.as_path(), &["ls-files", "-s", "--", "f"]), format!("100644 {blob} 0\tf"), "the commit keeps what --all selected");
    remove(world.root.as_path());
}

/// An obligation whose landing died before its intent on an unborn branch does not stop the initial commit:
/// there is no head to hold an entry,
/// so nothing is owed there.
#[test]
fn an_unborn_branch_with_an_orphaned_record_still_commits() {
    let root: PathBuf = crate::test_support::fixture("service-land-owed-unborn");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir_all(repo.as_path()).expect("repo");
    git(repo.as_path(), &["init", "--quiet", "-b", "main"]);
    std::fs::write(repo.join("first.txt"), "first\n").expect("write");
    git(repo.as_path(), &["add", "first.txt"]);
    let git_dir: PathBuf = repo.join(".git");
    let index: PathBuf = git_dir.join("index");
    let owed: crate::owed_index_update::Owed = crate::owed_index_update::Owed {
        seq: 1,
        repository: repo.clone(),
        git_dir: git_dir.clone(),
        index: index.clone(),
        reference: String::from("refs/heads/main"),
        old: "0".repeat(40),
        new: "0".repeat(40),
        paths: vec![b"first.txt".to_vec()],
        derived: false,
    };
    crate::owed_index_update::write(&owed).expect("the record a kill left");
    let left: Vec<String> = crate::owed_index_update::exclude_what_a_landed_commit_left(
        &crate::test_support::git_context(),
        repo.as_path(),
        git_dir.as_path(),
        index.as_path(),
        None,
        None,
        &crate::service_land_start::IndexWitness::Absent,
    )
    .expect("an unborn branch has no head, and the commit is not stopped for it");
    assert!(left.is_empty(), "nothing is left out on an unborn branch: {left:?}");
    remove(root.as_path());
}

/// A record that exists and cannot be read is still an obligation,
/// and it no longer stops the commit:
/// the landed deletion it could have named is left out,
/// and the notice says why.
#[test]
fn an_unreadable_record_leaves_the_landed_deletion_out_and_says_why() {
    let world: World = world("service-land-owed-unreadable-record");
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    let record: PathBuf = owed_of(&world, 1);
    std::fs::set_permissions(record.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o000)).expect("no read");
    let (found, unreadable): (Vec<crate::owed_index_update::Owed>, usize) = crate::owed_index_update::records_and_unreadable(world.repo.join(".git").as_path());
    assert!(found.is_empty(), "nothing can be read of it");
    assert_eq!(unreadable, 1, "and it counts as owed all the same");
    let commit_index: PathBuf = commit_index_of(&world);
    let mut note: Option<String> = None;
    let left: Vec<String> = crate::owed_index_repair::exclude_with_note(
        &world.context.git,
        world.repo.as_path(),
        world.repo.join(".git").as_path(),
        real_index_of(&world).as_path(),
        Some(commit_index.as_path()),
        Some(commit_index.as_path()),
        &crate::service_land_start::IndexWitness::Absent,
        &mut note,
    )
    .expect("the commit goes on");
    std::fs::set_permissions(record.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o644)).expect("readable again");
    assert_eq!(left, vec![String::from("one.txt")], "the deletion it could have named is left out");
    let said: String = note.expect("the notice says why");
    assert!(said.contains("cannot be read") && said.contains("one.txt"), "{said}");
    assert!(record.exists(), "the record is kept for the person");
    remove(world.root.as_path());
}

/// A Git whose head probe fails with something other than "no such object",
/// as a lock or an unreadable repository does.
fn git_with_a_failing_head_probe(world: &World) -> PathBuf {
    let program: PathBuf = world.root.join("failing-head-git");
    let script: String = format!(
        "#!/bin/sh\ncase \" $* \" in\n  *\" rev-parse \"*) echo 'fatal: cannot read the head' >&2; exit 128;;\nesac\nexec '{}' \"$@\"\n",
        crate::test_support::REAL_GIT
    );
    std::fs::write(program.as_path(), script).expect("script");
    std::fs::set_permissions(program.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("executable");
    return program;
}

/// A head probe that failed is a failure,
/// not an unborn branch:
/// the repair says so and touches nothing,
/// because taking it for an unborn head would write a removal.
#[test]
fn a_failed_head_probe_is_not_an_unborn_branch() {
    let world: World = world("service-land-owed-head-probe");
    std::fs::write(world.repo.join("f"), "A\n").expect("write");
    git(world.repo.as_path(), &["add", "f"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let landed: CommitRequest = request(&world, "landed", "f", "B\n");
    land_with_failed_index(&world, 1, &[landed]);
    let commit_index: PathBuf = commit_index_of(&world);
    let before: String = index_text(&world, commit_index.as_path(), &["ls-files", "-s"]);
    let mut context: crate::transaction_git::GitContext = world.context.git.clone();
    context.real_git = git_with_a_failing_head_probe(&world);
    let answered: Result<Vec<String>, String> = crate::owed_index_update::exclude_what_a_landed_commit_left(
        &context,
        world.repo.as_path(),
        world.repo.join(".git").as_path(),
        real_index_of(&world).as_path(),
        Some(commit_index.as_path()),
        Some(commit_index.as_path()),
        &crate::service_land_start::IndexWitness::Absent,
    );
    assert!(answered.is_err(), "a probe that failed is a failure: {answered:?}");
    assert_eq!(index_text(&world, commit_index.as_path(), &["ls-files", "-s"]), before, "and nothing was written");
    remove(world.root.as_path());
}

/// A record of another index names no path of this commit's index:
/// the moved-index comparison leaves that path alone.
#[test]
fn a_record_of_another_index_names_no_path_of_this_one() {
    let world: World = world("service-land-owed-other-index-names");
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    let mut records: Vec<crate::owed_index_update::Owed> = crate::owed_index_update::records_of(world.repo.join(".git").as_path());
    assert_eq!(records.len(), 1);
    records[0].index = world.root.join("other.index");
    crate::owed_index_update::write(&records[0]).expect("the record names another index");
    // The live index moves without this commit: the moved comparison runs, and the record is the only one that names one.txt.
    let capture: PathBuf = world.root.join("capture.index");
    std::fs::copy(real_index_of(&world), capture.as_path()).expect("the copy a commit makes");
    let commit_index: PathBuf = commit_index_of(&world);
    // The live index catches up with the landing, so the moved comparison lists one.txt although no record of this index names it.
    git(world.repo.as_path(), &["add", "one.txt"]);
    let left: Vec<String> = crate::owed_index_update::exclude_what_a_landed_commit_left(
        &world.context.git,
        world.repo.as_path(),
        world.repo.join(".git").as_path(),
        real_index_of(&world).as_path(),
        Some(commit_index.as_path()),
        Some(capture.as_path()),
        &crate::service_land_start::IndexWitness::Absent,
    )
    .expect("the repair has nothing to say about another index's paths");
    assert!(!left.contains(&String::from("one.txt")), "the other index's path is not left out: {left:?}");
    remove(world.root.as_path());
}

/// An obligation that cannot be read keeps a landed file from being recorded as deleted:
/// the commit goes on,
/// the deletion is left out,
/// and the notice says why.
#[test]
fn an_unreadable_obligation_keeps_a_landed_deletion_out() {
    let world: World = world("service-land-owed-unreadable-keeps");
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    let record: PathBuf = owed_of(&world, 1);
    std::fs::set_permissions(record.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o000)).expect("no read");
    let commit_index: PathBuf = commit_index_of(&world);
    let mut note: Option<String> = None;
    let left: Vec<String> = crate::owed_index_repair::exclude_with_note(
        &world.context.git,
        world.repo.as_path(),
        world.repo.join(".git").as_path(),
        real_index_of(&world).as_path(),
        Some(commit_index.as_path()),
        Some(commit_index.as_path()),
        &crate::service_land_start::IndexWitness::Absent,
        &mut note,
    )
    .expect("the commit goes on");
    std::fs::set_permissions(record.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o644)).expect("readable again");
    assert_eq!(left, vec![String::from("one.txt")], "the deletion it would have recorded is left out");
    let said: String = note.expect("the notice says why");
    assert!(said.contains("cannot be read") && said.contains("one.txt"), "{said}");
    let landed_blob: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD:one.txt"]);
    assert_eq!(index_text(&world, commit_index.as_path(), &["ls-files", "-s", "--", "one.txt"]), format!("100644 {landed_blob} 0\tone.txt"), "the landed file stays in the commit");
    remove(world.root.as_path());
}

/// What:
///  The notice a preparation prints reaches standard error while another thread holds its shared lock.
/// Why:
///  This write is on the commit's own path,
/// after the warnings,
/// so a lock a warning worker kept would hold the commit
///  here forever;
/// the notice is given to the descriptor with a ceiling instead,
/// and the commit goes on whatever happens.
#[cfg(unix)]
#[test]
fn a_notice_reaches_standard_error_while_its_shared_lock_is_held() {
    // The lock is reentrant for the thread that holds it, so the delivery runs on a thread of its own, as the production
    //  write does.
    let (done, finished) = std::sync::mpsc::channel::<bool>();
    let held = std::io::stderr().lock();
    let worker = std::thread::spawn(move || {
        let delivered: bool = crate::preparation_notice::deliver_notice_for_test("a notice holds no lock another writer needs\n");
        let _sent = done.send(delivered);
    });
    let delivered: bool = finished.recv_timeout(std::time::Duration::from_secs(5)).unwrap_or(false);
    drop(held);
    let _joined = worker.join();
    assert!(delivered, "the notice needed the lock a synchronous writer holds");
}

/// The longest a control waits for the landing of another thread to say what it waits for.
const A_NOTICE_WITHIN: std::time::Duration = std::time::Duration::from_secs(60);

/// The ceiling a control gives a landing whose lock a program holds and keeps.
const A_SHORT_CEILING: std::time::Duration = std::time::Duration::from_millis(500);

/// What:
///  An announcer that hands every notice of a batch to the control that reads them.
/// Why:
///   The wait for another program's `index.lock` is what these controls watch,
/// and the notices are its only statement that it began.
struct Told(std::sync::mpsc::Sender<crate::service_index_lock::Announce>);

impl crate::service_index_lock::Announcer for Told {
    fn announce(&self, note: crate::service_index_lock::Announce) {
        let _sent = self.0.send(note);
    }
}

/// What:
///  A program of this test that takes the worktree's index lock and keeps it open,
/// as another program's `git` does while it writes the index.
/// Why:
///   A lock nothing holds is removed by the landing's own rule,
/// so only a held one makes the wait last.
/// The holder lets go when `release` appears,
/// which keeps a control from waiting out a sleep.
fn hold_the_index(world: &World, release: &Path) -> (PathBuf, std::process::Child) {
    let lock: PathBuf = world.repo.join(".git/index.lock");
    let script: PathBuf = world.root.join("hold-the-index.sh");
    crate::test_support::executable(
        script.as_path(),
        format!(
            "#!/bin/sh\nexec 9>\"$1\"\nwhile [ ! -e \"$2\" ]; do /usr/bin/sleep 0.02; done\nrm -f \"$1\"\n",
        )
        .as_bytes(),
    );
    let holder: std::process::Child = crate::test_support::spawn_owned(std::process::Command::new(script.as_path()).arg(lock.as_path()).arg(release))
        .expect("the holder of the index runs");
    let began: std::time::Instant = std::time::Instant::now();
    while !lock.exists() {
        assert!(began.elapsed() < A_NOTICE_WITHIN, "the holder did not take {}", lock.display());
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    return (lock, holder);
}

/// The number of push records the worktree holds where nothing runs them.
fn held_push_records(world: &World) -> usize {
    let directory: PathBuf = crate::work_item::state_directory(crate::service_work::work_directory(world.repo.join(".git").as_path()).as_path(), crate::work_item::HELD);
    return std::fs::read_dir(directory.as_path()).map(|entries| { return entries.flatten().count(); }).unwrap_or(0);
}

/// Owner's rule "never discard user data",
/// and the SPEC's "a batch that never landed owes nothing":
/// a batch withdrawn at the index lock's ceiling never moved the branch,
/// so it owes no index update and no push,
/// and every command of the worktree must not read a record of it.
/// The records are taken after the lock,
/// before the branch moves,
/// so only the paths that can land take them.
#[test]
fn a_batch_withdrawn_at_the_index_lock_ceiling_owes_nothing() {
    let mut world: World = world("service-land-owed-withdrawn-at-the-lock");
    world.context.lock_ceiling = A_SHORT_CEILING;
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let waited: CommitRequest = request(&world, "waited", "one.txt", "one\n");
    let (_lock, mut holder): (PathBuf, std::process::Child) = hold_the_index(&world, world.root.join("release-the-index").as_path());
    let landing = land_batch(&world.context, 1, std::slice::from_ref(&waited));
    assert_eq!(landing.err(), Some(FailureCode::IndexInUse), "the commit is withdrawn at its ceiling");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), base, "the branch did not move");
    assert!(!owed_of(&world, 1).exists(), "a batch that never landed owes no index update: {}", owed_of(&world, 1).display());
    assert_eq!(held_push_records(&world), 0, "a batch that never landed held no push");
    // The holder is still there: what it holds is left where it is, as another program's lock is.
    let _ended = holder.kill();
    let _waited = holder.wait();
    remove(world.root.as_path());
}

/// A batch that waits for another program's `index.lock` owes nothing yet:
/// its branch has not moved,
/// so a commit prepared meanwhile must not read a record of it,
/// and must not say that a commit that landed did not finish updating the index.
#[test]
fn a_batch_that_waits_for_the_index_lock_owes_nothing_yet() {
    let world: World = world("service-land-owed-nothing-while-the-lock-is-waited-for");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let waited: CommitRequest = request(&world, "waited", "one.txt", "one\n");
    let release: PathBuf = world.root.join("release-the-index");
    let (_lock, mut holder): (PathBuf, std::process::Child) = hold_the_index(&world, release.as_path());
    let (told, heard): (std::sync::mpsc::Sender<crate::service_index_lock::Announce>, std::sync::mpsc::Receiver<crate::service_index_lock::Announce>) = std::sync::mpsc::channel();
    let context: LandContext = world.context.clone();
    let batch: Vec<CommitRequest> = vec![waited];
    let landing = std::thread::spawn(move || { return land_batch_announcing(&context, 1, &batch, &Told(told)); });
    let notice: crate::service_index_lock::Announce = heard.recv_timeout(A_NOTICE_WITHIN).expect("the landing says what it waits for");
    assert!(matches!(notice, crate::service_index_lock::Announce::Waiting(_)), "{notice:?}");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), base, "the batch has not landed while it waits");
    let owed: Vec<crate::owed_index_update::Owed> = crate::owed_index_update::records_of(world.repo.join(".git").as_path());
    assert!(owed.is_empty(), "a batch that has not landed owes nothing, so nothing of it is read: {owed:?}");
    assert_eq!(held_push_records(&world), 0, "a batch that has not landed holds no push");
    std::fs::write(release.as_path(), b"let go\n").expect("the holder is told to let go");
    let outcomes = landing.join().expect("the landing of the other thread").expect("the batch lands once the lock is free");
    assert!(matches!(outcomes.as_slice(), [Outcome::Landed { .. }]), "{outcomes:?}");
    assert!(!owed_of(&world, 1).exists(), "the landed batch installed its index, so it owes nothing any more");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "", "the worktree's index agrees with the branch");
    let _ended = holder.wait();
    remove(world.root.as_path());
}
