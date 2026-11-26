//! What:
//!  The one place this crate starts a child that is its own test binary running one named test,
//!  waits for it with a deadline,
//!  and reads what it said.
//! Why:
//!  A descriptor,
//!  a signal mask,
//!  a resource limit and the environment belong to one process,
//!  and this crate's tests run as threads of one process,
//!  so a control of any of them runs its body in a child.
//!  Written once here, every such child answers the same way:
//!  `spawn_owned` makes it lead a group that dies with the test thread and with this process (issue 6463, item 5),
//!  every wait has a deadline,
//!  and a child whose test did not pass fails the control with what the child said.
//!  Before this file held that shape, six bodies started such a child and 35 call sites built the command themselves;
//!  most were owned by nothing, `finished`'s predecessor waited with `Command::output` and no deadline,
//!  and only some callers checked that the child ran its test at all
//!  (`package/git-policy/cli/doc/debt-ledger-2026-11-25.md`, item 10).
//! Gotcha:
//!  A child that is meant to end by a signal, or one the control keeps and talks to,
//!  cannot use `passed`:
//!  `finished` answers the raw ending for the first, and `started` hands the running child to the second,
//!  whose own waits are bounded where they are.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! passed({ exactTest: 'mod::tests::the_control' });                       // one test, must pass
//! const gone = finished({ exactTest: name, environment: [[K, '1']] });    // undefined when it outlived the deadline
//! const helper = started({ exactTest: name, standardOutput: 'ignore' });  // talk to it, then end it
//! ```

/// What:
///  `Read` is how bytes come out of the pipe a child wrote,
///  and `AsRawFd` is the number of the descriptor a pipe is.
///  The siblings a reader might expect are `BufRead`, which adds line reading this file does not need,
///  and `AsFd`, which names a descriptor it cannot read through.
/// Why:
///  The read below needs both: `read` for the bytes and the descriptor to ask whether more are already there.
use std::io::Read;
/// What:
///  `AsRawFd` gives the number of the descriptor a pipe is.
/// Why:
///  The read below makes the pipe non-blocking through that number, so it never waits for a writer that keeps it open.
use std::os::fd::AsRawFd;
/// What:
///  `CommandExt` adds `arg0` (the name a program sees itself under) and `pre_exec` (the work between `fork` and `exec`).
/// Why:
///  One control runs a child under a Git subcommand's name, and three need work in the child before it execs.
use std::os::unix::process::CommandExt as _;
/// What:
///  `OsStr` is borrowed operating-system text of raw bytes (sibling: `String`, an owned UTF-8 buffer).
/// Why:
///  An environment value is bytes, not necessarily UTF-8, so the request names it by `OsStr`.
use std::ffi::OsStr;
/// What:
///  `PathBuf` is an owned filesystem path of raw OS bytes.
/// Why:
///  The path of this test binary is read once and kept for the command of each child.
use std::path::PathBuf;
/// What:
///  `Child` is a running process, `Command` its recipe, `Output` its ending and its two streams, `Stdio` one stream.
/// Why:
///  This file starts a child, waits for it and answers exactly these four.
use std::process::{Child, Command, Output, Stdio};
/// What:
///  `Duration` is a span and `Instant` a moment on the monotonic clock.
/// Why:
///  The deadline is a span measured from the moment the child starts.
use std::time::{Duration, Instant};

/// This test binary's own path,
/// for the child the command of each child starts from it.
///  The ban on `current_exe` (`clippy.toml`) is about the wrapper starting itself,
/// so this file is where that lint is excused for tests:
/// the program started here is libtest, never the wrapper.
#[allow(clippy::disallowed_methods, reason = "identifies the running test binary so a control can start a helper copy of it; the wrapper is never started this way")]
fn this_test_binary() -> PathBuf {
    return std::env::current_exe().expect("this test binary's own path");
}

/// What:
///  `spawn_owned` starts a process that dies with the test thread that started it,
///  whether the test passes, fails, panics, or the whole test process is killed.
///  In the binary controls' crate the same function is reached through that crate's `support` module.
/// Why:
///  A child no test thread owns outlives the run that started it (issue 6463),
/// and this file is the one place every such child is started.
use super::test_process_owner::spawn_owned;

