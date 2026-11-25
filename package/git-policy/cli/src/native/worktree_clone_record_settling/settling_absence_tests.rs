//! Controls for the pool's answer about one recorded clone:
//!  a name that answers with the recorded GUID stops the run,
//!  a name reused by another GUID stops it,
//!  a GUID carried by another name stops it,
//!  a parent carrying neither is a proved absence,
//!  and a refused listing is evidence rather than a stop.

use crate::worktree_clone_record_settling::settling_absence::{TheClonesPresence, the_clones_presence};
use crate::worktree_clone_record_settling::settling_fixtures::{CLONE_GUID, ScriptedProbe, a_guid, an_identity};

/// The clone's own name,
/// which is the record's,
/// so the design's proof reads the same names.
const CLONE: &str = "tank/worktree-clones/topic";

/// The parent the design pins.
const PARENT: &str = "tank/worktree-clones";

/// A clone that answers its own name with its own GUID stops the run.
#[test]
fn a_clone_that_stands_stops_the_run() {
    let probe: ScriptedProbe = ScriptedProbe::answering(vec![(CLONE, an_identity(CLONE_GUID, 7))]);
    let presence: TheClonesPresence = the_clones_presence(&probe, CLONE, a_guid(CLONE_GUID));
    assert_eq!(presence, TheClonesPresence::Stands { name: String::from(CLONE), guid: a_guid(CLONE_GUID) });
    assert!(presence.stops());
    assert!(presence.stop_sentence().expect("a stop").sentence().contains("git worktree remove"));
}

/// A recorded name that answers with another GUID is a reuse,
/// which refuses by design.
#[test]
fn a_reused_name_stops_the_run() {
    let probe: ScriptedProbe = ScriptedProbe::answering(vec![(CLONE, an_identity(CLONE_GUID + 1, 7))]);
    let presence: TheClonesPresence = the_clones_presence(&probe, CLONE, a_guid(CLONE_GUID));
    assert_eq!(presence, TheClonesPresence::TheNameHoldsAnother { name: String::from(CLONE), guid: a_guid(CLONE_GUID + 1) });
    let sentence: String = presence.stop_sentence().expect("a stop").sentence();
    assert!(sentence.contains("reused name is a refusal by design"), "{sentence}");
}

/// The recorded GUID answering under another name is the same clone renamed,
/// which proves nothing gone.
#[test]
fn a_guid_under_another_name_stops_the_run() {
    let other: String = format!("{PARENT}/another-name");
    let probe: ScriptedProbe = ScriptedProbe::empty().listing_children(&[other.as_str()]).answering_one(other.as_str(), an_identity(CLONE_GUID, 7));
    let presence: TheClonesPresence = the_clones_presence(&probe, CLONE, a_guid(CLONE_GUID));
    assert_eq!(presence, TheClonesPresence::AnswersElsewhere { found: other });
    assert!(presence.stop_sentence().expect("a stop").sentence().contains("under another name"));
}

/// A parent carrying neither the name nor the GUID is a proved absence,
/// and the run goes on.
#[test]
fn a_parent_with_neither_proves_the_absence() {
    let probe: ScriptedProbe = ScriptedProbe::empty().listing_children(&[&format!("{PARENT}/other")]);
    let presence: TheClonesPresence = the_clones_presence(&probe, CLONE, a_guid(CLONE_GUID));
    assert_eq!(presence, TheClonesPresence::ProvedGone);
    assert!(!presence.stops());
    assert!(presence.stop_sentence().is_none());
}

/// A listing the pool refused is the evidence the command exists for,
/// never a stop.
#[test]
fn a_refused_listing_is_evidence() {
    let probe: ScriptedProbe = ScriptedProbe::empty().refusing_the_listing("the pool answered EIO");
    let presence: TheClonesPresence = the_clones_presence(&probe, CLONE, a_guid(CLONE_GUID));
    let TheClonesPresence::Unprovable { evidence } = &presence else { panic!("the refusal is the unprovable arm, and this answered {presence:?}") };
    assert!(evidence.contains("the pool answered EIO"), "{evidence}");
    assert!(!presence.stops());
    assert!(presence.stop_sentence().is_none());
}
