//! Controls for the reads that part a locked holder's two routes:
//!  a task inside this account's own manager's service unit names that unit,
//!  a scope of that manager is never stopped,
//!  a holder of another account's manager or of the system manager is not this
//!  case,
//!  a line with no unified hierarchy names no unit,
//!  and a name the manager does not hold is refused rather than stopped.

use crate::process::a_locked_holders_own_unit::*;

/// The account these controls read for,
/// which is the handler's own account.
const OUR_UID: u32 = 0;

/// What:
///  One control-group line as the kernel prints it for a task of a transient
///  unit of this account's own manager.
/// Why:
///  The design's read takes the line's last segment and checks the path runs
///  through `user@<our number>.service`,
///  so a real line is the input the read is about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = '0::/user.slice/user-0.slice/user@0.service/app.slice/forced-act-holder.service';
/// ```
fn a_line_of_our_managers_unit() -> &'static str {
    return "0::/user.slice/user-0.slice/user@0.service/app.slice/forced-act-holder.service";
}

/// A task in this account's own manager's service unit names that unit.
#[test]
fn a_task_of_our_managers_service_unit_names_it() {
    let unit: String = the_unit_of_the_accounts_manager(a_line_of_our_managers_unit(), OUR_UID)
        .expect("the unit of this account's own manager");
    assert_eq!(unit, "forced-act-holder.service");
}

/// A scope of that manager is never stopped,
/// and the answer says which read said so.
#[test]
fn a_scope_of_that_manager_is_never_stopped() {
    let line: &str = "0::/user.slice/user-0.slice/user@0.service/session-3.scope";
    let refused: NotTheAccountsUnit =
        the_unit_of_the_accounts_manager(line, OUR_UID).expect_err("a scope is not a service unit");
    assert_eq!(refused, NotTheAccountsUnit::NotAServiceUnit);
    assert_eq!(refused.word(), "not-a-service-unit");
}

/// A holder of another account's manager,
/// or of the system manager,
/// is not the case the owner's answer stops.
#[test]
fn a_holder_of_another_manager_is_not_this_case() {
    let another_account: &str =
        "0::/user.slice/user-1000.slice/user@1000.service/app.slice/watcher.service";
    assert_eq!(
        the_unit_of_the_accounts_manager(another_account, OUR_UID)
            .expect_err("another account's unit"),
        NotTheAccountsUnit::NotTheAccountsManager
    );
    let the_system_manager: &str = "0::/system.slice/watcher.service";
    assert_eq!(
        the_unit_of_the_accounts_manager(the_system_manager, OUR_UID)
            .expect_err("the system manager's unit"),
        NotTheAccountsUnit::NotTheAccountsManager
    );
    let our_manager: &str = "0::/user.slice/user-0.slice/user@0.service/app.slice/watcher.service";
    assert_eq!(
        the_unit_of_the_accounts_manager(our_manager, OUR_UID).expect("our own unit"),
        "watcher.service"
    );
}

/// A line that holds no unified hierarchy names no unit,
/// which is the answer a kernel without one leaves.
#[test]
fn a_line_with_no_unified_hierarchy_names_no_unit() {
    let line: &str = "11:cpu,cpuacct:/user.slice/user-0.slice/session-3.scope";
    assert_eq!(
        the_unit_of_the_accounts_manager(line, OUR_UID).expect_err("no unified line"),
        NotTheAccountsUnit::NotTheAccountsManager
    );
}

/// A name the manager answered for is the unit to stop,
/// and a name it answered nothing for is refused.
#[test]
fn a_unit_the_manager_holds_is_stopped_and_one_it_does_not_is_refused() {
    assert_eq!(
        the_unit_the_manager_holds("watcher.service", Some("active"))
            .expect("the manager holds it"),
        "watcher.service"
    );
    let refused: NotTheAccountsUnit = the_unit_the_manager_holds("watcher.service", None)
        .expect_err("the manager does not hold it");
    assert_eq!(refused, NotTheAccountsUnit::TheManagerDoesNotHoldIt);
    assert_eq!(refused.word(), "the-manager-does-not-hold-it");
}

/// The whole read answers the unit of this account's own manager,
/// and the manager's answer decides whether the unit may be stopped.
#[test]
fn the_whole_read_answers_the_unit_or_the_managers_answer() {
    let unit: String =
        a_locked_holders_unit(a_line_of_our_managers_unit(), OUR_UID, Some("inactive"))
            .expect("the unit and the manager's answer together");
    assert_eq!(unit, "forced-act-holder.service");
    let refused: NotTheAccountsUnit =
        a_locked_holders_unit(a_line_of_our_managers_unit(), OUR_UID, None)
            .expect_err("the manager holds no such unit");
    assert_eq!(refused, NotTheAccountsUnit::TheManagerDoesNotHoldIt);
}
