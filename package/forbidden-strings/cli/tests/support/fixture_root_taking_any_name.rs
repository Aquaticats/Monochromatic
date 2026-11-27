//! What:
//!  Whether this control's fixture root is on a file system that takes any byte string as a name,
//!  and the re-execution that puts the control on a tmpfs where its root refuses such a name.
//! Why:
//!  The native rule-file control in `tests/embedding.rs` makes a rule file whose name is not valid UTF-8,
//!  and a file system may refuse that name:
//!  this host's home dataset does, with `utf8only=on`, answering `EILSEQ`
//!  (`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`, "Cause A"),
//!  and `CARGO_TARGET_TMPDIR`, where that control's fixture root is made, is inside the checkout on that dataset.
//!  A control that failed there would report a file system's refusal as a scanner defect,
//!  and one that returned early would pass a run in which it never ran (issue 6491).
//!  A tmpfs takes any name, so this file asks the fixture root whether it already takes the name,
//!  and only where it does not starts the same test binary again for the same test
//!  inside a user and mount namespace of its own,
//!  with a tmpfs mounted at the fixture root.
//!  That mount lives only inside the namespace:
//!  no other namespace sees it,
//!  nothing on the host is mounted,
//!  and the fixture root stays the run's own rather than `/tmp` or `/dev/shm`,
//!  whose backing device and quota this suite does not choose.
//!  The Git wrapper's crate answered the same refusal this way for issue 6448
//!  (`package/git-policy/cli/src/native/testing/fixture_root_taking_any_name.rs`);
//!  this is the scanner's own copy, because this crate does not depend on the wrapper's.
//!  The name asked about is written and taken away again, so a root that takes it is left as it was.
//! Gotcha:
//!  Where the root refuses the name and this host lets no unprivileged user namespace either,
//!  the control fails and names itself.
//!  No variable makes it skip,
//!  because a control that skipped proves nothing and a run in which it never ran would pass:
//!  such a host needs a fixture root on a file system that takes the name,
//!  which the containers this package's `test:container` task runs already have.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!inAFixtureRootTakingAnyName(fixtureRoot, 'public_embedding_runs_without_real_user_state')) return; // the inner half ran it
//! ```

/// What:
///  `Cow` is "a borrowed value or an owned one", which reads the same either way.
///  The siblings a reader might expect are `String` (always owned, so always a copy) and `&str` (borrowed only).
/// Why:
///  A child's output is usually valid UTF-8 and so is borrowed from the bytes already read,
///  and only an output with an invalid byte is copied and repaired.
use std::borrow::Cow;
/// What:
///  `CString` is an owned C string: bytes that end with one zero byte, which C and the kernel read.
///  Its siblings are `String` and `&str` (text that carries no trailing zero) and `Vec<u8>` (no trailing zero either).
/// Why:
///  The mount below hands the kernel three C strings, and the kernel reads until the zero byte.
#[cfg(target_os = "linux")]
use std::ffi::CString;
/// What:
///  `OsStrExt` adds `OsStr::from_bytes`, which builds a name out of raw bytes.
///  The sibling a reader might expect is `OsString::from`, which takes text and so could not name this probe.
/// Why:
///  The probe name is bytes that are not valid UTF-8, so it cannot come from a string literal alone.
use std::os::unix::ffi::OsStrExt as _;
/// What:
///  `Path` is a borrowed filesystem path and `PathBuf` an owned one.
/// Why:
///  The fixture root is lent to this file, and the probe path it makes is owned here and dropped at once.
use std::path::{
    Path,
    PathBuf,
};
/// What:
///  `Command` is a recipe for starting a program, `Output` is what it ended with and what its two streams held.
/// Why:
///  The half that runs inside the namespace is one program started once, and its output is how it answers.
#[cfg(target_os = "linux")]
use std::process::{
    Command,
    Output,
};

/// What:
///  The name written and removed again to learn whether a root takes a name that is not UTF-8.
///  Its last byte starts no character, which is the byte string this host's `utf8only` dataset refuses.
/// Why:
///  A root that takes this name can hold the rule file the control makes;
///  a root that answers `EILSEQ` for it cannot,
///  and that is the only answer this file reads as "the root refuses the name".
const PROBE_NAME: &[u8] = b"not-text-\xff";

