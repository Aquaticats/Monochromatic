//! What:
//!  The finish of a merge a command saved on its side ref:
//!  a branch that moved and kept the record,
//!  a staged edit and a directory that stop or do not stop the finish,
//!  a file whose name fills the file system's limit,
//!  a take failure that does or does not stop the command,
//!  the file a process holds open,
//!  a side ref another command repointed,
//!  and a merge saved for another branch's checkout.
//! Why:
//!  A saved merge is finished by the next command,
//!  so each refusal has to say whether that command stops and what it keeps.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(settled.record).toBe(true); expect(exists(aside)).toBe(true); expect(exists(merge)).toBe(true);
//! ```

use super::super::*;
use super::fixtures::{
    A_CONTROLS_MERGE,
    World,
    aside_names,
    dead_owner,
    deferred_record_of,
    directory_to_file_world,
    extra_change,
    place,
    record_of,
    stranded_name,
    world,
};
use super::readings::{
    judged_changes,
    snapshot,
};
use crate::merge_isolation_changes::file_identity;
use crate::merge_isolation_recovery_records::recover_interrupted_merges;
/// The install of a merge's index entries and files,
/// which a placement of the merge's own bytes makes.
use crate::merge_isolation_recovery_steps::install_entries;
use crate::test_support::{
    git,
    git_text,
    remove,
    spawn_owned,
};
use std::path::PathBuf;

/// What:
///  A saved merge whose branch advanced is kept and named,
///  and the command that finds it goes on.
/// Why:
///  The holder of the index commits an unrelated file,
/// so the branch is no longer the commit the merge was computed on;
///  the merge cannot land as it was saved,
/// and the command that reported it saved had already exited 0
///  (`SPEC.md:2187`).
/// Settling it as an ordinary moved-branch record dropped the record and the side ref,
/// so the
///  acknowledged merge vanished and its commit became collectable
/// (a cross-vendor review's finding on the second round).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = deferredRecord(world, 'refs/cli-git/kept/7'); commitUnrelated(); const outcomes = await recover(...); expect(sink).toContain('refs/cli-git/kept/7'); expect(await revParse(side)).toEqual(mergeOid); expect(exists(record)).toBe(false);
/// ```
#[test]
fn a_saved_merge_whose_branch_moved_is_kept_and_named() {
    let world: World = world("merge-recovery-saved-branch-moved");
    let record: PathBuf = deferred_record_of(
        &world,
        "17171717-0000-4000-8000-000000000017",
        "refs/cli-git/kept/7",
    );
    // The holder of the index commits an unrelated file: the branch is no longer the merge's first parent.
    std::fs::write(world.repo.join("z.txt"), b"committed meanwhile\n").expect("file");
    git(world.repo.as_path(), &["add", "--", "z.txt"]);
    git(
        world.repo.as_path(),
        &["commit", "--quiet", "--message=meanwhile", "--", "z.txt"],
    );
    let moved: String = git_text(world.repo.as_path(), &["rev-parse", "main"]);
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("the command goes on");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(
        text.contains("refs/cli-git/kept/7"),
        "where the merge is kept is said: {text}"
    );
    assert!(
        text.contains(world.new_oid.as_str()),
        "the merge commit is named: {text}"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "refs/cli-git/kept/7"]),
        world.new_oid,
        "the saved merge is kept"
    );
    assert!(
        !record.exists(),
        "the record is gone, so nothing stops the next command"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        moved,
        "the branch is the holder's"
    );
    assert_eq!(outcomes[0].action, MergeRecoveryAction::SavedBranchMoved);
    remove(world.parent.as_path());
}

