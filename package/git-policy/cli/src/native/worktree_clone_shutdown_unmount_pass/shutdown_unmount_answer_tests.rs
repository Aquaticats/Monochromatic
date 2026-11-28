//! What:
//!  Controls of the pass's answer:
//!  the summary's counts,
//!  the line each ending writes,
//!  the readiness the withdrawal found,
//!  and the sentence a boot that verified nothing answers.
//! Why:
//!  A person and a unit's journal read these lines and nothing else,
//!  so a summary that counted wrongly or a held line that named no holder would
//!  leave a machine going down with no answer to read
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 19,
//!  "Acceptance tests").
//!  The values are built here, so no control reads a machine.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = { boot, withdrawal, verifiedNothing: null, reached: [unmounted, held] };
//! outcome.lines();
//! ```

use super::{
    Holder,
    Holdings,
    MountEnding,
    PassOutcome,
    ReachedMount,
};
use crate::worktree_clone_mount_protocol::clone_mount_readiness::{
    ReadinessRecord,
    ReadyState,
    SweepState,
};
use crate::worktree_clone_mount_protocol::clone_mount_readiness_file_withdraw::{
    RecordStood,
    Withdrawal,
};
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::path::PathBuf;

/// The boot every control answers in.
const BOOT: &str = "0f3a5c7e-1b2d-4f60-8a9b-0c1d2e3f4a5b";

/// The record one control's boot published.
fn record() -> ReadinessRecord {
    return ReadinessRecord {
        boot: String::from(BOOT),
        pool_name: String::from("zpcachyos"),
        pool_state: ReadyState::Ready,
        checkout_path: String::from("/home/user/Mono"),
        checkout_state: ReadyState::Ready,
        sweep_nonce: RequestNonce::parse("1".repeat(32).as_str()).ok(),
        sweep_state: SweepState::Finished,
        time: String::from("2026-11-27T20:00:00-05:00"),
    };
}

/// The withdrawal of one boot's record.
fn withdrawal() -> Withdrawal {
    return Withdrawal {
        stood: RecordStood::Record(Box::new(record())),
        path: PathBuf::from("/run/git-worktree-clone-mount-readiness/result"),
    };
}

/// One mount the pass unmounted.
fn unmounted(slot: &str) -> ReachedMount {
    return ReachedMount {
        slot: String::from(slot),
        clone: format!("zpcachyos/nosnap/git-worktree-clones/{slot}"),
        slot_path: PathBuf::from(format!("/home/user/Mono/.claude/worktrees/{slot}")),
        ending: MountEnding::Unmounted {
            mount_id: 41,
        },
    };
}

/// One mount a task holds.
fn held(
    slot: &str,
    holdings: Result<Holdings, String>,
) -> ReachedMount {
    return ReachedMount {
        slot: String::from(slot),
        clone: format!("zpcachyos/nosnap/git-worktree-clones/{slot}"),
        slot_path: PathBuf::from(format!("/home/user/Mono/.claude/worktrees/{slot}")),
        ending: MountEnding::Held {
            mount_id: 42,
            said: String::from("a task holds the mount"),
            holdings,
        },
    };
}

/// The summary counts each ending and says no intent changed.
#[test]
fn the_summary_counts_every_ending_and_says_no_intent_changed() {
    let outcome: PassOutcome = PassOutcome {
        boot: String::from(BOOT),
        withdrawal: withdrawal(),
        verified_nothing: None,
        reached: vec![
            unmounted("one"),
            held(
                "two",
                Ok(Holdings {
                    holders: vec![Holder {
                        namespace: String::from("mnt:[4026532001]"),
                        task: PathBuf::from("/proc/1234"),
                        mounts: 1,
                    }],
                    incomplete_reads: 0,
                }),
            ),
            ReachedMount {
                slot: String::from("three"),
                clone: String::from("zpcachyos/nosnap/git-worktree-clones/three"),
                slot_path: PathBuf::from("/home/user/Mono/.claude/worktrees/three"),
                ending: MountEnding::Left {
                    said: String::from("another handler holds this slot's claim"),
                },
            },
        ],
    };
    assert_eq!(
        (outcome.unmounted(), outcome.held(), outcome.left()),
        (1, 1, 1)
    );
    let lines: Vec<String> = outcome.lines();
    assert_eq!(
        lines.len(),
        4,
        "one summary and one line per mount: {lines:?}"
    );
    let summary: &str = lines[0].as_str();
    assert!(
        summary.contains("readiness for boot 0f3a5c7e-1b2d-4f60-8a9b-0c1d2e3f4a5b was withdrawn before any mount was touched"),
        "the summary says the withdrawal came first, and this one says {summary}"
    );
    assert!(
        summary.contains(
            "3 mount(s) of this boot's clone container stood at a slot, 1 unmounted, 1 held, 1 left"
        ),
        "the summary counts every ending, and this one says {summary}"
    );
    assert!(
        summary.contains("no durable intent was changed"),
        "the summary says the pass changed no intent, and this one says {summary}"
    );
}

