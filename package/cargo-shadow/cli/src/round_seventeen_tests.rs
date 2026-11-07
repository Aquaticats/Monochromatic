//! What:
//! Round 17 tests:
//! the fixes asked for by the second certification pass (`cargo-shadow-cert2-sol-1.txt` to `-4.txt`).
//! Why:
//! The second pass found,
//! row by row,
//! assertions that a weaker outcome than the guarantee would also satisfy,
//! and bodies missing from
//! the packets;
//! each test here discriminates the guarantee from the weaker outcome named,
//! and the planted regression that turned the
//! old assertion red is in the contract's red-first list where one could be built.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('round seventeen', () => { it('a journal that lost its records is kept by recovery and refuses admission', ...) })
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use crate::crates::LockSnapshot;
use crate::journal::{begin, recover};
use crate::journal_recover::survivor_note;
use crate::journal_support::{journal_dirs, plant};
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/n/one";

/// The live text of the lockfile a killed run left.
const LEFT_BY_THE_KILLED_RUN: &str = "unchecked output of the killed run";

/// A snapshot of the lockfile of the case.
fn snapshot(source: &str) -> LockSnapshot {
    return LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: source.to_string() };
}

/// A disposable repository whose crate root holds the output of a killed run.
fn repository() -> tempfile::TempDir {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), LEFT_BY_THE_KILLED_RUN).expect("lock");
    return root;
}

/// A judge that refuses the live lockfile,
/// so that a recovery which goes ahead restores the snapshot.
fn refusing(_before: &[LockSnapshot], _live: &[LockSnapshot]) -> Vec<String> {
    return vec!["dep@1.0.1 refused".to_string()];
}

/// The text of the live lockfile.
fn live_lock(root: &Path) -> String {
    return fs::read_to_string(root.join(CRATE_DIR).join("Cargo.lock")).expect("lock");
}

/// Acceptance rows "V2,
/// amended c" and "V3,
/// V4" (certification,
/// second pass):
/// a nonempty journal that lost its containment record
/// (and,
/// in one case,
/// its ready record too) is a possible survivor for the real consumers,
/// not only for the classifier:
/// the recovery keeps the journal and its snapshots,
/// leaves the live output as it was and says that the record is missing,
/// and a new
/// transaction is refused for the same reason.
/// The control puts the empty record back,
/// after which the same recovery restores the snapshot,
/// so the refusal came from the loss.
#[test]
fn a_nonempty_journal_that_lost_its_records_is_kept_by_recovery_and_refuses_admission() {
    for (label, lose_ready) in [("the record lost", false), ("the record and the ready record lost", true)] {
        let root = repository();
        let journal: PathBuf = plant(root.path(), &[snapshot("original")], &[]);
        fs::remove_file(journal.join("groups")).expect("lose the record");
        if lose_ready {
            fs::remove_file(journal.join("ready")).expect("lose the ready record");
        }

        let note: Option<String> = survivor_note(&journal);
        let messages: Vec<String> = recover(root.path(), &refusing);
        let admission = begin(root.path(), &[snapshot("second")], false);

        assert!(note.is_some_and(|text| { return text.contains("missing"); }), "{label}: the classifier did not name the missing record");
        assert_eq!(live_lock(root.path()), LEFT_BY_THE_KILLED_RUN, "{label}: the recovery changed the live output: {messages:?}");
        assert_eq!(journal_dirs(root.path()).len(), 1, "{label}: the journal was removed or another appeared: {messages:?}");
        assert!(journal.join("0.lock").exists(), "{label}: the snapshot of the journal was lost");
        assert!(messages.iter().any(|message| { return message.contains("missing") && message.contains("cargo-shadow acknowledge"); }), "{label}: the recovery does not name the missing record: {messages:?}");
        assert!(admission.as_ref().is_err_and(|error| { return error.to_string().contains("missing"); }), "{label}: a transaction started beside a journal that lost its record: {:?}", admission.as_ref().err());
        if !lose_ready {
            fs::write(journal.join("groups"), "").expect("put the empty record back");
            let after: Vec<String> = recover(root.path(), &refusing);
            assert_eq!(live_lock(root.path()), "original", "{label}: the control: with the record back the recovery did not restore: {after:?}");
        }
    }
}
