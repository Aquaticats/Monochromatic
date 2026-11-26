//! What:
//!  The ref that keeps a capture alive while a saved merge is finished:
//!  a record written after the owners were read,
//!  the mode Git writes over a permission set after the capture,
//!  a special permission the path does not have,
//!  a capture another record still names,
//!  two finished records of one capture,
//!  and the deletion a lock blocked for the next call.
//! Why:
//!  Nothing orders two commands' recoveries,
//!  so the last one to finish drops the ref and a command that cannot drop it leaves the next one that job.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(refs()).toEqual([]); expect(refExists(capture)).toBe(true);
//! ```

use super::super::*;
use super::fixtures::{
    World,
    an_earlier_releases_name,
    dead_owner,
    half_carried,
    place,
    world,
};
use super::readings::the_mode_git_checks_out;
use crate::merge_isolation_journal::{
    intent_path,
    parse_merge_intent,
    write_merge_intent,
};
use crate::test_support::{
    REAL_GIT,
    executable,
    git,
    git_text,
    remove,
};
use crate::transaction_owner::current_transaction_owner;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  A merge that wrote its record after the owners were read keeps its copy in the merge's own hands.
/// Why:
///  The sweep of kept copies skips the copies of a merge that still runs,
/// and the keys it skipped were read before this copy was recorded:
/// a
///  merge that started in between would have its copy put back under the path it cleared,
/// which makes its own checkout fail.
/// The record is read again beside the copy it names,
/// so a merge that appeared after the scan still owns it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hooks.set('recovery-owners-planned', () => { record(live); write(aside); keepCopy(aside); rm('f.txt'); }); await recover(...); expect(exists('f.txt')).toBe(false);
/// ```
#[test]
fn a_merge_that_wrote_its_record_after_the_owners_were_read_keeps_its_copy() {
    let world: World = world("merge-recovery-live-record-after-owner-scan");
    let id: &str = "64646464-0000-4000-8000-000000000064";
    let repository = world.repo.clone();
    let git_directory = world.git_dir.clone();
    let old_oid = world.old_oid.clone();
    let new_oid = world.new_oid.clone();
    crate::service_fault::hooks::set(
        "recovery-owners-planned",
        std::rc::Rc::new(move || {
            let owner =
                current_transaction_owner(id, "2026-10-29T00:00:00.000Z").expect("the live owner");
            let intent: MergeIntent = MergeIntent {
                owner,
                repository_root: repository.clone(),
                target_ref: Some(String::from("refs/heads/main")),
                old_oid: old_oid.clone(),
                new_oid: new_oid.clone(),
                side_ref: None,
                carried: None,
            };
            write_merge_intent(git_directory.as_path(), &intent).expect("the new live record");
            let location = repository.join("f.txt");
            let aside = crate::merge_isolation_take_aside_names::aside_name(location.as_path(), id);
            std::fs::write(aside.as_path(), b"the live merge's copy\n")
                .expect("the live merge's copy");
            crate::merge_isolation_kept_copies::keep_the_copy(
                git_directory.as_path(),
                location.as_path(),
                aside.as_path(),
            );
            std::fs::remove_file(location.as_path()).expect("the live merge cleared the path");
        }),
    );
    let attempted =
        recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new());
    crate::service_fault::hooks::clear();
    attempted.expect("recovery");
    assert!(
        !world.repo.join("f.txt").exists(),
        "a command beside the live merge put its copy back under the path the merge cleared"
    );
    remove(world.parent.as_path());
}

