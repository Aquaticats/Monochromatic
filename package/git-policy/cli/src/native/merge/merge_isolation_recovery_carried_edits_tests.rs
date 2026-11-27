//! What:
//!  Finishing a saved merge over the person's own edits:
//!  a branch another program moved away from,
//!  a private name a killed carry left,
//!  edits newer than the capture,
//!  carried onto the moved head,
//!  a save at the copy's path during the scan,
//!  and the ref a settle leaves naming the merge.
//! Why:
//!  The person's work in the paths a merge changes is what the carry exists for,
//!  so a settle either carries it or says where it kept it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(read('f.txt')).toContain(thePersonsLine); expect(refs()).toContain(sideRef);
//! ```

use super::super::*;
use super::fixtures::{
    World,
    an_earlier_releases_name,
    dead_owner,
    half_carried,
    place,
    record_with_a_capture,
    world,
};
use crate::merge_isolation_journal::{
    intent_path,
    write_merge_intent,
};
use crate::test_support::{
    git,
    git_text,
    remove,
};
use std::path::PathBuf;

/// What:
///  A branch another program moved away from the merge is settled,
///  and the person's captured edit comes back into its file.
/// Why:
///  `git merge-base --is-ancestor` says "no" with status 1 and nothing on standard error,
/// and the recovery read that from
///  the failure text,
/// which holds only what Git printed,
/// so "the merge is not in the branch" was taken for a failure to read:
///  the merge exited 2,
///  every later command in the worktree exited 2 with the same record,
/// the person's edit was left only on a capture ref nothing names,
///  and the only way forward was to remove the record by hand
///  (a cross-vendor audit of alpha.10,
/// its third finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const capture = keepCapture(captureWork(['f.txt'])); record(capture); checkout('f.txt'); commit('other.txt', 'another tool'); const outcomes = await recover(...); expect(outcomes[0].action).toBe('branch-moved'); expect(read('f.txt')).toEqual(personBytes); expect(exists(record)).toBe(false);
/// ```
#[test]
fn a_branch_another_program_moved_away_from_the_merge_is_not_a_refusal() {
    let world: World = world("merge-recovery-branch-moved-away");
    let id: &str = "2a2a2a2a-0000-4000-8000-0000000000a2";
    let person: &[u8] = b"f.txt base\nthe person's line\n";
    std::fs::write(world.repo.join("f.txt"), person).expect("the person's edit");
    let context = crate::test_support::git_context();
    let capture: crate::merge_isolation_carry_parts::Capture =
        crate::merge_isolation_carry_ref::keep_capture(
            &context,
            world.repo.as_path(),
            &crate::merge_isolation_carry::capture_work(
                &context,
                world.repo.as_path(),
                &[b"f.txt".to_vec()],
            )
            .expect("the capture"),
        )
        .expect("kept");
    let record: PathBuf = record_with_a_capture(&world, id, &capture);
    // The clear the merge makes before Git's update, and then a commit another program makes on the branch.
    git(
        world.repo.as_path(),
        &["checkout", "--quiet", "--", "f.txt"],
    );
    std::fs::write(world.repo.join("other.txt"), b"another tool's file\n")
        .expect("the other program's file");
    git(world.repo.as_path(), &["add", "--", "other.txt"]);
    git(
        world.repo.as_path(),
        &["commit", "--quiet", "--message=another tool"],
    );
    let head: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    assert_ne!(
        head.trim(),
        world.new_oid,
        "the branch no longer names the merge"
    );
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("the command must not stop");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(
        outcomes[0].action,
        MergeRecoveryAction::BranchMoved,
        "the move is settled, not refused: {text}"
    );
    assert_eq!(
        std::fs::read(world.repo.join("f.txt")).expect("f.txt"),
        person,
        "the person's edit is back in its file: {text}"
    );
    assert!(
        !record.exists(),
        "the record goes with the settled merge: {text}"
    );
    remove(world.parent.as_path());
}

