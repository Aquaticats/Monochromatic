//! What:
//!  What the stop's own answer becomes in this folder's words:
//!  the refusal's word,
//!  the locked holder's sentence,
//!  and the journal line a stopped unit leaves.
//! Why:
//!  The stop lives in `process/a_service_unit_stopped_through_the_accounts_manager.rs`
//!  because it asks the account's own user manager over the session bus,
//!  and the pair of folders `process` and `service` is one the layout allow
//!  list already tolerates while this folder and that bus's folder are not
//!  (`doc/decision/cli-git-layout-rules-on-with-a-shrinking-allow-list.md`,
//!  "Decision").
//!  So the word a receipt carries,
//!  the sentence a person reads and the line the journal keeps are this
//!  folder's,
//!  and these controls hold the stop's answer to them.
//!  Never part of the release executable.

use crate::process::a_service_unit_stopped_through_the_accounts_manager::{
    StoppedUnit,
    UnitNotStopped,
};
use crate::worktree_clone_forced_removal::forced_removal_words::RefusalWord;
use crate::worktree_clone_forced_removal::the_acts_journal_lines::{
    AStoppedUnit,
    TheActsJournal,
};
use crate::worktree_clone_forced_removal::the_refusal_sentences::{
    TheHolder,
    WhyNotStopped,
    a_locked_holder,
};
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;

/// The slot the removal is about,
/// as the sentence spells it.
const SLOT: &str = "one";

/// The unit these controls read about,
/// as the manager spells it.
const UNIT: &str = "forced-act-holder.service";

/// The unit's own control-group path,
/// as the task's line printed it.
const CGROUP: &str = "/user.slice/user-0.slice/user@0.service/app.slice/forced-act-holder.service";

/// What:
///  One answer a stop may give,
///  with the manager's words the other three cannot write.
/// Why:
///  The sentence repeats what the manager or the wait said,
///  so the control hands in the answer it is about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = { kind: 'the-stop-refused', unit: UNIT, said: 'the manager answered ...' };
/// ```
fn an_answer() -> UnitNotStopped {
    return UnitNotStopped::TheStopRefused {
        unit: String::from(UNIT),
        said: String::from("the manager answered org.freedesktop.systemd1.NoSuchUnit: gone"),
    };
}

/// The stop's word is the locked holder's own word in this folder's list,
/// so a receipt's failed step says what the design says.
#[test]
fn the_stops_word_is_this_folders_locked_holder_word() {
    let answer: UnitNotStopped = an_answer();
    assert_eq!(answer.word(), RefusalWord::HolderLocked.word());
    assert_eq!(
        RefusalWord::parse(answer.word()),
        Some(RefusalWord::HolderLocked)
    );
}

/// The stop's answer is the sentence's stop-failed clause,
/// and the sentence a person reads carries the manager's own words and the
/// command that stops the unit.
#[test]
fn the_stops_answer_is_the_sentences_stop_failed_clause() {
    let answer: UnitNotStopped = an_answer();
    // What: the manager's words, held in a value the sentence may borrow.
    // Why:  the sentence holds a reference, and the answer's own text must outlive it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const said = answer.said();
    // ```
    let said: String = answer.said();
    let why: WhyNotStopped<'_> = WhyNotStopped::TheStopFailed {
        said: said.as_str(),
    };
    let sentence: String = a_locked_holder(
        SLOT,
        &TheHolder {
            noun: "unit",
            named: UNIT,
        },
        "mnt:[4026532001]",
        "/checkout/one",
        11,
        &why,
    );
    assert!(sentence.contains(answer.said().as_str()), "{sentence}");
    assert!(sentence.contains(UNIT), "{sentence}");
    assert!(
        sentence.contains("systemctl --user stop"),
        "the sentence names the command that stops it: {sentence}"
    );
    assert!(
        sentence.contains("Nothing was moved, unmounted or destroyed"),
        "the sentence says what was not changed: {sentence}"
    );
}

/// A stopped unit carries the three values the journal's own line prints,
/// and the line names the unit,
///  its control group and the namespace.
#[test]
fn a_stopped_unit_reaches_the_journals_stopped_unit_line() {
    // What: the stop's own answer, as the act would record it.
    // Why:  the journal line and the stop must spell the same three values.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const stopped = { unit: UNIT, cgroup: CGROUP, namespace: 'mnt:[4026532001]' };
    // ```
    let stopped: StoppedUnit = StoppedUnit {
        unit: String::from(UNIT),
        cgroup: String::from(CGROUP),
        namespace: String::from("mnt:[4026532001]"),
    };
    let nonce: RequestNonce =
        RequestNonce::parse("0123456789abcdef0123456789abcdef").expect("32 lowercase hex digits");
    let journal: TheActsJournal<'_> = TheActsJournal {
        dataset: "tank/clones/one",
        nonce: &nonce,
    };
    let line: String = journal.a_stopped_unit(&AStoppedUnit {
        unit: stopped.unit.as_str(),
        cgroup: stopped.cgroup.as_str(),
        namespace: stopped.namespace.as_str(),
    });
    assert!(line.contains("unit-stopped"), "{line}");
    assert!(line.contains(UNIT), "{line}");
    assert!(line.contains(CGROUP), "{line}");
    assert!(line.contains("mnt:[4026532001]"), "{line}");
    assert!(line.contains("start-the-unit"), "{line}");
}
