//! Controls for the clause a command reads when its wait for the commit service ended:
//!  every answer of the manager and every request of this command has its own words,
//!  the unit this command asked about is named,
//!  and no cell claims something the wait did not learn.

use super::*;

/// The unit every control here asked the manager about.
const UNIT: &str = "monochromatic-git-policy-cli@0123456789abcdef0123456789abcdef.service";

/// What:
///  Every state of the wait reads as itself,
///  and the two answers that are not states are never told alike.
/// Why:
///  A unit the manager reports as `activating` is one whose start is under way,
///  and a manager that answered it holds no such unit answered every poll:
///  a sentence that read either as "nothing asked" or as "no answer" sends a reader after the wrong fault.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(didNotStart(unit, { kind: 'state', text: 'activating' }, 'nothing')).toContain('under way');
/// ```
#[test]
fn every_state_of_the_wait_reads_as_itself() {
    let activating: String = did_not_start(Some(UNIT), &ManagerLastWords::State(String::from("activating")), RequestedStart::Nothing);
    assert!(activating.contains(UNIT), "the unit is named: {activating}");
    assert!(activating.contains("under way"), "a start the manager reports is said to be under way: {activating}");
    assert!(activating.contains("asked it for nothing itself"), "and what this command asked is said: {activating}");
    let not_held: String = did_not_start(Some(UNIT), &ManagerLastWords::NotHeld, RequestedStart::Start);
    assert!(not_held.contains(UNIT), "the unit is named: {not_held}");
    assert!(not_held.contains("holds no such unit"), "the manager's own answer is carried: {not_held}");
    assert!(not_held.contains("asked it to start"), "and the start this command asked for: {not_held}");
    assert!(!not_held.contains("no answer"), "and never that the manager was silent: {not_held}");
    let silent: String = did_not_start(Some(UNIT), &ManagerLastWords::Silent, RequestedStart::Start);
    assert!(silent.contains("gave no answer"), "a manager that answered nothing says so: {silent}");
    let failed: String = did_not_start(Some(UNIT), &ManagerLastWords::State(String::from("failed")), RequestedStart::ResetAndStart);
    assert!(failed.contains("forget that"), "the reset this command asked for is named: {failed}");
    let restarted: String = did_not_start(Some(UNIT), &ManagerLastWords::State(String::from("active")), RequestedStart::Restart);
    assert!(restarted.contains("to restart"), "the restart this command asked for is named: {restarted}");
    let no_manager: String = did_not_start(None, &ManagerLastWords::Silent, RequestedStart::Nothing);
    assert!(no_manager.contains("no service manager"), "a wait with no manager says so: {no_manager}");
}

/// What:
///  Every clause avoids the implementation words a person is never told
///  (`service_messages::DENYLIST`).
/// Why:
///  The clause is printed inside the message of a failure, which the denylist guards,
///  and a word that names how the service works is what the rules forbid there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const clause of allClauses()) expect(denylistHit(clause)).toBeUndefined();
/// ```
#[test]
fn no_clause_names_how_the_service_works() {
    let mut clauses: Vec<String> = Vec::new();
    for answer in [ManagerLastWords::State(String::from("activating")), ManagerLastWords::State(String::from("inactive")), ManagerLastWords::NotHeld, ManagerLastWords::Silent] {
        for asked in [RequestedStart::Nothing, RequestedStart::Start, RequestedStart::ResetAndStart, RequestedStart::Restart] {
            clauses.push(did_not_start(Some(UNIT), &answer, asked));
        }
    }
    clauses.push(did_not_start(None, &ManagerLastWords::Silent, RequestedStart::Nothing));
    for clause in clauses {
        for forbidden in crate::service_messages::DENYLIST {
            assert!(!clause.to_lowercase().contains(forbidden), "{forbidden:?} is a word the person is never told: {clause}");
        }
    }
}
