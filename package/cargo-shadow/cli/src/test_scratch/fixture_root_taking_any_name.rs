//! What:
//! Whether a control's fixture root takes a name that is not valid UTF-8,
//! and the re-execution that puts the control on a tmpfs where the root refuses such a name.
//! Why:
//! `journal::journal_tests::a_manifest_path_that_is_not_utf8_is_recovered_exactly` makes a directory whose name is not UTF-8,
//! and a file system may refuse that name:
//! the dataset behind this crate's build folder does,
//! under `utf8only=on`,
//! answering `EILSEQ`,
//! and the tests make their fixtures in `test-scratch` beside the test executable (`src/test_scratch.rs`),
//! which is on that dataset (issue 6550).
//! A tmpfs takes any name,
//! so this file asks the fixture root whether it already takes the name,
//! and only where it does not starts the same test binary again for the same test
//! inside a user and mount namespace of its own,
//! with a tmpfs mounted at the fixture root.
//! That mount lives only inside the namespace:
//! no other namespace sees it,
//! nothing on the host is mounted,
//! and the fixture root stays the run's own rather than `/tmp` or `/dev/shm`,
//! whose backing device and quota this suite does not choose.
//! The Git wrapper (`package/git-policy/cli/src/native/testing/fixture_root_taking_any_name.rs`)
//! and the forbidden-strings scanner (`package/forbidden-strings/cli/tests/support/fixture_root_taking_any_name.rs`)
//! answered the same refusal this way,
//! for issues 6448 and 6491;
//! this is the shadow's own copy,
//! because this crate depends on neither.
//! Gotcha:
//! Where the root refuses the name and this host lets no unprivileged user namespace either,
//! the control panics with this package's own line for a test that did not run
//! (`cargo-shadow tests: not run [<name>]: ...`,
//! which the `test` task reads),
//! so no run counts it as one that passed.

/// What:
/// `Bound` is how long one run may take and what to print when it does not finish,
/// and `output_within` runs a program
/// under one and hands back what it wrote.
/// The sibling a reader might expect is `Command::output`,
/// which waits forever.
/// Why:
/// The re-executed test is a program of this suite,
/// and a child that never ended would hold the whole run.
use crate::bounded_run::{
    Bound,
    output_within,
};
/// What:
/// `NOT_RUN_PREFIX` is the text this package prints before a test that could not run,
/// `cargo-shadow tests: not run [`,
/// and the `test` task reads that line to leave such a test out of its totals.
/// The sibling a reader might expect is `test_scratch::skip_or_fail`,
/// which prints a skip the task does not read.
/// Why:
/// A control that failed here without that line would be read as a defect of the tool it guards.
use crate::cgroup_test_support::NOT_RUN_PREFIX;
/// What:
/// `CStr` is borrowed C text:
/// bytes that end with one zero byte,
/// which the kernel reads.
/// The siblings a reader might expect are `CString` (the owned one) and `&str` (text with no trailing zero).
/// Why:
/// The mount takes its options as this type,
/// and this file gives it none.
use std::ffi::CStr;
/// What:
/// `OsStrExt` adds `OsStr::from_bytes`,
/// which makes a name out of raw bytes.
/// The sibling a reader might expect is `OsString::from`,
/// which takes text and so could not name this probe.
/// Why:
/// The probe's last byte starts no character,
/// so the name cannot come from a string literal alone.
use std::os::unix::ffi::OsStrExt as _;
/// What:
/// `Path` is a borrowed file-system path and `PathBuf` an owned one.
/// Why:
/// The fixture root is lent to this file,
/// and the probe path it makes is owned here and taken away again.
use std::path::{
    Path,
    PathBuf,
};
/// What:
/// `Command` is a recipe for starting a program,
/// `Output` is what it wrote and how it ended,
/// and `Stdio` names one of
/// its three streams.
/// Why:
/// The same test is started again inside a namespace,
/// and what it said is the evidence that it ran.
use std::process::{
    Command,
    Output,
    Stdio,
};
/// What:
/// `Duration` is a span of time.
/// Why:
/// Each bound below is a span measured from the moment the program starts.
use std::time::Duration;

