//! What:
//!  The comparison of the bounded receipt protocols (`model_helper_bound_r20.rs`): for each, whether some schedule leaves a receipt with no
//!  executor, the most processes alive for the lane at once, and the most synchronisations a client does in the foreground.
//! Why:
//!  The owner picks the live-helper bound from this table (handover, round 18). The numbers are asserted so that a change to a protocol or
//!  to the model changes the table on purpose.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('the built protocol bounds helpers by clients', () => { expect(report(Built).maxLive).toBe(CLIENTS); });
//! ```

use crate::model_helper_bound_r20::{CLIENTS, Findings, Protocol, report};

/// The report of `protocol`, refusing a search that did not cover every state.
fn run(protocol: Protocol) -> Findings {
    let found: Findings = report(protocol);
    // A search that stops at a miss has not covered every state; the numbers of a miss are those of the states seen until it.
    assert!(found.exhausted || found.miss.is_some(), "{:?} did not exhaust its states", protocol);
    return found;
}

/// The built release, recheck and exit: no miss, and as many live helpers as there are clients that launch in the release step (the bound
/// grows with the clients, which is the finding of review 11).
#[test]
fn the_built_protocol_misses_nothing_and_its_helpers_grow_with_the_clients() {
    let found: Findings = run(Protocol::Built);
    assert_eq!(found.miss, None);
    assert_eq!(found.max_live, CLIENTS);
    assert_eq!(found.max_client_syncs, 0);
}

/// The reviewer's singleton retry owner: no miss; one owner and one worker alive at most; no foreground synchronisation.
#[test]
fn the_singleton_retry_owner_misses_nothing_and_keeps_two_processes() {
    let found: Findings = run(Protocol::SingletonOwner);
    assert_eq!(found.miss, None, "{:?}", found.miss);
    assert_eq!(found.max_live, 2);
    assert_eq!(found.max_client_syncs, 0);
}

/// Inline forcing only while sealed: it still misses when no service runs and the lane is not sealed.
#[test]
fn forcing_inline_only_while_sealed_still_misses_when_no_service_runs() {
    let found: Findings = run(Protocol::SealedInline);
    let trace: Vec<String> = found.miss.expect("a miss");
    assert!(trace.iter().any(|step| { return step.contains("leaves its receipt enrolled"); }), "{:?}", trace);
    assert_eq!(found.max_live, 1);
}

/// Inline forcing whenever the token is held, the token kept to exit: no miss, one process alive at most, and a client that finds the token
/// held pays two synchronisations (the receipt file and its directory).
#[test]
fn forcing_inline_whenever_the_token_is_held_misses_nothing_and_keeps_one_process() {
    let found: Findings = run(Protocol::TokenHeldInline);
    assert_eq!(found.miss, None, "{:?}", found.miss);
    assert_eq!(found.max_live, 1);
    assert_eq!(found.max_client_syncs, 2);
}

/// The baton: no miss, at most one owner and one worker, and an owner only while no service runs.
#[test]
fn the_baton_between_the_service_and_the_retry_owner_misses_nothing() {
    let found: Findings = run(Protocol::Baton);
    assert_eq!(found.miss, None, "{:?}", found.miss);
    assert_eq!(found.max_live, 2);
    assert_eq!(found.max_client_syncs, 0);
}

/// The control: the token kept to exit with nothing to retry misses a receipt.
#[test]
fn the_token_kept_to_exit_with_nothing_to_retry_misses_a_receipt() {
    let found: Findings = run(Protocol::TokenToExit);
    assert!(found.miss.is_some());
    assert_eq!(found.max_live, 1);
}
