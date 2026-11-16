//! What:
//!  The two facts a held push record carries besides the work item itself:
//!  which command will decide the landed
//!  commit's post-commit checks,
//!  and the verdict that command left when it decided.
//! Why:
//!  A held record waits for the command that landed the commit to run its post-commit gate.
//! While that command
//!  lives,
//! the wait is its own;
//! when the command is gone,
//! nothing in the process table will ever promote or forget the
//!  record,
//! and the branch's backup waits on a command that cannot answer
//! (`doc/handover/questions-for-the-owner.md`,
//! "A commit whose command died before its checks ran now stays local").
//!  A record that says who would decide,
//! and what it decided before it stopped,
//! is settled from its own words instead of from a guess:
//!  a recorded pass is released,
//! a recorded refusal is kept and named,
//! and a record with no verdict stays exactly where the
//!  owner's rule puts it,
//! with the words naming what holds it and what sends it.
//!
//!  The holder is spelled `<boot id>/<pid>/<birth identity>`,
//! where the birth identity is the one every lock of this program records
//! (`process_identity.rs`):
//!  the PID alone would say nothing once the kernel reuses it,
//! and the boot id keeps a record of an earlier boot from naming a live process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const holding = holdingIn(recordText); const gone = provenGone(holding.holder);
//! ```

use crate::claim_time::{current_boot_id, is_boot_id};
use crate::process_identity::{birth_text, current_birth_identity, process_birth_identity};
use serde_json::{Map, Value};

/// The record field naming the command that will decide the commit's checks.
pub const HOLDER_FIELD: &str = "holder";

/// The record field naming what that command decided.
pub const VERDICT_FIELD: &str = "verdict";

/// The record field holding the gate's own words when it refused the commit.
pub const REASON_FIELD: &str = "reason";

/// What a command decided about the landed commit's post-commit checks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Verdict = 'undecided' | 'passed' | 'blocked';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// No command has decided yet,
    /// or the record is one an earlier release wrote.
    Undecided,
    /// The post-commit gate approved the backup,
    /// so the push may run.
    Passed,
    /// The post-commit gate refused the backup,
    /// so the commit stays local.
    Blocked,
}

impl Verdict {
    /// The word the record holds.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// verdictName(verdict) // 'undecided' | 'passed' | 'blocked'
    /// ```
    pub fn name(self) -> &'static str {
        return match self {
            Verdict::Undecided => "undecided",
            Verdict::Passed => "passed",
            Verdict::Blocked => "blocked",
        };
    }

    /// The verdict a record's word names;
    /// a word this release does not know is no verdict,
    /// which keeps the record held and named.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// verdictOf('passed') // Verdict.Passed
    /// ```
    pub fn of(word: &str) -> Verdict {
        return match word {
            "passed" => Verdict::Passed,
            "blocked" => Verdict::Blocked,
            _ => Verdict::Undecided,
        };
    }
}

/// What:
///  The holding one held record states:
/// who will decide,
/// what was decided,
/// and the gate's words when it refused.
/// Why:
///   A record an earlier release wrote states none of this,
/// and reads as undecided with no holder,
/// which is exactly
///  what its silence means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Holding = { holder: string; verdict: Verdict; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Holding {
    /// The command that will decide the commit's checks,
    /// or empty when no record names one.
    pub holder: String,
    /// What that command decided.
    pub verdict: Verdict,
    /// The gate's own words when it refused the commit.
    pub reason: String,
}

impl Holding {
    /// A record that says nothing about its command or its verdict.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const undecided = { holder: '', verdict: 'undecided', reason: '' };
    /// ```
    pub fn undecided() -> Holding {
        return Holding { holder: String::new(), verdict: Verdict::Undecided, reason: String::new() };
    }
}

/// What:
///  This command's own holder text,
/// the one a record it writes names.
/// Why:
///   The hold is written before the answer reaches the command,
/// so only the command itself can say who will decide the checks;
/// a
///  reading that fails leaves the field empty,
/// which never releases a record and never hides it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holder = `${bootId()}/${process.pid}/${birthIdentity()}`;
/// ```
pub fn this_command() -> String {
    let Ok(birth) = current_birth_identity() else {
        return String::new();
    };
    return format!("{}/{}/{}", current_boot_id(), std::process::id(), birth);
}

