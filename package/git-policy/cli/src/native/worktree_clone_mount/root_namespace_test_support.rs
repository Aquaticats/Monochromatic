//! What:
//!  Whether a control is running inside a user namespace of its own,
//!  where this run's own files are root's,
//!  and the re-execution that puts it there.
//! Why:
//!  A control that needs a file root owns cannot make one as the account the tests run as,
//!  and the store the route reads is refused unless root owns it:
//!  `unshare --user --map-root-user` makes the run root inside a namespace of its own,
//!  so a fixture written there is root's and the same test can prove both the accept and the refusal.
//!  One variable says which half a run is,
//!  and the outer half asserts the inner one really ran and passed,
//!  so a namespace that ran nothing fails rather than passing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!inARootNamespace('SLOT_IDENTITY_STORE_ROOT_SIDE', 'worktree_clone_mount::...::tests::a_test')) return; // skipped: no namespace
//! ```

use std::process::Command;

/// The program that makes the namespace,
/// with the user map that makes this run root inside it.
const UNSHARE: &str = "unshare";

/// The variable a run sets to state that this host lets no unprivileged user namespace,
/// which is the one acknowledgement that lets these controls skip instead of failing.
///
/// Why:
///  A control that needs a namespace and quietly returns passes a run in which it never ran,
///  and the gate's verdict then says nothing about it;
///  the repository's btrfs controls fail unless the run acknowledges the missing capability,
///  and this is the same rule for the kernel's namespaces.
pub(crate) const NO_USER_NAMESPACES_ACKNOWLEDGED_VARIABLE: &str = "CLI_GIT_TEST_NO_USER_NAMESPACES";

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
#[allow(clippy::disallowed_methods, reason = "starts this test binary, not the wrapper, under `unshare`; the ban on `current_exe` guards the wrapper's own start")]
pub(crate) fn in_a_root_namespace(variable: &str, test: &str) -> bool {
    if std::env::var_os(variable).is_some() {
        return true;
    }
    let probe: std::process::Output = Command::new(UNSHARE).args(["--user", "--map-root-user", "/bin/true"]).output().expect("unshare runs");
    if !probe.status.success() {
        let said: String = String::from_utf8_lossy(probe.stderr.as_slice()).trim().to_string();
        let acknowledged: bool = std::env::var_os(NO_USER_NAMESPACES_ACKNOWLEDGED_VARIABLE).is_some_and(|value| { return !value.is_empty(); });
        assert!(
            acknowledged,
            "the control {test} needs an unprivileged user namespace, and this host refused one: {said}; set {NO_USER_NAMESPACES_ACKNOWLEDGED_VARIABLE}=1 to state that this host has none"
        );
        eprintln!("NAMESPACE CONTROL NOT RUN: {test} (this host lets no unprivileged user namespace: {said})");
        return false;
    }
    let done: std::process::Output = Command::new(UNSHARE)
        .args(["--user", "--map-root-user"])
        .arg(std::env::current_exe().expect("this test binary"))
        .args(["--exact", test, "--nocapture"])
        .env(variable, "1")
        .output()
        .expect("the inner half runs");
    let said: String = String::from_utf8_lossy(done.stdout.as_slice()).to_string() + String::from_utf8_lossy(done.stderr.as_slice()).as_ref();
    assert!(done.status.success(), "the inner half failed:\n{said}");
    assert!(said.contains("1 passed"), "the inner half did not run:\n{said}");
    return false;
}
