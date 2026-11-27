//! What:
//!  Controls for the two facts this session names:
//!  a build made for tests answers a runtime directory and a bus address only from what the control that
//!  started its body handed in,
//!  and never from the environment that body inherited.
//! Why:
//!  A test process's own environment names this account's runtime directory and this account's bus,
//!  so a route that read them would reach the account's own service manager from a test run
//!  (issue 6586).
//!  Each control runs its body in a child of the test binary,
//!  because the environment belongs to the whole process a threaded run shares
//!  (`testing/process_wide_state_child_test_support.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // each control: with the switch set the answer is the switch's, and with it unset the inherited name is ignored
//! ```

use super::{
    BUS_ADDRESS_SWITCH,
    RUNTIME_DIRECTORY_SWITCH,
    bus_address,
    runtime_directory,
};
use crate::test_support::{
    fixture,
    remove,
};
use crate::testing::process_wide_state_child_test_support::run_the_body_with_an_environment_of_its_own;
use std::ffi::OsStr;
use std::path::PathBuf;

/// What:
///  The runtime directory a control hands its body,
///  which is a fixture of its own holding one marker file.
/// Why:
///  The child a control runs its body in makes its own fixtures,
///  so the body cannot compare the answer with a path this process computed:
///  the marker file is a fact of the folder that both processes can see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handedIn = fixture('session-places-handed-in'); writeFileSync(join(handedIn, THE_MARKER), '');
/// ```
fn a_fixture_directory_holding_a_marker(name: &str) -> PathBuf {
    let made: PathBuf = fixture(name);
    std::fs::write(made.join(THE_MARKER), b"").expect("the fixture's marker");
    return made;
}

/// The file a fixture of this file holds so its child can tell which folder answered,
/// whose siblings are the fixtures' own names, which differ between processes.
const THE_MARKER: &str = "this-is-the-folder-the-control-handed-in";

/// What:
///  A build made for tests answers the runtime directory its control handed in,
///  and never the one its body inherited.
/// Why:
///  The inherited name is this account's own runtime directory in every run on this host,
///  so an answer taken from it is the route that hands the account's manager a unit for a fixture.
///  The handed-in folder holds a marker file and the inherited one holds none,
///  so an answer taken from the wrong folder is a folder without the marker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(existsSync(join(runtimeDirectory(), THE_MARKER))).toBe(true);
/// ```
#[test]
fn a_build_made_for_tests_answers_the_runtime_directory_a_control_handed_in() {
    // What:     the folder the control hands in through the switch, holding its marker, and the one its body
    //           inherits, which holds nothing.
    // Why:      the inherited one stands for this account's runtime directory, which the answer must not be.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const handedIn = fixtureDirectoryHoldingAMarker('session-places-handed-in'); const inherited = fixture('session-places-inherited');
    // ```
    let handed_in: PathBuf = a_fixture_directory_holding_a_marker("session-places-handed-in");
    let inherited: PathBuf = fixture("session-places-inherited");
    // What:     the environment the body is given: the switch names `handed_in`, and the inherited variable
    //           names `inherited` so an answer taken from it is the other folder.
    // Why:      a body whose inherited name were unset would also answer nothing, and would pass without the
    //           switch being read at all.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const named = [[RUNTIME_DIRECTORY_SWITCH, handedIn], ['XDG_RUNTIME_DIR', inherited]];
    // ```
    let named: [(&str, &OsStr); 2] = [
        (RUNTIME_DIRECTORY_SWITCH, handed_in.as_os_str()),
        ("XDG_RUNTIME_DIR", inherited.as_os_str()),
    ];
    run_the_body_with_an_environment_of_its_own(
        module_path!(),
        "a_build_made_for_tests_answers_the_runtime_directory_a_control_handed_in",
        named.as_slice(),
        &[],
        || {
            let answered: Option<PathBuf> = runtime_directory();
            let Some(directory) = answered else {
                panic!(
                    "the runtime directory of a build made for tests is the one its control handed in"
                )
            };
            assert!(
                directory.join(THE_MARKER).is_file(),
                "the runtime directory of a build made for tests is the one its control handed in, and this \
                 answer is {}: it holds no marker",
                directory.display()
            );
        },
    );
    remove(handed_in.as_path());
    remove(inherited.as_path());
}

/// What:
///  A build made for tests answers the session-bus address its control handed in,
///  and never the one its body inherited.
/// Why:
///  An inherited address is this account's own session bus,
///  so an answer taken from it is the route that provisions on this account's manager.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(busAddress()).toBe(handedIn); expect(busAddress()).not.toBe(inherited);
/// ```
#[test]
fn a_build_made_for_tests_answers_the_bus_address_a_control_handed_in() {
    // What:     the two addresses as text, since a bus address is text a parser splits, unlike a path.
    // Why:      the route compares the address's own `unix:path=` entry, so the answer has to be that text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const handedIn = 'unix:path=/nonexistent/handed-in-bus'; const inherited = 'unix:path=/nonexistent/inherited-bus';
    // ```
    let handed_in: &str = "unix:path=/nonexistent/handed-in-bus";
    let inherited: &str = "unix:path=/nonexistent/inherited-bus";
    let named: [(&str, &OsStr); 2] = [
        (BUS_ADDRESS_SWITCH, OsStr::new(handed_in)),
        ("DBUS_SESSION_BUS_ADDRESS", OsStr::new(inherited)),
    ];
    run_the_body_with_an_environment_of_its_own(
        module_path!(),
        "a_build_made_for_tests_answers_the_bus_address_a_control_handed_in",
        named.as_slice(),
        &[],
        || {
            let answered: Option<String> = bus_address();
            assert_eq!(
                answered.as_deref(),
                Some(handed_in),
                "the bus address of a build made for tests is the one its control handed in"
            );
            assert_ne!(
                answered.as_deref(),
                Some(inherited),
                "and never the one the body inherited, which is this account's own bus"
            );
        },
    );
}

/// What:
///  A body whose control handed in nothing gets no runtime directory and no bus address,
///  whatever its own environment names.
/// Why:
///  This is the answer that keeps a test process off the account's manager even when another control of
///  the same process already remembered the running executable,
///  which is the order the route provisions in (issue 6586).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(runtimeDirectory()).toBeUndefined(); expect(busAddress()).toBeUndefined();
/// ```
#[test]
fn a_body_whose_control_handed_in_nothing_gets_neither_fact() {
    // What:     the two inherited names, set to places that exist, so an answer taken from them would be visible.
    // Why:      unset names would answer nothing for a reason other than the one this control is about.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const named = [['XDG_RUNTIME_DIR', inherited], ['DBUS_SESSION_BUS_ADDRESS', 'unix:path=/nonexistent/inherited-bus']];
    // ```
    let inherited: PathBuf = fixture("session-places-nothing-inherited");
    let named: [(&str, &OsStr); 2] = [
        ("XDG_RUNTIME_DIR", inherited.as_os_str()),
        (
            "DBUS_SESSION_BUS_ADDRESS",
            OsStr::new("unix:path=/nonexistent/inherited-bus"),
        ),
    ];
    run_the_body_with_an_environment_of_its_own(
        module_path!(),
        "a_body_whose_control_handed_in_nothing_gets_neither_fact",
        named.as_slice(),
        &[],
        || {
            assert_eq!(
                runtime_directory(),
                None,
                "a body that was handed no runtime directory has none, whatever it inherited"
            );
            assert_eq!(
                bus_address(),
                None,
                "a body that was handed no bus address has none, whatever it inherited"
            );
        },
    );
    remove(inherited.as_path());
}