/// What:
///  A saved merge whose file carries a permission the person set after the capture ends at the mode Git writes.
/// Why:
///  The merge's install writes the merge's own file as Git's own update writes it:
///  created with `0666` and narrowed by the process's umask,
///  so a `chmod 0600` made after the capture is not put back.
///  Real Git writes such a file the same way,
///  because it records the executable bit alone
///  (owner's decision of 2026-11-12T12:26:24-05:00,
///  "As real Git,
///  both cases";
///  `SPEC.md`,
///  "Merges and commits never fail").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// capture(); chmod('f.txt', 0o600); record(sideRef); await recover(...); expect(mode('f.txt')).toBe(gitWrites('f.txt')); expect(read('f.txt')).toContain("the person's line");
/// ```
#[test]
fn a_saved_merge_writes_the_mode_git_writes_over_a_permission_set_after_the_capture() {
    let world: World = world("merge-recovery-saved-merge-mode");
    let id: &str = "62626262-0000-4000-8000-000000000062";
    let side_ref: &str = "refs/cli-git/kept/62";
    std::fs::write(world.repo.join("f.txt"), b"f.txt base\nthe person's line\n")
        .expect("the person's edit");
    let context = crate::test_support::git_context();
    let capture = crate::merge_isolation_carry::keep_capture(
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
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(
        world.repo.join("f.txt"),
        std::fs::Permissions::from_mode(0o600),
    )
    .expect("the newer chmod");
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
    let _record: PathBuf =
        write_merge_intent(world.git_dir.as_path(), &intent).expect("the record");
    let outcomes =
        recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new())
            .expect("recovery");
    let mode = std::fs::metadata(world.repo.join("f.txt"))
        .expect("metadata")
        .permissions()
        .mode()
        & 0o777;
    let after = std::fs::read(world.repo.join("f.txt")).expect("f.txt");
    assert_eq!(
        outcomes[0].action,
        MergeRecoveryAction::Completed,
        "the merge is made over the person's own bytes"
    );
    assert!(
        String::from_utf8_lossy(after.as_slice()).contains("the person's line"),
        "their line is carried"
    );
    assert_eq!(
        mode,
        the_mode_git_checks_out(&world, "f.txt"),
        "the file ends at the mode Git's own checkout writes, not the one set after the capture: f.txt holds {:?}",
        String::from_utf8_lossy(after.as_slice())
    );
    remove(world.parent.as_path());
}

/// What:
///  A copy that is the only entry carrying a special permission bit stays.
/// Why:
///  The two entries hold one run of bytes and differ in a bit the person can set:
/// the copy is the only one with
///  setuid,
/// so removing it loses that permission while the line says nothing of theirs was lost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write('f.txt', bytes, 0o755); write(aside, bytes, 0o4755); keepCopy(aside); await recover(...); expect(exists(aside)).toBe(true);
/// ```
#[test]
fn a_copy_with_a_special_permission_the_path_does_not_have_stays() {
    let world: World = world("merge-recovery-copy-special-mode");
    let location: PathBuf = world.repo.join("f.txt");
    let aside: PathBuf = an_earlier_releases_name(location.as_path());
    std::fs::write(location.as_path(), b"same uncommitted bytes\n").expect("the location");
    std::fs::write(aside.as_path(), b"same uncommitted bytes\n").expect("the copy");
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(location.as_path(), std::fs::Permissions::from_mode(0o755))
        .expect("the location mode");
    std::fs::set_permissions(aside.as_path(), std::fs::Permissions::from_mode(0o4755))
        .expect("the copy's setuid mode");
    crate::merge_isolation_kept_copies::keep_the_copy(
        world.git_dir.as_path(),
        location.as_path(),
        aside.as_path(),
    );
    recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new())
        .expect("recovery");
    let mode = std::fs::metadata(location.as_path())
        .expect("metadata")
        .permissions()
        .mode();
    assert!(
        aside.exists(),
        "the only entry carrying the setuid bit was removed; the path's mode is {mode:o}"
    );
    remove(world.parent.as_path());
}

