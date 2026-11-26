//! What:
//!  Whether a save beside a path is the merge's own bytes:
//!  a save that arrives after a kill,
//!  one whose reading was cut short,
//!  one whose reading is gone,
//!  a take that recorded nothing,
//!  a merge that still runs and names the copy,
//!  a program still holding it open,
//!  a mode the path does not have,
//!  and a pipe beside the path.
//! Why:
//!  A save the command cannot prove is its own may be the person's,
//!  so no copy is deleted and no command waits on a pipe.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(read(copy)).toEqual(theBytesOfACommit(world, oid, path)); expect(exists(copy)).toBe(true);
//! ```

use super::super::*;
use super::fixtures::{World, an_earlier_releases_name, half_carried, place, record_of, world};
use super::readings::the_bytes_of_a_commit;
use crate::test_support::remove;
use std::path::{Path, PathBuf};
use crate::transaction_owner::{TransactionOwner, current_transaction_owner};

/// What:
///  A save that holds exactly the merge's bytes and arrives after a kill stands,
///  because the reading of what Git's update left travels beside the record.
/// Why:
///  The recovery of a killed merge had no reading of its own,
/// so a person's save whose bytes accept every line the
///  merge brought was read as the merge's own file and replaced with their older layer,
/// and the bytes of their newer
///  decision existed nowhere (`merge-carry-review-round-2-findings-1`,
/// its third finding).
/// The reading the command that moved the checkout made is
///  what tells the two apart,
/// so a recovery reads it too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const capture = halfCarried(world); writeLandedRecord(gitDir, id, landedOf(root, ['f.txt'])); write('f.txt', mergeVersion); await recover(...); expect(read('f.txt')).toEqual(mergeVersion); expect(sink).toContain('changed');
/// ```
#[test]
fn a_save_of_the_merges_own_bytes_after_a_kill_stands() {
    let world: World = world("merge-recovery-landed-reading");
    let id: &str = "26262626-0000-4000-8000-000000000026";
    let _capture: crate::merge_isolation_carry_parts::Capture = half_carried(&world, id);
    let merged: Vec<u8> = the_bytes_of_a_commit(&world, world.new_oid.as_str(), "f.txt");
    // What the command that moved the checkout read, written where the recovery of a kill finds it.
    let landed: Vec<crate::merge_isolation_carry_landed::LandedFile> =
        crate::merge_isolation_carry_landed::landed_of(world.repo.as_path(), &[b"f.txt".to_vec()]);
    crate::merge_isolation_carry_landed::write_landed_record(world.git_dir.as_path(), id, landed.as_slice());
    // The person's editor saves its buffer, which holds the merge's version and no longer their older line.
    std::fs::write(world.repo.join("f.txt"), merged.as_slice()).expect("the save");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::AlreadyLanded, "the merge is the branch's");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(std::fs::read(world.repo.join("f.txt")).expect("f.txt"), merged, "the newer save stands");
    assert!(text.contains("f.txt"), "the line names the path it left alone: {text}");
    assert!(text.contains("changed"), "the line says the path changed after the capture: {text}");
    assert!(!text.contains("git merge refs/heads/"), "the line never calls a repository ref the merge the person ran: {text}");
    remove(world.parent.as_path());
}

