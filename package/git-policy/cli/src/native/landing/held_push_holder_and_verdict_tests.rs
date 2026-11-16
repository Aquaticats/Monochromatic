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
use crate::process_identity::{LINUX_IDENTITY_PREFIX, current_birth_identity};
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

/// The birth of a process that never ran:
/// A boot ID of a boot that has ended, which is no boot of this host.
const AN_ENDED_BOOT: &str = "0123456789abcdef0123456789abcdef";

/// The birth of a process that never ran:
/// the largest start tick a 64-bit number holds,
/// written as this program writes one,
/// which no process is born at.
/// Why:
///   The table pairs it with the number of a process that runs to say what a record of an earlier process reads as,
/// and it is a birth a live process can never have.
const NO_SUCH_BIRTH: &str = "linux:18446744073709551615";

/// The answer one row of the identity table requires.
/// Why:
///   `proven_gone` answers yes or no,
/// and this table states the three states of the world that answer must not confuse:
/// a holder that is gone,
/// a holder that runs,
/// and a reader that cannot tell.
/// `Alive` and `Unknown` are both "no" to this reader,
/// which is why the claim's own table (`lock/single_owner_claim_tests.rs`) is the one that tells them apart.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Required {
    /// Nothing of the holder runs:
    /// the record may be settled.
    Gone,
    /// The holder is the process that has the number now:
    /// the record waits for it.
    Alive,
    /// The reader cannot tell:
    /// the record stays as it is.
    Unknown,
}

/// What the boot part of a holder text holds,
/// which is the one proof of an end that stands apart from the birth.
enum BootState {
    /// This host's boot ID, as this program writes it.
    ThisBoot,
    /// A boot ID of a boot that has ended, which ends every process in it.
    AnEndedBoot,
    /// This host's boot ID in the spelling the kernel prints, with its dashes.
    ThisBootDashed,
    /// A text that names no boot ID at all.
    Unreadable,
    /// The empty part a writer leaves when it cannot read its boot ID.
    Absent,
}

/// The process number a holder text names, and which process has it now,
/// which is the other proof of an end that stands apart from the birth.
enum NumberState {
    /// A number no process has.
    Free,
    /// The number of the process that wrote the record.
    TheWriter,
    /// The number of a later process, which the recorded birth is not.
    ALaterProcess,
}

/// What the birth part of a holder text holds.
enum BirthState {
    /// The birth of the record, in the spelling this program writes.
    Written,
    /// That birth in another spelling: a leading zero on its start tick.
    Respelled,
    /// That birth with padding before the colon of its tag, which is the same tag in another spelling.
    PaddedTag,
    /// Nothing but a space.
    Blank,
    /// The empty text.
    Empty,
    /// This platform's tag and no tick.
    NoTick,
}

/// One row of the identity table:
/// the three parts of a holder text, and the answer the rule owes that state.
/// Why:
///  A row states the boot and the number,
/// which are the proofs that stand apart from the birth,
/// beside the birth itself,
/// so a reader that uses an unreadable part as proof is named by the row it broke.
struct Row {
    what: &'static str,
    boot: BootState,
    number: NumberState,
    birth: BirthState,
    due: Required,
}

/// The birth `birth` with a space between its tag and the colon, which is the same tag in another spelling.
/// Why:
///   A field that was padded on the left of a colon names the same tag,
/// and a reader that compared it with the kernel's text would read another process.
fn birth_with_padding_before_the_colon(birth: &str) -> String {
    let Some((tag, tick)) = birth.split_once(':') else {
        panic!("the birth {birth} carries no tag this control can pad");
    };
    return format!("{tag} :{tick}");
}

/// The holder text of one row,
/// with every part the row does not name taken from this process.
/// Why:
///   A control that respells one part must leave the other two as the writer would have written them.
fn holder_text_of(row: &Row, live_birth: &str, free: i64, own: i64) -> String {
    let boot: String = match row.boot {
        BootState::ThisBoot => String::from(current_boot_id()),
        BootState::AnEndedBoot => String::from(AN_ENDED_BOOT),
        BootState::ThisBootDashed => boot_with_the_kernels_dashes(current_boot_id()),
        BootState::Unreadable => String::from("an-earlier-boot"),
        BootState::Absent => String::new(),
    };
    let (number, value): (i64, &str) = match row.number {
        NumberState::Free => (free, live_birth),
        NumberState::TheWriter => (own, live_birth),
        NumberState::ALaterProcess => (own, NO_SUCH_BIRTH),
    };
    let birth: String = match row.birth {
        BirthState::Written => value.to_owned(),
        BirthState::Respelled => birth_with_a_leading_zero(value),
        BirthState::PaddedTag => birth_with_padding_before_the_colon(value),
        BirthState::Blank => String::from(" "),
        BirthState::Empty => String::new(),
        BirthState::NoTick => String::from(LINUX_IDENTITY_PREFIX),
    };
    return format!("{boot}/{number}/{birth}");
}