/// What:
///  A staged edit of a path the saved merge replaces stops the finish,
///  and the staged content is still in the index.
/// Why:
///  The judgement reads the worktree,
/// where the branch's content stands,
/// so a version staged only in the index was
///  overwritten with the merge's without a warning or a copy
/// (a cross-vendor review's finding on the second round).
/// The staged version has no other name,
/// so the finish must stop and name the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write('f.txt', C); git add f.txt; write('f.txt', A); await expect(recover(...)).rejects.toMatch(/f.txt/); expect(indexOf('f.txt')).toEqual(C);
/// ```
#[test]
fn a_staged_edit_of_a_merge_path_stops_the_finish_and_is_kept() {
    let world: World = world("merge-recovery-staged-only");
    // Git installed the merge and died before the branch moved; another program then staged its own version of `f.txt`
    //  over the merge's entry, and the worktree was left as the merge wrote it.
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
        dead_owner("18181818-0000-4000-8000-000000000018"),
        Some("refs/heads/main"),
    );
    std::fs::write(world.repo.join("f.txt"), b"f.txt staged by someone else\n")
        .expect("the staged version");
    git(world.repo.as_path(), &["add", "--", "f.txt"]);
    // The worktree goes back to what the merge wrote there, so only the index holds the person's version.
    std::fs::write(world.repo.join("f.txt"), b"f.txt feature\n")
        .expect("the worktree as the merge left it");
    let staged_entry: String = git_text(
        world.repo.as_path(),
        &["ls-files", "--stage", "--", "f.txt"],
    );
    assert!(
        staged_entry.contains("f.txt"),
        "the index holds the staged entry under test: {staged_entry}"
    );
    let before = snapshot(&world);
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect_err("a staged edit of a merge path stops the finish");
    assert!(
        refused.0.contains("f.txt"),
        "the path is named: {}",
        refused.0
    );
    assert!(
        refused.0.contains("index"),
        "it says the index holds it: {}",
        refused.0
    );
    assert_eq!(
        git_text(
            world.repo.as_path(),
            &["ls-files", "--stage", "--", "f.txt"]
        ),
        staged_entry,
        "the staged version is still in the index"
    );
    assert_eq!(snapshot(&world), before, "nothing of the merge was written");
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        world.old_oid,
        "the branch did not move"
    );
    remove(world.parent.as_path());
}

/// What:
///  A directory the merge replaces with a file does not stop the finish,
///  because the directory holds the branch's own files that the merge deletes.
/// Why:
///  `worktree_state` reads a directory as neither the old nor the new content,
/// so the saved merge refused at every
///  later command,
/// and restoring the path from the branch recreated the same directory
/// (a cross-vendor review's finding on the
///  second round).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = directoryToFile(); await recover(...); expect(read('dir')).toEqual(fileContent); expect(exists('dir/child.txt')).toBe(false);
/// ```
#[test]
fn a_saved_merge_that_replaces_a_directory_with_a_file_finishes() {
    let world: World = directory_to_file_world("merge-recovery-directory-to-file");
    let _record: PathBuf = deferred_record_of(
        &world,
        "19191919-0000-4000-8000-000000000019",
        "refs/cli-git/kept/9",
    );
    assert!(
        world.repo.join("dir/child.txt").is_file() && !world.repo.join("dir").is_file(),
        "the old tree holds a directory"
    );
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("the merge is finished, not refused");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    assert_eq!(
        std::fs::read(world.repo.join("dir")).expect("the file the merge adds"),
        b"the merge's file\n",
        "the path is the merge's file"
    );
    assert!(
        !world.repo.join("dir/child.txt").exists(),
        "the path the merge deletes is gone"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        world.new_oid
    );
    remove(world.parent.as_path());
}

