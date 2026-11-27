//! What:
//! Round 6 tests of the recovery of journals that never became ready and of the ready record's payload set.
//! Why:
//! Fifth review 5b2 findings 3,
//! 4 and 5:
//! unready cleanup must not delete a nonredundant payload or make a durable copy
//! disappear before its live equivalent is durable,
//! and the ready record must name exactly the payload the indexes imply.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('recovery of unready journals', () => { it(...) })
//! ```

use std::fs;
use std::path::{
    Path,
    PathBuf,
};

use crate::crates::LockSnapshot;
use crate::journal::{
    begin,
    recover,
};
use crate::journal_recover::read_ready;
use crate::journal_support::{
    journal_dirs,
    plant,
};
use crate::manifest_edit::ManifestEdit;
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/r/one";

/// A snapshot of the lockfile of the case.
fn snapshot(source: &str) -> LockSnapshot {
    return LockSnapshot {
        crate_dir: CRATE_DIR.to_string(),
        source: source.to_string(),
    };
}

/// A repository with one crate whose lockfile and manifest hold the given text.
fn repository(
    lock: &str,
    manifest: &str,
) -> tempfile::TempDir {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), lock).expect("lock");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.toml"), manifest).expect("manifest");
    return root;
}

/// The manifest edit of the case.
fn edit(
    root: &Path,
    before: &str,
) -> ManifestEdit {
    return ManifestEdit {
        crate_dir: CRATE_DIR.to_string(),
        path: root.join(CRATE_DIR).join("Cargo.toml"),
        before: before.to_string(),
        produced: None,
    };
}

/// Recovers with a policy that refuses everything it is asked about.
fn run_recovery(root: &Path) -> Vec<String> {
    return recover(
        root,
        &|_before: &[crate::crates::LockSnapshot], _live: &[crate::crates::LockSnapshot]| {
            return vec!["refused".to_string()];
        },
    );
}

/// A journal that crashed while its manifests were being journaled:
/// snapshots written,
/// no ready record.
/// `manifest` is the path the index names,
/// which is the crate's own `Cargo.toml` except where a case needs another name.
fn unready_journal_with_manifest(
    root: &Path,
    lock: &str,
    manifest_before: &str,
    manifest: &Path,
) -> PathBuf {
    let transaction = begin(root, &[snapshot(lock)], true).expect("begin");
    drop(transaction);
    let journal = journal_dirs(root).into_iter().next().expect("a journal");
    fs::write(journal.join("m0.toml"), manifest_before).expect("manifest snapshot");
    fs::write(
        journal.join("manifests.txt"),
        format!(
            "v1\n{CRATE_DIR}\t{}\nend 1\n",
            crate::journal_transaction::encode_path(manifest)
        ),
    )
    .expect("manifest index");
    return journal;
}

/// Round 6 (5b2 3):
/// equal lockfiles do not make a manifest snapshot redundant.
#[test]
fn an_unready_journal_whose_manifest_differs_from_its_snapshot_is_kept() {
    let root = repository("same lock", "manifest after");
    let journal = unready_journal_with_manifest(
        root.path(),
        "same lock",
        "manifest before",
        root.path().join(CRATE_DIR).join("Cargo.toml").as_path(),
    );

    let messages = run_recovery(root.path());

    assert!(
        journal.join("m0.toml").exists(),
        "the only copy of the manifest was deleted: {messages:?}"
    );
}

/// Round 6 (5b2 3):
/// a journal that lost its index but still holds payload is not empty.
#[test]
fn an_unready_journal_that_lost_its_index_but_holds_snapshots_is_kept() {
    let root = repository("same lock", "manifest");
    let journal = unready_journal_with_manifest(
        root.path(),
        "same lock",
        "manifest",
        root.path().join(CRATE_DIR).join("Cargo.toml").as_path(),
    );
    fs::remove_file(journal.join("index.txt")).expect("lose the index");

    let messages = run_recovery(root.path());

    assert!(
        journal.join("0.lock").exists(),
        "a snapshot was deleted without an index to judge it: {messages:?}"
    );
}

