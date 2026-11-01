//! Controls for the pure parts of a landing attempt.
//! The attempt itself runs against real repositories, in `binary_landing_tests.rs`.

use super::*;

/// Only a target that names a commit can be replayed onto.
#[test]
fn only_a_commit_target_names_a_commit() {
    assert!(names_commit(&Base::Commit("a".repeat(40))));
    assert!(!names_commit(&Base::Unborn));
}

/// Each failing step is worded by its own cause, with the text it carries.
#[test]
fn each_failure_is_worded_by_its_cause() {
    assert_eq!(LandingError::Git(GitFailure(String::from("git broke"))).to_string(), "git broke");
    assert_eq!(LandingError::Recovery(RecoveryError(String::from("recovery broke"))).to_string(), "recovery broke");
    assert_eq!(LandingError::Journal(String::from("journal broke")).to_string(), "journal broke");
    assert_eq!(LandingError::Release(String::from("release broke")).to_string(), "release broke");
    assert_eq!(
        LandingError::Phase(PhaseMarkerError::Io(std::io::Error::other("disk"))).to_string(),
        "transaction phase marker failed: disk"
    );
    assert!(
        LandingError::Lock(CoordinationFailure::Stop(String::from("lock broke")))
            .to_string()
            .contains("lock broke")
    );
}

/// A failure after the branch advanced names the landed commit and its reference, carries the cause and says not to retry.
#[test]
fn a_failure_after_the_advance_names_the_landed_commit_and_forbids_a_retry() {
    let failure: LandingError = LandingError::LandedThenFailed {
        oid: "a".repeat(40),
        target: String::from("refs/heads/main"),
        cause: Box::new(LandingError::Journal(String::from("marker broke"))),
    };
    let text: String = failure.to_string();
    assert!(text.contains(&format!("the commit {} landed on refs/heads/main", "a".repeat(40))), "{text}");
    assert!(text.contains("completing its records failed: marker broke"), "{text}");
    assert!(text.contains("do not retry it"), "{text}");
}