/// What:
///  A save of the merge's own bytes stands when the reading of what Git's update left was cut short,
///  and the bytes this merge carries for the path are kept beside it and named.
/// Why:
///  The reading is written under a private name and renamed onto its own,
/// so a kill cannot leave half of it;
///  a reading that is
///  present and cannot be read is still a state this command must not answer by writing over the file:
/// a save made
///  after the kill holds exactly the merge's own bytes,
/// and the byte comparison cannot tell it from Git's own write,
///  so the file is left as it is and the layer the carry would have written is kept at a copy the person is told about
/// (`SPEC.md`,
/// the copy whose bytes the file at its path holds again;
/// a Kimi K3.1 Pro review of `bc3d54f9cd`,
/// its fourth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const capture = halfCarried(world); writeLandedRecord(id, landedOf(['f.txt'])); truncate(landedOf(id), half); write('f.txt', mergeVersion); await recover(...); expect(read('f.txt')).toEqual(mergeVersion); const aside = asidesOf('f.txt')[0]; expect(read(aside)).toEqual(`${mergeVersion}${PERSON_LINE}\n`);
/// ```
#[test]
fn a_save_of_the_merges_own_bytes_stands_when_the_reading_was_cut_short() {
    let world: World = world("merge-recovery-torn-landed-reading");
    let id: &str = "27272727-0000-4000-8000-000000000027";
    let _capture: crate::merge_isolation_carry_parts::Capture = half_carried(&world, id);
    let merged: Vec<u8> = the_bytes_of_a_commit(&world, world.new_oid.as_str(), "f.txt");
    let landed: Vec<crate::merge_isolation_carry_landed::LandedFile> =
        crate::merge_isolation_carry_landed::landed_of(world.repo.as_path(), &[b"f.txt".to_vec()]);
    crate::merge_isolation_carry_landed::write_landed_record(world.git_dir.as_path(), id, landed.as_slice());
    // The bytes a killed write left before the record took its own name: half of a complete record.
    let record: PathBuf = crate::merge_isolation_carry_landed::landed_record_path(world.git_dir.as_path(), id);
    let whole: Vec<u8> = std::fs::read(record.as_path()).expect("the reading");
    std::fs::write(record.as_path(), &whole[..whole.len() / 2]).expect("the cut reading");
    std::fs::write(world.repo.join("f.txt"), merged.as_slice()).expect("the save");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::AlreadyLanded, "the merge is the branch's");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    let saved: Vec<u8> = std::fs::read(world.repo.join("f.txt")).expect("f.txt");
    assert_eq!(saved, merged, "the save stands: nothing proved it was the entry Git's update wrote");
    let kept: Vec<crate::merge_isolation_take_aside_names::Aside> = crate::merge_isolation_take_aside_names::asides_of(world.repo.join("f.txt").as_path());
    assert_eq!(kept.len(), 1, "the bytes this merge carried are kept beside the path");
    let carried: String = String::from_utf8(std::fs::read(kept[0].path.as_path()).expect("the kept layer")).expect("the kept layer is text");
    assert_eq!(carried, "f.txt feature\nthe person's line\n", "the layer the carry would have written is whole");
    assert!(text.contains(kept[0].path.to_string_lossy().as_ref()), "the copy is named: {text}");
    assert!(
        crate::merge_isolation_kept_copies::record_of_a_control(world.git_dir.as_path(), kept[0].path.as_path()).exists(),
        "the copy is recorded, so a later command decides it again"
    );
    remove(world.parent.as_path());
}

/// What:
///  The same,
/// for a record whose reading was removed before the record it belongs to.
/// Why:
///  A cleanup that deleted the reading first left a record with no reading,
/// and the next command then read the checkout
///  by bytes alone and wrote the older layer over a save made after the crash;
/// the removal now takes the record first,
/// so a kill
///  leaves a reading no command reads instead,
/// and the recovery keeps both whatever the state it was handed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const capture = halfCarried(world); rm(landedOf(id)); write('f.txt', mergeVersion); await recover(...); expect(read('f.txt')).toEqual(mergeVersion); expect(asidesOf('f.txt')).toHaveLength(1);
/// ```
#[test]
fn a_save_of_the_merges_own_bytes_stands_when_the_reading_is_gone() {
    let world: World = world("merge-recovery-no-landed-reading");
    let id: &str = "27272727-0000-4000-8000-0000000000a7";
    let _capture: crate::merge_isolation_carry_parts::Capture = half_carried(&world, id);
    let merged: Vec<u8> = the_bytes_of_a_commit(&world, world.new_oid.as_str(), "f.txt");
    std::fs::write(world.repo.join("f.txt"), merged.as_slice()).expect("the save");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::AlreadyLanded, "the merge is the branch's");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(std::fs::read(world.repo.join("f.txt")).expect("f.txt"), merged, "the save stands");
    let kept: Vec<crate::merge_isolation_take_aside_names::Aside> = crate::merge_isolation_take_aside_names::asides_of(world.repo.join("f.txt").as_path());
    assert_eq!(kept.len(), 1, "the bytes this merge carried are kept beside the path");
    assert_eq!(
        std::fs::read(kept[0].path.as_path()).expect("the kept layer"),
        b"f.txt feature\nthe person's line\n",
        "the layer the carry would have written is whole"
    );
    assert!(text.contains(kept[0].path.to_string_lossy().as_ref()), "the copy is named: {text}");
    remove(world.parent.as_path());
}