/// What:
///  A file whose name fills the file system's limit is taken aside and finished,
///  because the private name does not carry it whole.
/// Why:
///  The private name embedded the whole file name and then added a prefix,
/// a PID,
/// a count and a clock
///  reading,
/// so a valid file name made a name the file system refuses (`ENAMETOOLONG`),
/// and the retained record then
///  stopped every command (a cross-vendor review's finding on the second round).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const long = 'l'.repeat(240) + '.txt'; await installEntries(...); expect(read(long)).toEqual(mergeContent);
/// ```
#[test]
fn a_file_whose_name_is_at_the_file_systems_limit_is_taken_and_finished() {
    let world: World = world("merge-recovery-long-name");
    let (mut changes, mut judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let long: String = format!("{}.txt", "l".repeat(240));
    std::fs::write(world.repo.join(long.as_str()), b"old long\n").expect("a valid long file name");
    let identity: Option<crate::merge_isolation_changes::FileIdentity> =
        file_identity(world.repo.join(long.as_str()).as_path());
    let old_oid: String = git_text(
        world.repo.as_path(),
        &["hash-object", "-w", "--", long.as_str()],
    )
    .trim()
    .to_owned();
    let next_oid: String = git_text(world.repo.as_path(), &["hash-object", "-w", "--stdin"])
        .trim()
        .to_owned();
    changes.push(extra_change(
        long.as_str(),
        "100644",
        "100644",
        old_oid.as_str(),
        next_oid.as_str(),
    ));
    judgements.push(Judged {
        standing: Standing::Old,
        identity,
        content: Some(old_oid.clone()),
    });
    let mut said: Vec<u8> = Vec::new();
    let installed = install_entries(
        &place(&world),
        world.repo.as_path(),
        changes.as_slice(),
        judgements.as_slice(),
        false,
        A_CONTROLS_MERGE,
        &mut said,
    );
    assert!(
        installed.is_ok(),
        "a valid file name is taken and finished: {installed:?} {}",
        String::from_utf8_lossy(&said)
    );
    assert_eq!(
        aside_names(world.repo.as_path()),
        Vec::<String>::new(),
        "nothing is left at a private name"
    );
    remove(world.parent.as_path());
}

/// What:
///  A saved merge whose finish is refused says so and lets the command go on.
/// Why:
///  A record that cannot be finished must not stop unrelated commands
/// (the shape behind two cross-vendor review
///  findings):
/// nothing of a saved merge is in the checkout,
/// so a command that only reads it has nothing to
///  be stopped for,
/// and the line says where the merge is and what blocked it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// deferredRecord(); stage('f.txt', C); const outcomes = await recover(...); expect(outcomes[0].action).toBe('deferred-refused'); expect(indexOf('f.txt')).toEqual(C);
/// ```
#[test]
fn a_saved_merge_that_cannot_be_finished_does_not_stop_the_command() {
    let world: World = world("merge-recovery-saved-refused");
    let record: PathBuf = deferred_record_of(
        &world,
        "20202020-0000-4000-8000-000000000020",
        "refs/cli-git/kept/20",
    );
    std::fs::write(world.repo.join("f.txt"), b"f.txt staged by someone else\n")
        .expect("the staged version");
    git(world.repo.as_path(), &["add", "--", "f.txt"]);
    std::fs::write(world.repo.join("f.txt"), b"f.txt base\n").expect("the worktree back");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("the command goes on");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::DeferredRefused);
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(
        text.contains("refs/cli-git/kept/20") && text.contains("f.txt"),
        "where it is and what blocked it: {text}"
    );
    assert!(
        record.exists(),
        "the merge is still to be finished once the path is free"
    );
    assert!(
        git_text(
            world.repo.as_path(),
            &["ls-files", "--stage", "--", "f.txt"]
        )
        .contains("f.txt"),
        "the staged version is kept"
    );
    remove(world.parent.as_path());
}

/// What:
///  A begun merge whose finish is refused still stops the command.
/// Why:
///  The checkout may hold the merge's files,
/// so a command must not read it as another session's staged work;
/// only a
///  saved merge,
/// with nothing of it in the checkout,
/// goes on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// begunRecord(); write('f.txt', mine); await expect(recover(...)).rejects.toMatch(/f.txt/);
/// ```
#[test]
fn a_begun_merge_whose_finish_is_refused_still_stops_the_command() {
    let world: World = world("merge-recovery-begun-refused");
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
        dead_owner("21212121-0000-4000-8000-000000000021"),
        Some("refs/heads/main"),
    );
    std::fs::write(world.repo.join("f.txt"), b"my own edit\n").expect("the person's edit");
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect_err("a begun merge stops the command");
    assert!(refused.0.contains("f.txt"), "{}", refused.0);
    remove(world.parent.as_path());
}