/// What:
///  `own_group` makes the process group a child leads die with the test thread that started it,
///  for a child `spawn_owned` cannot start.
/// Why:
///  A child that must become a session leader cannot also be put in a group by `spawn_owned`,
/// because a group leader cannot start a session;
/// it leads a group of its own already, so only its number is given to the thread's fixtures.
use super::test_process_owner::own_group;

/// What:
///  How long a child of a test may run before the control counts it as a child that did not end.
///  The siblings a reader might expect are a per-call number, which each control would have to guess,
///  and no bound at all, which is what this file's predecessor had.
/// Why:
///  A run under `cargo test` has no per-test limit, so a child that hangs holds the whole run;
/// 120 s is the window the run's own watchdog uses (`QUIET_WINDOW` in `testing/test_support.rs`),
/// so a child that shows nothing is failed here with its own name before the watchdog ends the run,
/// and the container gate stops any test after five minutes (`.config/nextest.toml`),
/// which leaves room for this file's message to reach the log.
pub(crate) const CHILD_TEST_DEADLINE: Duration = Duration::from_secs(120);

/// How often a wait in this file looks at its child again.
const LOOK_EVERY: Duration = Duration::from_millis(20);

/// What:
///  The line libtest prints for a run of one test that passed:
///  one passed, none failed, none skipped.
/// Why:
///  A name that matches no test ends with success and runs nothing,
/// so only this line proves the child ran the test the control meant.
const ONE_TEST_PASSED: &str = "test result: ok. 1 passed; 0 failed";

/// What:
///  A program that runs the test binary as its own argument, with the arguments that come before it:
///  `unshare --user --map-root-user --mount --propagation private <this test binary> --exact <test>`.
///  The siblings a reader might expect are `program` alone (which cannot pass the test binary to it)
///  and one fixed program name (which the five namespace controls would each have to special-case).
/// Why:
///  A mount, a bind and a user namespace are made by `unshare`, and only the child inside them can run the test,
/// so one child start serves every control that needs a namespace of its own.
pub(crate) struct ProgramRunningTheTest<'a> {
    /// The program started, such as `unshare`.
    pub(crate) program: &'a str,
    /// The arguments it takes before the test binary: its own switches.
    pub(crate) arguments_before_the_test_binary: &'a [&'a str],
}