/// Refused:
///  no row of the identity table gives "gone" for a holder that runs,
///  and no row keeps a holder that cannot run.
/// Why:
///  The two rules pull against each other,
/// and every proof that stands on its own is used:
/// a boot ID of another boot ends every process in it,
/// a number no process has is the end of the command that wrote the record,
/// and only where the number names a process do the boot and the birth decide,
///  where a part this reader cannot compare answers unknown and keeps the record.
///  A boot part this reader cannot read is no end and hides no end:
/// the number's own proof still stands, which is what a holder proven dead by its absent number needs.
///  The birth spellings include the ones a person or a program produces without changing the value:
/// a leading zero on the start tick,
/// padding before the tag's colon,
/// a field of nothing but a space,
/// the empty text,
/// and this platform's tag with no tick.
#[test]
fn every_row_of_the_identity_table_gives_the_answer_the_rule_requires() {
    let own: i64 = i64::from(std::process::id());
    let live_birth: String = current_birth_identity().expect("this process has an identity");
    assert_ne!(AN_ENDED_BOOT, current_boot_id(), "the table needs a boot ID other than this host's");
    let free: i64 = crate::single_owner_claim::unused_pid_for_tests();
    let rows: [Row; 33] = [
        // The birth column, beside this host's boot.
        Row { what: "a free number and the birth as the writer writes it", boot: BootState::ThisBoot, number: NumberState::Free, birth: BirthState::Written, due: Required::Gone },
        Row { what: "a free number and the birth in another spelling", boot: BootState::ThisBoot, number: NumberState::Free, birth: BirthState::Respelled, due: Required::Gone },
        Row { what: "a free number and padding before the birth's colon", boot: BootState::ThisBoot, number: NumberState::Free, birth: BirthState::PaddedTag, due: Required::Gone },
        Row { what: "a free number and a blank birth", boot: BootState::ThisBoot, number: NumberState::Free, birth: BirthState::Blank, due: Required::Gone },
        Row { what: "a free number and an empty birth", boot: BootState::ThisBoot, number: NumberState::Free, birth: BirthState::Empty, due: Required::Gone },
        Row { what: "a free number and a birth that names no tick", boot: BootState::ThisBoot, number: NumberState::Free, birth: BirthState::NoTick, due: Required::Gone },
        Row { what: "the writer's number and the birth as the writer writes it", boot: BootState::ThisBoot, number: NumberState::TheWriter, birth: BirthState::Written, due: Required::Alive },
        Row { what: "the writer's number and the birth in another spelling", boot: BootState::ThisBoot, number: NumberState::TheWriter, birth: BirthState::Respelled, due: Required::Unknown },
        Row { what: "the writer's number and padding before the birth's colon", boot: BootState::ThisBoot, number: NumberState::TheWriter, birth: BirthState::PaddedTag, due: Required::Unknown },
        Row { what: "the writer's number and a blank birth", boot: BootState::ThisBoot, number: NumberState::TheWriter, birth: BirthState::Blank, due: Required::Unknown },
        Row { what: "the writer's number and an empty birth", boot: BootState::ThisBoot, number: NumberState::TheWriter, birth: BirthState::Empty, due: Required::Unknown },
        Row { what: "the writer's number and a birth that names no tick", boot: BootState::ThisBoot, number: NumberState::TheWriter, birth: BirthState::NoTick, due: Required::Unknown },
        Row { what: "a later process's number and the birth as the writer writes it", boot: BootState::ThisBoot, number: NumberState::ALaterProcess, birth: BirthState::Written, due: Required::Gone },
        Row { what: "a later process's number and the birth in another spelling", boot: BootState::ThisBoot, number: NumberState::ALaterProcess, birth: BirthState::Respelled, due: Required::Unknown },
        Row { what: "a later process's number and padding before the birth's colon", boot: BootState::ThisBoot, number: NumberState::ALaterProcess, birth: BirthState::PaddedTag, due: Required::Unknown },
        Row { what: "a later process's number and a blank birth", boot: BootState::ThisBoot, number: NumberState::ALaterProcess, birth: BirthState::Blank, due: Required::Unknown },
        Row { what: "a later process's number and an empty birth", boot: BootState::ThisBoot, number: NumberState::ALaterProcess, birth: BirthState::Empty, due: Required::Unknown },
        Row { what: "a later process's number and a birth that names no tick", boot: BootState::ThisBoot, number: NumberState::ALaterProcess, birth: BirthState::NoTick, due: Required::Unknown },
        // The boot column, beside the birth this program writes: a boot that ended ends every process in it.
        Row { what: "an ended boot and a free number", boot: BootState::AnEndedBoot, number: NumberState::Free, birth: BirthState::Written, due: Required::Gone },
        Row { what: "an ended boot and the writer's number", boot: BootState::AnEndedBoot, number: NumberState::TheWriter, birth: BirthState::Written, due: Required::Gone },
        Row { what: "an ended boot and a later process's number", boot: BootState::AnEndedBoot, number: NumberState::ALaterProcess, birth: BirthState::Written, due: Required::Gone },
        // A boot part this reader cannot compare is no end, and hides none: the number still decides.
        Row { what: "this boot with the kernel's dashes and a free number", boot: BootState::ThisBootDashed, number: NumberState::Free, birth: BirthState::Written, due: Required::Gone },
        Row { what: "this boot with the kernel's dashes and the writer's number", boot: BootState::ThisBootDashed, number: NumberState::TheWriter, birth: BirthState::Written, due: Required::Unknown },
        Row { what: "this boot with the kernel's dashes and a later process's number", boot: BootState::ThisBootDashed, number: NumberState::ALaterProcess, birth: BirthState::Written, due: Required::Unknown },
        Row { what: "a boot part that names no boot and a free number", boot: BootState::Unreadable, number: NumberState::Free, birth: BirthState::Written, due: Required::Gone },
        Row { what: "a boot part that names no boot and the writer's number", boot: BootState::Unreadable, number: NumberState::TheWriter, birth: BirthState::Written, due: Required::Unknown },
        Row { what: "a boot part that names no boot and a later process's number", boot: BootState::Unreadable, number: NumberState::ALaterProcess, birth: BirthState::Written, due: Required::Unknown },
        Row { what: "no boot part and a free number", boot: BootState::Absent, number: NumberState::Free, birth: BirthState::Written, due: Required::Gone },
        Row { what: "no boot part and the writer's number", boot: BootState::Absent, number: NumberState::TheWriter, birth: BirthState::Written, due: Required::Unknown },
        Row { what: "no boot part and a later process's number", boot: BootState::Absent, number: NumberState::ALaterProcess, birth: BirthState::Written, due: Required::Unknown },
        // A number no process has ends the record whatever else the text says, and the birth cannot soften an ended boot.
        Row { what: "a boot part that names no boot, a free number and a blank birth", boot: BootState::Unreadable, number: NumberState::Free, birth: BirthState::Blank, due: Required::Gone },
        Row { what: "an ended boot, the writer's number and padding before the birth's colon", boot: BootState::AnEndedBoot, number: NumberState::TheWriter, birth: BirthState::PaddedTag, due: Required::Gone },
        Row { what: "no boot part, a free number and a birth that names no tick", boot: BootState::Absent, number: NumberState::Free, birth: BirthState::NoTick, due: Required::Gone },
    ];
    // Every row is asked and every wrong answer is kept, so one run of this table names each row the rule did not reach.
    let mut wrong: Vec<String> = Vec::new();
    for row in &rows {
        let holder: String = holder_text_of(row, live_birth.as_str(), free, own);
        let gone: bool = proven_gone(holder.as_str());
        let expected_gone: bool = row.due == Required::Gone;
        if gone != expected_gone {
            wrong.push(format!("{}: the reader answered {} for {holder}, and the rule requires {:?}", row.what, match gone { true => "gone", false => "not gone" }, row.due));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