/// Round 6 (5b2 4):
/// a redundant journal is deleted only after the live files that make it redundant are durable.
#[test]
fn an_unready_journal_is_kept_when_the_live_files_cannot_be_made_durable() {
    let root = repository("same lock", "manifest");
    let journal = unready_journal_with_manifest(
        root.path(),
        "same lock",
        "manifest",
        root.path().join(CRATE_DIR).join("Cargo.toml").as_path(),
    );
    crate::failpoints::arm(crate::failpoints::Point::RecoverDurability);

    let messages = run_recovery(root.path());

    // The fault was reached by this state (an unrelated earlier refusal would leave it armed), and the message names it.
    assert!(
        !crate::failpoints::fires(crate::failpoints::Point::RecoverDurability),
        "the durability barrier was never reached: {messages:?}"
    );
    assert!(
        messages.iter().any(|message| {
            return message.contains("could not be made durable");
        }),
        "{messages:?}"
    );
    assert!(
        journal.exists(),
        "the durable copy was deleted before the live files were durable: {messages:?}"
    );

    // Control: the same state without the fault completes, so the retention above is the barrier's and not a refusal of the state.
    let again = run_recovery(root.path());
    assert!(
        !journal.exists(),
        "the redundant journal was not removed once the barrier succeeded: {again:?}"
    );
}

/// Issue 6550:
/// the live manifest of an unready journal is made durable by the path it really has,
/// so a manifest whose name is not UTF-8 keeps its journal when that barrier fails.
/// A recovery that carries the live paths as text asks for a barrier on a name no file has instead:
/// the file there is skipped as absent,
/// or a different file is synced where a directory with the name that text spells exists,
/// and the journal then goes while the live manifest was never made durable,
/// which is what the extra directory below makes this case report.
#[test]
fn an_unready_journal_of_a_manifest_that_is_not_utf8_is_kept_when_that_manifest_cannot_be_made_durable()
 {
    use std::os::unix::ffi::OsStringExt;
    // What:    `in_a_fixture_root_taking_any_name` answers whether this half may make the odd name here, and where the
    //  file system refuses it, starts this same test again inside a user and mount namespace with a tmpfs at the
    //  scratch root.
    // Why: the dataset behind this build folder refuses a name that is not UTF-8 (issue 6550).
    if !crate::fixture_root_taking_any_name::in_a_fixture_root_taking_any_name(
        crate::test_scratch::scratch_root().as_path(),
    ) {
        return;
    }
    // What: the bytes of the directory this case's manifest lives in, the last of which starts no character.
    // Why: the manifest this case is about is the one whose name is not text.
    let odd_name: &[u8] = b"odd-\xff-dir";
    let root = repository("same lock", "manifest");
    // What: `from_vec` builds an operating-system name out of raw bytes.
    // Why: `Path::join` with a `&str` could not spell this directory.
    let odd: PathBuf = root
        .path()
        .join(std::ffi::OsString::from_vec(odd_name.to_vec()));
    fs::create_dir_all(&odd).expect("a directory whose name is not UTF-8");
    let manifest: PathBuf = odd.join("Cargo.toml");
    fs::write(&manifest, "manifest").expect("the live manifest");
    // What: the same directory under the name a lossy conversion of those bytes spells, from that same conversion.
    // Why: a recovery that carried the manifest as text would sync this other file and call the barrier done,
    //  so the case has to hold one for that recovery to reach its own wrong answer here.
    let mangled: PathBuf = root
        .path()
        .join(String::from_utf8_lossy(odd_name).into_owned());
    fs::create_dir_all(&mangled).expect("the directory the text of that name spells");
    fs::write(mangled.join("Cargo.toml"), "manifest").expect("the file there");
    let journal =
        unready_journal_with_manifest(root.path(), "same lock", "manifest", manifest.as_path());
    crate::failpoints::arm_output_file_sync_of(manifest.as_path());

    let messages = run_recovery(root.path());

    // The barrier is armed for the manifest's own bytes, so a recovery that reached it disarmed it.
    let (file_armed, directory_armed) = crate::failpoints::path_targets_armed();
    assert!(
        !file_armed && !directory_armed,
        "the live manifest's own barrier was never reached, so the recovery asked for a name this file does not have: \
         {messages:?}"
    );
    assert!(
        messages.iter().any(|message| {
            return message.contains("could not be made durable");
        }),
        "{messages:?}"
    );
    assert!(
        journal.exists(),
        "the durable copy was deleted before the live manifest was durable: {messages:?}"
    );

    // Control: the same state without the fault completes, so the retention above is the barrier's and not a refusal of the state.
    let again = run_recovery(root.path());
    assert!(
        !journal.exists(),
        "the redundant journal was not removed once the barrier succeeded: {again:?}"
    );
}