/// What:
///  A capture that another record still names keeps its ref when the first of two records is discarded.
/// Why:
///  A merge that saves its result for a later finish writes a second record beside the first,
/// and both keep one
///  capture (`save_for_a_later_finish`).
/// The first record is discarded by the next command while the second waits for
///  its own finish,
/// and dropping the shared ref there left the person's bytes reachable by nothing:
/// `git gc` collects a
///  commit nothing points at,
/// and the wait may last until the next command after the lock frees
/// (a qwen4-max-preview audit of the saved merge,
/// its second finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// capture(); record({ sideRef: undefined, carried }); record({ sideRef, target: 'refs/heads/other', carried }); await recover(...); expect(carried.reference).toBe('refs/cli-git/carried/...'); expect(catFile(capture.commit)).toBe('commit');
/// ```
#[test]
fn a_capture_another_record_still_names_keeps_its_ref() {
    let world: World = world("merge-recovery-shared-capture");
    let first_id: &str = "11111111-0000-4000-8000-000000000071";
    let saved_id: &str = "22222222-0000-4000-8000-000000000072";
    let saved_ref: &str = "refs/cli-git/kept/72";
    std::fs::write(world.repo.join("f.txt"), b"f.txt base\nthe person's line\n")
        .expect("the person's edit");
    let context = crate::test_support::git_context();
    let capture = crate::merge_isolation_carry::keep_capture(
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
    let reference: String = capture
        .reference
        .clone()
        .expect("the ref that keeps the capture");
    let first: MergeIntent = MergeIntent {
        owner: dead_owner(first_id),
        repository_root: world.repo.clone(),
        target_ref: Some(String::from("refs/heads/main")),
        old_oid: world.old_oid.clone(),
        new_oid: world.new_oid.clone(),
        side_ref: None,
        carried: Some(capture.clone()),
    };
    let first_record: PathBuf =
        write_merge_intent(world.git_dir.as_path(), &first).expect("the first record");
    git(
        world.repo.as_path(),
        &["update-ref", saved_ref, world.new_oid.as_str()],
    );
    git(
        world.repo.as_path(),
        &["update-ref", "refs/heads/other", world.old_oid.as_str()],
    );
    // The save's record names a branch this worktree does not have checked out, so its own settle keeps it for the command that
    //  checks that branch out: the capture's ref must outlive the first record.
    let saved: MergeIntent = MergeIntent {
        owner: dead_owner(saved_id),
        repository_root: world.repo.clone(),
        target_ref: Some(String::from("refs/heads/other")),
        old_oid: world.old_oid.clone(),
        new_oid: world.new_oid.clone(),
        side_ref: Some(String::from(saved_ref)),
        carried: Some(capture.clone()),
    };
    let saved_record: PathBuf =
        write_merge_intent(world.git_dir.as_path(), &saved).expect("the save's record");
    let _outcomes =
        recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new())
            .expect("the command must not stop");
    assert!(
        !first_record.exists(),
        "the merge's first record is settled"
    );
    assert!(
        saved_record.exists(),
        "the save's record waits for the branch it is for"
    );
    assert_eq!(
        git_text(
            world.repo.as_path(),
            &["rev-parse", "--verify", reference.as_str()]
        )
        .trim(),
        capture.commit,
        "the ref that another record still names keeps the person's capture"
    );
    assert_eq!(
        git_text(
            world.repo.as_path(),
            &["cat-file", "-t", capture.commit.as_str()]
        )
        .trim(),
        "commit",
        "the capture's commit is still there"
    );
    remove(world.parent.as_path());
}

