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
use crate::test_support::spawn_owned;
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
    // Only a child that outlives the read of its identity has one: a child that has already ended, or a zombie the
    // parent has not reaped yet, answers "gone" (`process_identity.rs`), and a `true` raced this read in about one
    // run in ten.
    let mut child: std::process::Child = spawn_owned(std::process::Command::new("sleep").arg("30")).expect("a child starts");
    let pid: u32 = child.id();
    let birth: String = crate::process_identity::process_birth_identity(i64::from(pid)).expect("the table is readable").expect("the child runs");
    let _ended = child.kill();
    let _reaped = child.wait();
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
///  The other boot is written as a boot ID is written,
/// 32 lowercase hexadecimal digits,
/// because only such a text names a boot this reader can compare with its own:
/// a boot part in any other shape is no boot at all,
/// which is the control beside this one
/// (`a_live_holders_text_spelled_another_way_is_never_proven_gone`).
#[test]
fn another_boot_and_a_reused_number_are_both_gone() {
    let birth: String = current_birth_identity().expect("this process has an identity");
    let own: i64 = i64::from(std::process::id());
    let another_boot: &str = "0123456789abcdef0123456789abcdef";
    assert_ne!(another_boot, current_boot_id(), "the control needs a boot ID other than this host's");
    assert!(proven_gone(format!("{another_boot}/{own}/{birth}").as_str()), "a process of an earlier boot cannot run");
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
        format!("{boot}/{}/", std::process::id()),
        format!("{boot}/{}/linux:", std::process::id()),
    ] {
        assert!(!proven_gone(text.as_str()), "{text} proves nothing");
    }
}

/// What:
///  The three parts of this command's own holder text,
///  cut where `proven_gone` reads them.
/// Why:
///   A control that respells one part must leave the other two exactly as the writer wrote them,
/// and taking the parts from the writer itself keeps this control from drifting away from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [boot, pid, birth] = thisCommand().split('/');
/// ```
fn live_holder_parts() -> (String, String, String) {
    let text: String = this_command();
    let mut parts = text.splitn(3, '/');
    let (Some(boot), Some(pid), Some(birth)) = (parts.next(), parts.next(), parts.next()) else {
        panic!("this command's own holder text {text} does not name a boot, a process and a birth");
    };
    return (boot.to_owned(), pid.to_owned(), birth.to_owned());
}

/// What:
///  The birth `linux:8423337` with a leading zero on its start tick,
///  which is the same tick written the way a program that pads numbers writes it.
/// Why:
///   A start tick is a number,
/// and one number has one value however many zeros lead it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const padded = (birth) => { const [prefix, tick] = birth.split(':'); return `${prefix}:0${tick}`; };
/// ```
fn birth_with_a_leading_zero(birth: &str) -> String {
    let Some((prefix, tick)) = birth.split_once(':') else {
        panic!("the birth {birth} is no start tick this control can pad");
    };
    assert!(!tick.starts_with('0'), "the birth {birth} already leads its tick with a zero");
    return format!("{prefix}:0{tick}");
}

/// What:
///  The same birth with its prefix's letters in capitals.
/// Why:
///   A tag's case is part of how it is written and no part of what it names,
/// so a program that capitalizes a field writes the same birth another way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const capitalized = (birth) => { const [prefix, tick] = birth.split(':'); return `${prefix.toUpperCase()}:${tick}`; };
/// ```
fn birth_with_a_capitalized_prefix(birth: &str) -> String {
    let Some((prefix, tick)) = birth.split_once(':') else {
        panic!("the birth {birth} carries no prefix this control can capitalize");
    };
    return format!("{}:{tick}", prefix.to_uppercase());
}

/// What:
///  The same boot ID in the dashed form the kernel prints in its own file.
/// Why:
///   `read_boot_id` removes those dashes before it stores the boot,
/// and a tool that copies the kernel's file as it is writes the same boot another way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dashed = (boot) => `${boot.slice(0, 8)}-${boot.slice(8, 12)}-${boot.slice(12, 16)}-${boot.slice(16, 20)}-${boot.slice(20)}`;
/// ```
fn boot_with_the_kernels_dashes(boot: &str) -> String {
    assert_eq!(boot.len(), 32, "this host's boot ID {boot} is not the 32 digits a boot ID has");
    return format!("{}-{}-{}-{}-{}", &boot[0..8], &boot[8..12], &boot[12..16], &boot[16..20], &boot[20..32]);
}

/// Refused:
///  a record whose text spells one part of a live holder's identity another way is never proven gone,
///  and the commit that command landed stays held.
/// Why:
///   A part of the holder stands for one value in one spelling,
/// and a reader that compares two spellings of one value as two values proves a command gone that runs.
/// A held push settled on that proof is published before its post-commit checks have answered,
/// and the line the person reads says the landing command ended
/// (`service_handoff.rs`,
/// `held_push_line`);
/// and a record that says the checks were never decided stops holding the branch's push back
/// (`work_item_holding.rs`,
/// `undecided_commits`).
///  The spellings are the ones a person,
/// an older release,
/// a tool or a damaged file can produce without changing the value:
/// a boot ID's letters in capitals,
/// the kernel's dashes in the boot,
/// a leading zero on the start tick,
/// a space on either side of the birth,
/// the birth's tag in capitals,
/// and a boot part that names no boot ID at all,
/// which no reader can compare with its own boot.
/// A padded process number and one behind a plus sign are the same number,
/// so neither is a spelling the reader has to know:
/// those arms are the control of this control and pass on every reader.
#[test]
fn a_live_holders_text_spelled_another_way_is_never_proven_gone() {
    let (boot, pid, birth) = live_holder_parts();
    let as_written: String = format!("{boot}/{pid}/{birth}");
    assert!(!proven_gone(as_written.as_str()), "the control's own spelling names a command that runs: {as_written}");
    let uppercase_boot: String = boot.to_uppercase();
    assert_ne!(uppercase_boot, boot, "this host's boot ID {boot} has no letter whose case can change");
    let padded_tick: String = birth_with_a_leading_zero(birth.as_str());
    let mut arms: Vec<(&str, String)> = vec![
        ("the boot in capitals", format!("{uppercase_boot}/{pid}/{birth}")),
        ("the boot in the kernel's dashed form", format!("{}/{pid}/{birth}", boot_with_the_kernels_dashes(boot.as_str()))),
        ("a boot part that names no boot ID", format!("an-earlier-boot/{pid}/{birth}")),
        ("a boot part padded with a space", format!("{boot} /{pid}/{birth}")),
        ("a leading zero on the start tick", format!("{boot}/{pid}/{padded_tick}")),
        ("a space before the birth", format!("{boot}/{pid}/ {birth}")),
        ("a space after the birth", format!("{boot}/{pid}/{birth} ")),
        ("a padded process number", format!("{boot}/0{pid}/{birth}")),
        ("a process number behind a plus sign", format!("{boot}/+{pid}/{birth}")),
    ];
    if cfg!(target_os = "linux") {
        arms.push(("the birth's tag in capitals", format!("{boot}/{pid}/{}", birth_with_a_capitalized_prefix(birth.as_str()))));
    }
    // Every arm is asked and every wrong answer is kept, so one run names each spelling that was read as another process
    //  instead of the first one that was.
    let mut wrong: Vec<String> = Vec::new();
    for (what, text) in arms {
        if proven_gone(text.as_str()) {
            wrong.push(format!("{what}: the reader proved the holder of {text} gone, and that command runs"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