/// A planted ready journal with one manifest,
/// and the text of its ready record.
fn ready_journal(root: &Path) -> (PathBuf, String) {
    let journal = plant(root, &[snapshot("lock")], &[edit(root, "manifest before")]);
    let ready = fs::read_to_string(journal.join("ready")).expect("ready record");
    return (journal, ready);
}

/// A planted ready journal whose live lockfile has changed since the snapshot,
/// so that a recovery that accepted the journal would
/// restore the snapshot text.
fn ready_journal_of_a_changed_lock(root: &Path) -> (PathBuf, String) {
    let (journal, ready) = ready_journal(root);
    fs::write(
        root.join(CRATE_DIR).join("Cargo.lock"),
        "changed by the killed run",
    )
    .expect("the run changed the lock");
    return (journal, ready);
}

/// The live lockfile of the case.
fn live_lock(root: &Path) -> String {
    return fs::read_to_string(root.join(CRATE_DIR).join("Cargo.lock")).expect("the live lock");
}

/// Real recovery of a journal whose ready record is invalid (certification,
/// second pass):
/// the journal is kept as it is,
/// the live
/// files are not touched,
/// and the message says it is damaged.
/// The same journal with a valid record is restored,
/// which shows that
/// the rejection and not some other refusal kept it.
fn assert_recovery_keeps_a_damaged_ready_journal(
    root: &Path,
    journal: &Path,
    case: &str,
) {
    let messages = run_recovery(root);
    assert!(
        messages.iter().any(|message| {
            return message.contains("damaged");
        }),
        "{case}: recovery did not call the journal damaged: {messages:?}"
    );
    assert!(
        journal.exists(),
        "{case}: the journal of the damaged record was deleted: {messages:?}"
    );
    assert_eq!(
        live_lock(root),
        "changed by the killed run",
        "{case}: recovery acted on a journal whose record is invalid"
    );
    // The edited manifest is unchanged too (sixth pass, group 8): a recovery that restores it before validating the ready record,
    // then rejects the record, would split the pair while the assertions above still hold.
    assert_eq!(
        fs::read_to_string(root.join(CRATE_DIR).join("Cargo.toml")).expect("manifest"),
        "manifest",
        "{case}: recovery acted on the manifest of a journal whose record is invalid"
    );
}

/// Rewrites the ready record with the given file lines (name and length) and the matching end line.
fn write_ready(
    journal: &Path,
    lines: &[String],
) {
    let mut text = String::from("v1\nmanifests 1\n");
    for line in lines {
        text.push_str(line);
        text.push('\n');
    }
    text.push_str(&format!("end {}\n", lines.len()));
    fs::write(journal.join("ready"), text).expect("write ready");
}

/// The file lines of a ready record.
fn file_lines(ready: &str) -> Vec<String> {
    return ready
        .lines()
        .filter(|line| {
            return line.contains('\t');
        })
        .map(str::to_string)
        .collect();
}