/// What:
///  One request for a child that is this test binary running `exact_test`.
///  A field left out of a call is the plain case,
/// because `Default` sets the streams to two pipes and the deadline to `CHILD_TEST_DEADLINE`.
///  The siblings a reader might expect are one function per shape, which was the six bodies this replaced,
///  and a builder of nine methods, which states the same fields in nine more places.
/// Why:
///  The controls that start such a child differ in one or two things each,
/// and a field they leave out says the plain case once here instead of at every call.
///
/// # Examples
/// In TS you'd write (pseudocode):
/// ```ts
/// ChildTest { exactTest: 'mod::tests::c', environment: [[K, '1']], ..defaults() }
/// ```
pub(crate) struct ChildTest<'a> {
    /// What:
    ///  The path libtest answers to: the module path without the crate's own first segment, and the function's name,
    ///  such as `process::signal_relay::tests::helper_entry`.
    ///  Its sibling is the bare function name, which `--exact` would match nothing for.
    /// Why:
    ///  `--exact` matches this text and nothing else, so a wrong path runs no test and the `passed` check says so.
    pub(crate) exact_test: &'a str,
    /// What:
    ///  The variables the child gets, each a name and a value of raw operating-system text.
    ///  The siblings a reader might expect are `String` pairs (which refuse a value that is not UTF-8) and
    ///  `OsString` pairs (which are owned, so the request could not borrow them).
    /// Why:
    ///  The child is what learns it is the child, and which fixture it works in, from variables like these.
    pub(crate) environment: &'a [(&'a str, &'a OsStr)],
    /// What:
    ///  Variables the child must not inherit from this process, such as a switch a shell set for this run.
    /// Why:
    ///  A control that proves what the wrapper does with nothing set needs the switch removed, not set to empty.
    pub(crate) cleared_environment: &'a [&'a str],
    /// What:
    ///  Whether the test is marked `#[ignore]` and must be asked for with `--ignored`.
    /// Why:
    ///  A child body that is only ever started this way stays out of a plain run of its module.
    pub(crate) ignored: bool,
    /// What:
    ///  The program that runs the test binary as an argument, when the test needs a namespace of its own.
    ///  The sibling a reader might expect is `program_name`, which renames the test binary instead of wrapping it.
    /// Why:
    ///  `unshare` is the only way an unprivileged run makes a user or mount namespace, and it must start the program.
    pub(crate) running_program: Option<ProgramRunningTheTest<'a>>,
    /// What:
    ///  Arguments given to the test binary before `--exact`, such as the Git subcommand one control makes it look like.
    /// Why:
    ///  libtest reads them as words it does not know, and the program listing of another control needs them there.
    pub(crate) arguments_before_the_test_name: &'a [&'a str],
    /// What:
    ///  The name the test binary sees itself under, which is `git` in one control and its own file name otherwise.
    /// Why:
    ///  A process listing shows the name a program was started under, and a control proves the wrapper's children appear
    ///  there as `git <subcommand>`.
    pub(crate) program_name: Option<&'a str>,
    /// What:
    ///  Whether the child makes a session of its own, which `setsid` in `before_the_exec` needs,
    ///  and which a child started with `spawn_owned` cannot do.
    ///  The sibling a reader might expect is the field left out, and then `spawn_owned` starts the child.
    /// Why:
    ///  A group leader cannot start a session, so `spawn_owned`'s own `process_group` makes `setsid` fail;
    ///  such a child leads a group of its own already, and `own_group` hands that number to the thread's fixtures.
    pub(crate) takes_its_own_session: bool,
    /// What:
    ///  The work the child does between `fork` and `exec`: a umask, a resource limit, a session of its own.
    ///  The closures it takes may only call async-signal-safe functions, because the child shares this process's memory
    ///  until it execs.
    ///  The siblings a reader might expect are a `before_exec` per concern (six again) and setting these on the parent
    ///  (which would reach every other thread of the run).
    /// Why:
    ///  A umask, a limit and a session belong to one process, so each is set in the child that needs it and nowhere else.
    pub(crate) before_the_exec: Option<Box<dyn FnMut() -> std::io::Result<()> + Send + Sync>>,
    /// What:
    ///  The child's standard input, a pipe of its own by default nothing at all.
    /// Why:
    ///  A child that reads nothing must not read what this run reads, and one control writes to its child's input.
    pub(crate) standard_input: Stdio,
    /// What:
    ///  The child's standard output, a pipe this file reads by default.
    /// Why:
    ///  The line that says the test passed comes through this stream, and a control that only interacts with its child
    ///  answers nothing there and asks for the null device instead.
    pub(crate) standard_output: Stdio,
    /// What:
    ///  The child's standard error, a pipe this file reads by default.
    /// Why:
    ///  A failure of the child is read from here, and one control sends it to a file of its own.
    pub(crate) standard_error: Stdio,
    /// What:
    ///  How long the child may run before `finished` counts it as one that did not end.
    ///  The sibling a reader might expect is no field at all, and then every control that needs a short bound
    ///  (the controls of this file's own deadline) would have to wait for the long one.
    /// Why:
    ///  A control of the deadline reaches it in a moment, and every other control takes `CHILD_TEST_DEADLINE`.
    pub(crate) deadline: Duration,
}

/// What:
///  The plain request of a child:
///  no variable of its own,
///  one test that is not `#[ignore]`d,
///  this test binary started directly,
///  two pipes and a deadline of `CHILD_TEST_DEADLINE`.
/// Why:
///  A control states only the fields it needs, and every field it leaves out means the same plain thing here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function plainRequest(): ChildTest { return { exactTest: '', environment: [], clearedEnvironment: [], ignored: false, runningProgram: undefined, argumentsBeforeTheTestName: [], programName: undefined, beforeTheExec: undefined, stdin: 'ignore', stdout: 'pipe', stderr: 'pipe', deadline: CHILD_TEST_DEADLINE }; }
/// ```
impl Default for ChildTest<'_> {
    fn default() -> Self {
        return ChildTest {
            exact_test: "",
            environment: &[],
            cleared_environment: &[],
            ignored: false,
            running_program: None,
            arguments_before_the_test_name: &[],
            program_name: None,
            takes_its_own_session: false,
            before_the_exec: None,
            standard_input: Stdio::null(),
            standard_output: Stdio::piped(),
            standard_error: Stdio::piped(),
            deadline: CHILD_TEST_DEADLINE,
        };
    }
}