/// What:
///  A take a killed placement left with no record of itself is put back before a landed merge is finished,
///  so the person's file is not left absent while its bytes wait beside the path.
/// Why:
///  The take registers a kept copy only after the placement returns,
/// so a kill between the rename and the placement leaves a
///  private name no record names;
/// the recovery of a merge the branch already names carried the capture without putting
///  that name back,
/// read the empty path as somebody else's write,
/// and dropped the record and the capture ref with the person's
///  file still absent (a Kimi K3.1 Pro review of `bc3d54f9cd`,
/// its first finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const capture = halfCarried(world); writeLandedRecord(id); rename('f.txt', asideName('f.txt', id)); await recover(...); expect(read('f.txt')).toEqual("f.txt feature\nthe person's line\n"); expect(exists(aside)).toBe(false); expect(exists(record)).toBe(false);
/// ```
#[test]
fn a_take_that_recorded_nothing_is_put_back_before_a_landed_merge_finishes() {
    let world: World = world("merge-recovery-unrecorded-take");
    let id: &str = "28282828-0000-4000-8000-000000000082";
    let _capture: crate::merge_isolation_carry_parts::Capture = half_carried(&world, id);
    let record: PathBuf = crate::merge_isolation_journal::intent_path(world.git_dir.as_path(), id);
    let landed: Vec<crate::merge_isolation_carry_landed::LandedFile> =
        crate::merge_isolation_carry_landed::landed_of(world.repo.as_path(), &[b"f.txt".to_vec()]);
    crate::merge_isolation_carry_landed::write_landed_record(world.git_dir.as_path(), id, landed.as_slice());
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = crate::merge_isolation_take_aside_names::aside_name(location.as_path(), id);
    std::fs::rename(location.as_path(), aside.as_path()).expect("the take the killed placement left");
    // The bytes the placement had written under its own private name, which no record names and the next command makes again.
    let private: PathBuf = world.repo.join(format!(".cli-git-carry-{}-0-1-f.txt", std::process::id()));
    std::fs::write(private.as_path(), b"f.txt feature\nthe person's line\n").expect("the placement's own bytes");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("recovery");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::AlreadyLanded, "the merge is the branch's");
    assert_eq!(
        std::fs::read(location.as_path()).expect("the person's file is back"),
        b"f.txt feature\nthe person's line\n",
        "the file is back with the merge and the person's own line: {text}"
    );
    assert!(!aside.exists(), "the entry that was put back is not left beside the path: {text}");
    assert!(!record.exists(), "the record goes once every byte of the person's is in the file");
    remove(world.parent.as_path());
}