/// Round 6 (5b2 5):
/// a ready record that leaves out a manifest snapshot is rejected,
/// so a truncated snapshot cannot pass unchecked.
#[test]
fn a_ready_record_that_omits_a_manifest_snapshot_is_rejected() {
    let root = repository("lock", "manifest");
    let (journal, ready) = ready_journal_of_a_changed_lock(root.path());
    let kept: Vec<String> = file_lines(&ready)
        .into_iter()
        .filter(|line| {
            return !line.starts_with("m0.toml");
        })
        .collect();
    write_ready(&journal, &kept);

    assert!(
        read_ready(&journal).is_err(),
        "an omitted manifest snapshot passed"
    );
    assert_recovery_keeps_a_damaged_ready_journal(
        root.path(),
        &journal,
        "an omitted manifest snapshot",
    );
}

/// Control of the three tests that follow:
/// a journal with a valid ready record and a changed live lockfile is restored by the same
/// recovery,
/// so a kept journal there is the rejection of the record.
#[test]
fn a_journal_with_a_valid_ready_record_is_restored_by_the_same_recovery() {
    let root = repository("lock", "manifest");
    let (journal, _ready) = ready_journal_of_a_changed_lock(root.path());

    let messages = run_recovery(root.path());

    assert_eq!(
        live_lock(root.path()),
        "lock",
        "the control did not restore the snapshot: {messages:?}"
    );
    assert!(
        !journal.exists(),
        "the control journal was not completed: {messages:?}"
    );
}

/// Round 6 (5b2 5):
/// a duplicate entry is rejected.
#[test]
fn a_ready_record_with_a_duplicate_entry_is_rejected() {
    let root = repository("lock", "manifest");
    let (journal, ready) = ready_journal_of_a_changed_lock(root.path());
    let mut lines = file_lines(&ready);
    lines.push(lines[0].clone());
    write_ready(&journal, &lines);

    assert!(read_ready(&journal).is_err(), "a duplicate entry passed");
    assert_recovery_keeps_a_damaged_ready_journal(root.path(), &journal, "a duplicate entry");
}

/// Round 6 (5b2 5):
/// an entry that is no part of the payload is rejected.
#[test]
fn a_ready_record_with_an_unexpected_entry_is_rejected() {
    let root = repository("lock", "manifest");
    let (journal, ready) = ready_journal_of_a_changed_lock(root.path());
    let mut lines = file_lines(&ready);
    let length = fs::metadata(journal.join("owner")).expect("owner").len();
    lines.push(format!("owner\t{length}"));
    write_ready(&journal, &lines);

    assert!(read_ready(&journal).is_err(), "an unexpected entry passed");
    assert_recovery_keeps_a_damaged_ready_journal(root.path(), &journal, "an unexpected entry");
}

/// Round 6 (5b 4):
/// the advice about exhausted temporary names never offers a copy-and-delete as safe,
/// because a program that still has the file open keeps writing to the original.
#[test]
fn the_exhausted_names_advice_does_not_recommend_archiving_and_names_the_open_file_risk() {
    let message = crate::journal_replace::names_exhausted_message(Path::new("/x"), "Cargo.lock");

    assert!(!message.contains("archive"), "{message}");
    assert!(message.contains("open"), "{message}");
}