/// What:
///  Start the child `request` names and answer it running,
///  owned by the test thread's fixtures and by this process (through `spawn_owned`).
/// Why:
///  A control that talks to its child, or waits for it with a bound of its own, needs the process itself;
///  `finished` and `passed` are this function plus the wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function started(request: ChildTest): Child { return spawnOwned(theCommand(request)); }
/// ```
pub(crate) fn started(request: ChildTest<'_>) -> Child {
    let takes_its_own_session: bool = request.takes_its_own_session;
    let mut command: Command = command_of(request);
    if takes_its_own_session {
        let child: Child = command.spawn().expect("the child test starts");
        own_group(&child);
        return child;
    }
    return spawn_owned(&mut command).expect("the child test starts");
}

/// What:
///  Start the child `request` names, wait until it ends or `request.deadline` passes,
///  and answer what it wrote and how it ended,
///  or nothing when the deadline passed and the child was killed.
/// Why:
///  A wait without a bound holds the whole run when the child never ends,
/// and a control that answers nothing at the deadline is a failed control that names the child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finished(request: ChildTest): Output | undefined {
///   const child = started(request); const began = now();
///   for (;;) {
///     if (child.exited) return output(child);
///     if (now() - began >= request.deadline) { child.kill(); child.wait(); return undefined; }
///     sleep(LOOK_EVERY);
///   }
/// }
/// ```
pub(crate) fn finished(request: ChildTest<'_>) -> Option<Output> {
    let deadline: Duration = request.deadline;
    let mut child: Child = started(request);
    let begun: Instant = Instant::now();
    let status: std::process::ExitStatus = loop {
        if let Some(status) = child.try_wait().expect("the child's status") {
            break status;
        }
        if begun.elapsed() >= deadline {
            // The child is killed and then reaped, so the group `spawn_owned` owns holds no process the control lost.
            let _killed: std::io::Result<()> = child.kill();
            let _reaped: std::io::Result<std::process::ExitStatus> = child.wait();
            return None;
        }
        std::thread::sleep(LOOK_EVERY);
    };
    let stdout: Vec<u8> = what_the_pipe_holds(child.stdout.take());
    let stderr: Vec<u8> = what_the_pipe_holds(child.stderr.take());
    return Some(Output { status, stdout, stderr });
}

/// What:
///  Start the child `request` names, wait for the one test it runs,
///  and answer what it wrote once that test passed.
///  A child that did not end, failed, or ran no test fails this call with everything the child said.
/// Why:
///  This is the whole point of the child:
///  the parent's assertion is the child's result,
/// and a name that matched no test would otherwise pass a run that proved nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function passed(request: ChildTest): Output {
///   const output = finished(request); assert(output, 'deadline'); assert(output.status.ok); assert(output.stdout.includes(ONE_TEST_PASSED)); return output;
/// }
/// ```
pub(crate) fn passed(request: ChildTest<'_>) -> Output {
    let name: &str = request.exact_test;
    let deadline: Duration = request.deadline;
    let Some(output) = finished(request) else {
        panic!(
            "the child test {name} did not end within {} s, so it was killed: give its `ChildTest` a longer `deadline` if its work is slower than that on this host",
            deadline.as_secs_f64()
        );
    };
    let said: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    let heard: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert!(output.status.success(), "the child test {name} failed with {:?}:\n{said}\n{heard}", output.status.code());
    assert!(
        said.contains(ONE_TEST_PASSED),
        "the child test {name} ran no single passing test (a name that matches no test ends successfully and runs nothing):\n{said}\n{heard}"
    );
    return output;
}

