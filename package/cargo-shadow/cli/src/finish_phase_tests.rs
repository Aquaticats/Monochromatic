//! What:
//! Round 7 tests of the three phases of finishing a transaction and of how an update joins them with its own outcome.
//! Why:
//! Sixth review 6b2 5 and 6c R1:
//! only the removal of a journal that is already marked done is a warning,
//! and a failed mandatory phase must not be hidden behind an earlier error.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('finish phases', () => { it('a completion failure is not cleanup', ...) })
//! ```

use std::fs;

use crate::crates::LockSnapshot;
use crate::failpoints::{Point, arm};
use crate::journal::begin;
use crate::journal_finish::{FinishError, settle_finish};
use crate::journal_support::journal_dirs;
use crate::test_scratch::scratch;
use crate::update::UpdateError;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/f/one";

/// A snapshot of the lockfile of the case.
fn snapshot() -> LockSnapshot {
    return LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: "original".to_string() };
}

/// A disposable repository whose crate root holds a lockfile.
fn repository() -> tempfile::TempDir {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), "original").expect("lock");
    return root;
}

/// Round 7 (6b2 5):
/// a completion marker that cannot be published is not cleanup:
/// the error says so, and the journal is still active (not renamed).
#[test]
fn a_completion_failure_is_not_cleanup_and_leaves_the_journal_active() {
    let root = repository();
    let transaction = begin(root.path(), &[snapshot()], false).expect("begin");
    arm(Point::FinishCompletion);

    let error: FinishError = transaction.finish().expect_err("the completion failed");

    assert!(!error.cleanup_only(), "{error}");
    let journals = journal_dirs(root.path());
    assert_eq!(journals.len(), 1);
    assert!(!journals[0].to_string_lossy().ends_with(".done"), "the journal was marked done although the marker failed");
}

/// Round 7 (6b2 5):
/// a removal that fails after the marker is cleanup only.
#[test]
fn a_removal_failure_after_the_marker_is_cleanup_only() {
    let root = repository();
    let transaction = begin(root.path(), &[snapshot()], false).expect("begin");
    arm(Point::FinishCleanup);

    let error: FinishError = transaction.finish().expect_err("the removal failed");

    assert!(error.cleanup_only(), "{error}");
}

/// Round 7 (6c R1, the update half):
/// an update that failed and whose mandatory finishing failed too reports both.
#[test]
fn an_update_error_with_a_failed_mandatory_phase_reports_both() {
    let original = UpdateError::Other { message: "cargo could not resolve".to_string() };

    let joined = settle_finish(Err(original), Err(FinishError::outputs(std::io::Error::other("disk said no"))), Vec::new(), Vec::new());

    let text: String = joined.expect_err("an error").to_string();
    assert!(text.contains("cargo could not resolve") && text.contains("disk said no"), "{text}");
}

/// A failed removal of a disposable journal does not change an update error.
#[test]
fn an_update_error_stands_alone_when_only_the_cleanup_failed() {
    let original = UpdateError::Other { message: "cargo could not resolve".to_string() };

    let joined = settle_finish(Err(original), Err(FinishError::cleanup(std::io::Error::other("busy"))), Vec::new(), Vec::new());

    let text: String = joined.expect_err("an error").to_string();
    assert_eq!(text, "cargo could not resolve");
}