/// The unmounted line names the slot, the clone and the mount.
#[test]
fn the_unmounted_line_names_the_slot_the_clone_and_the_mount() {
    let outcome: PassOutcome = PassOutcome {
        boot: String::from(BOOT),
        withdrawal: withdrawal(),
        verified_nothing: None,
        reached: vec![unmounted("one")],
    };
    let line: String = outcome.lines()[1].clone();
    assert!(
        line.contains("unmounted slot `one` at /home/user/Mono/.claude/worktrees/one of `zpcachyos/nosnap/git-worktree-clones/one`, which was mount 41"),
        "the line names all three, and this one says {line}"
    );
}

/// The held line names its holders and says nothing was forced.
#[test]
fn the_held_line_names_its_holders_and_says_nothing_was_forced() {
    let outcome: PassOutcome = PassOutcome {
        boot: String::from(BOOT),
        withdrawal: withdrawal(),
        verified_nothing: None,
        reached: vec![held(
            "two",
            Ok(Holdings {
                holders: vec![Holder {
                    namespace: String::from("mnt:[4026532001]"),
                    task: PathBuf::from("/proc/1234"),
                    mounts: 2,
                }],
                incomplete_reads: 3,
            }),
        )],
    };
    let line: String = outcome.lines()[1].clone();
    assert!(
        line.contains("the all-namespace scan named `mnt:[4026532001]` read through /proc/1234 with 2 mount(s) of it (3 read(s) it could not complete)"),
        "the line names the holder and the incompletes, and this one says {line}"
    );
    assert!(
        line.contains("Nothing was forced and nothing was detached"),
        "the line says what the pass did not do, and this one says {line}"
    );
}

/// A scan that found nothing says so rather than printing an empty list.
#[test]
fn a_scan_that_found_no_holder_says_so() {
    let outcome: PassOutcome = PassOutcome {
        boot: String::from(BOOT),
        withdrawal: withdrawal(),
        verified_nothing: None,
        reached: vec![held(
            "two",
            Ok(Holdings {
                holders: Vec::new(),
                incomplete_reads: 0,
            }),
        )],
    };
    let line: String = outcome.lines()[1].clone();
    assert!(
        line.contains("the all-namespace scan named no namespace holding a mount of it"),
        "the line says the scan named nothing, and this one says {line}"
    );
}

/// A scan that could not be made is that sentence and not a clear machine.
#[test]
fn a_scan_that_could_not_be_made_says_why() {
    let outcome: PassOutcome = PassOutcome {
        boot: String::from(BOOT),
        withdrawal: withdrawal(),
        verified_nothing: None,
        reached: vec![held(
            "two",
            Err(String::from(
                "no holder scan could be made, because `one` answered no GUID to bind one to",
            )),
        )],
    };
    let line: String = outcome.lines()[1].clone();
    assert!(
        line.contains("no holder scan could be made"),
        "the line carries the scan's refusal, and this one says {line}"
    );
}

/// A boot that verified nothing says why and unmounted nothing.
#[test]
fn a_boot_that_verified_nothing_says_why() {
    let outcome: PassOutcome = PassOutcome {
        boot: String::from(BOOT),
        withdrawal: Withdrawal {
            stood: RecordStood::Nothing,
            path: PathBuf::from("/run/git-worktree-clone-mount-readiness/result"),
        },
        verified_nothing: Some(String::from(
            "this boot published no readiness record, so nothing here says it verified the pool and the checkout",
        )),
        reached: Vec::new(),
    };
    let lines: Vec<String> = outcome.lines();
    assert_eq!(lines.len(), 1, "a pass with nothing to do answers one line");
    let summary: &str = lines[0].as_str();
    assert!(
        summary.contains("this boot had published no readiness record"),
        "the summary says what stood, and this one says {summary}"
    );
    assert!(
        summary.contains("so no mount was unmounted and no durable intent was changed"),
        "the summary says what it cost, and this one says {summary}"
    );
}

/// A record this route's reader refuses says so in the summary.
#[test]
fn an_unreadable_record_is_named_in_the_summary() {
    let outcome: PassOutcome = PassOutcome {
        boot: String::from(BOOT),
        withdrawal: Withdrawal {
            stood: RecordStood::Unreadable {
                said: String::from("the file is not a readiness record"),
            },
            path: PathBuf::from("/run/git-worktree-clone-mount-readiness/result"),
        },
        verified_nothing: None,
        reached: Vec::new(),
    };
    let summary: String = outcome.lines()[0].clone();
    assert!(
        summary.contains(
            "a readiness record this route's reader refuses was removed before any mount was touched"
        ),
        "the summary says the file was removed and why, and this one says {summary}"
    );
}