/// What:
///  The name a killed carry wrote its bytes under is removed when the merge is finished,
///  so the worktree holds no half-applied result.
/// Why:
///  A kill between writing those bytes and renaming them into place leaves an untracked
///  `.cli-git-carry-*` file beside the path,
/// and nothing ended it:
/// later commands put the path back and exit 0,
/// and the copy stayed
///  for good (a cross-vendor audit of alpha.10,
/// its fifth finding).
///  The bytes are a temporary of the placing,
/// made again by the next carry and held in the object store either
///  way,
/// so the name goes with the merge that wrote it and no other merge's does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const capture = halfCarried(world); private = carryName('f.txt', id); write(private, 'made again'); await recover(...); expect(exists(private)).toBe(false); expect(exists(record)).toBe(false);
/// ```
#[test]
fn a_private_name_a_killed_carry_left_is_removed_when_the_merge_finishes() {
    let world: World = world("merge-recovery-carry-name-left");
    let id: &str = "2b2b2b2b-0000-4000-8000-0000000000b2";
    let _capture: crate::merge_isolation_carry_parts::Capture = half_carried(&world, id);
    let record: PathBuf = crate::merge_isolation_journal::intent_path(world.git_dir.as_path(), id);
    let left: PathBuf = world.repo.join(format!(
        ".cli-git-carry-{}-{}-{}-0-1",
        crate::merge_isolation_take_aside_names::name_key("f.txt"),
        crate::merge_isolation_take_aside_names::name_key(id),
        std::process::id()
    ));
    std::fs::write(left.as_path(), b"f.txt feature\nthe person's line\n")
        .expect("the bytes the killed placing had written");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("recovery");
    assert_eq!(
        outcomes[0].action,
        MergeRecoveryAction::AlreadyLanded,
        "the merge is the branch's"
    );
    assert!(
        !record.exists(),
        "the record goes once the merge is finished"
    );
    assert!(
        !left.exists(),
        "the name a killed carry left goes with the merge that wrote it"
    );
    assert_eq!(
        std::fs::read(world.repo.join("f.txt")).expect("f.txt"),
        b"f.txt feature\n",
        "the file holds the merge's own bytes, which nothing proves are not a save made after a kill"
    );
    remove(world.parent.as_path());
}

/// What:
///  A merge saved behind an index lock that never began is installed when the lock is gone,
///  even though the person's own edits stand in the paths it changes,
///  and their work is carried into the file.
/// Why:
///  A merge that waits out a held `index.lock` saves itself on a ref and exits 0,
/// and the next command must finish it.
///  Where the person has unsaved edits in the merge's paths,
/// the finish refused for ever:
/// every command exited 0
///  with "was not finished ... was interrupted while Git updated these files",
/// the branch never advanced,
/// and the only ways
///  offered were to discard the edit or to abandon the merge
/// (a cross-vendor audit of alpha.10,
/// its fourth finding).
///  The capture the record keeps holds the file's bytes and its mode,
/// so installing the merge over them
///  loses nothing,
/// and the carry writes them back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = world(); write('f.txt', personEdit); const capture = keepCapture(captureWork(['f.txt'])); updateRef('refs/cli-git/kept/9', world.newOid); record({ carried: capture, sideRef: 'refs/cli-git/kept/9' }); const outcomes = await recover(...); expect(outcomes[0].action).toBe('completed'); expect(read('f.txt')).toEqual(merged); expect(exists(record)).toBe(false); expect(refExists('refs/cli-git/kept/9')).toBe(false);
/// ```
#[test]
fn a_saved_merge_over_the_persons_kept_edits_is_installed_and_carries_them() {
    let world: World = world("merge-recovery-saved-behind-a-lock");
    let id: &str = "2c2c2c2c-0000-4000-8000-0000000000c2";
    let side_ref: &str = "refs/cli-git/kept/9";
    std::fs::write(world.repo.join("f.txt"), b"f.txt base\nthe person's line\n")
        .expect("the person's edit");
    let context = crate::test_support::git_context();
    let capture: crate::merge_isolation_carry_parts::Capture =
        crate::merge_isolation_carry_ref::keep_capture(
            &context,
            world.repo.as_path(),
            &crate::merge_isolation_carry::capture_work(
                &context,
                world.repo.as_path(),
                &[b"f.txt".to_vec()],
            )
            .expect("the capture"),
        )
        .expect("kept");
    git(
        world.repo.as_path(),
        &["update-ref", side_ref, world.new_oid.as_str()],
    );
    let intent: MergeIntent = MergeIntent {
        owner: dead_owner(id),
        repository_root: world.repo.clone(),
        target_ref: Some(String::from("refs/heads/main")),
        old_oid: world.old_oid.clone(),
        new_oid: world.new_oid.clone(),
        side_ref: Some(String::from(side_ref)),
        carried: Some(capture),
    };
    let record: PathBuf = write_merge_intent(world.git_dir.as_path(), &intent).expect("the record");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("the command must not stop");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(
        outcomes[0].action,
        MergeRecoveryAction::Completed,
        "the saved merge is finished: {text}"
    );
    assert_eq!(
        std::fs::read(world.repo.join("f.txt")).expect("f.txt"),
        b"f.txt feature\nthe person's line\n",
        "the merge's line and the person's line are both in the file: {text}"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "refs/heads/main"]),
        world.new_oid,
        "the branch names the merge"
    );
    assert!(
        !record.exists(),
        "the record goes once the merge landed: {text}"
    );
    let kept_refs: String = git_text(
        world.repo.as_path(),
        &["for-each-ref", "--format=%(refname)", "refs/cli-git"],
    );
    assert!(
        !kept_refs.contains(side_ref),
        "the ref that kept the merge goes with the record: {kept_refs} {text}"
    );
    remove(world.parent.as_path());
}