/// What:
///  Two commands that finish the two records of one capture,
///  each deciding before either removes its record,
///  leave no ref behind.
/// Why:
///  A saved merge writes two records that name one capture,
/// and the ref that keeps the capture is dropped by whichever command finishes last.
/// Nothing orders two commands' recoveries,
/// so both can look at the record set before either removes its record:
/// if each answers "the other one still needs it",
/// both remove their records and the ref is left with nobody to drop it
/// (the second review of this branch,
/// its second finding,
/// which is the price of the fix for the alpha.14 audit's third finding).
/// Each finish writes the sibling that says its carry has finished before it decides
/// (`merge_isolation_journal.rs`,
/// `mark_carry_finished`),
/// so a record whose carry is finished is no claim on the ref,
/// and a deletion of a name already gone is what the deletion wanted.
/// The control takes both decisions before it acts on either,
/// which is the interleaving,
/// and then removes both records as the two commands would.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// settled(a); settled(b);
/// const first = drop(capture, a); const second = drop(capture, b); rm(a); rm(b);
/// expect(refs.filter((one) => one.startsWith('refs/cli-git/carried/'))).toEqual([]);
/// ```
#[test]
fn two_finished_records_of_one_capture_leave_no_ref_behind() {
    let world: World = world("merge-recovery-two-finished-records");
    std::fs::write(world.repo.join("f.txt"), b"f.txt base\nthe person's line\n")
        .expect("the person's edit");
    let context = crate::test_support::git_context();
    let capture = crate::merge_isolation_carry::keep_capture(
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
    let reference: String = capture
        .reference
        .clone()
        .expect("the ref that keeps the capture");
    let mut records: Vec<PathBuf> = Vec::new();
    for id in [
        "11111111-0000-4000-8000-000000000081",
        "22222222-0000-4000-8000-000000000082",
    ] {
        let intent: MergeIntent = MergeIntent {
            owner: dead_owner(id),
            repository_root: world.repo.clone(),
            target_ref: Some(String::from("refs/heads/main")),
            old_oid: world.old_oid.clone(),
            new_oid: world.new_oid.clone(),
            side_ref: None,
            carried: Some(capture.clone()),
        };
        records.push(write_merge_intent(world.git_dir.as_path(), &intent).expect("a record"));
    }
    // Each command's carry has finished, which is what it writes beside its record before it decides.
    for record in &records {
        std::fs::write(record.with_extension("settled"), b"")
            .expect("the marker of a finished carry");
    }
    // The interleaving: both commands decide before either removes its record, so each drop is asked while both records are
    //  still there, which is the ordering the two commands can realise.
    let first: bool = crate::merge_isolation_carry::drop_capture_ref(
        &context,
        world.git_dir.as_path(),
        world.repo.as_path(),
        &capture,
        Some(records[0].as_path()),
    );
    assert!(first, "the first finish may go on");
    let second: bool = crate::merge_isolation_carry::drop_capture_ref(
        &context,
        world.git_dir.as_path(),
        world.repo.as_path(),
        &capture,
        Some(records[1].as_path()),
    );
    assert!(
        second,
        "a ref that is already gone is what the second deletion wanted"
    );
    for record in &records {
        std::fs::remove_file(record.as_path()).expect("the record goes");
    }
    assert_eq!(
        git_text(
            world.repo.as_path(),
            &[
                "for-each-ref",
                "--format=%(refname)",
                crate::merge_isolation_carry::CARRIED_REF_PREFIX
            ]
        )
        .trim(),
        "",
        "no ref is left once both records are gone (the ref that was {reference})"
    );
    remove(world.parent.as_path());
}

/// A stand-in for real Git that refuses the first deletion of a capture ref,
/// as another command's lock on that ref makes Git refuse it,
/// and answers everything else itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shim = (parent) => executable('locked-capture-deletion-git', `#!/bin/sh ... : > .git/$4.lock; realGit "$@"; rm -f .git/$4.lock ...`);
/// ```
fn git_that_locks_the_first_capture_deletion(parent: &Path) -> PathBuf {
    let shim: PathBuf = parent.join("locked-capture-deletion-git");
    let marker: PathBuf = parent.join("the-deletion-was-locked");
    let log: PathBuf = parent.join("shim-invocations.txt");
    let script: String = format!(
        "#!/bin/sh\necho \"$*\" >> '{log}'\ndeletes=no\nref=\nlast=\nfor argument in \"$@\"; do\n  if [ \"$argument\" = update-ref ]; then deletes=yes; fi\n  if [ \"$last\" = -d ]; then ref=\"$argument\"; fi\n  last=\"$argument\"\ndone\nif [ \"$deletes\" = yes ] && [ -n \"$ref\" ]; then\n  case \"$ref\" in\n    refs/cli-git/carried/*)\n      if [ ! -e '{marker}' ]; then\n        : > '{marker}'\n        lock=\"$({real} rev-parse --absolute-git-dir)/$ref.lock\"\n        : > \"$lock\"\n        {real} \"$@\"\n        status=$?\n        rm -f \"$lock\"\n        exit $status\n      fi\n      ;;\n  esac\nfi\nexec {real} \"$@\"\n",
        marker = marker.display(),
        log = log.display(),
        real = REAL_GIT
    );
    executable(shim.as_path(), script.as_bytes());
    return shim;
}

/// What:
///  A capture ref whose deletion another command's lock blocked is deleted by the next call,
///  because the merge's record,
///  which is the only thing that still names it,
///  stayed.
/// Why:
///  Removing the record before the ref left a name nothing retried and a capture nothing needed,
/// and a
///  person had to clean it up by hand (a cross-vendor audit of alpha.12,
/// "Capture-ref cleanup is never retried after a
///  transient failure").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const first = await dropTheRecordsRefsThenTheRecord(placeWithShim, ...); expect(first).toBe(false); expect(refExists(ref)).toBe(true); const second = await dropTheRecordsRefsThenTheRecord(place, ...); expect(second).toBe(true); expect(refExists(ref)).toBe(false);
/// ```
#[test]
fn a_capture_ref_a_lock_blocked_is_deleted_by_the_next_call() {
    let world: World = world("merge-recovery-capture-ref-locked");
    let id: &str = "2b2b2b2b-0000-4000-8000-00000000002b";
    let capture: crate::merge_isolation_carry_parts::Capture = half_carried(&world, id);
    let reference: String = capture.reference.clone().expect("the capture's ref");
    let record: PathBuf = intent_path(world.git_dir.as_path(), id);
    let bytes: Vec<u8> = std::fs::read(record.as_path()).expect("the record");
    let intent: MergeIntent = parse_merge_intent(bytes.as_slice()).expect("a record");
    let shim: PathBuf = git_that_locks_the_first_capture_deletion(world.parent.as_path());
    let mut locked: RecoveryPlace = place(&world);
    locked.git.real_git = shim;
    let run_cleanup = |place: RecoveryPlace, kept_record: PathBuf, kept_intent: MergeIntent| {
        return crate::lazy_runtime::run_on_runtime(async move {
            return crate::merge_isolation_recovery_carry::drop_the_records_refs_then_the_record(
                &place,
                place.cwd.as_path(),
                kept_record.as_path(),
                &kept_intent,
                crate::merge_isolation_recovery_carry::SideRef::Drop,
            )
            .await
            .expect("the cleanup");
        });
    };
    let cleared: bool =
        run_cleanup(locked, record.clone(), intent.clone()).expect("the runtime ran the cleanup");
    assert!(
        !cleared,
        "the record stays while the ref it names could not go"
    );
    assert!(record.exists(), "the record is kept");
    assert_eq!(
        git_text(
            world.repo.as_path(),
            &[
                "for-each-ref",
                "--format=%(refname)",
                "refs/cli-git/carried"
            ]
        )
        .trim(),
        reference
    );
    let retried: bool = run_cleanup(place(&world), record.clone(), intent.clone())
        .expect("the runtime ran the retry");
    assert!(retried, "the next call removes both once the lock is gone");
    assert!(!record.exists(), "the record goes with the ref");
    assert_eq!(
        git_text(
            world.repo.as_path(),
            &[
                "for-each-ref",
                "--format=%(refname)",
                "refs/cli-git/carried"
            ]
        )
        .trim(),
        "",
        "no name is left"
    );
    remove(world.parent.as_path());
}
