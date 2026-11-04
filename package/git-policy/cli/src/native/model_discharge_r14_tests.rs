//! What:
//!  The controls of dependency discharge: R1 retired and its request deleted, an old segment deleted under a checkpoint, and a
//!  checkpoint whose transitive dependency is absent; each red under revision 13 and green under 14, over every surviving subset.
//! Why:
//!  Controls 8, 9 and 12 of the thirteenth review.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(disagreements('fourteen')).toEqual([]); expect(disagreements('thirteen')).toContainEqual(...);
//! ```

use crate::model_discharge_r14::{ADOPT1, ADOPT2, CKPT, CKPT2, DEPX, OUT1, REFX, REQ1, REQ2, Rules, SEG1, accepts, disagreements, spec};

/// Control 8: R1 and R2 adopted in one chain, R1 retired (its outcome is in the ledger) and its request deleted, restart. Revision 13
/// cannot validate R1's adopt and drops R2 (a false drop); revision 14 discharges R1's dependency by the terminal outcome.
#[test]
fn a_reclaimed_request_does_not_drop_the_later_adoption() {
    let present: u32 = ADOPT1 | ADOPT2 | REQ2 | OUT1;
    assert!(spec(present, ADOPT2));
    assert!(!accepts(Rules::Thirteen, present, ADOPT2), "revision 13 drops R2");
    assert!(accepts(Rules::Fourteen, present, ADOPT2));
}

/// Control 9: the old segment is deleted after the checkpoint is durable. Revision 13's checkpoint names its predecessor and becomes
/// unrecoverable (a false drop); revision 14's checkpoint is self-contained.
#[test]
fn a_checkpoint_survives_the_deletion_of_its_predecessor() {
    assert!(!accepts(Rules::Thirteen, CKPT, CKPT));
    assert!(accepts(Rules::Fourteen, CKPT, CKPT));
}

/// Control 12: a checkpoint references a record whose own dependency is absent. Revision 13 checks only the reference's presence and
/// trusts it (a false accept); revision 14 validates the transitive closure.
#[test]
fn a_checkpoint_with_an_absent_transitive_dependency_is_not_trusted() {
    let present: u32 = CKPT2 | REFX;
    assert!(!spec(present, CKPT2));
    assert!(accepts(Rules::Thirteen, present, CKPT2), "revision 13 trusts it");
    assert!(!accepts(Rules::Fourteen, present, CKPT2));
    assert!(accepts(Rules::Fourteen, present | DEPX, CKPT2));
}

/// Over every surviving subset: revision 14 never disagrees with the specification, revision 13 does (and the disagreements include
/// both kinds, a false drop and a false accept).
#[test]
fn revision_fourteen_agrees_with_the_specification_everywhere_and_thirteen_does_not() {
    assert!(disagreements(Rules::Fourteen).is_empty());
    let thirteen = disagreements(Rules::Thirteen);
    assert!(thirteen.iter().any(|entry| { return entry.2; }), "a false accept exists");
    assert!(thirteen.iter().any(|entry| { return !entry.2; }), "a false drop exists");
    assert!(thirteen.iter().any(|entry| { return entry.1 == ADOPT2 && entry.0 & REQ1 == 0; }));
    assert!(thirteen.iter().any(|entry| { return entry.1 == CKPT && entry.0 & SEG1 == 0; }));
}