/// What:
/// The variable the outer half sets for the child it starts,
/// so the re-executed test knows which half of itself it is.
/// Why:
/// One name serves every control,
/// because the child is started with `--exact` for exactly one test,
/// so no other test of the same binary ever reads it,
/// and the kernel's own user-id mapping is asked too before anything is mounted.
const INSIDE_VARIABLE: &str = "CARGO_SHADOW_FIXTURE_ROOT_ANY_NAME";

/// What:
/// The program that makes the namespace,
/// `unshare`,
/// and the switches it takes before the program it runs in it.
/// Why:
/// `--map-root-user` makes this run root inside a user namespace of its own,
/// `--mount` gives it a mount namespace whose mounts no other namespace sees,
/// and `--propagation private` keeps every mount made there from reaching the host's mount table.
const UNSHARE: &str = "unshare";

/// What:
/// Those switches,
/// one word each.
/// Why:
/// `Command::args` takes them as a list,
/// so they are written as one.
const NAMESPACE_SWITCHES: &[&str] = &[
    "--user",
    "--map-root-user",
    "--mount",
    "--propagation",
    "private",
];

/// What:
/// The line libtest prints for a run of one test that passed:
/// one passed,
/// none failed.
/// Why:
/// A name that matches no test ends successfully and runs nothing,
/// so only this line proves the child ran the test this file asked it for.
const ONE_TEST_PASSED: &str = "test result: ok. 1 passed; 0 failed";

/// What:
/// How long the re-executed test may run before this file counts it as one that did not end.
/// The sibling a reader might expect is no bound at all,
/// which is what `Command::output` gives.
/// Why:
/// The control makes one directory and one file in a tmpfs and reads one journal,
/// which the case finished in 0.07 s on this host on 2026-11-27,
/// so this bound only ends a child that never ends.
const CHILD_DEADLINE: Duration = Duration::from_secs(120);

/// What:
/// How long `unshare` is given to say whether this host lets a user namespace:
/// it runs `/bin/true` and nothing else.
/// Why:
/// A host that refuses the switch answers at once,
/// and one that cannot start `unshare` at all must not hold the run.
const NAMESPACE_QUESTION_DEADLINE: Duration = Duration::from_secs(30);

/// What:
/// Whether the caller may go on to make a fixture under `root` whose name is not UTF-8,
/// starting the same test inside a namespace of its own and mounting a tmpfs at `root` when it may not.
/// `false` means this run is the outer half and the inner one has already run and passed.
/// Why:
/// `false` is returned only once the inner half passed,
/// because a control that passed without running would claim coverage it does not have;
/// a host with no unprivileged user namespace panics here with this package's line for a test that did not run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function inAFixtureRootTakingAnyName(root: string): boolean;
/// ```
pub(crate) fn in_a_fixture_root_taking_any_name(root: &Path) -> bool {
    // What: this process is the half the outer one started, inside a namespace of its own.
    // Why: only there may a tmpfs be mounted, and only there does this file mount one.
    if started_inside_a_user_namespace_of_its_own() {
        mount_a_tmpfs(root);
        // What: `assert!` stops the run unless its condition holds.
        // Why: a tmpfs that refused the name too would let the control fail later for a reason that reads as a tool defect.
        assert!(
            a_name_that_is_not_utf8_can_be_made(root),
            "the control mounted a tmpfs at {} because the file system there refused a name that is not UTF-8, and the \
             tmpfs refused it too",
            root.display()
        );
        // What: `true` is the answer "make the fixture here".
        // Why: inside the namespace the root takes the name now.
        return true;
    }
    if a_name_that_is_not_utf8_can_be_made(root) {
        return true;
    }
    start_the_same_test_in_a_namespace_of_its_own(root);
    // What: `false` is the answer "the inner half ran this control".
    // Why: this half must not make the fixture too, or the control would run twice.
    return false;
}

