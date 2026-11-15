//! Controls for the clause a command reads when its wait for the commit service ended:
//!  every state of the wait has its own words,
//!  each names the unit this command asked about,
//!  and none of them says a start was asked for when none was.

use super::*;

/// The unit every control here asked the manager about.
const UNIT: &str = "monochromatic-git-policy-cli@0123456789abcdef0123456789abcdef.service";

/// What:
///  Each state of the wait reads as itself:
///  a start that was asked for and one that was not are never told alike.
/// Why:
///  The two have different next steps,
///  and a reader sent after the wrong one is what left the merges of 2026-11-14T20:51-05:00 unexplained.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theStartThatDidNotCome(unit, 'failed', true)).toContain('was asked to start');
/// ```
#[test]
fn every_state_of_the_wait_reads_as_itself() {
    let asked_and_failed: String = the_start_that_did_not_come(Some(UNIT), Some("failed"), true);
    assert!(asked_and_failed.contains(UNIT), "the unit is named: {asked_and_failed}");
    assert!(asked_and_failed.contains("was asked to start"), "the start is named as asked for: {asked_and_failed}");
    assert!(asked_and_failed.contains("is failed"), "the manager's own word is carried: {asked_and_failed}");
    let quietly_inactive: String = the_start_that_did_not_come(Some(UNIT), Some("inactive"), false);
    assert!(quietly_inactive.contains(UNIT), "the unit is named: {quietly_inactive}");
    assert!(quietly_inactive.contains("nothing asked it to start"), "no start is claimed: {quietly_inactive}");
    let asked_and_silent: String = the_start_that_did_not_come(Some(UNIT), None, true);
    assert!(asked_and_silent.contains("was asked to start"), "the start is named as asked for: {asked_and_silent}");
    assert!(asked_and_silent.contains("gave no answer"), "the silence is named: {asked_and_silent}");
    let unreachable: String = the_start_that_did_not_come(Some(UNIT), None, false);
    assert!(unreachable.contains(UNIT), "the unit is named: {unreachable}");
    assert!(unreachable.contains("could not be asked"), "the manager's silence is named: {unreachable}");
    let no_manager_at_all: String = the_start_that_did_not_come(None, None, false);
    assert!(no_manager_at_all.contains("no service manager"), "a wait with no manager says so: {no_manager_at_all}");
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
    for clause in [
        the_start_that_did_not_come(Some(UNIT), Some("failed"), true),
        the_start_that_did_not_come(Some(UNIT), Some("inactive"), false),
        the_start_that_did_not_come(Some(UNIT), None, true),
        the_start_that_did_not_come(Some(UNIT), None, false),
        the_start_that_did_not_come(None, None, false),
    ] {
        for forbidden in crate::service_messages::DENYLIST {
            assert!(!clause.to_lowercase().contains(forbidden), "{forbidden:?} is a word the person is never told: {clause}");
        }
    }
}
