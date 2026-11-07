//! Controls for the words that say how to change the push deadline.

use super::*;

/// In a terminal the variable is the remedy;
/// in the service the user manager's command and the restart are,
/// and the
///  service text never tells a person to set a variable in a terminal.
#[test]
fn the_service_advice_reaches_the_service_and_the_terminal_advice_names_the_variable() {
    let terminal: String = extend_clause_for(false);
    assert_eq!(terminal, format!("set {PUSH_DEADLINE_VARIABLE} to allow longer"));
    let service: String = extend_clause_for(true);
    assert!(service.contains(format!("systemctl --user set-environment {PUSH_DEADLINE_VARIABLE}=").as_str()), "{service}");
    assert!(service.contains(RESTART_COMMAND), "{service}");
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