/// What:
///  The same,
/// for a path whose bytes are newer than the capture the record saved:
///  nothing is written over them,
///  the saved commit stays on its ref,
///  and the line says what to run.
/// Why:
///  The capture holds the file as the merge found it,
/// and the person may have saved again since.
///  Installing the
///  merge would take the file aside and remove it once the merge's content stood there,
/// so their newest bytes would
///  be gone;
/// the record settles instead,
/// the merge's commit stays on its ref,
/// and the line names that ref and the
///  `git merge` command that makes the merge on what the file holds now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const capture = keepCapture(captureWork(['f.txt'])); write('f.txt', newerEdit); updateRef('refs/cli-git/kept/9', newOid); record({ carried: capture, sideRef }); const outcomes = await recover(...); expect(outcomes[0].action).toBe('deferred'); expect(read('f.txt')).toEqual(newerEdit); expect(exists(record)).toBe(false); expect(refExists(sideRef)).toBe(true);
/// ```
#[test]
fn a_saved_merge_whose_paths_hold_edits_newer_than_its_capture_settles_and_names_the_ref() {
    let world: World = world("merge-recovery-saved-newer-edit");
    let id: &str = "2d2d2d2d-0000-4000-8000-0000000000d2";
    let side_ref: &str = "refs/cli-git/kept/10";
    let person: &[u8] = b"f.txt base\nthe person's line\ntheir later line\n";
    std::fs::write(world.repo.join("f.txt"), b"f.txt base\nthe person's line\n")
        .expect("the person's edit");
    let context = crate::test_support::git_context();
    let capture: crate::merge_isolation_carry_parts::Capture =
        crate::merge_isolation_carry_ref::keep_capture(
            &context,
            world.repo.as_path(),
            &crate::merge_isolation_carry::capture_work(
                &context,
                world.repo.as_path(),
                &[b"f.txt".to_vec()],
            )
            .expect("the capture"),
        )
        .expect("kept");
    // The save after the capture, which the merge cannot see.
    std::fs::write(world.repo.join("f.txt"), person).expect("the later save");
    git(
        world.repo.as_path(),
        &["update-ref", side_ref, world.new_oid.as_str()],
    );
    let intent: MergeIntent = MergeIntent {
        owner: dead_owner(id),
        repository_root: world.repo.clone(),
        target_ref: Some(String::from("refs/heads/main")),
        old_oid: world.old_oid.clone(),
        new_oid: world.new_oid.clone(),
        side_ref: Some(String::from(side_ref)),
        carried: Some(capture),
    };
    let record: PathBuf = write_merge_intent(world.git_dir.as_path(), &intent).expect("the record");
    let mut said: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect("the command must not stop");
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert_eq!(
        outcomes[0].action,
        MergeRecoveryAction::DeferredRefused,
        "the merge settles, it does not refuse: {text}"
    );
    assert_eq!(
        std::fs::read(world.repo.join("f.txt")).expect("f.txt"),
        person,
        "the newer save is untouched: {text}"
    );
    assert!(
        !record.exists(),
        "the record goes, so no later command meets it again: {text}"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "--verify", side_ref]),
        world.new_oid,
        "the saved merge commit stays on its ref: {text}"
    );
    assert!(
        text.contains(side_ref),
        "the line names the ref that keeps the merge: {text}"
    );
    assert!(
        text.contains(&format!("git merge {}", world.new_oid)),
        "the line says how to make the merge: {text}"
    );
    remove(world.parent.as_path());
}