/// What:
/// The name written and removed again to learn whether a root takes a name that is not UTF-8:
/// `odd-name-<this process>-` followed by a byte that starts no character.
/// Why:
/// The scratch root is shared by every test of this binary,
/// so a fixed name could be a file another run made;
/// the process number makes the probe this run's,
/// and its last byte is still the byte string this host's `utf8only` dataset refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function probeName(): Uint8Array { return concat(utf8(`odd-name-${process.pid}-`), Uint8Array.of(0xff)); }
/// ```
fn probe_name() -> Vec<u8> {
    // What: `format!(...).into_bytes()` builds the text of the name and takes the bytes behind it.
    // Why: the byte appended next is not text, so the name cannot stay a `String`.
    let mut name: Vec<u8> = format!("odd-name-{}-", std::process::id()).into_bytes();
    // What: `push` appends one byte, `0xFF`, which starts no UTF-8 character.
    // Why: this is the byte the file system refuses, and the one the control's own directory name holds.
    name.push(0xFF);
    return name;
}

/// What:
/// Whether a file whose name is not valid UTF-8 can be made in `root`,
/// which the probe removes again so the fixture root is left as it was found.
/// Why:
/// A refusal other than `EILSEQ` is not the file system declining this name:
/// it is a fixture root this control cannot use,
/// and it is stated rather than read as "this root takes the name".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aNameThatIsNotUtf8CanBeMade(root: string): boolean;
/// ```
fn a_name_that_is_not_utf8_can_be_made(root: &Path) -> bool {
    // What: `join` appends the probe's bytes to the root, and `OsStr::from_bytes` builds a name from them.
    // Why: the probe path is the only place a name that is not text is made here.
    let probe: PathBuf = root.join(std::ffi::OsStr::from_bytes(probe_name().as_slice()));
    // What: `File::create_new` is the kernel's `O_CREAT | O_EXCL`: it makes the file and fails instead of truncating
    //  one that is already there.
    // Why: a probe left behind by a run that had this very process number must fail this control,
    //  never be overwritten and then removed as though this run had made it.
    let made: std::io::Result<std::fs::File> = std::fs::File::create_new(probe.as_path());
    match made {
        // What: `Ok(_made)` is the success variant of `Result`, its file dropped at the end of the arm.
        // Why: the name was made, which is the answer; the probe is taken away before the answer goes out.
        Ok(_made) => {
            std::fs::remove_file(probe.as_path()).expect("the probe name is taken away again");
            return true;
        }
        // What: `Err(error)` is the failure variant, `raw_os_error` the kernel's own number behind it, and
        //  `Errno::ILSEQ` the number the kernel gives a byte string the file system's name encoding refuses.
        // Why: that refusal is the one this whole file is about, told apart from a root this control cannot use.
        Err(error) if error.raw_os_error() == Some(rustix::io::Errno::ILSEQ.raw_os_error()) => {
            return false;
        }
        Err(error) => panic!(
            "the fixture root {} was asked whether it takes a name that is not UTF-8, with the probe {}: {error}",
            root.display(),
            probe.display()
        ),
    }
}

/// What:
/// Whether this process is the half the outer one started inside a user namespace of its own:
/// the variable the outer half sets is there,
/// and the kernel's mapping of user ids puts this process's id 0 somewhere other than 0 outside.
/// Why:
/// A variable can be inherited from anything,
/// and a process that read one as "I am inside" without being inside
/// would mount a tmpfs in the namespace it shares with the rest of the machine,
/// over a folder other tests use;
/// the kernel's mapping is the answer to which namespace this is,
/// and a user namespace of this account's own maps 0 to another id.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startedInsideAUserNamespaceOfItsOwn(): boolean {
///   if (process.env[INSIDE_VARIABLE] === undefined) return false;
///   const [inside, outside] = readFileSync('/proc/self/uid_map', 'utf8').split('\n')[0].trim().split(/\s+/);
///   if (inside !== '0' || outside === '0') throw new Error(`${INSIDE_VARIABLE} is set outside a namespace of its own`);
///   return true;
/// }
/// ```
fn started_inside_a_user_namespace_of_its_own() -> bool {
    if std::env::var_os(INSIDE_VARIABLE).is_none() {
        return false;
    }
    // What: the kernel's mapping of this process's user ids, one line per range of ids.
    // Why: its first line says which id inside this namespace stands for which id outside it.
    let map: String = std::fs::read_to_string("/proc/self/uid_map")
        .expect("the kernel's mapping of this process's user ids");
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
        "the variable {INSIDE_VARIABLE} is set in a process whose id 0 is this machine's own id 0, so this is no user \
         namespace of its own and a tmpfs mounted here would be mounted on the machine: /proc/self/uid_map holds {first}"
    );
    return true;
}

