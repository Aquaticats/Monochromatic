//! What:
//!  Whether the running control's fixture root is on a file system that takes any byte string as a name,
//!  and the re-execution that puts it on a tmpfs where the root it would use refuses such a name.
//! Why:
//!  A control that makes a file or folder whose name is not valid UTF-8 cannot run where the name is
//!  refused,
//!  as this host's home dataset refuses it under `utf8only=on`,
//!  answering `EILSEQ`
//!  (`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`,
//!  "Cause A"),
//!  and a control that quietly skipped there would pass a run in which it never ran.
//!  A tmpfs takes any name and holds real files,
//!  folders and repositories,
//!  so the first question asked is whether the run's own fixture root already takes the name,
//!  which is the gate's answer because its root is a tmpfs the container makes
//!  (`bin/native-gate-fixture-root.ts`);
//!  where the answer is no,
//!  the same test binary is started inside a user and mount namespace of its own
//!  (`super::root_namespace_test_support`),
//!  and the tmpfs it mounts at the fixture root lives only inside that namespace:
//!  no other namespace sees it,
//!  nothing on the host is mounted by this round,
//!  and the fixture root is the run's own rather than `/tmp` or `/dev/shm`,
//!  whose backing device and quota this suite does not choose.
//!  The name is written and taken away again,
//!  so a root that takes it is left as it was.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!inARootTakingAnyName(fixtureRoot(), 'repair::package_repair_prewarm::tests::a_test')) return; // only where the run acknowledged that this host has no user namespaces
//! ```

use super::root_namespace_test_support::in_a_root_namespace;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

/// The name written and removed again to learn whether a root takes a name that is not UTF-8.
/// Its last byte starts no character,
/// which is the byte string this host's `utf8only` dataset refuses.
const PROBE_NAME: &[u8] = b"not-text-\xff";

/// The variable the inner half sets,
/// so the re-executed test knows which half of itself it is.
///
/// Why one name and not one per control:
///  the variable is set in the child's environment only,
///  and the child is started for exactly one test (`--exact`),
///  so no other test of the same binary ever reads it.
const INSIDE_VARIABLE: &str = "CLI_GIT_FIXTURE_ROOT_ANY_NAME";

/// What:
///  Whether the caller may go on to make a fixture under `root` whose name is not UTF-8,
///  starting the same test inside a namespace of its own and mounting a tmpfs at `root` when it may not.
/// `false` means this run is the outer half:
///  the inner one has already run and passed,
///  or this host lets no unprivileged user namespace and the run said so.
/// Why:
///   `false` is returned only once the inner half passed,
///  or after the run acknowledged that this host has no user namespaces,
///  because a control that silently passed without running would claim coverage it does not have;
///  `in_a_root_namespace` fails the control when the host refuses a namespace that the run did not
///  acknowledge,
///  and prints one line naming the control that did not run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function inARootTakingAnyName(root: string, test: string): boolean;
/// ```
pub(crate) fn in_a_root_taking_any_name(root: &Path, test: &str) -> bool {
    if a_name_that_is_not_utf8_can_be_made(root) {
        return true;
    }
    if !in_a_root_namespace(INSIDE_VARIABLE, test) {
        return false;
    }
    mount_a_tmpfs(root);
    // RED RUN: the fix is not called. stop_gaining_capabilities_across_exec();
    assert!(
        a_name_that_is_not_utf8_can_be_made(root),
        "the control {test} mounted a tmpfs at {} because the file system there refused a name that is not UTF-8, and the \
         tmpfs refused it too",
        root.display()
    );
    return true;
}

/// What:
///  Whether a file whose name is not valid UTF-8 can be made in `root`,
///  which the probe removes again so the fixture root is left as it was found.
/// Why:
///   A refusal other than `EILSEQ` is not the file system declining this name:
///  it is a fixture root this control cannot use,
///  and it is stated rather than read as "this root takes the name".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aNameThatIsNotUtf8CanBeMade(root: string): boolean;
/// ```
fn a_name_that_is_not_utf8_can_be_made(root: &Path) -> bool {
    let probe: PathBuf = root.join(std::ffi::OsStr::from_bytes(PROBE_NAME));
    match std::fs::write(probe.as_path(), b"") {
        Ok(()) => {
            std::fs::remove_file(probe.as_path()).expect("the probe name is taken away again");
            return true;
        }
        Err(error) if error.raw_os_error() == Some(libc::EILSEQ) => return false,
        Err(error) => panic!("the fixture root {} was asked whether it takes a name that is not UTF-8: {error}", root.display()),
    }
}

/// What:
///  Mount one tmpfs at the fixture root of this run.
/// Why:
///   It is called only inside the namespace the control was re-executed in,
///  where this process is root and may mount;
///  the mount's options are the ones the gate's fixture root is made with:
///  `rw`,
///  the size the file system gives it,
///  and the mode the kernel gives a tmpfs root,
///  so a fixture here behaves as the fixture the gate makes does,
///  and a program a control starts from a fixture is runnable there as it is in the gate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mountATmpfs(root: string): void; // throws when the mount is refused
/// ```
fn mount_a_tmpfs(root: &Path) {
    let source: CString = CString::new("tmpfs").expect("a file system type holds no NUL");
    let target: CString = CString::new(root.as_os_str().as_bytes()).expect("a fixture path holds no NUL");
    let kind: CString = CString::new("tmpfs").expect("a file system type holds no NUL");
    // SAFETY: the three names are NUL-terminated and outlive the call, and `mount` reads no other memory of ours.
    let mounted: libc::c_int = unsafe { libc::mount(source.as_ptr(), target.as_ptr(), kind.as_ptr(), 0, std::ptr::null()) };
    assert_eq!(mounted, 0, "a tmpfs is mounted at the fixture root {}: {}", root.display(), std::io::Error::last_os_error());
}

/// What:
///  Stop every program started from here on from gaining a capability,
/// without taking away the ones this process already holds.
/// Why:
///   This process is the root of the user namespace the control was re-executed in,
///  and the kernel gives the root of a program it starts every capability of that namespace
///  unless the process carries `SECURE_NOROOT`:
///  `handle_privileged_root` overwrites the permitted set with the bounding set and makes it effective
///  (`security/commoncap.c`).
///  A process that holds no capability is refused by the capability module before any confined profile is asked,
///  so a profile's refusal is never logged,
///  and the fixture's `git` reaches the hardlink check of a read-only object as an ordinary account's does
///  (`fs/namei.c`, `safe_hardlink_source`;
///  `doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`,
///  the `dac_override` section).
///  The bit is set after the tmpfs is mounted and this process keeps its own capabilities,
///  so the fixtures this inner half goes on to make are root's and written as they were.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// syscall('prctl', PR_SET_SECUREBITS, SECBIT_NOROOT); // throws when the kernel refuses it
/// ```
fn stop_gaining_capabilities_across_exec() {
    // SAFETY: `prctl` with `PR_SET_SECUREBITS` reads the integers it is given and touches no memory of ours.
    let refused: libc::c_int = unsafe { libc::prctl(libc::PR_SET_SECUREBITS, libc::SECBIT_NOROOT, 0, 0, 0) };
    assert_eq!(
        refused,
        0,
        "this inner half stops gaining capabilities across exec: {}",
        std::io::Error::last_os_error()
    );
}
