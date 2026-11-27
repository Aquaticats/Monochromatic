//! Controls for the walk to the first process:
//!  a chain reads whole and its protected set spares the session and the ancestors,
//!  a caller whose number names another process is gone,
//!  a parent that changed between the two reads is refused,
//!  a start time that runs backwards is refused,
//!  and a refused read is blind.

use crate::worktree_clone_forced_removal::caller_chain_walk::*;
use crate::worktree_clone_forced_removal::caller_property::CallerIdentity;
use crate::worktree_clone_forced_removal::forced_removal_words::RefusalWord;
use crate::worktree_clone_forced_removal::process_session_record::{
    CommandWord,
    ProcessRecord,
    ReadOutcome,
};
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::collections::BTreeMap;

/// One process record a control's fixture answers.
fn record(
    pid: i32,
    parent: i32,
    session: i32,
    started: u64,
) -> ProcessRecord {
    return ProcessRecord {
        pid,
        command: CommandWord::of("sh"),
        state: 'S',
        parent,
        group: pid,
        session,
        started,
    };
}

/// A caller at 500 whose parent is 400,
/// whose parent is the first process.
fn a_chain() -> BTreeMap<i32, ReadOutcome> {
    let mut table: BTreeMap<i32, ReadOutcome> = BTreeMap::new();
    table.insert(500, ReadOutcome::Found(record(500, 400, 500, 900)));
    table.insert(400, ReadOutcome::Found(record(400, 1, 300, 800)));
    return table;
}

/// The identity a caller property would carry for that chain.
fn an_identity() -> CallerIdentity {
    return CallerIdentity {
        nonce: RequestNonce::parse("0123456789abcdef0123456789abcdef").expect("a nonce"),
        pid: 500,
        started: 900,
        parent: 400,
        parent_started: 800,
        boot: String::from("6f0f1a2b-3c4d-4e5f-8a9b-0c1d2e3f4a5b"),
    };
}

/// A reader over one fixture table,
/// answering gone for a number the table lacks.
fn reader(table: BTreeMap<i32, ReadOutcome>) -> impl Fn(i32) -> ReadOutcome {
    return move |pid: i32| return table.get(&pid).cloned().unwrap_or(ReadOutcome::Gone);
}

/// The walk reads the chain and its protected set spares the caller,
/// the session's tasks and the ancestors,
/// and spares nothing else.
#[test]
fn the_walk_reads_the_chain_and_its_protected_set() {
    let chain: CallerChain =
        walk_the_caller_chain(reader(a_chain()), &an_identity()).expect("the chain reads");
    assert_eq!(chain.caller.pid, 500);
    assert_eq!(chain.ancestors.len(), 1);
    assert_eq!(chain.ancestors[0].pid, 400);
    assert!(chain.spares(&record(500, 400, 500, 900)));
    assert_eq!(
        chain.why_spared(&record(500, 400, 500, 900)),
        Some(SparedBy::Caller)
    );
    // A task of the caller's session, holding nothing to do with the chain.
    assert!(chain.spares(&record(501, 1, 500, 950)));
    assert_eq!(
        chain.why_spared(&record(501, 1, 500, 950)),
        Some(SparedBy::Session)
    );
    // The ancestor, by number and start time.
    assert_eq!(
        chain.why_spared(&record(400, 1, 300, 800)),
        Some(SparedBy::Ancestor)
    );
    // A process that took the ancestor's number later is not the ancestor.
    assert_eq!(chain.why_spared(&record(400, 1, 300, 801)), None);
    // A holder of another session is not spared.
    assert_eq!(chain.why_spared(&record(600, 1, 600, 900)), None);
}

/// A wrapper whose number now names another process is gone,
/// and so is one whose recorded parent has gone.
#[test]
fn a_reused_or_missing_process_is_gone() {
    let mut elsewhere: BTreeMap<i32, ReadOutcome> = a_chain();
    elsewhere.insert(500, ReadOutcome::Found(record(500, 400, 500, 999)));
    assert_eq!(
        walk_the_caller_chain(reader(elsewhere), &an_identity()),
        Err(ChainRefusal::CallerGone)
    );
    let mut without_parent: BTreeMap<i32, ReadOutcome> = a_chain();
    without_parent.remove(&400);
    assert_eq!(
        walk_the_caller_chain(reader(without_parent), &an_identity()),
        Err(ChainRefusal::ParentGone)
    );
}

/// A wrapper whose record names another parent than the property is refused before the walk.
#[test]
fn a_changed_parent_is_refused() {
    let mut changed: BTreeMap<i32, ReadOutcome> = a_chain();
    changed.insert(500, ReadOutcome::Found(record(500, 401, 500, 900)));
    assert_eq!(
        walk_the_caller_chain(reader(changed), &an_identity()),
        Err(ChainRefusal::ParentChanged {
            recorded: 400,
            found: 401
        })
    );
}

/// A parent whose start time is later than its child's is refused,
/// which the kernel cannot record for a real parent.
#[test]
fn a_start_time_that_runs_backwards_is_refused() {
    let mut backwards: BTreeMap<i32, ReadOutcome> = a_chain();
    backwards.insert(400, ReadOutcome::Found(record(400, 1, 300, 901)));
    assert_eq!(
        walk_the_caller_chain(reader(backwards), &an_identity()),
        Err(ChainRefusal::StartOrder {
            child: 500,
            child_started: 900,
            parent_started: 901
        })
    );
}

/// A parent whose number is the recorded one and whose start time is not is not the recorded parent.
#[test]
fn the_recorded_parents_start_time_is_proved() {
    let mut replaced: BTreeMap<i32, ReadOutcome> = a_chain();
    replaced.insert(400, ReadOutcome::Found(record(400, 1, 300, 700)));
    assert_eq!(
        walk_the_caller_chain(reader(replaced), &an_identity()),
        Err(ChainRefusal::ParentGone)
    );
}

/// A process that took the caller's number after it is not the caller,
/// so the protected set does not spare it.
#[test]
fn a_reused_callers_number_is_not_spared() {
    let chain: CallerChain =
        walk_the_caller_chain(reader(a_chain()), &an_identity()).expect("the chain reads");
    // Another session, so only the caller's own half could spare it, and its start time is not the caller's.
    assert_eq!(chain.why_spared(&record(500, 1, 777, 901)), None);
    assert!(!chain.spares(&record(500, 1, 777, 901)));
}

/// A chain whose records name each other in a way that never reaches the first process is refused,
/// and not read as a set that stops at the caller.
#[test]
fn a_chain_that_never_reaches_the_first_process_is_refused() {
    let mut looping: BTreeMap<i32, ReadOutcome> = BTreeMap::new();
    looping.insert(500, ReadOutcome::Found(record(500, 400, 500, 900)));
    looping.insert(400, ReadOutcome::Found(record(400, 400, 300, 800)));
    assert_eq!(
        walk_the_caller_chain(reader(looping), &an_identity()),
        Err(ChainRefusal::TooDeep)
    );
}

/// A refused read is a blind spot and not an absence.
#[test]
fn a_refused_read_is_blind() {
    let mut blinded: BTreeMap<i32, ReadOutcome> = a_chain();
    blinded.insert(
        400,
        ReadOutcome::Blind {
            errno: 13,
            word: RefusalWord::HolderBlind,
        },
    );
    assert_eq!(
        walk_the_caller_chain(reader(blinded), &an_identity()),
        Err(ChainRefusal::Blind {
            pid: 400
        })
    );
}