/// What:
///  A file a process has open is never removed with its private name,
///  and a file nothing has open is.
/// Why:
///  A writer that opened the file before the take writes into the inode the take renamed,
/// so removing the
///  private name deletes the only name that edit has;
/// a check just before the unlink cannot close that,
/// but a
///  scan for a process that has the file open can prove the file has no writer left,
/// which is the proof the commit
///  service's lock removal acts on (a cross-vendor review's finding on the second round).
/// The limit is the same as that removal's:
/// a process in another
///  user namespace,
/// or one that cannot be read,
/// is not seen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = holderOf(aside); finishAsides([take], sink); expect(exists(aside)).toBe(true); expect(sink).toContain(aside); release(held); finishAsides([take], sink); expect(exists(aside)).toBe(false);
/// ```
#[test]
fn a_file_a_process_holds_open_is_kept_and_a_file_nothing_holds_is_removed() {
    let world: World = world("merge-recovery-held-open-aside");
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = stranded_name(location.as_path(), A_CONTROLS_MERGE);
    let expected: String = git_text(world.repo.as_path(), &["hash-object", "--", "f.txt"])
        .trim()
        .to_owned();
    std::fs::rename(location.as_path(), aside.as_path()).expect("a take");
    let taken = file_identity(aside.as_path()).expect("the identity the take checked");
    let entries: Vec<TakenOnDisk> = vec![TakenOnDisk {
        location: location.clone(),
        path: b"f.txt".to_vec(),
        aside: aside.clone(),
        taken,
        expected: expected.clone(),
    }];
    // A writer that had the file open before the take holds the very inode the take renamed; the scan leaves this process out,
    //  so the holder is a child of it, as an editor would be.
    let kept_open: std::fs::File =
        std::fs::File::open(aside.as_path()).expect("the writer's descriptor to hand on");
    let mut holder: std::process::Command = std::process::Command::new("sleep");
    holder
        .arg("120")
        .stdin(std::process::Stdio::from(kept_open));
    let mut writer: std::process::Child =
        spawn_owned(&mut holder).expect("a writer holding the file open");
    drop(holder);
    let mut said: Vec<u8> = Vec::new();
    finish_asides(&place(&world), world.repo.as_path(), &entries, &mut said)
        .expect("the asides are finished");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(
        aside.exists(),
        "a file a process holds open is never removed"
    );
    assert!(
        text.contains(aside.to_string_lossy().as_ref()),
        "the private name is said: {text}"
    );
    // With the writer gone, nothing proves a write can still arrive, and the private name goes.
    writer.kill().expect("the writer ends");
    writer.wait().expect("reaped");
    let mut afterwards: Vec<u8> = Vec::new();
    finish_asides(
        &place(&world),
        world.repo.as_path(),
        &entries,
        &mut afterwards,
    )
    .expect("the asides are finished");
    assert!(
        !aside.exists(),
        "a file nothing holds open is removed once its content was checked"
    );
    remove(world.parent.as_path());
}

/// What:
///  A staged edit of a path the merge changes stops the settling of a branch that moved,
///  and the staged version stays in the index.
/// Why:
///  The straight completion path refuses a foreign staged entry,
/// and the moved-branch settling had no such check,
/// so
///  the person's staged version of a path the merge changes was overwritten with the moved head's and the record
///  removed
/// (a cross-vendor review of the third round).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readTree(repo, old, merged); commitUnrelated(); stage('f.txt', C); await expect(recover(...)).rejects.toMatch(/f.txt/);
/// ```
#[test]
fn a_staged_edit_of_a_merge_path_stops_the_moved_branch_settling() {
    let world: World = world("merge-recovery-moved-staged");
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
    // A commit on top of the old head that leaves `f.txt` alone, checked out: the branch moved.
    std::fs::write(world.repo.join("z.txt"), b"committed meanwhile\n").expect("file");
    git(world.repo.as_path(), &["add", "--", "z.txt"]);
    git(
        world.repo.as_path(),
        &["commit", "--quiet", "--message=meanwhile", "--", "z.txt"],
    );
    let moved: String = git_text(world.repo.as_path(), &["rev-parse", "main"]);
    // Another program stages its own version of `f.txt` and puts the worktree back to the merge's content.
    std::fs::write(world.repo.join("f.txt"), b"f.txt staged by someone else\n")
        .expect("the staged version");
    git(world.repo.as_path(), &["add", "--", "f.txt"]);
    std::fs::write(world.repo.join("f.txt"), b"f.txt feature\n")
        .expect("the worktree as the merge left it");
    let staged_entry: String = git_text(
        world.repo.as_path(),
        &["ls-files", "--stage", "--", "f.txt"],
    );
    let _record: PathBuf = record_of(
        &world,
        dead_owner("22222222-0000-4000-8000-000000000022"),
        Some("refs/heads/main"),
    );
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect_err("a staged edit of a merge path stops the settling");
    assert!(
        refused.0.contains("f.txt"),
        "the path is named: {}",
        refused.0
    );
    assert_eq!(
        git_text(
            world.repo.as_path(),
            &["ls-files", "--stage", "--", "f.txt"]
        ),
        staged_entry,
        "the staged version is kept"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        moved,
        "the branch is the moved one"
    );
    remove(world.parent.as_path());
}