/// What:
/// Mount one tmpfs at `root`,
/// with the options the kernel gives a tmpfs nobody sized:
/// `rw`,
/// the size the file system gives it,
/// and the mode the kernel gives a tmpfs root.
/// Why:
/// It is called only inside the namespace a control was re-executed in,
/// where this process is root and may mount,
/// and the mount lives in that namespace alone;
/// a fixture that holds one directory,
/// one manifest and one journal needs no size of its own.
/// Gotcha:
/// `None::<&CStr>` is the mount's options argument:
/// the kernel reads that argument as C text,
/// and this mount asks for the default options.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mountATmpfs(root: string): void; // throws when the mount is refused
/// ```
fn mount_a_tmpfs(root: &Path) {
    // What: `rustix::mount::mount` is the kernel's `mount`: the file-system type, the folder it goes on, the same type
    //  again, its flags, and its options.
    // Why: it is the safe wrapper this crate already depends on, so no raw pointer is written here.
    let mounted: Result<(), rustix::io::Errno> = rustix::mount::mount(
        "tmpfs",
        root,
        "tmpfs",
        rustix::mount::MountFlags::empty(),
        None::<&CStr>,
    );
    assert!(
        mounted.is_ok(),
        "a tmpfs is mounted at the fixture root {}: {:?}",
        root.display(),
        mounted.err()
    );
}

/// What:
/// The name this test runs under,
/// as the runner gave it:
/// `journal::journal_tests::a_manifest_path_that_is_not_utf8_is_recovered_exactly` and the like.
/// Why:
/// The child is started for exactly this test,
/// so the name comes from the running thread instead of a second copy of it in the source,
/// which could go stale when a test is renamed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theRunningTest(): string { const name = currentThreadName(); if (name === undefined) throw new Error('the runner named no thread'); return name; }
/// ```
fn the_running_test() -> String {
    // What: `current()` is the thread this test runs in, kept in a binding so its name may be read from it.
    // Why: `name()` lends a view of that thread, and the thread has to outlive the view.
    let running: std::thread::Thread = std::thread::current();
    // What: `name()` is the name the runner gave this thread, or none when it gave none.
    // Why: libtest starts each test in a thread named after its path, which is what `--exact` matches.
    let given: Option<&str> = running.name();
    match given {
        // What: `name.to_string()` copies the borrowed name into an owned `String`.
        // Why: the name outlives the thread handle it was read from.
        Some(name) => return name.to_string(),
        None => panic!(
            "the runner named no thread for this control, so the test binary cannot be started for exactly this test"
        ),
    }
}

