//! What:
//!  Whether a control is running inside a user and mount namespace of its own,
//!  where this run's own files are root's,
//!  and the re-execution that puts it there.
//! Why:
//!  A control that needs a file root owns cannot make one as the account the tests run as,
//!  and the store the route reads is refused unless root owns it:
//!  `unshare --user --map-root-user --mount --propagation private` makes the run
//!  root and gives it a mount namespace whose mounts stay inside it,
//!  so a fixture written there is root's,
//!  a control can mount a tmpfs that no other namespace sees,
//!  and the same test can prove both the accept and the refusal.
//!  One variable says which half a run is,
//!  and the outer half asserts the inner one really ran and passed,
//!  so a namespace that ran nothing fails rather than passing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!inARootNamespace('SLOT_IDENTITY_STORE_ROOT_SIDE', 'worktree_clone_mount::...::tests::a_test')) { /* the child ran and passed */ }
//! ```

/// The program that makes the namespace,
/// with the user map that makes this run root inside it.
const UNSHARE: &str = "unshare";

/// The switches `unshare` takes before the program it runs in the namespace.
const NAMESPACE_SWITCHES: &[&str] = &[
    "--user",
    "--map-root-user",
    "--mount",
    "--propagation",
    "private",
];

/// The variable a run sets to state that this host lets no unprivileged user namespace,
/// which is the one acknowledgement that lets these controls skip instead of failing.
///
/// Why:
///  A control that needs a namespace and quietly returns passes a run in which it never ran,
///  and the gate's verdict then says nothing about it;
///  the repository's btrfs controls fail unless the run acknowledges the missing capability,
///  and this is the same rule for the kernel's namespaces.
pub(crate) const NO_USER_NAMESPACES_ACKNOWLEDGED_VARIABLE: &str = "CLI_GIT_TEST_NO_USER_NAMESPACES";

/// The one request for a child of this test binary, the program that runs it, and the check on its one test.
use super::child_test_process::{
    ChildTest,
    ProgramRunningTheTest,
    passed,
};
/// `OsStr` is borrowed operating-system text of raw bytes.
use std::ffi::OsStr;

/// What:
///  Whether the running control is the inner half,
///  inside a user namespace of its own;
///  when it is not,
///  the same test binary is started there and this test runs as root in that child.
/// Why:
///   The test binary is the only thing that can run the test,
///  so the outer half names the test by its own path and states which half the child is;
///  a child that ran nothing or failed makes the outer half fail,
///  and a host without user namespaces says the control did not run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const inner = inARootNamespace(variable, 'worktree_clone_mount::...::tests::a_test'); // false when skipped or after the child passed
/// ```
///
/// # Errors
/// Panics when the child half failed or ran no test,
/// because a control that did not run must not count as one that passed.
pub(crate) fn in_a_root_namespace(
    variable: &str,
    test: &str,
) -> bool {
    if std::env::var_os(variable).is_some() {
        return true;
    }
    let probe: std::process::Output = std::process::Command::new(UNSHARE)
        .args(NAMESPACE_SWITCHES)
        .arg("/bin/true")
        .output()
        .expect("unshare runs");
    if !probe.status.success() {
        let said: String = String::from_utf8_lossy(probe.stderr.as_slice())
            .trim()
            .to_string();
        let acknowledged: bool = std::env::var_os(NO_USER_NAMESPACES_ACKNOWLEDGED_VARIABLE)
            .is_some_and(|value| {
                return !value.is_empty();
            });
        assert!(
            acknowledged,
            "the control {test} needs an unprivileged user namespace, and this host refused one: {said}; set {NO_USER_NAMESPACES_ACKNOWLEDGED_VARIABLE}=1 to state that this host has none"
        );
        eprintln!(
            "NAMESPACE CONTROL NOT RUN: {test} (this host lets no unprivileged user namespace: {said})"
        );
        return false;
    }
    let environment: [(&str, &OsStr); 1] = [(variable, OsStr::new("1"))];
    let done = passed(ChildTest {
        exact_test: test,
        environment: environment.as_slice(),
        running_program: Some(ProgramRunningTheTest {
            program: UNSHARE,
            arguments_before_the_test_binary: NAMESPACE_SWITCHES,
        }),
        ..ChildTest::default()
    });
    // What: the inner half's own error lines, printed here when it passed.
    // Why:
    //  A control that could not do what it is for prints one line naming itself, as the btrfs and ZFS controls do,
    //  and a line written inside the namespace would otherwise die in this child's captured output;
    //  the gate asks nextest for a passing test's output (`--success-output=final`), so this line reaches its log.
    let inner_errors: String = String::from_utf8_lossy(done.stderr.as_slice())
        .trim()
        .to_string();
    if !inner_errors.is_empty() {
        eprintln!("{inner_errors}");
    }
    return false;
}
