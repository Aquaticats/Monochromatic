//! What:
//!  Whether this control's fixture root is on a file system that takes any byte string as a name,
//!  and the re-execution that puts the control on a tmpfs where its root refuses such a name.
//! Why:
//!  The native rule-file control in `tests/embedding.rs` makes a rule file whose name is not valid UTF-8,
//!  and a file system may refuse that name:
//!  this host's home dataset does,
//! with `utf8only=on`,
//! answering `EILSEQ`
//!  (`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`,
//! "Cause A"),
//!  and `CARGO_TARGET_TMPDIR`,
//! where that control's fixture root is made,
//! is on that dataset.
//!  A control that failed there would report a file system's refusal as a scanner defect,
//!  and one that returned early would pass a run in which it never ran (issue 6491).
//!  A tmpfs takes any name,
//! so this file asks the fixture root whether it already takes the name,
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
//!  this is the scanner's own copy,
//! because this crate does not depend on the wrapper's.
//!  Unlike that copy,
//! this one sets no `SECURE_NOROOT`:
//!  the wrapper sets it so a fixture's Git asks the capability module for no capability its confinement refuses,
//!  and the only program this control starts is its own test binary,
//!  which needs none (issue 6459 holds the wrapper's reason).
//!  The name asked about is written and taken away again,
//! so a root that takes it is left as it was.
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
///  `Cow` is "a borrowed value or an owned one",
/// which reads the same either way;
///  the siblings a reader might expect are `String` (always owned,
/// so always a copy) and `&str` (borrowed only).
/// Why:
///  The inner half's output is usually valid UTF-8 and so is borrowed from the bytes already read,
///  and only an output with an invalid byte is copied and repaired.
#[cfg(target_os = "linux")]
use std::borrow::Cow;
/// What:
///  `CString` is an owned C string:
/// bytes that end with one zero byte,
/// which C and the kernel read.
///  Its siblings are `String` and `&str` (text that carries no trailing zero) and `Vec<u8>` (no trailing zero either).
/// Why:
///  The mount below hands the kernel three C strings,
/// and the kernel reads until the zero byte.
#[cfg(target_os = "linux")]
use std::ffi::CString;
/// What:
///  `Read` is how bytes come out of the pipe a child wrote.
///  The sibling a reader might expect is `BufRead`,
/// which adds line reading this file does not need.
/// Why:
///  The inner half's two streams are pipes,
/// and this file reads what they hold once the child has ended.
#[cfg(target_os = "linux")]
use std::io::Read as _;
/// What:
///  `OsStrExt` adds `OsStr::from_bytes`,
/// which builds a name out of raw bytes.
///  The sibling a reader might expect is `OsString::from`,
/// which takes text and so could not name this probe.
/// Why:
///  The probe name is bytes that are not valid UTF-8,
/// so it cannot come from a string literal alone.
use std::os::unix::ffi::OsStrExt as _;
/// What:
///  `Path` is a borrowed filesystem path and `PathBuf` an owned one.
/// Why:
///  The fixture root is lent to this file,
/// and the probe path it makes is owned here and dropped at once.
use std::path::{
    Path,
    PathBuf,
};
/// What:
///  `Command` is a recipe for starting a program,
///  `Child` is that program running,
///  and `ExitStatus` is how it ended.
/// Why:
///  The child is started,
/// looked at while it runs so a child that never ends can be ended here,
///  and its ending is what the control asserts.
#[cfg(target_os = "linux")]
use std::process::{
    Child,
    Command,
    ExitStatus,
};
/// What:
///  `Duration` is a span and `Instant` a moment on the monotonic clock.
/// Why:
///  The child's deadline is a span measured from the moment it starts.
#[cfg(target_os = "linux")]
use std::time::{
    Duration,
    Instant,
};

/// What:
///  The variable the outer half sets for the child it starts,
/// so the re-executed test knows which half it is.
/// Why:
///  One name serves every control,
/// because the child is started with `--exact` for exactly one test,
///  so no other test of the same binary ever reads it.
#[cfg(target_os = "linux")]
const INSIDE_VARIABLE: &str = "FORBIDDEN_STRINGS_FIXTURE_ROOT_ANY_NAME";

