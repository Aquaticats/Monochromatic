//! Controls for the names of a work record's id:
//! the branch digest a push carries,
//! the suffixes an id may hold,
//! and the length those keep inside a file name.

use super::*;
use crate::service_work_handlers_replanning_and_acceptance::SERVICE_KINDS;
use crate::work_item::MAX_OBJECT_NAME_LENGTH;

/// A push's id carries a digest of its branch,
/// which is sixteen lowercase hexadecimal digits and differs between two branches of one commit.
#[test]
fn a_pushs_digest_names_its_branch() {
    let main: String = branch_digest("refs/heads/main");
    assert_eq!(main.len(), 16);
    assert!(main.as_bytes().iter().all(|digit| return digit.is_ascii_digit() || (b'a'..=b'f').contains(digit)), "{main}");
    assert_eq!(main, branch_digest("refs/heads/main"), "the same branch gives the same digest");
    assert_ne!(main, branch_digest("refs/heads/main-two"), "another branch gives another digest");
    assert!(is_id_suffix(&WorkKind::Push, format!("abc-{main}").as_str()));
}

/// What:
///  Every kind the service claims has an id the recovery reads as one of this release's.
/// Why:
///   A claim a killed service left behind is settled by what `is_valid_id` answers,
/// and an id it cannot spell is filed as another release's and given up on
///  instead of retried,
/// so a kind missing from that reading loses the work that asked for it.
#[test]
fn every_kind_the_service_claims_has_an_id_the_recovery_reads() {
    let object_name: String = "a".repeat(MAX_OBJECT_NAME_LENGTH);
    for kind in SERVICE_KINDS {
        let id: String = format!("{}-{object_name}", kind.name());
        assert!(is_valid_id(id.as_str()), "{id:?} names no item, so a claim of it would be given up on");
    }
}

/// What follows a kind in an id is the object alone,
/// or for a push the object and a digest,
/// and anything else names no item.
#[test]
fn a_suffix_is_what_the_kind_names() {
    assert!(is_id_suffix(&WorkKind::Review, "abc"));
    assert!(is_id_suffix(&WorkKind::Push, "abc-0123456789abcdef"));
    // A push record an earlier release named by the kind and the object alone keeps its name.
    assert!(is_id_suffix(&WorkKind::Push, "abc"));
    for refused in ["", "-0123456789abcdef", "abc-", "abc-0123456789ABCDEF", "abc-0123456789abcde", "abc/d", "abc~0123456789abcdef"] {
        assert!(!is_id_suffix(&WorkKind::Push, refused), "{refused:?} names no push");
    }
    assert!(!is_id_suffix(&WorkKind::Review, "abc-0123456789abcdef"), "a review's id is the object alone");
    assert!(is_id_suffix(&WorkKind::Push, "a".repeat(MAX_OBJECT_NAME_LENGTH).as_str()), "an object name may be as long as the limit");
    assert!(is_id_suffix(&WorkKind::Push, format!("{}-0123456789abcdef", "a".repeat(MAX_OBJECT_NAME_LENGTH)).as_str()), "and it carries a digest after it");
    assert!(!is_id_suffix(&WorkKind::Push, "a".repeat(MAX_OBJECT_NAME_LENGTH + 1).as_str()), "one character more names nothing");
    assert_eq!(MAX_ID_SUFFIX_LENGTH, MAX_OBJECT_NAME_LENGTH + 1 + 16);
}

/// What:
///  The commit a push record's name is for,
/// under the name an earlier release wrote and under this one's.
/// Why:
///   A command that reads a held record says which commit is held,
/// and this release's name carries the branch digest after the object,
///  which is not part of any commit's name.
#[test]
fn a_push_records_name_gives_the_commit_it_is_for() {
    let oid: &str = "0123456789abcdef0123456789abcdef01234567";
    let named: String = format!("push-{oid}-{}", branch_digest("refs/heads/main"));
    assert_eq!(object_of_a_record_name(named.as_str()), Some(oid), "this release's name");
    assert_eq!(object_of_a_record_name(format!("push-{oid}").as_str()), Some(oid), "an earlier release's name");
    assert_eq!(object_of_a_record_name("maintenance-abc"), None, "only a push carries a branch digest");
    assert_eq!(object_of_a_record_name("push"), None, "a name with no object names no commit");
    assert_eq!(object_of_a_record_name("push-"), None, "and neither does an empty one");
}