/// What:
///  The variable the inner half finds set, so the re-executed test knows which half of itself it is.
/// Why:
///  One name serves every control, because the child is started with `--exact` for exactly one test,
///  so no other test of the same binary ever reads it.
#[cfg(target_os = "linux")]
const INSIDE_VARIABLE: &str = "FORBIDDEN_STRINGS_FIXTURE_ROOT_ANY_NAME";

/// What:
///  The program that makes the namespace, `unshare`, and the switches it takes before the program it runs in it.
/// Why:
///  `--map-root-user` makes this run root inside a user namespace of its own,
///  `--mount` gives it a mount namespace whose mounts no other namespace sees,
///  and `--propagation private` keeps every mount made there from reaching the host's mount table.
#[cfg(target_os = "linux")]
const UNSHARE: &str = "unshare";

/// What:
///  Those switches, one word each.
/// Why:
///  `Command::args` takes them as a list, so they are written as one.
#[cfg(target_os = "linux")]
const NAMESPACE_SWITCHES: &[&str] = &[
    "--user",
    "--map-root-user",
    "--mount",
    "--propagation",
    "private",
];

/// What:
///  The line libtest prints for a run of one test that passed:
///  one passed, none failed.
/// Why:
///  A name that matches no test ends with success and runs nothing,
///  so only this line proves the child ran the test this file asked it for.
#[cfg(target_os = "linux")]
const ONE_TEST_PASSED: &str = "test result: ok. 1 passed; 0 failed";

/// What:
///  Whether the caller may go on to make a fixture under `root` whose name is not UTF-8,
///  starting the same test inside a namespace of its own and mounting a tmpfs at `root` when it may not.
///  `false` means this run is the outer half and the inner one has already run and passed.
/// Why:
///  `false` is returned only once the inner half passed,
///  because a control that passed without running would claim coverage it does not have;
///  a host that lets no unprivileged user namespace fails the control here and names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function inAFixtureRootTakingAnyName(root: string, test: string): boolean;
/// ```
#[cfg(target_os = "linux")]
pub(crate) fn in_a_fixture_root_taking_any_name(
    root: &Path,
    test: &str,
) -> bool {
    if std::env::var_os(INSIDE_VARIABLE).is_some() {
        mount_a_tmpfs(root);
        assert!(
            a_name_that_is_not_utf8_can_be_made(root),
            "the control {test} mounted a tmpfs at {} because the file system there refused a name that is not UTF-8, \
             and the tmpfs refused it too",
            root.display()
        );
        return true;
    }
    if a_name_that_is_not_utf8_can_be_made(root) {
        return true;
    }
    start_the_same_test_in_a_namespace_of_its_own(root, test);
    return false;
}

/// What:
///  The same question on a platform that has no unprivileged user namespace:
///  a fixture root there has to take the name itself, or the control fails and names itself.
/// Why:
///  Returning true on a root that refuses the name would let the control fail later with a file system's
///  refusal, which reads as a scanner defect;
///  and returning false without a namespace would be the silent skip this file exists to prevent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function inAFixtureRootTakingAnyName(root: string, test: string): boolean {
///   if (!aNameThatIsNotUtf8CanBeMade(root)) throw new Error(`${test} needs a fixture root on a file system that takes any name; ${root} refuses this one and this platform makes no namespace to mount a tmpfs in`);
///   return true;
/// }
/// ```
#[cfg(not(target_os = "linux"))]
pub(crate) fn in_a_fixture_root_taking_any_name(
    root: &Path,
    test: &str,
) -> bool {
    assert!(
        a_name_that_is_not_utf8_can_be_made(root),
        "the control {test} makes a rule file whose name is not valid UTF-8, the file system at {} refuses that name, \
         and this platform makes no unprivileged user namespace to mount a tmpfs in: run this control where the \
         fixture root is on a file system that takes any name",
        root.display()
    );
    return true;
}