/// What:
///  A copy a merge that still runs names is left alone by a command beside it.
/// Why:
///  A live merge keeps a copy because a path it is still writing was in the way,
/// and a command that put the copy
///  back under that merge made Git refuse the checkout it was about to make,
/// so a command with nothing to do with the
///  merge turned a safe merge into a failed one
/// (a Kimi K3.1 Pro review of `bc3d54f9cd`,
/// its second finding;
/// `SPEC.md`,
/// the record of a merge whose owner still runs).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordOf(liveOwner); aside = asideName('f.txt', id); keepCopy(aside); rm('f.txt'); await recover(...); expect(exists('f.txt')).toBe(false); expect(exists(aside)).toBe(true);
/// ```
#[test]
fn a_copy_a_merge_that_still_runs_names_is_left_alone() {
    let world: World = world("merge-recovery-live-merge-copy");
    let id: &str = "29292929-0000-4000-8000-000000000092";
    let owner: TransactionOwner = current_transaction_owner(id, "2026-10-29T00:00:00.000Z").expect("the owner of this control");
    let record: PathBuf = record_of(&world, owner, Some("refs/heads/main"));
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = crate::merge_isolation_take_aside_names::aside_name(location.as_path(), id);
    std::fs::write(aside.as_path(), b"the person's untracked bytes\n").expect("the copy the live merge kept");
    crate::merge_isolation_kept_copies::keep_the_copy(world.git_dir.as_path(), location.as_path(), aside.as_path());
    std::fs::remove_file(location.as_path()).expect("the path the live merge cleared");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("recovery");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::OwnerActive, "the record of a merge that still runs is not settled");
    assert!(std::fs::symlink_metadata(location.as_path()).is_err(), "the path the live merge cleared stays free: {text}");
    assert!(aside.exists(), "the copy the live merge kept is not put back under it: {text}");
    assert!(!text.contains("put back"), "the command says nothing about putting a copy back: {text}");
    assert!(record.exists(), "the record of the merge that still runs stays");
    remove(world.parent.as_path());
}

/// What:
///  A copy a program still has open is not removed when the file at its path holds the same bytes.
/// Why:
///  A write through a descriptor opened before the take lands in the inode the copy names,
/// and an unlink of that name
///  while the writer lives leaves the write with no path at all,
/// so the copy goes only on the proof the aside's own end
///  asks for:
/// no process this command can read holds it open
/// (a Kimi K3.1 Pro review of `bc3d54f9cd`,
/// its third finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write('f.txt', bytes); write(aside, bytes); keepCopy(aside); holder = sleep(stdin = open(aside)); await recover(...); expect(exists(aside)).toBe(true); holder.kill(); await recover(...); expect(exists(aside)).toBe(false);
/// ```
#[test]
fn a_copy_a_program_still_holds_open_is_not_removed() {
    let world: World = world("merge-recovery-copy-held-open");
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = an_earlier_releases_name(location.as_path());
    let kept: &[u8] = b"same bytes\n";
    std::fs::write(aside.as_path(), kept).expect("the copy");
    std::fs::write(location.as_path(), kept).expect("the person's file holds the same bytes");
    crate::merge_isolation_kept_copies::keep_the_copy(world.git_dir.as_path(), location.as_path(), aside.as_path());
    let mut holder_command: std::process::Command = std::process::Command::new("sleep");
    holder_command
        .arg("120")
        .stdin(std::process::Stdio::from(std::fs::File::open(aside.as_path()).expect("the copy's descriptor")));
    let mut holder: std::process::Child = crate::test_support::spawn_owned(&mut holder_command).expect("a program holding the copy open");
    let mut said: Vec<u8> = Vec::new();
    let _first = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("recovery");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(aside.exists(), "the copy a program holds open stays: {text}");
    assert!(text.contains("holds the copy open"), "the line says why the copy stays: {text}");
    assert_eq!(std::fs::read(aside.as_path()).expect("the copy"), kept, "and its bytes are whole");
    holder.kill().expect("the holder ends");
    holder.wait().expect("the holder is reaped");
    let mut later: Vec<u8> = Vec::new();
    let _outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut later).expect("recovery");
    let ending: String = String::from_utf8(later).expect("the notices are UTF-8");
    assert!(!aside.exists(), "the copy goes once nothing holds it open: {ending}");
    assert!(ending.contains("removed"), "the line says the copy went: {ending}");
    remove(world.parent.as_path());
}

/// Set or clear the executable bits of one fixture entry.
fn set_executable(path: &Path, executable: bool) {
    use std::os::unix::fs::PermissionsExt;
    let mode: u32 = if executable { 0o755 } else { 0o644 };
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).expect("the mode of the fixture entry");
}

