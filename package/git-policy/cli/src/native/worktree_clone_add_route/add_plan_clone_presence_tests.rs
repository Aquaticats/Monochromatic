//! What:
//!  Controls for the pool's answer about the clone one standing plan records,
//!  and for the bound the add's waiting walk takes.
//! Why:
//!  The command a person reads beside a plan follows from this answer,
//!  and the answer must be the removal's own proof translated,
//!  so these controls drive a scripted pool and assert the four arms the
//!  command list reads,
//!  including a container longer than one walk takes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(thePresenceOfThePlansClone(probe, plan)).toEqual({ kind: 'gone-proved' });
//! ```

use super::the_presence_of_the_plans_clone;
use crate::worktree_clone_binding_record::add_state_commands::{
    ClonePresence,
    PlanHeadWord,
};
use crate::worktree_clone_binding_record::plan_folder_read::APlanThatNamesTheClone;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::MAXIMUM_CHILDREN;
use crate::worktree_clone_record_settling::settling_fixtures::{
    ScriptedProbe,
    a_guid,
    an_identity,
};
use std::path::PathBuf;

/// The clone the fixture plan records,
/// which is one direct child of the container below.
const CLONE: &str = "tank/worktree-clones/topic";

/// The container the design pins the clone under.
const CONTAINER: &str = "tank/worktree-clones";

/// The GUID the fixture plan records for its clone.
const CLONE_GUID: u64 = 4_821_193_048_571_903_102;

/// One plan naming the clone above,
/// which every control here asks the pool about.
fn the_plan() -> APlanThatNamesTheClone {
    return APlanThatNamesTheClone {
        plan: PathBuf::from(
            "/repo/.git/monochromatic-git-policy-cli/worktree-clones/v1/plans/4821193048571903102",
        ),
        clone_dataset: String::from(CLONE),
        clone_guid: a_guid(CLONE_GUID),
        head: PlanHeadWord::Present,
        every_tuple_planned: true,
    };
}

/// The plan's clone answering its own name with its own GUID is a clone that stands.
#[test]
fn a_clone_that_answers_its_own_name_stands() {
    let probe: ScriptedProbe = ScriptedProbe::answering(vec![(CLONE, an_identity(CLONE_GUID, 7))]);
    assert_eq!(
        the_presence_of_the_plans_clone(&probe, &the_plan()),
        ClonePresence::Stands
    );
}

/// The recorded GUID answering under another name is the same clone renamed,
/// so it stands for this question too.
#[test]
fn a_clone_that_answers_under_another_name_stands() {
    let other: String = format!("{CONTAINER}/another-name");
    let probe: ScriptedProbe = ScriptedProbe::empty()
        .listing_children(&[other.as_str()])
        .answering_one(other.as_str(), an_identity(CLONE_GUID, 7));
    assert_eq!(
        the_presence_of_the_plans_clone(&probe, &the_plan()),
        ClonePresence::Stands
    );
}

/// A container carrying neither the name nor the GUID proves the clone gone.
#[test]
fn a_container_with_neither_proves_the_clone_gone() {
    let probe: ScriptedProbe =
        ScriptedProbe::empty().listing_children(&[&format!("{CONTAINER}/other")]);
    assert_eq!(
        the_presence_of_the_plans_clone(&probe, &the_plan()),
        ClonePresence::GoneProved
    );
}

/// A container whose listing refused proves nothing either way.
#[test]
fn a_refused_listing_proves_nothing() {
    let probe: ScriptedProbe = ScriptedProbe::empty().refusing_the_listing("the pool answered EIO");
    let ClonePresence::GoneUnprovable {
        evidence,
    } = the_presence_of_the_plans_clone(&probe, &the_plan())
    else {
        panic!("a refused listing is an unproved absence")
    };
    assert!(evidence.contains("the pool answered EIO"), "{evidence}");
}

/// The recorded name answering another clone's GUID is the one state no command
/// of this release settles,
/// and the answer names the GUID that took the name.
#[test]
fn a_reused_name_names_the_clone_that_took_it() {
    let probe: ScriptedProbe =
        ScriptedProbe::answering(vec![(CLONE, an_identity(CLONE_GUID + 1, 7))]);
    assert_eq!(
        the_presence_of_the_plans_clone(&probe, &the_plan()),
        ClonePresence::TheNameAnswersAnotherClone {
            live_guid: (CLONE_GUID + 1).to_string()
        }
    );
}

/// A container holding one child more than a walk takes answers an unproved
/// absence rather than walking for ever,
/// and the evidence names the bound.
#[test]
fn a_container_past_the_bound_proves_nothing() {
    let children: Vec<String> = (0..=MAXIMUM_CHILDREN)
        .map(|index| return format!("{CONTAINER}/generated-{index}"))
        .collect();
    let names: Vec<&str> = children.iter().map(|name| return name.as_str()).collect();
    let probe: ScriptedProbe = ScriptedProbe::empty().listing_children(names.as_slice());
    let ClonePresence::GoneUnprovable {
        evidence,
    } = the_presence_of_the_plans_clone(&probe, &the_plan())
    else {
        panic!("a container past the bound is an unproved absence")
    };
    assert!(
        evidence.contains(&MAXIMUM_CHILDREN.to_string()),
        "the evidence names the bound: {evidence}"
    );
    assert!(
        evidence.contains("nothing proves this plan's clone gone"),
        "{evidence}"
    );
}

/// A container holding exactly the bound is walked whole,
/// which is what tells the bound from a truncation.
#[test]
fn a_container_at_the_bound_is_walked_whole() {
    let children: Vec<String> = (0..MAXIMUM_CHILDREN)
        .map(|index| return format!("{CONTAINER}/generated-{index}"))
        .collect();
    let names: Vec<&str> = children.iter().map(|name| return name.as_str()).collect();
    let probe: ScriptedProbe = ScriptedProbe::empty().listing_children(names.as_slice());
    assert_eq!(
        the_presence_of_the_plans_clone(&probe, &the_plan()),
        ClonePresence::GoneProved
    );
}
