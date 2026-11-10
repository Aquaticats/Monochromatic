//! What:
//!  Controls for the holder and the verdict a held push record carries:
//!  the words a record keeps,
//!  and the proof
//!  that the command they name is gone.
//! Why:
//!  The settlement of a held push rests on both:
//! a record that cannot be read must keep its hold,
//! and a
//!  command that cannot be proven gone must keep it too,
//! so both "yes" and "no" are controlled here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(holdingIn(withHolding(record, holding))).toEqual(holding); expect(provenGone(deadChildHolder())).toBe(true);
//! ```

use super::{Holding, Verdict, holding_in, pid_of, proven_gone, this_command, with_holding};
use crate::claim_time::current_boot_id;
use crate::process_identity::current_birth_identity;
use crate::work_item::{WorkItem, WorkKind};
use std::path::PathBuf;

/// One held push record's text,
/// as the queue writes it.
fn record_text() -> String {
    return WorkItem {
        kind: WorkKind::Push,
        repository: PathBuf::from("/repository"),
        git_dir: PathBuf::from("/repository/.git"),
        branch_ref: String::from("refs/heads/main"),
        oid: String::from("0123456789abcdef0123456789abcdef01234567"),
        parent: String::from("fedcba9876543210fedcba9876543210fedcba98"),
        created: String::from("2026-11-10T17:50:25.196Z"),
        tries: 0,
    }
    .encode();
}

/// What:
///  The text of a record whose command is gone,
/// as `this_command` would have spelled it for a process that has ended.
/// Why:
///   A born-and-ended process is the one holder a control can make without a race.
fn holder_of_an_ended_process(pid: u32, birth: &str) -> String {
    return format!("{}/{pid}/{birth}", current_boot_id());
}

/// What:
///  A record gains the holding and keeps the item it already stated.
/// Why:
///   The holding is written by replacing the record whole,
/// so a reader that knew the item must find it again beside the
///  new words.
#[test]
fn a_record_keeps_its_item_and_gains_the_holding() {
    let text: String = record_text();
    let before: WorkItem = WorkItem::decode(text.as_str()).expect("the record the queue writes");
    let holding: Holding = Holding { holder: String::from("boot/42/linux:99"), verdict: Verdict::Blocked, reason: String::from("a secret was found") };
    let written: String = with_holding(text.as_str(), &holding).expect("one JSON object");
    let after: WorkItem = WorkItem::decode(written.as_str()).expect("the record with its holding");
    assert_eq!(after, before, "the item is the one the queue wrote");
    assert_eq!(holding_in(written.as_str()), holding, "the holding is what was written");
}

/// What:
///  A record an earlier release wrote states no holding,
/// and reads as undecided with no command.
/// Why:
///   That silence is what keeps such a record held and named instead of released:
/// nothing ever said its checks passed.
#[test]
fn a_record_without_a_holding_reads_as_undecided() {
    let text: String = record_text();
    assert_eq!(holding_in(text.as_str()), Holding::undecided());
    assert!(!proven_gone(""), "no command is named, so none is proven gone");
}

/// What:
///  A text that is not one JSON object gains no holding.
/// Why:
///   A record the queue can still read by name must never be replaced by words this module guessed at.
#[test]
fn a_text_that_is_not_a_record_gains_no_holding() {
    let holding: Holding = Holding { holder: String::from("boot/42/linux:99"), verdict: Verdict::Passed, reason: String::new() };
    assert_eq!(with_holding("not a record", &holding), None);
    assert_eq!(with_holding("[1, 2]", &holding), None);
    assert_eq!(holding_in("not a record"), Holding::undecided());
}

/// What:
///  A command that has ended is proven gone,
/// by its number naming no process.
/// Why:
///   This is the proof the settlement turns on:
/// the command that would have decided the checks cannot answer.
#[test]
fn a_command_that_ended_is_proven_gone() {
    let mut child = std::process::Command::new("true").spawn().expect("a child starts");
    let pid: u32 = child.id();
    let birth: String = crate::process_identity::process_birth_identity(i64::from(pid)).expect("the table is readable").expect("the child runs");
    let _ended = child.wait();
    assert!(proven_gone(holder_of_an_ended_process(pid, birth.as_str()).as_str()), "a process that ended is gone");
}

/// What:
///  A command that still runs is not proven gone,
/// which is the control of the proof's "yes".
/// Why:
///   A record whose command lives waits for that command's own answer,
/// so no look may settle it.
#[test]
fn a_command_that_runs_is_not_proven_gone() {
    let birth: String = current_birth_identity().expect("this process has an identity");
    let holder: String = this_command();
    assert_eq!(pid_of(holder.as_str()), Some(i64::from(std::process::id())), "the holder names this command: {holder}");
    assert!(holder.ends_with(birth.as_str()), "and its birth identity: {holder}");
    assert!(!proven_gone(holder.as_str()), "this process runs, so it is not gone");
    assert!(pid_of(birth.as_str()).is_none(), "a birth identity alone is no holder text");
}

/// What:
///  A record of another boot is gone,
/// and a number that was reused names another process,
/// which is gone too.
/// Why:
///   Neither can be the command that landed the commit:
/// one cannot be running after the boot it was born in,
/// and the other is a
///  process younger than the record.
#[test]
fn another_boot_and_a_reused_number_are_both_gone() {
    let birth: String = current_birth_identity().expect("this process has an identity");
    let own: i64 = i64::from(std::process::id());
    assert!(proven_gone(format!("an-earlier-boot/{own}/{birth}").as_str()), "a process of an earlier boot cannot run");
    assert!(proven_gone(format!("{}/{own}/{birth}9", current_boot_id()).as_str()), "another birth identity is another process");
    assert!(!proven_gone(format!("{}/{own}/{birth}", current_boot_id()).as_str()), "the live command of this process is not gone");
}

/// What:
///  A holder text this release cannot read is not proven gone.
/// Why:
///   A proof is required before a record is settled,
/// and a text that is not one proves nothing either way.
#[test]
fn an_unreadable_holder_is_not_proven_gone() {
    let boot: &str = current_boot_id();
    for text in [
        String::from("nothing"),
        String::from("boot"),
        String::from("boot/"),
        format!("{boot}/not-a-number/linux:1"),
        format!("{boot}/0/linux:1"),
        format!("{boot}/-4/linux:1"),
    ] {
        assert!(!proven_gone(text.as_str()), "{text} proves nothing");
    }
}
