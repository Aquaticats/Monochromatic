//! Controls for the reuse of a role name:
//!  every stop's sentence names what stopped it and a way forward,
//!  and no sentence is empty.

use crate::worktree_clone_record_settling::settling_refusal::SettlingStop;
use std::path::PathBuf;

/// Every stop a control can build answers with a sentence naming the file or value it stopped on.
#[test]
fn every_stop_builds_a_sentence() {
    let stops: Vec<SettlingStop> = vec![
        SettlingStop::Usage {
            said: String::from("no slot stands"),
        },
        SettlingStop::PlaceholderLeft {
            flag: String::from("--confirm-clone-guid"),
            value: String::from("<digits>"),
        },
        SettlingStop::TheSlotIsNamedTwice {
            asked: String::from("topic"),
            confirmed: String::from("other"),
        },
        SettlingStop::TheSlotIsNotThisRecord {
            asked: String::from("topic"),
            recorded: String::from("other"),
        },
        SettlingStop::TheCloneGuidDiffers {
            typed: String::from("1"),
            wanted: String::from("2"),
        },
        SettlingStop::NoRecord {
            path: PathBuf::from("/repo/.git/records/topic.binding"),
        },
        SettlingStop::RecordUnreadable {
            path: PathBuf::from("/repo/.git/records/topic.binding"),
            said: String::from("it does not exist"),
        },
        SettlingStop::NoCloneRecorded {
            slot: String::from("topic"),
            phase: String::from("registered"),
        },
        SettlingStop::TheRecordIsIncomplete {
            said: String::from("cloneGuid is missing"),
        },
        SettlingStop::TheSlotCouldNotBeRead {
            path: PathBuf::from("/repo/.claude/worktrees/topic"),
            said: String::from("permission denied"),
        },
        SettlingStop::TheClaimIsHeld {
            slot: String::from("topic"),
            pid: 48213,
        },
        SettlingStop::TheClaimCannotBeTaken {
            slot: String::from("topic"),
            said: String::from("the folder is missing"),
        },
        SettlingStop::TheCloneStillStands {
            clone: String::from("tank/clones/topic"),
            guid: String::from("4821193048571903102"),
        },
        SettlingStop::TheNameHoldsAnother {
            name: String::from("tank/clones/topic"),
            guid: String::from("1"),
        },
        SettlingStop::TheCloneAnswersElsewhere {
            found: String::from("tank/clones/other"),
        },
        SettlingStop::AMountAtTheSlot {
            described: String::from("`tank/clones/topic` at the slot"),
        },
        SettlingStop::TheMountTableCouldNotBeRead {
            said: String::from("the table is empty"),
        },
        SettlingStop::ThePointerNamesAnother {
            found: String::from("gitdir: /elsewhere"),
        },
        SettlingStop::TheRegistrationIsAnother {
            recorded: String::from("1:2:3"),
            live: String::from("4:5:6"),
        },
        SettlingStop::TheRegistrationDoesNotAnswer {
            path: PathBuf::from("/repo/.git/worktrees/topic"),
            said: String::from("gone"),
        },
        SettlingStop::TheLockIsNotThisAdds {
            path: PathBuf::from("/repo/.git/worktrees/topic/locked"),
        },
        SettlingStop::TheOriginCleanupDisagrees {
            field: String::from("originSnapshotGuid"),
            recorded: String::from("1"),
            standing: String::from("2"),
        },
        SettlingStop::TheOriginCleanupCouldNotBeRead {
            path: PathBuf::from("/repo/.git/cleanups/origin"),
            said: String::from("it does not read"),
        },
        SettlingStop::ThePlanCouldNotBeRead {
            path: PathBuf::from("/repo/.git/plans/1"),
            said: String::from("it does not read"),
        },
        SettlingStop::ThePlanIsTheFinishPasss {
            path: PathBuf::from("/repo/.git/plans/1"),
        },
        SettlingStop::ThePlanDisagrees {
            field: String::from("target"),
            binding: String::from("/a"),
            plan: String::from("/b"),
        },
        SettlingStop::TheRecordCouldNotGo {
            path: PathBuf::from("/repo/.git/records/topic.binding"),
            said: String::from("permission denied"),
        },
        SettlingStop::TheRecipeStopped {
            said: String::from("a name still stands"),
        },
        SettlingStop::TheClaimCannotBeReleased {
            slot: String::from("topic"),
            said: String::from("the file is gone"),
        },
    ];
    for stop in stops {
        let sentence: String = stop.sentence();
        assert!(!sentence.trim().is_empty(), "{stop:?}");
        assert!(sentence.len() > 40, "{stop:?} printed `{sentence}`");
    }
}

/// The claim's sentence names the process a person can look for.
#[test]
fn the_live_owner_is_named_by_its_process() {
    let sentence: String = SettlingStop::TheClaimIsHeld {
        slot: String::from("topic"),
        pid: 48213,
    }
    .sentence();
    assert!(sentence.contains("process 48213"), "{sentence}");
}
