//! What:
//!  Controls for the destruction proof:
//!  a clone that is gone,
//!  a clone that still answers,
//!  a pool that could not be read,
//!  and a record with no GUID to prove gone.
//! Why:
//!  The release takes this value as its only permission,
//!  so a proof that could be made without a live read would release a lock over a live clone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const proof = proveTheRecordedCloneIsGone('slot', recorded, { kind: 'absent' });
//! ```

use super::{
    DestructionRefusal,
    TheLiveCloneAnswer,
    prove_the_recorded_clone_is_gone,
};
use crate::worktree_clone_managed_lock::RecordedClone;
use crate::worktree_clone_mount_protocol::DatasetGuid;

/// The clone one record names.
fn recorded() -> RecordedClone {
    return RecordedClone {
        dataset: String::from("pool/clones/locked-slot"),
        guid: Some(DatasetGuid::parse("18446744073709551615").expect("the GUID reads")),
    };
}

/// A clone that is gone makes the proof,
/// and it carries the slot and the clone it was made for.
#[test]
fn an_absent_clone_makes_the_proof() {
    let clone: RecordedClone = recorded();
    let proof =
        prove_the_recorded_clone_is_gone("locked-slot", &clone, &TheLiveCloneAnswer::Absent)
            .expect("the proof is made");
    assert_eq!(proof.slot(), "locked-slot");
    assert_eq!(proof.clone_dataset(), "pool/clones/locked-slot");
    assert_eq!(
        proof.clone_guid(),
        clone.guid.expect("the fixture holds a GUID")
    );
}

/// A clone that still answers refuses,
/// and the refusal names what answered.
#[test]
fn a_clone_that_still_answers_refuses() {
    let refused: DestructionRefusal = prove_the_recorded_clone_is_gone(
        "locked-slot",
        &recorded(),
        &TheLiveCloneAnswer::Answers {
            dataset: String::from("pool/clones/locked-slot"),
            guid: DatasetGuid::parse("7").expect("the GUID reads"),
        },
    )
    .expect_err("a live clone refuses the proof");
    assert!(matches!(
        refused,
        DestructionRefusal::TheCloneStillAnswers { .. }
    ));
    assert!(
        refused.words().contains("still answers on the pool"),
        "{}",
        refused.words()
    );
}

/// A pool that could not be read proves nothing either way.
#[test]
fn an_unreadable_pool_refuses() {
    let refused: DestructionRefusal = prove_the_recorded_clone_is_gone(
        "locked-slot",
        &recorded(),
        &TheLiveCloneAnswer::Unreadable {
            reason: String::from("the pool's list was refused"),
        },
    )
    .expect_err("an unreadable pool refuses the proof");
    assert!(matches!(refused, DestructionRefusal::Unreadable { .. }));
    assert!(
        refused.words().contains("the pool's list was refused"),
        "{}",
        refused.words()
    );
}

/// A record with no GUID names nothing a proof could be about.
#[test]
fn a_record_with_no_guid_refuses() {
    let mut clone: RecordedClone = recorded();
    clone.guid = None;
    let refused: DestructionRefusal =
        prove_the_recorded_clone_is_gone("locked-slot", &clone, &TheLiveCloneAnswer::Absent)
            .expect_err("no GUID refuses the proof");
    assert_eq!(
        refused,
        DestructionRefusal::NoGuidToProveGone {
            dataset: String::from("pool/clones/locked-slot")
        }
    );
}