/// What:
///  The holding a record's text states,
/// with every field it does not state left as [`Holding::undecided`] leaves it.
/// Why:
///   The record is one JSON object per held push,
/// so the item and the holding are read from the same bytes and cannot disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holding = holdingIn(text); // { holder: '', verdict: 'undecided', reason: '' } when absent
/// ```
/// The text one field of a record holds,
/// or empty when it holds none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fieldText = (fields, name) => fields[name] ?? '';
/// ```
fn field_text(fields: &Map<String, Value>, name: &str) -> String {
    return fields.get(name).and_then(Value::as_str).unwrap_or_default().to_owned();
}

pub fn holding_in(text: &str) -> Holding {
    let Ok(value) = serde_json::from_str::<Value>(text) else {
        return Holding::undecided();
    };
    let Some(fields) = value.as_object() else {
        return Holding::undecided();
    };
    let verdict: String = field_text(fields, VERDICT_FIELD);
    return Holding { holder: field_text(fields, HOLDER_FIELD), verdict: Verdict::of(verdict.as_str()), reason: field_text(fields, REASON_FIELD) };
}

/// What:
///  The same record with the holding written into it,
/// or nothing when the text is not one JSON object.
/// Why:
///   The holding is published by replacing the record whole,
/// so a reader sees the item it already knew and the new words together,
/// never one
///  without the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withHolding(text, 'boot/pid/linux:42', 'passed', '') // one JSON line
/// ```
pub fn with_holding(text: &str, holding: &Holding) -> Option<String> {
    let value: Value = serde_json::from_str(text).ok()?;
    let mut fields: Map<String, Value> = value.as_object()?.clone();
    fields.insert(String::from(HOLDER_FIELD), Value::from(holding.holder.as_str()));
    fields.insert(String::from(VERDICT_FIELD), Value::from(holding.verdict.name()));
    fields.insert(String::from(REASON_FIELD), Value::from(holding.reason.as_str()));
    return Some(format!("{}\n", Value::Object(fields)));
}

/// What:
///  The process number a holder text names,
/// for words that ask the person to look at that command.
/// Why:
///   The number is part of the text,
/// and a text this release cannot read names no command,
/// which the words say instead of
///  inventing one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pid = pidOf(holder); // undefined for an unreadable text
/// ```
pub fn pid_of(holder: &str) -> Option<i64> {
    let mut parts = holder.split('/');
    let _boot: &str = parts.next()?;
    let pid: &str = parts.next()?;
    let Ok(number) = pid.parse::<i64>() else {
        return None;
    };
    // Zero names no process, so a text that holds it names no command.
    return (number > 0).then_some(number);
}

/// What:
///  Whether the command a holder text names is provably gone,
/// so nothing it would have decided can be expected.
/// Why:
///   Only a proof settles a record:
/// a text this release cannot read,
/// a part that is not the value it stands for in the spelling this program writes that value,
/// a `/proc` that refuses an answer,
/// and a process of another boot
///  number all keep the record as it is.
/// A part of a holder stands for one value in one spelling,
/// and two spellings of one boot or one birth compare as two of them,
/// which proves a command gone that runs and settles a held push its own gate has not decided
/// (`service_handoff.rs`,
/// `held_push_line`,
/// `work_item_holding.rs`,
/// `undecided_commits`).
/// So a boot part is a boot ID (`claim_time`,
/// `is_boot_id`) and a birth part is a birth this reader can compare (`process_identity`,
/// `birth_text`),
/// and a part in any other shape is no part this reader can compare.
///  What each part can decide on its own decides first,
/// because the rules pull against each other:
/// nothing may prove a live holder gone,
/// and a holder that can never run must not keep a record that no command can settle.
/// A boot that has ended ends every process in it,
/// whatever the birth says;
/// a PID that names no process is the command's own end,
/// whether
///  it finished,
/// crashed or was killed,
/// whatever the birth says;
/// and only where a process has the number does the birth decide,
///  because that is the one place a birth is compared with anything.
///  A boot part this reader cannot read is neither an end nor a hiding place for one:
/// the number's own proof still stands,
/// which is what a command proven dead by its absent number needs.
///  Where the number names a process,
/// the boot and the birth must both be spellings this reader can compare:
/// a boot it cannot place leaves open that the number belongs to another boot or machine,
/// and a birth it cannot compare says nothing about that process.
/// A record that names no boot is settled by a number that answers nothing,
/// and by nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gone = bootEnded(boot) || pidIsFree(pid) || (bootIsThisOne(boot) && canCompare(birth) && birthIdentityOf(pid) !== birth);
/// ```
pub fn proven_gone(holder: &str) -> bool {
    return holder_answer(holder) == HolderAnswer::Gone;
}