/// What:
/// Ask this host whether it lets an unprivileged user namespace,
/// and where it does not panic with this package's line for a test that did not run.
/// Why:
/// The question is asked of the kernel,
/// by running `/bin/true` inside the namespace,
/// so the answer is the one the child would get;
/// and the line names the test and says why,
/// which the `test` task reads,
/// so a run on such a host counts this control as one that did not run instead of as a defect.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function askThisHostForAUserNamespace(test: string, root: string): void; // throws with the not-run line
/// ```
fn ask_this_host_for_a_user_namespace(
    test: &str,
    root: &Path,
) {
    let mut command: Command = Command::new(UNSHARE);
    command.args(NAMESPACE_SWITCHES).arg("/bin/true");
    let bound: Bound = Bound {
        limit: NAMESPACE_QUESTION_DEADLINE,
        what: format!("{UNSHARE} asked whether this host lets a user namespace"),
        rerun: format!("{UNSHARE} {} /bin/true", NAMESPACE_SWITCHES.join(" ")),
    };
    // What: `output_within` starts the program and hands back what it wrote, or the error that it could not be run.
    // Why: a host where `unshare` cannot be started at all answers the same question with the same answer.
    let asked: Output = match output_within(&mut command, None, &bound) {
        Ok(output) => output,
        Err(error) => panic!(
            "{NOT_RUN_PREFIX}{test}]: the file system at {} refuses a name that is not UTF-8, and this suite could not \
             ask this host for a user namespace to mount a tmpfs there ({error})",
            root.display()
        ),
    };
    if asked.status.success() {
        return;
    }
    // What: what `unshare` said when it was refused, as text, with its line ends taken off.
    // Why: the refusal is the reason this control cannot run, and the person reading the log needs it.
    let said: String = String::from_utf8_lossy(asked.stderr.as_slice())
        .trim()
        .to_string();
    panic!(
        "{NOT_RUN_PREFIX}{test}]: the file system at {} refuses a name that is not UTF-8, and this host lets no \
         unprivileged user namespace to mount a tmpfs there ({said}); it runs where the file system behind the fixture \
         root takes any name, or where this account may run `{UNSHARE} {}`",
        root.display(),
        NAMESPACE_SWITCHES.join(" ")
    );
}

/// What:
/// Start this test binary again for this test inside a user and mount namespace of its own,
/// and require it to end with that one test passed.
/// Why:
/// The test binary is the only program that can run the test,
/// and the inner half mounts the tmpfs before the test's own body makes its fixture;
/// a child that ran nothing,
/// failed or did not end fails this call with everything the child said,
/// which is the guard against the control quietly skipping.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startTheSameTestInANamespaceOfItsOwn(root: string): void; // throws when the child did not pass that one test
/// ```
fn start_the_same_test_in_a_namespace_of_its_own(root: &Path) {
    let test: String = the_running_test();
    ask_this_host_for_a_user_namespace(test.as_str(), root);
    // What: this test binary's own path, as the runner started it.
    // Why: the child is the same program, so it can run the same test.
    let executable: PathBuf = std::env::current_exe().expect("this test binary's own path");
    let mut command: Command = Command::new(UNSHARE);
    command.args(NAMESPACE_SWITCHES).arg(executable.as_path());
    // What: `--exact` runs only the named test, `--nocapture` lets its own prints reach the pipe read below,
    //  and one thread keeps the child's body the only test it runs.
    // Why: a child that ran other tests would report their work as this control's.
    command.args(["--exact", test.as_str(), "--nocapture", "--test-threads=1"]);
    command.env(INSIDE_VARIABLE, "1");
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let bound: Bound = Bound {
        limit: CHILD_DEADLINE,
        what: format!("the test {test} inside a user and mount namespace of its own"),
        rerun: format!(
            "{UNSHARE} {} {} --exact {test} --nocapture --test-threads=1",
            NAMESPACE_SWITCHES.join(" "),
            executable.display()
        ),
    };
    let done: Output = match output_within(&mut command, None, &bound) {
        Ok(output) => output,
        Err(error) => panic!(
            "the control {test} needs a fixture root that takes a name which is not UTF-8, {} does not take it, and the \
             test could not be run inside a namespace of its own: {error}",
            root.display()
        ),
    };
    // What: the child's two streams as text, a byte that is not text replaced rather than refused.
    // Why: the two lines below quote what the child said, and a harness line may hold any byte.
    let said: String = String::from_utf8_lossy(done.stdout.as_slice()).into_owned();
    let heard: String = String::from_utf8_lossy(done.stderr.as_slice()).into_owned();
    assert!(
        done.status.success(),
        "the control {test} ran inside a user and mount namespace of its own, with a tmpfs at {} because the file \
         system there refused a name that is not UTF-8, and it failed with {:?}:\n{said}\n{heard}",
        root.display(),
        done.status.code()
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