/// What:
///  A copy whose permissions the file at its path does not have stays,
///  and is named with that reason.
/// Why:
///  One run of bytes is not one file:
/// a copy that is executable where its path is not is the only executable copy of
///  those bytes,
/// and removing it loses the person's mode while the line says nothing of theirs was lost
/// (a Kimi K3.1 Pro review of `bc3d54f9cd`,
/// its sixth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write('f.txt', bytes, 0o644); write(aside, bytes, 0o755); keepCopy(aside); await recover(...); expect(exists(aside)).toBe(true); chmod('f.txt', 0o755); await recover(...); expect(exists(aside)).toBe(false);
/// ```
#[test]
fn a_copy_with_a_mode_the_path_does_not_have_stays() {
    let world: World = world("merge-recovery-copy-mode");
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = an_earlier_releases_name(location.as_path());
    std::fs::write(aside.as_path(), b"same bytes\n").expect("the copy");
    std::fs::write(location.as_path(), b"same bytes\n").expect("the person's file holds the same bytes");
    set_executable(aside.as_path(), true);
    set_executable(location.as_path(), false);
    crate::merge_isolation_kept_copies::keep_the_copy(world.git_dir.as_path(), location.as_path(), aside.as_path());
    let mut said: Vec<u8> = Vec::new();
    let _first = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said).expect("recovery");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(aside.exists(), "the only executable copy stays: {text}");
    assert!(text.contains("same permissions"), "the line says why the copy stays: {text}");
    // The person makes their file executable, which is the end that line named.
    set_executable(location.as_path(), true);
    let mut later: Vec<u8> = Vec::new();
    let _outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut later).expect("recovery");
    let ending: String = String::from_utf8(later).expect("the notices are UTF-8");
    assert!(!aside.exists(), "the copy goes once the path holds its bytes and its permissions: {ending}");
    assert!(ending.contains("removed"), "the line says the copy went: {ending}");
    remove(world.parent.as_path());
}

/// What:
///  A pipe standing where a copy's path belongs does not hold the command.
/// Why:
///  The decision about a copy compares the bytes of the file at its path,
/// and a pipe has none:
/// `git hash-object` on one
///  waits for a writer that may never come,
/// so a command that met a pipe never finished
/// (`AGENTS.md`,
/// "nothing
///  should ever wait forever or indefinitely").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write(aside, bytes); mkfifo('f.txt'); keepCopy(aside); const answered = await within(5, recover(...)); expect(answered).toBe(true); expect(exists(aside)).toBe(true);
/// ```
#[test]
fn a_pipe_beside_a_path_does_not_hold_the_command() {
    let world: World = world("merge-recovery-pipe-beside-a-path");
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = an_earlier_releases_name(location.as_path());
    std::fs::write(aside.as_path(), b"same bytes\n").expect("the copy");
    std::fs::remove_file(location.as_path()).expect("the path the take emptied");
    let made: std::process::Output = std::process::Command::new("mkfifo").arg(&location).output().expect("mkfifo starts");
    assert!(made.status.success(), "the pipe is made: {}", String::from_utf8_lossy(made.stderr.as_slice()));
    crate::merge_isolation_kept_copies::keep_the_copy(world.git_dir.as_path(), location.as_path(), aside.as_path());
    let git_dir: PathBuf = world.git_dir.clone();
    let recovery_place: RecoveryPlace = place(&world);
    let deadline: std::time::Duration = std::time::Duration::from_secs(20);
    // A reading of the pipe waits for a writer that never comes, so the probe thread is not joined: one that cannot answer
    //  within the deadline is left to its wait and the control fails with what it has, because joining it here would hold this
    //  test on the very wait it forbids (`parallel_test_support::finished_within_at`).
    let answered: Option<Vec<u8>> = crate::parallel_test_support::finished_within_at(deadline, move || {
        let mut said: Vec<u8> = Vec::new();
        let _outcomes = recover_interrupted_merges(&recovery_place, git_dir.as_path(), &mut said).expect("recovery");
        return said;
    });
    let Some(said) = answered else {
        panic!("a command did not answer within {deadline:?} for a pipe beside a copy");
    };
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(aside.exists(), "the copy stays: a pipe holds no bytes to compare: {text}");
    remove(world.parent.as_path());
}
