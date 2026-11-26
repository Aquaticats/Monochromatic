//! What:
//!  A path that changed after the merge judged it:
//!  a file saved or created then,
//!  a private name a killed take left,
//!  a file that appeared where the merge deletes,
//!  a take or a write that fails after taking files,
//!  and a mode or bytes changed under the same length and time.
//! Why:
//!  The person's work outranks the merge:
//!  what stands at the path when the write comes is kept and named,
//!  never replaced.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(after).toEqual(before); expect(asideNames(repo)).toHaveLength(1);
//! ```

use super::super::*;
use super::fixtures::{
    A_CONTROLS_MERGE,
    World,
    aside_names,
    dead_owner,
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
/// The install of a merge's index entries and files,
/// which a placement of the merge's own bytes makes,
/// and the put-back of the files a killed take left beside their paths.
use crate::merge_isolation_recovery_steps::{
    install_entries,
    put_back_stranded_takes_awaiting,
};
use crate::test_support::{
    git,
    git_text,
    remove,
};
use std::path::PathBuf;

/// What:
///  A file saved after recovery judged it is never replaced by the merge.
/// Why:
///  The judgement and the write of a merged file were separate steps,
/// so an editor that saved in between lost the
///  edit with no notice (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 4).
/// The take checks
///  what it took against the file system identity the judgement read,
/// so the edit stays and the merge stops,
/// naming
///  the path and keeping the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const (changes, judged) = judge(world); write('f.txt', edit); await expect(installEntries(place, root, changes, judged, sink)).rejects.toMatch(/f.txt/); expect(read('f.txt')).toEqual(edit);
/// ```
#[test]
fn a_file_saved_after_the_judgement_is_not_replaced() {
    let world: World = world("merge-recovery-edit-after-judging");
    // Git had not written this path yet: the checkout holds the old content, so the merge writes it.
    let (changes, judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let before = snapshot(&world);
    // The person's editor saves one of the merge's paths after the judgement.
    std::fs::write(
        world.repo.join("f.txt"),
        b"an edit saved after the judgement\n",
    )
    .expect("the edit");
    let mut said: Vec<u8> = Vec::new();
    let refused = install_entries(
        &place(&world),
        world.repo.as_path(),
        changes.as_slice(),
        judgements.as_slice(),
        false,
        A_CONTROLS_MERGE,
        &mut said,
    )
    .expect_err("a file that changed after the judgement is not the merge's to replace");
    assert!(
        refused.error.0.contains("f.txt"),
        "the path is named: {}",
        refused.error.0
    );
    assert!(
        refused
            .error
            .0
            .contains("changed after this command judged them"),
        "the reason is stated: {}",
        refused.error.0
    );
    assert_eq!(
        std::fs::read(world.repo.join("f.txt")).expect("the edit"),
        b"an edit saved after the judgement\n",
        "the edit is kept"
    );
    let after = snapshot(&world);
    assert_eq!(after.0, before.0, "the index is exactly as it was");
    assert_eq!(
        after.1, " M f.txt",
        "the checkout is exactly as it was, and the merge is still unfinished"
    );
    assert_eq!(
        aside_names(world.repo.as_path()),
        Vec::<String>::new(),
        "nothing was left at a private name"
    );
    remove(world.parent.as_path());
}

/// What:
///  A file that appeared where the merge adds one is refused,
///  put back,
///  and never replaced.
/// Why:
///  The judged state there was `Missing`,
/// so any file is someone else's;
/// without the take the merge would have written over it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rm('dir/new.txt'); const (changes, judged) = judge(world); write('dir/new.txt', mine); await expect(installEntries(...)).rejects.toMatch(/dir\/new.txt/); expect(read('dir/new.txt')).toEqual(mine);
/// ```
#[test]
fn a_file_created_after_the_judgement_is_not_replaced() {
    let world: World = world("merge-recovery-create-after-judging");
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
    // The merge adds `dir/new.txt`; it is gone when the paths are judged, so its judged state is `Missing`.
    std::fs::remove_file(world.repo.join("dir/new.txt")).expect("the merge's file goes");
    let (changes, judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    std::fs::write(world.repo.join("dir/new.txt"), b"a file of my own\n")
        .expect("the person's file");
    let mut said: Vec<u8> = Vec::new();
    let refused = install_entries(
        &place(&world),
        world.repo.as_path(),
        changes.as_slice(),
        judgements.as_slice(),
        false,
        A_CONTROLS_MERGE,
        &mut said,
    )
    .expect_err("a file that appeared where the merge adds one is not the merge's to replace");
    assert!(
        refused.error.0.contains("dir/new.txt"),
        "the path is named: {}",
        refused.error.0
    );
    assert_eq!(
        std::fs::read(world.repo.join("dir/new.txt")).expect("the person's file"),
        b"a file of my own\n",
        "the file is put back"
    );
    assert_eq!(
        aside_names(world.repo.as_path()),
        Vec::<String>::new(),
        "nothing was left at a private name"
    );
    remove(world.parent.as_path());
}

/// What:
///  A file a killed take set aside is put back by the next command when its path is free,
///  and the command says so.
/// Why:
///  A kill between the rename of a take and the write that follows leaves the file under a private name and its
///  path missing,
/// so recovery judged the path as someone else's and stopped every command,
/// while the person's file sat
///  where nothing looks
/// (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 4's take,
/// round
///  two).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rename('f.txt', stranded); await recoverInterruptedMerges(place, dir, sink); expect(sink).toContain('put back'); expect(exists(stranded)).toBe(false);
/// ```
#[test]
fn a_file_a_killed_take_set_aside_is_put_back_and_reported() {
    let world: World = world("merge-recovery-stranded-take");
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
    let location: PathBuf = world.repo.join("f.txt");
    let stranded: PathBuf =
        stranded_name(location.as_path(), "15151515-0000-4000-8000-000000000015");
    let content: Vec<u8> = std::fs::read(location.as_path()).expect("the file Git wrote");
    std::fs::rename(location.as_path(), stranded.as_path()).expect("a killed take");
    let record: PathBuf = record_of(
        &world,
        dead_owner("15151515-0000-4000-8000-000000000015"),
        Some("refs/heads/main"),
    );
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("recovery");
    assert_eq!(
        outcomes[0].action,
        MergeRecoveryAction::Completed,
        "the merge is finished"
    );
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(
        text.contains("put back") && text.contains("f.txt"),
        "the putting back is said: {text}"
    );
    assert_eq!(
        std::fs::read(location.as_path()).expect("the file is back"),
        content,
        "the bytes are the ones that were set aside"
    );
    assert!(!stranded.exists(), "the private name is gone");
    assert!(!record.exists());
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        world.new_oid
    );
    remove(world.parent.as_path());
}

/// What:
///  A file a killed take set aside is kept beside a path that now holds something else,
///  and the private name is said.
/// Why:
///  Nothing of the person's may be removed,
/// so both files stay and the person is told where the second one is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rename('f.txt', stranded); write('f.txt', mine); const error = await recover(...); expect(error).toMatch(/f.txt/); expect(sink).toContain(stranded); expect(exists(stranded)).toBe(true);
/// ```
#[test]
fn a_file_a_killed_take_set_aside_is_kept_beside_a_path_that_holds_something_else() {
    let world: World = world("merge-recovery-stranded-take-both");
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
    let location: PathBuf = world.repo.join("f.txt");
    let stranded: PathBuf =
        stranded_name(location.as_path(), "16161616-0000-4000-8000-000000000016");
    std::fs::rename(location.as_path(), stranded.as_path()).expect("a killed take");
    std::fs::write(location.as_path(), b"my own file\n").expect("the person's file");
    let _record: PathBuf = record_of(
        &world,
        dead_owner("16161616-0000-4000-8000-000000000016"),
        Some("refs/heads/main"),
    );
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect_err("the path holds someone else's content");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(
        text.contains("f.txt") && text.contains(stranded.to_string_lossy().as_ref()),
        "the private name is said: {text}"
    );
    assert!(refused.0.contains("f.txt"), "{}", refused.0);
    assert_eq!(
        std::fs::read(location.as_path()).expect("the person's file"),
        b"my own file\n",
        "nothing was removed"
    );
    assert!(stranded.exists(), "the file that was set aside is kept");
    remove(world.parent.as_path());
}

/// What:
///  A private name no record claims is never put back on a guess,
///  and a copy whose bytes Git holds is removed and said so,
///  while one a record claims is put back.
/// Why:
///  A merge that ended keeps no record,
/// so a file it left beside a path belongs to no checkout state at all;
/// putting it back when a later
///  merge found the path free brought back a file the person had deleted,
/// and the later merge carried that older version
///  forward
///  (a cross-vendor review of the second round,
/// its third finding).
///  A copy the merge's own commit holds the bytes of is the second copy of them,
/// so it
///  goes and the line names the commit,
/// and the path the person emptied stays empty.
/// The claimed file is the control:
/// the same step still puts back what a live record names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// record(deadOwner('2020...')); rename('f.txt', asideOf('f.txt', '2020...')); rename('gone.txt', asideOf('gone.txt', '2121...')); await putBackStrandedTakes(...); expect(exists(asideOf('gone.txt'))).toBe(false); expect(exists('gone.txt')).toBe(false); expect(sink).toContain('removed');
/// ```
#[test]
fn a_private_name_no_record_claims_is_never_put_back_on_a_guess() {
    let world: World = world("merge-recovery-aside-without-record");
    let (changes, _judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    // One merge is still unfinished, and its own take waits beside `f.txt`.
    let record: PathBuf = record_of(
        &world,
        dead_owner("20202020-0000-4000-8000-000000000020"),
        Some("refs/heads/main"),
    );
    let claimed_location: PathBuf = world.repo.join("f.txt");
    let claimed: PathBuf = stranded_name(
        claimed_location.as_path(),
        "20202020-0000-4000-8000-000000000020",
    );
    std::fs::rename(claimed_location.as_path(), claimed.as_path())
        .expect("a take of the record's merge");
    // A merge that ended left this one, and no record names it any more.
    let ended_location: PathBuf = world.repo.join("gone.txt");
    let ended: PathBuf = stranded_name(
        ended_location.as_path(),
        "21212121-0000-4000-8000-000000000021",
    );
    std::fs::rename(ended_location.as_path(), ended.as_path())
        .expect("a take of a merge that ended");
    let mut said: Vec<u8> = Vec::new();
    crate::lazy_runtime::run_on_runtime(put_back_stranded_takes_awaiting(
        &place(&world).git,
        world.repo.as_path(),
        world.git_dir.as_path(),
        changes.as_slice(),
        None,
        "2a2a2a2a-0000-4000-8000-00000000002a",
        &mut said,
    ))
    .expect("the runtime of the controls");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(
        !claimed.exists(),
        "a file a record claims loses its private name"
    );
    assert!(
        claimed_location.exists(),
        "the claimed path holds its file again"
    );
    assert!(
        !ended.exists(),
        "a copy whose bytes the merge's commit holds is removed"
    );
    assert!(
        !ended_location.exists(),
        "the path a person emptied stays empty"
    );
    assert!(
        text.contains(ended.to_string_lossy().as_ref()),
        "the copy that went is named: {text}"
    );
    assert!(
        text.contains("git restore"),
        "the line says how to reach the bytes: {text}"
    );
    assert!(
        record.exists(),
        "the record is not the putting back's to remove"
    );
    remove(world.parent.as_path());
}

/// What:
///  A path the merge deletes that someone fills after it was judged is not deleted.
/// Why:
///  The take removes the judged file by its rename,
/// so the later removal has nothing to do with the merge and would
///  delete whatever appeared there
/// (a cross-vendor review's finding on the round-one take).
/// The judged state here is `Missing`
///  because the path is free when the judgements are made,
/// which is the seam for a file that appears later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const (changes, judged) = judge(world); write('gone.txt', mine); await installEntries(place, root, changes, judged, sink); expect(read('gone.txt')).toEqual(mine);
/// ```
#[test]
fn a_file_that_appears_where_the_merge_deletes_is_not_deleted() {
    let world: World = world("merge-recovery-delete-race");
    // `gone.txt` is a path the merge deletes; it is free when the judgements are made.
    std::fs::remove_file(world.repo.join("gone.txt")).expect("the judged state is `Missing`");
    let (changes, judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    std::fs::write(
        world.repo.join("gone.txt"),
        b"a file made after the judgement\n",
    )
    .expect("the person's file");
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
    assert!(installed.is_ok(), "{installed:?}");
    assert_eq!(
        std::fs::read(world.repo.join("gone.txt")).expect("the person's file"),
        b"a file made after the judgement\n",
        "a file that appeared is never deleted"
    );
    remove(world.parent.as_path());
}

/// What:
///  A file written after it was taken aside is kept rather than dropped with the private name,
///  and the private name is said.
/// Why:
///  A program that already had the file open writes into the inode the take renamed,
/// after the check,
/// so
///  dropping the aside would lose that edit when the writer closes
/// (a cross-vendor review's finding on the round-one take).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const take = takeAside(file); write(aside, edit); const left = finishAsides([take], sink); expect(exists(aside)).toBe(true); expect(sink).toContain(aside);
/// ```
#[test]
fn a_file_written_after_it_was_taken_is_kept_and_named() {
    let world: World = world("merge-recovery-written-after-take");
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = stranded_name(location.as_path(), A_CONTROLS_MERGE);
    let expected: String = git_text(world.repo.as_path(), &["hash-object", "--", "f.txt"])
        .trim()
        .to_owned();
    std::fs::rename(location.as_path(), aside.as_path()).expect("a take");
    let taken = file_identity(aside.as_path()).expect("the identity the take checked");
    // The program that had the file open writes its edit into the inode the take holds.
    std::fs::write(aside.as_path(), b"an edit written after the check\n")
        .expect("the writer's edit");
    let mut said: Vec<u8> = Vec::new();
    let entries: Vec<TakenOnDisk> = vec![TakenOnDisk {
        location: location.clone(),
        path: b"f.txt".to_vec(),
        aside: aside.clone(),
        taken,
        expected: expected.clone(),
    }];
    finish_asides(&place(&world), world.repo.as_path(), &entries, &mut said)
        .expect("the asides are finished");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(
        text.contains("f.txt") && text.contains(aside.to_string_lossy().as_ref()),
        "the private name is said: {text}"
    );
    assert!(aside.exists(), "the file written after the check is kept");
    assert_eq!(
        std::fs::read(aside.as_path()).expect("the writer's edit"),
        b"an edit written after the check\n",
        "the edit is not lost"
    );
    remove(world.parent.as_path());
}

/// What:
///  A take that fails after another file was taken puts that file back.
/// Why:
///  The take pass returned on the error of one path and left every earlier one under its private name,
/// where the
///  next command refuses the path and nothing looks
/// (a cross-vendor review's finding on the round-one take).
/// The failing take is a path in a directory that refuses the rename,
/// which this control skips where it runs as
///  root and permissions do not apply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const (changes, judged) = judge(world); changes.push(extra('sealed/zeta.txt')); chmod('sealed', 0o555); await expect(installEntries(...)).rejects; expect(read('f.txt')).toEqual(old); expect(asides()).toEqual([]);
/// ```
#[cfg(unix)]
#[test]
fn a_take_that_fails_puts_the_files_it_already_took_back() {
    use std::os::unix::fs::PermissionsExt;
    if unsafe { libc::geteuid() } == 0 {
        eprintln!(
            "skipped: a directory that refuses writes does not stop root, so this control proves nothing here"
        );
        return;
    }
    let world: World = world("merge-recovery-take-error");
    let (mut changes, mut judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let old: Vec<u8> = std::fs::read(world.repo.join("f.txt")).expect("the old content");
    // A path of this control's own, in a directory that refuses the rename into it; it is judged last, so the merge's own
    //  files are taken before it.
    std::fs::create_dir(world.repo.join("sealed")).expect("the sealed directory");
    std::fs::write(world.repo.join("sealed/zeta.txt"), b"old zeta\n").expect("the file");
    let identity: Option<crate::merge_isolation_changes::FileIdentity> =
        file_identity(world.repo.join("sealed/zeta.txt").as_path());
    let old_oid: String = git_text(
        world.repo.as_path(),
        &["hash-object", "--", "sealed/zeta.txt"],
    )
    .trim()
    .to_owned();
    changes.push(extra_change(
        "sealed/zeta.txt",
        "100644",
        "100644",
        old_oid.as_str(),
        old_oid.as_str(),
    ));
    judgements.push(Judged {
        standing: Standing::Old,
        identity,
        content: Some(old_oid.clone()),
    });
    std::fs::set_permissions(
        world.repo.join("sealed"),
        std::fs::Permissions::from_mode(0o555),
    )
    .expect("a directory that refuses writes");
    let mut said: Vec<u8> = Vec::new();
    let failed = install_entries(
        &place(&world),
        world.repo.as_path(),
        changes.as_slice(),
        judgements.as_slice(),
        false,
        A_CONTROLS_MERGE,
        &mut said,
    )
    .expect_err("the take of a path in a read-only directory fails");
    let _writable = std::fs::set_permissions(
        world.repo.join("sealed"),
        std::fs::Permissions::from_mode(0o755),
    );
    assert!(!failed.error.0.is_empty(), "the failure is named");
    assert_eq!(
        std::fs::read(world.repo.join("f.txt")).expect("the file"),
        old,
        "the file taken before the failure is put back"
    );
    assert_eq!(
        std::fs::read(world.repo.join("sealed/zeta.txt")).expect("the file"),
        b"old zeta\n",
        "the file that failed is where it was"
    );
    assert_eq!(
        aside_names(world.repo.as_path()),
        Vec::<String>::new(),
        "nothing is left at a private name"
    );
    remove(world.parent.as_path());
}

/// What:
///  A write of the index that Git refuses puts back every file that was taken aside.
/// Why:
///  The index write and the checkout both come after the take,
/// and their error arms deleted the originals instead of
///  restoring them,
/// leaving a file missing that the next command then refuses
///  (a cross-vendor review's finding on the round-one take).
/// The index write is made to fail by an object id no repository holds,
/// which needs no permission to arrange.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const (changes, judged) = judge(world); changes.push(extra('zeta.txt', old, '1234...')); await expect(installEntries(...)).rejects; expect(read('f.txt')).toEqual(old); expect(asides()).toEqual([]);
/// ```
#[test]
fn a_write_that_fails_puts_the_files_it_already_took_back() {
    let world: World = world("merge-recovery-write-error");
    let (mut changes, mut judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    std::fs::write(world.repo.join("zeta.txt"), b"old zeta\n").expect("the file");
    let identity: Option<crate::merge_isolation_changes::FileIdentity> =
        file_identity(world.repo.join("zeta.txt").as_path());
    let old_oid: String = git_text(
        world.repo.as_path(),
        &["hash-object", "-w", "--", "zeta.txt"],
    )
    .trim()
    .to_owned();
    // An object no repository holds: the index write and the checkout both refuse it.
    let absent: String = String::from("1234567890123456789012345678901234567890");
    // The failing path is written first, so the checkout stops before it writes any file the merge changes.
    changes.insert(
        0,
        extra_change(
            "zeta.txt",
            "100644",
            "100644",
            old_oid.as_str(),
            absent.as_str(),
        ),
    );
    judgements.insert(
        0,
        Judged {
            standing: Standing::Old,
            identity,
            content: Some(old_oid.clone()),
        },
    );
    let mut said: Vec<u8> = Vec::new();
    let failed = install_entries(
        &place(&world),
        world.repo.as_path(),
        changes.as_slice(),
        judgements.as_slice(),
        false,
        A_CONTROLS_MERGE,
        &mut said,
    )
    .expect_err("an object the repository does not hold stops the write");
    assert!(!failed.error.0.is_empty(), "the failure is named");
    // The path whose write Git refused holds its old content again; the files Git did write hold the merge's.
    assert_eq!(
        std::fs::read(world.repo.join("zeta.txt")).expect("the file Git did not write"),
        b"old zeta\n",
        "the file is put back where it was"
    );
    assert_eq!(
        std::fs::read(world.repo.join("f.txt")).expect("the file Git wrote"),
        b"f.txt feature\n",
        "what Git wrote stays written"
    );
    assert_eq!(
        aside_names(world.repo.as_path()),
        Vec::<String>::new(),
        "nothing is left at a private name"
    );
    remove(world.parent.as_path());
}

/// What:
///  A file whose permissions changed after it was judged is not replaced.
/// Why:
///  The identity the take compares holds the device,
/// the inode,
/// the size and the modification time,
/// none of which a
///  `chmod` changes,
/// so an executable bit the person set was lost without a word
///  (a cross-vendor review's finding on the round-one take).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const (changes, judged) = judge(world); chmod('f.txt', 0o755); await expect(installEntries(...)).rejects.toMatch(/f.txt/); expect(mode('f.txt')).toEqual(0o755);
/// ```
#[cfg(unix)]
#[test]
fn a_file_whose_mode_changed_after_the_judgement_is_not_replaced() {
    use std::os::unix::fs::PermissionsExt;
    let world: World = world("merge-recovery-mode-after-judging");
    let (changes, judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let location: PathBuf = world.repo.join("f.txt");
    std::fs::set_permissions(location.as_path(), std::fs::Permissions::from_mode(0o755))
        .expect("the person's executable bit");
    let mut said: Vec<u8> = Vec::new();
    let refused = install_entries(
        &place(&world),
        world.repo.as_path(),
        changes.as_slice(),
        judgements.as_slice(),
        false,
        A_CONTROLS_MERGE,
        &mut said,
    )
    .expect_err("a path whose mode changed after the judgement is not the merge's to replace");
    assert!(refused.error.0.contains("f.txt"), "{}", refused.error.0);
    let mode: u32 = std::fs::metadata(location.as_path())
        .expect("the file")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o755, "the person's executable bit is kept");
    remove(world.parent.as_path());
}

/// What:
///  A file whose bytes were replaced with the same length,
///  and whose modification time was put back,
///  is not taken for the file that was judged.
/// Why:
///  The identity a take compares is the device,
/// the inode,
/// the size,
/// the mode and the modification time,
/// and a program that
///  writes the same length and restores the time leaves every one of them as it was,
/// so the merge replaced an
///  edit that was never hashed or copied
/// (a cross-vendor review of the third round,
/// the second of its timing findings).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const (changes, judged) = judge(world); writeSameLength('f.txt', edit); restoreMtime('f.txt', judged.identity.modified); await expect(installEntries(...)).rejects.toMatch(/f.txt/); expect(read('f.txt')).toEqual(edit);
/// ```
#[test]
fn a_file_whose_bytes_changed_with_its_length_and_time_kept_is_not_replaced() {
    use std::time::{
        Duration,
        SystemTime,
        UNIX_EPOCH,
    };
    let world: World = world("merge-recovery-same-length-edit");
    let (changes, judgements): (Vec<Change>, Vec<Judged>) = judged_changes(&world);
    let location: PathBuf = world.repo.join("f.txt");
    let judged: &Judged = &judgements[changes
        .iter()
        .position(|change| {
            return change.path == b"f.txt";
        })
        .expect("the path")];
    let kept_time = judged.identity.expect("the identity the judgement read");
    let modified: SystemTime = UNIX_EPOCH
        + Duration::new(
            u64::try_from(kept_time.modified_seconds).expect("a time"),
            u32::try_from(kept_time.modified_nanos).expect("nanoseconds"),
        );
    // The same length as "f.txt base\n", written over the same inode, and the modification time put back.
    std::fs::write(location.as_path(), b"f.txt EDIT\n").expect("the person's edit");
    std::fs::File::options()
        .write(true)
        .open(location.as_path())
        .expect("open")
        .set_modified(modified)
        .expect("the time put back");
    assert_eq!(
        file_identity(location.as_path()),
        Some(kept_time),
        "the identity is the one that was judged"
    );
    let mut said: Vec<u8> = Vec::new();
    let refused = install_entries(
        &place(&world),
        world.repo.as_path(),
        changes.as_slice(),
        judgements.as_slice(),
        false,
        A_CONTROLS_MERGE,
        &mut said,
    )
    .expect_err("bytes that changed are not the bytes that were judged");
    assert!(
        refused.error.0.contains("f.txt"),
        "the path is named: {}",
        refused.error.0
    );
    assert_eq!(
        std::fs::read(location.as_path()).expect("the file"),
        b"f.txt EDIT\n",
        "the person's bytes are kept"
    );
    remove(world.parent.as_path());
}

/// What:
///  A taken file whose bytes were replaced,
///  with its length and modification time kept,
///  is kept rather than removed with its private name.
/// Why:
///  A program that had the file open keeps its descriptor across the rename,
/// and a write through it lands in the file
///  that waits aside;
/// the identity cannot see that,
/// but the bytes can,
/// and a file whose bytes are not the ones that
///  were judged is kept and named
/// (a cross-vendor review of the third round,
/// the first and second of its timing findings).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const take = takeAside(file); writeSameLength(take.aside, edit); restoreMtime(take.aside, take.taken.modified); const left = finishAsides([take], sink); expect(exists(take.aside)).toBe(true); expect(sink).toContain(take.aside);
/// ```
#[test]
fn a_taken_file_whose_bytes_changed_is_kept_and_named() {
    use std::time::{
        Duration,
        SystemTime,
        UNIX_EPOCH,
    };
    let world: World = world("merge-recovery-same-length-aside");
    let location: PathBuf = world.repo.join("f.txt");
    let expected: String = git_text(world.repo.as_path(), &["hash-object", "--", "f.txt"])
        .trim()
        .to_owned();
    let aside: PathBuf = stranded_name(location.as_path(), A_CONTROLS_MERGE);
    std::fs::rename(location.as_path(), aside.as_path()).expect("a take");
    let taken = file_identity(aside.as_path()).expect("the identity the take checked");
    // A write through the descriptor a program kept: the same length, and the time put back.
    std::fs::write(aside.as_path(), b"f.txt EDIT\n").expect("the writer's edit");
    let modified: SystemTime = UNIX_EPOCH
        + Duration::new(
            u64::try_from(taken.modified_seconds).expect("a time"),
            u32::try_from(taken.modified_nanos).expect("nanoseconds"),
        );
    std::fs::File::options()
        .write(true)
        .open(aside.as_path())
        .expect("open")
        .set_modified(modified)
        .expect("the time put back");
    assert_eq!(
        file_identity(aside.as_path()),
        Some(taken),
        "the identity is the one that was judged"
    );
    let entries: Vec<TakenOnDisk> = vec![TakenOnDisk {
        location: location.clone(),
        path: b"f.txt".to_vec(),
        aside: aside.clone(),
        taken,
        expected,
    }];
    let mut said: Vec<u8> = Vec::new();
    finish_asides(&place(&world), world.repo.as_path(), &entries, &mut said)
        .expect("the asides are finished");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(aside.exists(), "a file whose bytes changed is kept");
    assert!(
        text.contains(aside.to_string_lossy().as_ref()),
        "the private name is said: {text}"
    );
    assert_eq!(
        std::fs::read(aside.as_path()).expect("the writer's edit"),
        b"f.txt EDIT\n",
        "the edit is not lost"
    );
    remove(world.parent.as_path());
}