/// What:
///  A directory that the branch's own files explain does not stop the settling of a branch that moved.
/// Why:
///  The moved-branch settling judged paths with its own loop,
/// so the directory-to-file shape refused it for good,
/// as the
///  straight path did before its own rule
/// (a cross-vendor review of the third round,
/// and the brief's item 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = directoryToFileWorld(); await readTree(repo, old, merged, { updateWorktree: false }); commitUnrelated(); expect((await recover(...))[0].action).toBe('branch-moved');
/// ```
#[test]
fn a_directory_that_explains_itself_does_not_stop_the_moved_branch_settling() {
    let world: World = directory_to_file_world("merge-recovery-moved-directory");
    // The merge's index is installed and the worktree still holds the branch's directory.
    git(
        world.repo.as_path(),
        &[
            "read-tree",
            "-m",
            world.old_oid.as_str(),
            world.new_oid.as_str(),
        ],
    );
    // A commit on top of the old head leaves the directory alone: the branch moved.
    std::fs::write(world.repo.join("z.txt"), b"committed meanwhile\n").expect("file");
    git(world.repo.as_path(), &["add", "--", "z.txt"]);
    git(
        world.repo.as_path(),
        &["commit", "--quiet", "--message=meanwhile", "--", "z.txt"],
    );
    let moved: String = git_text(world.repo.as_path(), &["rev-parse", "main"]);
    let _record: PathBuf = record_of(
        &world,
        dead_owner("23232323-0000-4000-8000-000000000023"),
        Some("refs/heads/main"),
    );
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("the settling goes on");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::BranchMoved);
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        moved,
        "the branch is the moved one"
    );
    assert!(
        world.repo.join("dir/child.txt").is_file(),
        "the branch's directory is still the checkout's"
    );
    remove(world.parent.as_path());
}

/// What:
///  A saved merge whose install stops after the index was written is reported as begun,
///  and the command stops.
/// Why:
///  The catch for a saved merge accepted every error and said nothing of the merge was in the checkout,
/// although the
///  index had been written and a file had been moved aside
/// (a cross-vendor review of the third round,
/// finding 5).
/// A partly installed merge read by a command as another session's work is exactly what the stop exists for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = directoryToFileWorld(); deferredRecord(); const held = holdOpen('dir/child.txt'); await expect(recover(...)).rejects.toMatch(/partly installed/); release(held);
/// ```
#[test]
fn a_saved_merge_that_is_partly_installed_stops_the_command() {
    let world: World = directory_to_file_world("merge-recovery-partly-installed");
    let _record: PathBuf = deferred_record_of(
        &world,
        "24242424-0000-4000-8000-000000000024",
        "refs/cli-git/kept/24",
    );
    // A process that can be read holds the file the merge deletes open, so its private name is kept and the directory the
    //  merge replaces cannot be emptied: the checkout then fails after the index was written.
    let kept_open: std::fs::File =
        std::fs::File::open(world.repo.join("dir/child.txt")).expect("the file to keep open");
    let mut holder: std::process::Command = std::process::Command::new("sleep");
    holder
        .arg("120")
        .stdin(std::process::Stdio::from(kept_open));
    let mut writer: std::process::Child =
        spawn_owned(&mut holder).expect("a process holding the file open");
    drop(holder);
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect_err("a partly installed merge stops the command");
    writer.kill().expect("the process ends");
    writer.wait().expect("reaped");
    assert!(
        refused.0.contains("partly installed"),
        "it says the merge is partly installed: {}",
        refused.0
    );
    assert!(
        !world.repo.join("dir/child.txt").exists(),
        "the file was moved aside before the failure"
    );
    remove(world.parent.as_path());
}