/// What:
///  Whether a file whose name is not valid UTF-8 can be made in `root`,
///  which the probe removes again so the fixture root is left as it was found.
/// Why:
///  A refusal other than `EILSEQ` is not the file system declining this name:
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
        Err(error) => panic!(
            "the fixture root {} was asked whether it takes a name that is not UTF-8: {error}",
            root.display()
        ),
    }
}

/// What:
///  Mount one tmpfs at `root`, with the options the container gate's own fixture root is made with:
///  `rw`, the size the file system gives it, and the mode the kernel gives a tmpfs root.
/// Why:
///  It is called only inside the namespace a control was re-executed in,
///  where this process is root and may mount,
///  and the mount lives in that namespace alone;
///  the missing size means a tmpfs the kernel sizes itself,
///  which is right for a fixture that holds one rule file and its compiled artifact.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mountATmpfs(root: string): void; // throws when the mount is refused
/// ```
#[cfg(target_os = "linux")]
fn mount_a_tmpfs(root: &Path) {
    let source: CString = CString::new("tmpfs").expect("a file system type holds no NUL");
    let target: CString =
        CString::new(root.as_os_str().as_bytes()).expect("a fixture path holds no NUL");
    let kind: CString = CString::new("tmpfs").expect("a file system type holds no NUL");
    // SAFETY: all three names are NUL-terminated and outlive the call, and `mount` reads no other memory of ours.
    let mounted: libc::c_int = unsafe {
        libc::mount(
            source.as_ptr(),
            target.as_ptr(),
            kind.as_ptr(),
            0,
            std::ptr::null(),
        )
    };
    assert_eq!(
        mounted,
        0,
        "a tmpfs is mounted at the fixture root {}: {}",
        root.display(),
        std::io::Error::last_os_error()
    );
}

/// What:
///  Start this test binary again for `test` inside a user and mount namespace of its own,
///  and require it to end with that one test passed.
/// Why:
///  The test binary is the only thing that can run the test,
///  so the outer half names the test by its own path and states which half the child is;
///  a child that ran nothing, failed, or could not be started fails this call with everything the child said,
///  which is the whole point of the child and the guard against the control quietly skipping.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startTheSameTestInANamespaceOfItsOwn(root: string, test: string): void; // throws when the child did not pass that one test
/// ```
#[cfg(target_os = "linux")]
fn start_the_same_test_in_a_namespace_of_its_own(
    root: &Path,
    test: &str,
) {
    let executable: PathBuf = std::env::current_exe().expect("this test binary's own path");
    let mut command: Command = Command::new(UNSHARE);
    command.args(NAMESPACE_SWITCHES).arg(executable.as_path());
    // `--exact` runs only the named test, `--nocapture` lets its own prints reach the pipe read below,
    // and one thread keeps the child's body the only test it runs.
    command.args(["--exact", test, "--nocapture", "--test-threads=1"]);
    command.env(INSIDE_VARIABLE, "1");
    let output: Output = command.output().unwrap_or_else(|error| {
        panic!(
            "the control {test} needs a fixture root that takes a name which is not UTF-8, {} does not take it, and \
             {UNSHARE} could not be started to make a namespace with one: {error}",
            root.display()
        )
    });
    let said: Cow<'_, str> = String::from_utf8_lossy(output.stdout.as_slice());
    let heard: Cow<'_, str> = String::from_utf8_lossy(output.stderr.as_slice());
    assert!(
        output.status.success(),
        "the control {test} ran inside a user and mount namespace of its own, with a tmpfs at {} because the file \
         system there refused a name that is not UTF-8, and it failed with {:?}:\n{said}\n{heard}",
        root.display(),
        output.status.code()
    );
    assert!(
        said.contains(ONE_TEST_PASSED),
        "the control {test} was started inside a namespace of its own for the fixture root {}, and the child ran no \
         single passing test (a name that matches no test ends successfully and runs nothing):\n{said}\n{heard}",
        root.display()
    );
    eprintln!(
        "the control {test} ran inside a user and mount namespace of its own, with a tmpfs at {}, because the file \
         system there refused a name that is not UTF-8",
        root.display()
    );
}