/// What:
///  The command that starts the test binary on one test,
///  with everything `request` names and nothing else.
///  The sibling a reader might expect is the child itself, which `started` answers.
/// Why:
///  The six bodies this replaced each built this command, and this is the one place that decides the order of its words,
///  its streams and the request's variables.
///  One control hands this recipe to release code that runs a command with bounds of its own
///  (`push::bounded_push`), and one writes it into a shell script, so building it is separate from starting it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function commandOf(request: ChildTest): Command {
///   const c = request.runningProgram ? program(request.runningProgram, [binary]) : named(binary, request.programName);
///   c.args([...request.argumentsBeforeTheTestName, '--exact', request.exactTest, '--nocapture', '--test-threads=1']);
///   if (request.ignored) c.arg('--ignored');
///   for (const [k, v] of request.environment) c.env(k, v);
///   for (const k of request.clearedEnvironment) c.envRemove(k);
///   return c.stdin(request.standardInput).stdout(request.standardOutput).stderr(request.standardError);
/// }
/// ```
pub(crate) fn command_of(request: ChildTest<'_>) -> Command {
    let binary: PathBuf = this_test_binary();
    let mut command: Command = match request.running_program {
        Some(running) => {
            let mut outer: Command = Command::new(running.program);
            outer.args(running.arguments_before_the_test_binary).arg(binary.as_path());
            outer
        }
        None => {
            let mut own: Command = Command::new(binary.as_path());
            if let Some(name) = request.program_name {
                own.arg0(name);
            }
            own
        }
    };
    command.args(request.arguments_before_the_test_name);
    // `--exact` matches the path exactly, `--nocapture` lets the child's own prints reach the pipe read above, and one
    //  thread keeps the child's body the only test it runs.
    command.args(["--exact", request.exact_test, "--nocapture", "--test-threads=1"]);
    if request.ignored {
        command.arg("--ignored");
    }
    // The cleared names go first, so a control that both inherits a switch from this process and sets its own lands
    //  on the value it asked for.
    for variable in request.cleared_environment {
        command.env_remove(variable);
    }
    for (variable, value) in request.environment {
        command.env(variable, value);
    }
    command.stdin(request.standard_input).stdout(request.standard_output).stderr(request.standard_error);
    if let Some(hook) = request.before_the_exec {
        // SAFETY: the request's own documentation says its closure calls only async-signal-safe functions.
        unsafe {
            command.pre_exec(hook);
        }
    }
    return command;
}

/// What:
///  Every byte the pipe still holds once the child that wrote it has ended,
/// read without waiting for the pipe to end.
/// Why:
///  A process the child started inherits the child's pipe, and the pipe ends only when every process holding it lets go,
///  which one that never ends never does;
/// a read that waited for the end would wait for a process the control never started to watch,
///  where everything the child wrote is already in the pipe when the child ends (issue 6463).
/// Gotcha:
///  A read that starts before the child ends is not what this does: the pipe is read once the child is gone,
///  so a child that writes more than the pipe holds (64 KiB) still blocks until the deadline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whatThePipeHolds(pipe?: Readable): Uint8Array { if (!pipe) return []; setNonBlocking(pipe); const held = []; for (;;) { const c = pipe.read(8192); if (c.length === 0 || wouldBlock) return held; held.push(...c); } }
/// ```
fn what_the_pipe_holds(pipe: Option<impl Read + AsRawFd>) -> Vec<u8> {
    let Some(mut held) = pipe else {
        return Vec::new();
    };
    let descriptor: std::os::fd::RawFd = held.as_raw_fd();
    // SAFETY: the descriptor belongs to `pipe`, which outlives both calls, and neither call takes a pointer.
    let flags: libc::c_int = unsafe { libc::fcntl(descriptor, libc::F_GETFL) };
    if flags < 0 {
        eprintln!("the child's output could not be read without waiting for its end (errno {flags}): nothing of it is collected");
        return Vec::new();
    }
    // SAFETY: as above; the flag is set on this copy of the descriptor only.
    let _set: libc::c_int = unsafe { libc::fcntl(descriptor, libc::F_SETFL, flags | libc::O_NONBLOCK) };
    let mut collected: Vec<u8> = Vec::new();
    let mut buffer: [u8; 8_192] = [0; 8_192];
    loop {
        match held.read(buffer.as_mut_slice()) {
            // Zero bytes read is the end of the pipe, so everything the child wrote has been collected.
            Ok(0) => return collected,
            Ok(read) => collected.extend_from_slice(buffer.get(..read).unwrap_or(buffer.as_slice())),
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            // Nothing is there now, and the child has ended, so nothing of the child's will arrive later.
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return collected,
            Err(error) => {
                eprintln!("the child's output ended in an error ({error}): what is collected is what came before it");
                return collected;
            }
        }
    }
}