/// What:
///  The program that makes the namespace,
/// `unshare`,
/// and the switches it takes before the program it runs in it.
/// Why:
///  `--map-root-user` makes this run root inside a user namespace of its own,
///  `--mount` gives it a mount namespace whose mounts no other namespace sees,
///  and `--propagation private` keeps every mount made there from reaching the host's mount table.
#[cfg(target_os = "linux")]
const UNSHARE: &str = "unshare";

/// What:
///  Those switches,
/// one word each.
/// Why:
///  `Command::args` takes them as a list,
/// so they are written as one.
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
///  one passed,
/// none failed.
/// Why:
///  A name that matches no test ends with success and runs nothing,
///  so only this line proves the child ran the test this file asked it for.
#[cfg(target_os = "linux")]
const ONE_TEST_PASSED: &str = "test result: ok. 1 passed; 0 failed";

/// What:
///  How long the inner half may run before this file counts it as a child that did not end.
///  The sibling a reader might expect is no bound at all,
/// which is what a plain `Command::output` gives.
/// Why:
///  A child that never ends would hold the whole test run,
/// and a run under `cargo nextest` has no per-test
///  stop of its own here;
///  the control itself ended in about 20 ms on this host on 2026-11-27 in a release build,
///  so this bound is far above its work and only ends a child that never ends.
#[cfg(target_os = "linux")]
const CHILD_DEADLINE: Duration = Duration::from_secs(120);

