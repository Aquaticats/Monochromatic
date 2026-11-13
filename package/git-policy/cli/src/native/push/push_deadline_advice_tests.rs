//! Controls for the words that say how to change the push deadline.

use super::*;

/// What:
///  In a terminal the variable is the remedy;
///  a person reading a message from inside the service is told to set it for the command that starts the service and to end
///  the one that is running,
///  because a service keeps the environment it was started with.
/// Why:
///  The wrapper starts the service now, so there is no manager to set an environment on:
/// the old words named `systemctl --user set-environment`, which no longer reaches anything.
#[test]
fn the_service_advice_reaches_the_service_and_the_terminal_advice_names_the_variable() {
    let terminal: String = extend_clause_for(false);
    assert_eq!(terminal, format!("set {PUSH_DEADLINE_VARIABLE} to allow longer"));
    let service: String = extend_clause_for(true);
    assert!(service.contains(format!("export {PUSH_DEADLINE_VARIABLE}=").as_str()), "{service}");
    assert!(service.contains(RESTART_COMMAND), "{service}");
    assert!(!service.contains("systemctl"), "no manager's command is offered: {service}");
    assert!(!service.contains(format!("set {PUSH_DEADLINE_VARIABLE} to allow longer").as_str()), "{service}");
    assert_eq!(limit_clause_for(false), format!("{PUSH_DEADLINE_VARIABLE} sets the limit"));
    assert_eq!(limit_clause_for(true), service);
}

/// The test program is not the service,
/// so the running program's advice is the terminal's.
#[test]
fn a_program_that_is_not_the_service_gets_the_terminal_advice() {
    assert_eq!(extend_clause(), extend_clause_for(false));
    assert_eq!(limit_clause(), limit_clause_for(false));
}