/// Acceptance row V5 (certification,
/// fourth pass):
/// the recovery's own output barriers are the real `make_durable` calls for the
/// lockfiles and the journaled manifests,
/// each failed on its own at its production operation.
/// The root is accepted by the policy and
/// kept (no replacement runs for it,
/// so no other durability supplies the evidence):
/// a failed file or directory sync is a mandatory
/// failure,
/// the fault is consumed,
/// and the journal stays;
/// the control without a fault completes and removes the journal.
#[test]
fn the_recovery_makes_a_kept_roots_lockfile_and_manifest_durable_through_the_real_barrier() {
    use crate::failpoints::{
        Point,
        arm,
        fires,
    };
    for fault in [Point::OutputFileSync, Point::OutputDirectorySync] {
        let root = repository("lock", "manifest after");
        let manifest_edit = edit(root.path(), "manifest before");
        let journal = plant(root.path(), &[snapshot("lock")], &[manifest_edit]);
        fs::write(
            root.path().join(CRATE_DIR).join("Cargo.lock"),
            "output the policy accepts",
        )
        .expect("the killed run's output");
        arm(fault);

        let report = crate::journal::recover_report(
            root.path(),
            &|_before: &[crate::crates::LockSnapshot], _live: &[crate::crates::LockSnapshot]| {
                return Vec::new();
            },
        );

        assert!(
            !fires(fault),
            "{fault:?}: the barrier was never reached at its production operation"
        );
        assert!(
            report.mandatory_failure,
            "{fault:?}: a failed output barrier of the recovery was not a mandatory failure: {:?}",
            report.messages
        );
        assert!(
            journal.exists(),
            "{fault:?}: the journal was removed after a failed output barrier"
        );
        assert_eq!(live_lock(root.path()), "output the policy accepts");
    }
    // The control: no fault.
    let root = repository("lock", "manifest after");
    let journal = plant(
        root.path(),
        &[snapshot("lock")],
        &[edit(root.path(), "manifest before")],
    );
    fs::write(
        root.path().join(CRATE_DIR).join("Cargo.lock"),
        "output the policy accepts",
    )
    .expect("the killed run's output");
    let _ = crate::bounded_run::take_trace_for_test();

    let report = crate::journal::recover_report(
        root.path(),
        &|_before: &[crate::crates::LockSnapshot], _live: &[crate::crates::LockSnapshot]| {
            return Vec::new();
        },
    );

    assert!(
        !report.mandatory_failure && !journal.exists(),
        "the control did not complete: {:?}",
        report.messages
    );
    let events: Vec<String> = crate::bounded_run::take_trace_for_test();
    let durable: Vec<String> = events
        .iter()
        .filter(|event| {
            return event.starts_with("durable ");
        })
        .cloned()
        .collect();
    assert_eq!(
        durable,
        vec![
            "durable Cargo.lock".to_string(),
            "durable Cargo.toml".to_string()
        ],
        "the recovery did not make both outputs durable before completing"
    );
    // The recovery's order too (sixth pass, group 8): both output barriers, then the marker, its parent barrier, and removal.
    let ordered: Vec<String> = events
        .into_iter()
        .filter(|event| {
            return event.starts_with("durable ")
                || event.starts_with("marker ")
                || event.starts_with("cleanup ");
        })
        .collect();
    assert_eq!(
        ordered,
        vec![
            "durable Cargo.lock",
            "durable Cargo.toml",
            "marker renamed",
            "marker parent synced",
            "cleanup attempted"
        ],
        "the recovery did not finish in the order outputs, marker, marker barrier, removal: {ordered:?}"
    );
}

/// Acceptance row V5 (certification,
/// fourth pass):
/// the recovery of a journal that never became ready reports its mandatory failures
/// through `recover_report`:
/// a failed durability barrier of the live files is a mandatory failure,
/// and a failed removal after the
/// marker is a cleanup only.
#[test]
fn an_unready_journal_reports_its_mandatory_and_cleanup_failures_through_the_report() {
    use crate::failpoints::{
        Point,
        arm,
        fires,
    };
    for (fault, mandatory) in [
        (Point::RecoverDurability, true),
        (Point::OutputFileSync, true),
        (Point::OutputDirectorySync, true),
        (Point::CompletionSync, true),
        (Point::FinishCleanup, false),
    ] {
        let root = repository("same lock", "manifest");
        let journal = unready_journal_with_manifest(
            root.path(),
            "same lock",
            "manifest",
            root.path().join(CRATE_DIR).join("Cargo.toml").as_path(),
        );
        arm(fault);

        let report = crate::journal::recover_report(
            root.path(),
            &|_before: &[crate::crates::LockSnapshot], _live: &[crate::crates::LockSnapshot]| {
                return vec!["refused".to_string()];
            },
        );

        assert!(!fires(fault), "{fault:?}: the fault was never reached");
        assert_eq!(
            report.mandatory_failure, mandatory,
            "{fault:?}: {:?}",
            report.messages
        );
        assert_eq!(
            journal.exists(),
            mandatory && !matches!(fault, Point::CompletionSync),
            "{fault:?}: the journal is not in the state this failure leaves: {:?}",
            report.messages
        );
    }
}