/// What:
///  How often the wait for that child looks at it again.
/// Why:
///  A look costs one `waitpid` call,
/// so looking every 20 ms keeps the wait bounded without a busy loop.
#[cfg(target_os = "linux")]
const LOOK_EVERY: Duration = Duration::from_millis(20);

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
    if started_inside_a_user_namespace_of_its_own() {
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
///  a fixture root there has to take the name itself,
/// or the control fails and names itself.
/// Why:
///  Returning true on a root that refuses the name would let the control fail later with a file system's
///  refusal,
/// which reads as a scanner defect;
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
///  The name written and removed again to learn whether a root takes a name that is not UTF-8:
///  `not-text-<this process>-` followed by a byte that starts no character.
/// Why:
///  The fixture root is this build folder's own `tmp` folder,
/// which every test of every binary in the package shares,
///  so a fixed name could be a file another run made;
///  the process number makes the probe this run's,
/// and its last byte is still the byte string this host's
///  `utf8only` dataset refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function probeName(): Uint8Array { return concat(utf8(`not-text-${process.pid}-`), Uint8Array.of(0xff)); }
/// ```
fn probe_name() -> Vec<u8> {
    let mut name: Vec<u8> = format!("not-text-{}-", std::process::id()).into_bytes();
    name.push(0xFF);
    return name;
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
    let probe: PathBuf = root.join(std::ffi::OsStr::from_bytes(probe_name().as_slice()));
    // What: `create_new` is the kernel's `O_CREAT | O_EXCL`: it makes the file, and fails instead of truncating one
    //  that is already there.
    // Why: a probe left behind by a run that had this very process number must fail this control,
    //  never be overwritten and then removed as though this run had made it.
    let made: std::io::Result<std::fs::File> = std::fs::File::create_new(probe.as_path());
    match made {
        Ok(_made) => {
            std::fs::remove_file(probe.as_path()).expect("the probe name is taken away again");
            return true;
        }
        Err(error) if error.raw_os_error() == Some(libc::EILSEQ) => return false,
        Err(error) => panic!(
            "the fixture root {} was asked whether it takes a name that is not UTF-8, with the probe {}: {error}",
            root.display(),
            probe.display()
        ),
    }
}

/// What:
///  Whether this process is the half the outer one started inside a user namespace of its own:
///  the variable the outer half sets is there,
/// and the kernel's mapping of user ids puts this process's id 0
///  somewhere other than 0 outside.
/// Why:
///   A variable can be inherited from anything,
/// and a process that read one as "I am inside" without being inside
///  would mount a tmpfs in the namespace it shares with the rest of the machine,
/// over a folder other tests use;
///  the kernel's mapping is the answer to which namespace this is,
/// and only a user namespace maps 0 to another id.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startedInsideAUserNamespaceOfItsOwn(): boolean {
///   if (process.env[INSIDE_VARIABLE] === undefined) return false;
///   const [inside, outside] = readFileSync('/proc/self/uid_map', 'utf8').split('\n')[0].trim().split(/\s+/);
///   if (outside === '0') throw new Error(`${INSIDE_VARIABLE} is set in a process whose id 0 is the machine's 0, so this is not a namespace of its own`);
///   return true;
/// }
/// ```
#[cfg(target_os = "linux")]
fn started_inside_a_user_namespace_of_its_own() -> bool {
    if std::env::var_os(INSIDE_VARIABLE).is_none() {
        return false;
    }
    let map: String = std::fs::read_to_string("/proc/self/uid_map")
        .expect("the kernel's mapping of this process's user ids");
    // What: the first line's first three numbers: this namespace's starting id, the id it is outside, and how many.
    // Why: the root of a user namespace this file made has its 0 mapped to a nonzero id outside.
    let first: &str = map
        .lines()
        .next()
        .expect("a process always has one line of user-id mapping");
    let mut fields = first.split_whitespace();
    let inside: &str = fields
        .next()
        .expect("the mapping names the id inside the namespace");
    let outside: &str = fields
        .next()
        .expect("the mapping names the id it stands for outside");
    assert_eq!(
        inside, "0",
        "the variable {INSIDE_VARIABLE} is set in a process that is not id 0 of a namespace, and this file sets it \
         only for the root of one: /proc/self/uid_map holds {first}"
    );
    assert_ne!(
        outside, "0",
        "the variable {INSIDE_VARIABLE} is set in a process whose id 0 is this machine's own id 0, so this is no \
         user namespace of its own and a tmpfs mounted here would be mounted on the machine: /proc/self/uid_map \
         holds {first}"
    );
    return true;
}

/// What:
///  Mount one tmpfs at `root`,
/// with the options the container gate's own fixture root is made with:
///  `rw`,
/// the size the file system gives it,
/// and the mode the kernel gives a tmpfs root.
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
///  wait for it with a deadline,
/// and require it to end with that one test passed.
/// Why:
///  The test binary is the only thing that can run the test,
///  so the outer half names the test by its own path and states which half the child is;
///  a child that ran nothing,
/// failed,
/// did not end,
/// or could not be started fails this call with everything the child
///  said,
/// which is the whole point of the child and the guard against the control quietly skipping.
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
    command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child: Child = command.spawn().unwrap_or_else(|error| {
        panic!(
            "the control {test} needs a fixture root that takes a name which is not UTF-8, {} does not take it, and \
             {UNSHARE} could not be started to make a namespace with one: {error}",
            root.display()
        )
    });
    let began: Instant = Instant::now();
    let status: ExitStatus = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if began.elapsed() < CHILD_DEADLINE => std::thread::sleep(LOOK_EVERY),
            Ok(None) => {
                // An inner half that never ends is ended here, so the run is not held by it,
                // and its two pipes are left unread because a process it left running may still hold them.
                let _killed: std::io::Result<()> = child.kill();
                let _reaped: std::io::Result<ExitStatus> = child.wait();
                panic!(
                    "the control {test} did not end within {} s inside the namespace it was started in, so it was \
                     killed: give it a longer `CHILD_DEADLINE` if its work is slower than that on this host",
                    CHILD_DEADLINE.as_secs()
                );
            }
            Err(error) => panic!("the control {test}'s child could not be looked at: {error}"),
        }
    };
    // The child's own test ends its child before it returns, so once it is gone nothing holds either pipe open.
    let mut said_bytes: Vec<u8> = Vec::new();
    let mut heard_bytes: Vec<u8> = Vec::new();
    child
        .stdout
        .take()
        .expect("the child's standard output is a pipe")
        .read_to_end(&mut said_bytes)
        .expect("the child's standard output is read");
    child
        .stderr
        .take()
        .expect("the child's standard error is a pipe")
        .read_to_end(&mut heard_bytes)
        .expect("the child's standard error is read");
    let said: Cow<'_, str> = String::from_utf8_lossy(said_bytes.as_slice());
    let heard: Cow<'_, str> = String::from_utf8_lossy(heard_bytes.as_slice());
    assert!(
        status.success(),
        "the control {test} ran inside a user and mount namespace of its own, with a tmpfs at {} because the file \
         system there refused a name that is not UTF-8, and it failed with {:?}:\n{said}\n{heard}",
        root.display(),
        status.code()
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