/// What:
///  A side ref another command repointed is not deleted as if it still held this merge.
/// Why:
///  The deletion named only the ref,
/// so a ref that had come to hold another merge's commit was deleted with it,
/// and that
///  merge's only name went with it
/// (a cross-vendor review of the third round,
/// finding 6).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = deferredRecord(world, 'refs/cli-git/kept/25'); await git(['update-ref', '--no-deref', 'refs/cli-git/kept/25', other]); await recover(...); expect(await revParse('refs/cli-git/kept/25')).toEqual(other);
/// ```
#[test]
fn a_side_ref_another_command_repointed_is_not_deleted() {
    let world: World = world("merge-recovery-side-ref-repointed");
    let _record: PathBuf = deferred_record_of(
        &world,
        "25252525-0000-4000-8000-000000000025",
        "refs/cli-git/kept/25",
    );
    // Another command puts its own commit on that ref, the only name it has.
    let other: String = git_text(
        world.repo.as_path(),
        &["commit-tree", "HEAD^{tree}", "-m", "another merge"],
    );
    git(
        world.repo.as_path(),
        &[
            "update-ref",
            "--no-deref",
            "refs/cli-git/kept/25",
            other.as_str(),
        ],
    );
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("the merge is finished");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "refs/cli-git/kept/25"]),
        other,
        "the other command's ref is kept"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        world.new_oid,
        "this merge landed"
    );
    remove(world.parent.as_path());
}

/// What:
///  A merge saved for one branch is never installed into another branch's checkout.
/// Why:
///  The settlement resolved the recorded ref,
/// which still held the same commit,
/// and installed the merge into whatever the
///  worktree had checked out,
/// moving the recorded branch from a checkout that was not its own
///  (a cross-vendor review of the third round,
/// finding 7).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = deferredRecord(world, 'refs/cli-git/kept/26'); await git(['branch', 'other']); await git(['switch', 'other']); const outcomes = await recover(...); expect(await symbolicRef()).toEqual('refs/heads/other'); expect(await revParse('main')).toEqual(oldOid); expect(exists(record)).toBe(true);
/// ```
#[test]
fn a_saved_merge_for_another_branch_is_not_installed_into_the_checked_out_one() {
    let world: World = world("merge-recovery-other-branch");
    let record: PathBuf = deferred_record_of(
        &world,
        "26262626-0000-4000-8000-000000000026",
        "refs/cli-git/kept/26",
    );
    git(world.repo.as_path(), &["branch", "--quiet", "other"]);
    git(world.repo.as_path(), &["switch", "--quiet", "other"]);
    let before = snapshot(&world);
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("the command goes on");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(
        text.contains("refs/heads/main"),
        "the branch the merge is for is named: {text}"
    );
    assert_eq!(outcomes[0].action, MergeRecoveryAction::DeferredRefused);
    assert_eq!(
        git_text(world.repo.as_path(), &["symbolic-ref", "HEAD"]),
        "refs/heads/other",
        "the checkout stays on its own branch"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        world.old_oid,
        "the merge's branch did not move"
    );
    assert_eq!(
        snapshot(&world),
        before,
        "nothing of the merge is in that checkout"
    );
    assert!(
        record.exists(),
        "the record is kept for when its branch is checked out"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "refs/cli-git/kept/26"]),
        world.new_oid,
        "the side ref is kept"
    );
    remove(world.parent.as_path());
}