/// What a holder text says about the command that wrote it:
/// that it is gone,
/// that it is the process that has the number now,
/// or that this reader cannot tell.
/// The project decided the last one on 2026-11-16 ("A: stays, command says how"):
/// the record stays,
/// and past a patience a command prints which record it is,
/// whose,
/// how old,
/// and the exact `rm -- <path>` (`doc/handover/cli-git-rust-implementation.md`,
/// "Owner decisions in force").
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HolderAnswer {
    /// Nothing the text names runs now.
    Gone,
    /// The number names the very process the text names.
    Alive,
    /// The text cannot be compared with what the process table answers,
    /// so nothing shows whether its command still runs.
    CannotBeTold,
}

/// What `holder` answers,
/// by the rule `proven_gone` states.
pub fn holder_answer(holder: &str) -> HolderAnswer {
    if holder.is_empty() {
        return HolderAnswer::CannotBeTold;
    }
    let mut parts = holder.split('/');
    let (Some(boot), Some(pid), Some(birth)) = (parts.next(), parts.next(), parts.next()) else {
        return HolderAnswer::CannotBeTold;
    };
    let here: &str = current_boot_id();
    // A boot part this reader can place: a boot ID in the one spelling both are written in, on a reader that knows its
    //  own boot. A part this reader cannot read is not a boot of this host, and a record that names no boot is not
    //  placed on this one either.
    let boot_is_a_boot_id: bool = !boot.is_empty() && is_boot_id(boot) && !here.is_empty();
    // The boot part proves an end on its own only when it is a boot ID of another boot: a boot that ended ends every
    //  process in it. A part this reader cannot read proves nothing here, and it hides nothing either, so a number that
    //  answers nothing still ends the record below.
    let boot_is_this_one: bool = boot_is_a_boot_id && boot == here;
    if boot_is_a_boot_id && !boot_is_this_one {
        return HolderAnswer::Gone;
    }
    let Ok(number) = pid.parse::<i64>() else {
        return HolderAnswer::CannotBeTold;
    };
    // Zero names no process, so a text that holds it names no command and proves nothing.
    if number <= 0 {
        return HolderAnswer::CannotBeTold;
    }
    return match process_birth_identity(number) {
        // No process has this number now, so the command that wrote the record has ended whatever its other parts say.
        Ok(None) => HolderAnswer::Gone,
        // The number names a process, so the rest of the text decides: it is another process only when the boot is this
        //  one and the birth is a spelling this reader can compare, and otherwise nothing is shown either way.
        Ok(Some(_now)) if !boot_is_this_one || !birth_text(birth).can_be_compared() => HolderAnswer::CannotBeTold,
        Ok(Some(now)) if now == birth => HolderAnswer::Alive,
        Ok(Some(_reused)) => HolderAnswer::Gone,
        // The process table could not be read, so nothing is shown either way.
        Err(_unreadable) => HolderAnswer::CannotBeTold,
    };
}

/// Controls for the holder and the verdict stay out of the release executable.
#[cfg(test)]
#[path = "held_push_holder_and_verdict_tests.rs"]
mod tests;