/// What:
///  A merge that landed and was then built on carries the person's captured edit onto the moved head,
///  instead of reading the record it wrote before the head moved.
/// Why:
///  The reading of what Git's update left belongs to the instant of that update;
/// after another command commits on top of the merge,
///  every captured path holds that commit's file and no longer the entry the reading names,
/// so a carry that judged the moved head by the old reading left the person's line out of the file
///  and removed the record that named it.
/// The moved head is read here,
/// so the entry standing at the path is the one the carry judges.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const capture = halfCarried(world); writeLandedRecord(...); commit('later head'); await recover(...); expect(read('f.txt')).toContain("the person's line");
/// ```
#[test]
fn a_merge_built_on_after_it_landed_carries_the_persons_edit_onto_the_moved_head() {
    let world: World = world("merge-recovery-carry-onto-stale-landed");
    let id: &str = "61616161-0000-4000-8000-000000000061";
    let _capture = half_carried(&world, id);
    let landed =
        crate::merge_isolation_carry_landed::landed_of(world.repo.as_path(), &[b"f.txt".to_vec()]);
    crate::merge_isolation_carry_landed::write_landed_record(
        world.git_dir.as_path(),
        id,
        landed.as_slice(),
    );
    std::fs::write(
        world.repo.join("f.txt"),
        b"f.txt written by the later commit\n",
    )
    .expect("the later edit");
    git(world.repo.as_path(), &["add", "--", "f.txt"]);
    git(
        world.repo.as_path(),
        &["commit", "--quiet", "--message=later head"],
    );
    let record = intent_path(world.git_dir.as_path(), id);
    let outcomes =
        recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new())
            .expect("recovery");
    let after = std::fs::read(world.repo.join("f.txt")).expect("f.txt");
    assert!(
        String::from_utf8_lossy(after.as_slice()).contains("the person's line"),
        "the captured edit was neither carried nor left in a named copy: f.txt holds {after:?}, the record {} exists: {}",
        record.display(),
        record.exists()
    );
    assert_eq!(
        outcomes[0].action,
        MergeRecoveryAction::AlreadyLanded,
        "the merge landed and the moved head was carried onto"
    );
    remove(world.parent.as_path());
}

