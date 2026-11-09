//! Controls for the deadline of the repository link check;
//! the answers of the check itself are in the shared table (`repository_link_shared_table_tests.rs`).

use super::*;
use crate::repository_link_answer::AnswerKind;
use crate::repository_link_identity::LinkPhase;
use crate::repository_link_probe::EntryKind;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// A probe that never answers a `lstat`,
/// the way a hung drive does.
struct StalledProbe;

impl LinkProbe for StalledProbe {
    fn lstat(&self, _path: &Path) -> EntryKind {
        loop {
            std::thread::park();
        }
    }
    fn readlink(&self, _path: &Path) -> String {
        return String::new();
    }
    fn realpath(&self, _path: &Path) -> Option<PathBuf> {
        return None;
    }
    fn read_text(&self, _path: &Path) -> String {
        return String::new();
    }
    fn uuid_of(&self, _path: &Path) -> String {
        return String::new();
    }
    fn is_mount_point(&self, _path: &Path) -> bool {
        return false;
    }
}

/// An identity that names a link path and nothing else.
fn identity() -> LinkIdentity {
    return LinkIdentity {
        link_path: String::from("/home/u/Monochromatic"),
        link_text: String::from("/mnt/860/repository/Monochromatic"),
        canonical_root: String::from("/mnt/860/repository/Monochromatic"),
        mount_point: String::from("/mnt/860"),
        drive_uuid: String::from("DRIVE"),
        nvme_uuid: String::from("NVME"),
        root_commit: String::from("abc123"),
        phase: LinkPhase::After,
    };
}

/// A filesystem that never replies is answered as "the drive did not answer" inside the deadline,
/// with the plain message that names the link path.
#[test]
fn a_stalled_drive_is_answered_within_the_deadline() {
    let started: Instant = Instant::now();
    let deadline: Duration = Duration::from_millis(300);
    let answer: LinkAnswer = check_repository_link_within(&identity(), Arc::new(StalledProbe), deadline);
    assert_eq!(answer.kind, AnswerKind::DriveDidNotAnswer);
    assert!(!answer.ready);
    assert!(answer.message.contains("/home/u/Monochromatic"), "{}", answer.message);
    assert!(answer.message.contains("nothing was written"), "{}", answer.message);
    // The claim is the answer, its kind and its message: the deadline passing is what ends the wait, so a slow host only makes
    //  this ceiling easier to hold. Measured on this host at a load average of 47: the answer came at 0.31 s for a deadline of
    //  0.3 s, so ten times the deadline carries 10 times the figure it was measured at.
    assert!(started.elapsed() < deadline * 10, "the answer came after {:?}", started.elapsed());
}

/// The default deadline is the plan's "a few seconds":
/// above zero,
/// and never an indefinite wait.
#[test]
fn the_default_deadline_is_a_few_seconds() {
    assert!(DEFAULT_DEADLINE > Duration::ZERO);
    assert!(DEFAULT_DEADLINE <= Duration::from_secs(10));
}

/// A missing identity answers with the TypeScript check's message.
#[test]
fn a_missing_identity_has_the_plain_message() {
    let answer: LinkAnswer = answer_when_identity_is_missing();
    assert_eq!(answer.kind, AnswerKind::IdentityMissing);
    assert!(answer.message.contains("repository-link-record"), "{}", answer.message);
}