/// What:
///  A save at the copy's own path,
/// made after the bytes were compared,
/// keeps the copy.
/// Why:
///  The copy is removed because the file at its path holds the same bytes;
/// a save that arrives between that comparison and the
///  unlink leaves the copy the only name of the older uncommitted bytes,
/// so the last look before the removal asks the path again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write('f.txt', unique); write(aside, unique); keepCopy(aside); hooks.set('copy-scanned', () => write('f.txt', newer)); await recover(...); expect(exists(aside)).toBe(true);
/// ```
#[test]
fn a_save_at_the_copys_path_during_the_scan_keeps_the_copy() {
    let world: World = world("merge-recovery-copy-location-save");
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = an_earlier_releases_name(location.as_path());
    let unique: &[u8] = b"uncommitted bytes held only in these two entries\n";
    std::fs::write(location.as_path(), unique).expect("the location");
    std::fs::write(aside.as_path(), unique).expect("the copy");
    crate::merge_isolation_kept_copies::keep_the_copy(
        world.git_dir.as_path(),
        location.as_path(),
        aside.as_path(),
    );
    let saved: PathBuf = location.clone();
    crate::service_fault::hooks::set(
        "copy-scanned",
        std::rc::Rc::new(move || {
            std::fs::write(saved.as_path(), b"a newer save made during recovery\n")
                .expect("the concurrent save");
        }),
    );
    let attempted =
        recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new());
    crate::service_fault::hooks::clear();
    attempted.expect("recovery");
    let after = std::fs::read(location.as_path()).expect("the newer save");
    assert!(
        aside.exists(),
        "recovery deleted the only remaining name of the old uncommitted bytes, and the path holds {:?}",
        String::from_utf8_lossy(after.as_slice())
    );
    remove(world.parent.as_path());
}

/// What:
///  A saved merge settled over the person's newer edits leaves a ref that still names its commit.
/// Why:
///  The record goes and the line says the merge commit is kept on the side ref,
/// so a ref that another command repointed would leave the merge
///  commit reachable by nothing and the line false:
/// the commit is kept on a name of this command's own while the record still exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// capture(); write('f.txt', newer); updateRef(sideRef, newOid); record(sideRef); updateRef(sideRef, oldOid); await recover(...); expect(for-each-ref --contains newOid).not.toBe('');
/// ```
#[test]
fn settling_a_saved_merge_keeps_a_ref_that_names_the_merge() {
    let world: World = world("merge-recovery-saved-merge-repointed-ref");
    let id: &str = "63636363-0000-4000-8000-000000000063";
    let side_ref: &str = "refs/cli-git/kept/63";
    std::fs::write(world.repo.join("f.txt"), b"f.txt base\nthe person's line\n")
        .expect("the captured edit");
    let context = crate::test_support::git_context();
    let capture = crate::merge_isolation_carry_ref::keep_capture(
        &context,
        world.repo.as_path(),
        &crate::merge_isolation_carry::capture_work(
            &context,
            world.repo.as_path(),
            &[b"f.txt".to_vec()],
        )
        .expect("the capture"),
    )
    .expect("kept");
    std::fs::write(
        world.repo.join("f.txt"),
        b"a newer edit after the capture\n",
    )
    .expect("the newer edit");
    git(
        world.repo.as_path(),
        &["update-ref", side_ref, world.new_oid.as_str()],
    );
    let intent: MergeIntent = MergeIntent {
        owner: dead_owner(id),
        repository_root: world.repo.clone(),
        target_ref: Some(String::from("refs/heads/main")),
        old_oid: world.old_oid.clone(),
        new_oid: world.new_oid.clone(),
        side_ref: Some(String::from(side_ref)),
        carried: Some(capture),
    };
    let record = write_merge_intent(world.git_dir.as_path(), &intent).expect("the record");
    git(
        world.repo.as_path(),
        &["update-ref", side_ref, world.old_oid.as_str()],
    );
    let outcomes =
        recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new())
            .expect("recovery");
    let containing = git_text(
        world.repo.as_path(),
        &[
            "for-each-ref",
            "--format=%(refname)",
            "--contains",
            world.new_oid.as_str(),
        ],
    );
    assert!(
        !record.exists(),
        "the record goes, so no later command meets it again"
    );
    assert_eq!(
        outcomes[0].action,
        MergeRecoveryAction::DeferredRefused,
        "the merge is settled over the person's newer edit"
    );
    assert!(
        !containing.trim().is_empty(),
        "the saved merge commit {:?} has no ref after recovery removed its record",
        world.new_oid
    );
    remove(world.parent.as_path());
}
