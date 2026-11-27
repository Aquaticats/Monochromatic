//! What:
//!  The controls of the wait in `binary_support_observation_bound.rs`
//!  and of its faces in `binary_support_observation.rs`:
//!  the program a bounded line names,
//!  the bound counted from the command's own start,
//!  a command that ends by itself with its own status or signal,
//!  a death by the signal the bounded line's own `timeout` sends at its bound,
//!  one this wait ends at its bound,
//!  a descendant in a session of its own that holds one of the command's pipes,
//!  the two bounds one reading of a command's line gives the wait,
//!  and the hand a signal at the bound goes through.
//! Why:
//!  The bound is this wait's own promise and the ending is its own report:
//!  one timer must be on the command,
//!  a command that ended by itself must come back with its own status or signal,
//!  a command this wait ended must say so,
//!  a pipe another process holds must not hold the call past the bound,
//!  and no signal may name a process number whose process may be gone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const seen = observe(boundedScript('kill -KILL $$', 30)); // answered, never reported as the bound's doing
//! ```

/// The shared wait these controls are of.
use super::observe;
/// The disposable fixture directory,
/// the one writer of executable fixture files,
/// and its removal.
use crate::support::{
    Fixture,
    executable,
    fixture,
    remove,
};
/// The wait itself and what it answers,
/// for the controls that give a wait a child of their own,
/// the two bounds one reading of a command's line gives it,
/// and the shared backstop a command whose line states none runs under.
use crate::support_observation_bound::{
    Ending,
    LineBound,
    UNSTATED_BOUND,
    line_bound_of,
    wait_under_bound,
};
/// The five folders a record of the after-commit queue stands in,
/// which the wait's control moves one record through.
use git_policy_cli::work_item::{
    CLAIMED,
    DONE,
    FAILED,
    PENDING,
    RECOVERING,
};
/// `process_group` puts a command of a control in a group of its own.
use std::os::unix::process::CommandExt as _;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};
/// The command builder and the child's pipes.
use std::process::{
    Child,
    Command,
    Stdio,
};
/// The clock a control measures its wait with,
/// and the sleeps of its own waits.
use std::time::{
    Duration,
    Instant,
};

/// The program whose form a bound takes in this harness:
///  `/usr/bin/timeout`,
///  from `binary_support.rs`'s `bounded_for`.
const TIMEOUT: &str = "/usr/bin/timeout";

/// The shell every scripted command of these controls is.
const SHELL: &str = "/bin/sh";

/// The bound these controls give a command this wait must end,
/// in seconds:
///  the number `timeout` is given,
///  and the value the wait reads back as the bound that line states.
const ONE_SECOND: u64 = 1;

/// The bound these controls give a command that ends by itself,
/// in seconds:
///  ample for a shell that exits at once,
///  and the bound the command's own line is read for.
const AMPLE_SECONDS: u64 = 30;

/// How long a descendant of a control holds the pipes it inherited,
/// in seconds.
/// Why longer than every bound here and short of the runner's five-minute stop:
///  a wait that reads a pipe to its end waits for this descendant,
///  so the control shows that wait,
///  and a wait that keeps its bound answers long before it.
const HOLDS_SECONDS: u64 = 30;

/// How long a control lets its wait take before it says the wait lost its bound.
/// Why five seconds:
///  the one-second bound these controls state,
///  the grace the wait gives a held pipe,
///  and room for the scheduler
///  of a loaded host;
///  the descendant these controls start holds its pipes for `HOLDS_SECONDS`,
///  so a wait that waits for it shows itself
///  far past this window.
const PATIENCE: Duration = Duration::from_secs(5);

/// The bound a control gives a wait whose command states a shorter bound in its own line,
/// in seconds.
/// Why five:
///  the line's own timer then fires with this wait's time still to spare,
///  so the exit is what this wait reads and its deadline is not what decides,
///  which is the order issue 6324 saw on a loaded host
///  and which no control can force by holding this process back.
const OUTLASTING_THE_LINES_BOUND_SECONDS: u64 = 5;

/// The bound this control gives the command it lets run past it,
/// in seconds,
/// and the wait it hands that command.
/// Why four:
///  the wait must answer in a quarter of this bound,
///  and it must be the whole bound that a wait counting from its call would spend,
///  so each side of the comparison is four times away from the other one.
///  Measured on 2026-11-25 with the bound a second: the wait took 0.65 to 2.46 ms quiet,
///  and 233 and 238 ms in two of eight runs on this host, spent on its `/proc` read (27 and
///  50 ms), its two pipe-drain thread creations (114 and 118 ms) and the kill with its reap
///  (51 and 82 ms), so a fifth of a second left no room for the work it was measuring.
const SPENT_BOUND_SECONDS: u64 = 4;

/// How much longer than its bound the command of the control below runs before the wait is
/// reached,
/// so that nothing of the bound is left.
const SPENT_OVERRUN: Duration = Duration::from_millis(200);

/// How long a wait may take when the bound it is given has already passed.
/// Why a quarter of the bound:
///  the wait has nothing left to wait for,
///  so this covers the signal and the reap together with the threads and the `/proc` reads it makes
///  before it can answer,
///  and a wait that counts its bound from the moment it is called takes the whole bound instead.
const ALREADY_SPENT_PATIENCE: Duration = Duration::from_millis(SPENT_BOUND_SECONDS * 1_000 / 4);

/// The bound the control below hands its command,
/// built from the seconds it states.
fn spent_bound() -> Duration {
    return Duration::from_secs(SPENT_BOUND_SECONDS);
}

/// What:
///  A command in the harness's own bounded form,
///  built as `bounded_for` builds one:
///  `timeout --signal=KILL <seconds> <script>`,
///  with the fixture's hermetic environment and the fixture as its directory.
/// Why:
///  Every command of the binary controls carries its bound in its own line,
///  and the bound this wait keeps is the one that line states;
///  the wait starts the program such a line names,
///  which it does from the environment the line sets.
fn bounded_script(
    made: &Fixture,
    script: &Path,
    seconds: u64,
) -> Command {
    let mut command: Command = Command::new(TIMEOUT);
    command
        .arg("--signal=KILL")
        .arg(seconds.to_string())
        .arg(script);
    crate::support::hermetic(&mut command, made, std::ffi::OsStr::new("/usr/bin:/bin"));
    command.current_dir(made.root.as_path());
    return command;
}

/// One executable script in the fixture's directory.
fn script(
    made: &Fixture,
    name: &str,
    body: &str,
) -> PathBuf {
    let path: PathBuf = made.root.join(name);
    executable(path.as_path(), body.as_bytes());
    return path;
}

/// The text of a panic a control caught,
/// whether the payload was a `String` or a `&str`.
fn message_of(payload: Box<dyn std::any::Any + Send>) -> String {
    return match payload.downcast_ref::<String>() {
        Some(text) => text.clone(),
        None => match payload.downcast_ref::<&str>() {
            Some(text) => (*text).to_string(),
            None => String::from("(a panic with no text)"),
        },
    };
}

/// What a descendant of these controls is:
///  a process in a session of its own that holds the pipes it inherited
///  until the fixture releases it through a named pipe.
/// Why:
///  A descendant that must outlive its command cannot be signalled by its control:
///  a control may signal only a process it started itself,
///  and this one is started by the command.
///  So the control releases it by writing one line into a pipe of the fixture's own,
///  and reads a marker the descendant writes as it goes,
///  which is also what shows that nothing of the control outlived it.
fn descendant(release_and_gone: (&Path, &Path)) -> String {
    let (release, gone) = release_and_gone;
    return format!(
        "/usr/bin/setsid {SHELL} -c 'read -r line < \"{}\"; printf gone > \"{}\"' &\n",
        release.display(),
        gone.display()
    );
}

/// A named pipe the fixture can release a descendant through.
fn named_pipe(path: &Path) {
    let named: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_encoded_bytes())
        .expect("a fixture path holds no NUL");
    // SAFETY: `mkfifo` reads the NUL-terminated path it is given and touches no memory of this process.
    let made: libc::c_int = unsafe { libc::mkfifo(named.as_ptr(), 0o600) };
    assert_eq!(
        made,
        0,
        "the fixture makes the pipe {}: {}",
        path.display(),
        std::io::Error::last_os_error()
    );
}

/// Release the descendant that holds `release` open,
/// and wait until it says it is gone.
/// Why the wait is bounded:
///  a descendant that never exits would hold the fixture's pipes for ever,
///  so a control that cannot see it go fails naming it.
fn released(release_and_gone: (&Path, &Path)) {
    let (release, gone) = release_and_gone;
    let mut writing: std::fs::File = std::fs::OpenOptions::new()
        .write(true)
        .open(release)
        .expect("the descendant holds the other end of the release pipe");
    let _wrote: std::io::Result<()> = std::io::Write::write_all(&mut writing, b"go\n");
    let began: Instant = Instant::now();
    while !gone.exists() {
        assert!(
            began.elapsed() < PATIENCE,
            "the descendant released through {} did not say it was gone within {PATIENCE:?}",
            release.display()
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// A child of the control's own in a process group of its own,
/// with its pipes,
/// ready for `wait_under_bound`.
fn spawned_beside_the_control(command: &mut Command) -> (Child, libc::pid_t) {
    let child: Child = command
        .process_group(0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the command starts");
    let group: libc::pid_t =
        libc::pid_t::try_from(child.id()).expect("a process ID fits the pid type");
    return (child, group);
}

/// What:
///  The two bounds one reading of a command's line gives the wait,
///  and the value the shipped faces hand on,
///  with no clock in it:
///  a line of this harness's bounded form states the bound it names and this wait keeps the same one,
///  and a line that states none leaves the shared backstop.
/// Why:
///  The stated half is what tells the death the line's own `timeout` deals from a command's own `SIGKILL`,
///  and on the shipped path it is invisible:
///  the wait keeps the bound the line states,
///  so its own deadline is the earlier of the two and decides first in every run that is not late.
///  A wiring that dropped the stated half therefore left every control green
///  (found by the integration pass of 2026-11-20,
/// which passed `None` for it and ran the same controls),
///  which is the path issue 6324 was about.
///  So the reading is asserted here,
///  and the shipped face's own line is asserted to hand that reading to the wait,
///  because no clock in a control can reach the stated half on a quiet host.
#[test]
fn a_lines_two_bounds_are_read_once_and_reach_the_wait() {
    let made: Fixture = fixture("observe-line-bounds");
    let command: PathBuf = script(
        &made,
        "keeps-alive",
        format!("#!/bin/sh\nexec /usr/bin/sleep {HOLDS_SECONDS}\n").as_str(),
    );
    // A line of the bounded form states the seconds it names, and this wait keeps the same bound.
    let wrapped: Command = bounded_script(&made, command.as_path(), ONE_SECOND);
    let line: LineBound = line_bound_of(&wrapped);
    assert_eq!(
        line.stated,
        Some(Duration::from_secs(ONE_SECOND)),
        "a line of the bounded form states the seconds it names"
    );
    assert_eq!(
        line.kept,
        Duration::from_secs(ONE_SECOND),
        "this wait keeps the bound the command's own line states"
    );
    // A line that names no `timeout` states nothing, so the shared backstop is what this wait keeps.
    let mut plain: Command = Command::new(SHELL);
    plain.arg(command.as_path());
    let unstated: LineBound = line_bound_of(&plain);
    assert_eq!(
        unstated.stated, None,
        "a line that is not the bounded form states no bound of its own"
    );
    assert_eq!(
        unstated.kept, UNSTATED_BOUND,
        "a command whose line states no bound runs under the shared backstop"
    );
    // A `timeout` asked for no limit states nothing either: nothing waits without a bound.
    let no_limit: Command = bounded_script(&made, command.as_path(), 0);
    assert_eq!(
        line_bound_of(&no_limit).stated,
        None,
        "`timeout --signal=KILL 0` asks for no limit, so the line states no bound"
    );
    // The shipped faces hand the wait the bounds one reading gave, and never a pair of their own.
    let faces: &str = include_str!("binary_support_observation.rs");
    assert!(
        faces.contains("wait_under_bound(child, Some(group), line_bound_of(command), input)"),
        "the shipped face hands the wait the bounds one reading of the command's line gave, never a pair it made itself"
    );
    remove(&made);
}

/// What:
///  A bound counts from the command's own start,
///  so a command that has already run past its bound is ended when the wait is called,
///  not one bound later.
/// Why:
///  A control states a bound on the command,
///  and the command began before this wait was reached:
///  a bound counted from the call would give the command up to twice the bound it was given.
#[test]
fn a_bound_counts_from_the_commands_own_start() {
    let made: Fixture = fixture("observe-spent-bound");
    let command: PathBuf = script(
        &made,
        "runs-past-its-bound",
        format!("#!/bin/sh\nexec /usr/bin/sleep {HOLDS_SECONDS}\n").as_str(),
    );
    // A child of this control's own with no timer around it,
    //  so the only bound on it is the one the wait is given.
    let mut started: Command = Command::new(SHELL);
    started
        .arg(command.as_path())
        .current_dir(made.root.as_path());
    let (child, group): (Child, libc::pid_t) = spawned_beside_the_control(&mut started);
    // The command runs for longer than its own bound before the wait is called at all.
    std::thread::sleep(spent_bound() + SPENT_OVERRUN);
    let began: Instant = Instant::now();
    // The command's own line states no bound of its own: it is a script of this control's, not a bounded form.
    let answer = wait_under_bound(
        child,
        Some(group),
        LineBound {
            kept: spent_bound(),
            stated: None,
        },
        b"",
    );
    let elapsed: Duration = began.elapsed();
    println!("the wait answered {:?} after {elapsed:?}", answer.ending);
    assert_eq!(
        answer.ending,
        Ending::AtTheBound,
        "a command past its bound is ended by this wait, not by itself"
    );
    assert!(
        elapsed < ALREADY_SPENT_PATIENCE,
        "the bound counts from the command's own start, and the wait took {elapsed:?} though nothing of the bound was left"
    );
    remove(&made);
}

/// What:
///  A command's own timer is never taken for this wait's ending,
///  even when the wait is reached long after the command started.
/// Why:
///  The review that found this ran a `timeout --signal=KILL 1` child,
///  held this process back a hundred milliseconds,
///  and saw the inner timer's death answered as the command's own;
///  the bound now counts from the child's own start,
///  which no work of this process after the spawn can move,
///  so the wait is the one that ends it.
#[test]
fn the_inner_timer_of_a_child_is_never_the_ending_this_wait_reports() {
    let made: Fixture = fixture("observe-inner-timer");
    let command: PathBuf = script(
        &made,
        "kept-alive",
        format!("#!/bin/sh\nexec /usr/bin/sleep {HOLDS_SECONDS}\n").as_str(),
    );
    let mut wrapped: Command = bounded_script(&made, command.as_path(), ONE_SECOND);
    let (child, group): (Child, libc::pid_t) = spawned_beside_the_control(&mut wrapped);
    // The permitted interleaving the review reproduced: this process does other work before it waits.
    std::thread::sleep(Duration::from_millis(100));
    let answer = wait_under_bound(
        child,
        Some(group),
        LineBound {
            kept: Duration::from_secs(1),
            stated: Some(Duration::from_secs(ONE_SECOND)),
        },
        b"",
    );
    println!(
        "the wait answered {:?} with status {:?}",
        answer.ending, answer.status
    );
    assert_ne!(
        answer.ending,
        Ending::Itself,
        "the child's own `timeout` fired inside the bound this wait states, so this wait must not call that ending the child's own"
    );
    remove(&made);
}

/// What:
///  A death by the `SIGKILL` the bounded line's own `timeout` sends at the bound is the bound's doing,
///  though this wait read the exit rather than its own deadline running out first.
/// Why:
///  Issue 6324's gate failure:
///  `timeout --signal=KILL 1 <a script that never answers>` dies by `SIGKILL` itself at its one second,
///  because `SIGKILL` cannot be ignored and it signals its own group,
///  and this wait then answers `code: None` with no ending named,
///  as though the command had ended by itself.
///  The two timers keep one value and this wait's deadline is the earlier of them,
///  so a `SIGKILL` at the bound is this wait's to claim.
///  The order is forced here with no host load:
///  the wait is given a bound longer than the one the line states,
///  so the line's own timer fires while this wait is still waiting for the exit and the exit is what it reads.
#[test]
fn a_death_by_the_lines_own_timeout_at_its_bound_is_the_bounds_ending() {
    let made: Fixture = fixture("observe-line-timer-at-the-bound");
    let command: PathBuf = script(
        &made,
        "kept-alive",
        format!("#!/bin/sh\nexec /usr/bin/sleep {HOLDS_SECONDS}\n").as_str(),
    );
    let mut wrapped: Command = bounded_script(&made, command.as_path(), ONE_SECOND);
    let (child, group): (Child, libc::pid_t) = spawned_beside_the_control(&mut wrapped);
    let began: Instant = Instant::now();
    let answer = wait_under_bound(
        child,
        Some(group),
        LineBound {
            kept: Duration::from_secs(OUTLASTING_THE_LINES_BOUND_SECONDS),
            stated: Some(Duration::from_secs(ONE_SECOND)),
        },
        b"",
    );
    let elapsed: Duration = began.elapsed();
    println!(
        "the wait answered {:?} with status {:?} after {elapsed:?}",
        answer.ending, answer.status
    );
    assert_eq!(
        std::os::unix::process::ExitStatusExt::signal(&answer.status),
        Some(libc::SIGKILL),
        "the line's own `timeout` ends its group and itself with `SIGKILL` at the one second it states: {:?}",
        answer.status
    );
    assert_eq!(
        answer.ending,
        Ending::AtTheBound,
        "a `SIGKILL` death at the bound the command's own line states is the bound's doing, not an ending the command chose"
    );
    remove(&made);
}

/// What:
///  A command that kills itself with `SIGKILL` before the bound its line states keeps that ending,
///  though its line's `timeout` dies by the same signal.
/// Why:
///  The line's `timeout` re-raises on itself the signal its command died by (coreutils' `cleanup` in `src/timeout.c`),
///  so a `SIGKILL` a command chose and the `SIGKILL` the bound deals look the same in the status,
///  and the bound the line states,
///  already reached,
///  is what tells them apart.
///  Without that,
///  this wait would call a command's own death the bound's.
///  The line states the ample bound,
///  and the wait keeps the same one,
///  so no stall of this process short of the whole bound can make the read late enough to reach it:
///  the command dies a moment after it starts,
///  and the exit is what the wait reads.
#[test]
fn a_command_that_kills_itself_before_its_bound_keeps_its_own_ending() {
    let made: Fixture = fixture("observe-own-kill-before-the-bound");
    let command: PathBuf = script(&made, "kills-itself", "#!/bin/sh\nkill -KILL $$\n");
    let mut wrapped: Command = bounded_script(&made, command.as_path(), AMPLE_SECONDS);
    let (child, group): (Child, libc::pid_t) = spawned_beside_the_control(&mut wrapped);
    let answer = wait_under_bound(
        child,
        Some(group),
        LineBound {
            kept: Duration::from_secs(AMPLE_SECONDS),
            stated: Some(Duration::from_secs(AMPLE_SECONDS)),
        },
        b"",
    );
    println!(
        "the wait answered {:?} with status {:?}",
        answer.ending, answer.status
    );
    assert_eq!(
        std::os::unix::process::ExitStatusExt::signal(&answer.status),
        Some(libc::SIGKILL),
        "the line's `timeout` re-raises the signal its command died by, so the status is still a `SIGKILL` death: {:?}",
        answer.status
    );
    assert_eq!(
        answer.ending,
        Ending::Itself,
        "the command killed itself a moment after it started, so no bound of its line had been reached"
    );
    remove(&made);
}

/// What:
///  The signal at the bound reaches the command while it is still this wait's to signal,
///  and reaches no other process.
/// Why:
///  A command number is reused once its process is reaped,
///  so a signal sent after a reap can end another command of the account;
///  this control starts a second child of its own beside the command,
///  and the signal that ends the command must leave that one running.
#[test]
fn the_signal_at_the_bound_reaches_the_command_and_no_other_process() {
    let made: Fixture = fixture("observe-signal-identity");
    let command: PathBuf = script(
        &made,
        "waits-for-ever",
        format!("#!/bin/sh\nexec /usr/bin/sleep {HOLDS_SECONDS}\n").as_str(),
    );
    let mut started: Command = bounded_script(&made, command.as_path(), ONE_SECOND);
    let (child, group): (Child, libc::pid_t) = spawned_beside_the_control(&mut started);
    // A second child of this control's own, in a group of its own, that no signal may reach.
    let mut beside: Command = Command::new("/usr/bin/sleep");
    beside.arg(HOLDS_SECONDS.to_string());
    beside.current_dir(made.root.as_path());
    let mut untouched: Child = beside
        .process_group(0)
        .spawn()
        .expect("the second child starts");
    let answer = wait_under_bound(
        child,
        Some(group),
        LineBound {
            kept: Duration::from_secs(1),
            stated: Some(Duration::from_secs(ONE_SECOND)),
        },
        b"",
    );
    let stayed: bool = untouched
        .try_wait()
        .expect("the second child's state")
        .is_none();
    let _ended: std::io::Result<()> = untouched.kill();
    let _waited: std::io::Result<std::process::ExitStatus> = untouched.wait();
    assert_eq!(
        answer.ending,
        Ending::AtTheBound,
        "the wait ended the command at its bound"
    );
    assert_eq!(
        std::os::unix::process::ExitStatusExt::signal(&answer.status),
        Some(libc::SIGKILL),
        "a command whose status is a `SIGKILL` death was alive when that signal went out: {:?}",
        answer.status
    );
    assert!(
        stayed,
        "the signal reached only the command, and the second child of this control still ran"
    );
    remove(&made);
}

/// What:
///  `THE_WAIT` is every file the wait is made of,
///  each with its text and the text that proves it is still about the wait.
/// Why:
///  The ban below is about the wait,
///  and the alpha.48 split moved nine items out of `binary_support_observation.rs` and seven out of
///  `binary_support_observation_bound.rs` into two siblings,
///  so a ban over the two originals stopped reading half of it (issue 6599).
///  This folder is a target of its own,
///  where the shared scan of `source_guard` is not compiled,
///  so the files are named here:
///  each receiver is named,
///  and each entry states the text the file is still about,
///  so a file that leaves the wait fails beside a file that was never added.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_WAIT = [{ file: 'binary_support_observation.rs', source: read('binary_support_observation.rs'), about: 'wait_under_bound(' }, ...];
/// ```
const THE_WAIT: &[(&str, &str, &str)] = &[
    (
        "binary_support_observation.rs",
        include_str!("binary_support_observation.rs"),
        "wait_under_bound(",
    ),
    (
        "binary_support_observation_bound.rs",
        include_str!("binary_support_observation_bound.rs"),
        "libc::WNOWAIT",
    ),
    (
        "binary_support_observation_bound_what_the_wait_answers_with.rs",
        include_str!("binary_support_observation_bound_what_the_wait_answers_with.rs"),
        "LineBound",
    ),
    (
        "binary_support_observation_waiting_for_the_after_commit_work.rs",
        include_str!("binary_support_observation_waiting_for_the_after_commit_work.rs"),
        "settle_service_work(",
    ),
];

/// What:
///  `THE_BOUNDED_WAIT` is the one file whose order this control pins.
/// Why:
///  The look before the signal and the pin the command is signalled through are a sequence inside one file,
///  and the control names it so a move fails instead of passing over a file it never read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_BOUNDED_WAIT = 'binary_support_observation_bound.rs';
/// ```
const THE_BOUNDED_WAIT: &str = "binary_support_observation_bound.rs";

/// What:
///  `files_that_left_the_wait(sources)` is every file of `THE_WAIT` whose text no longer holds what that file is
///  about.
/// Why:
///  A file that stopped holding its own names has been split again,
///  and a ban that went on reading it would guard the less without saying so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const filesThatLeftTheWait = (sources) => sources.filter(([, source, about]) => !source.includes(about)).map(([file]) => file);
/// ```
fn files_that_left_the_wait(sources: &[(&str, &str, &str)]) -> Vec<String> {
    return sources
        .iter()
        .filter(|(_file, source, about)| {
            return !source.contains(*about);
        })
        .map(|(file, _source, about)| return format!("{file} holds no `{about}`"))
        .collect();
}

/// What:
///  `lines_holding(source, needles)` is every line of one source that holds one of `needles`,
///  comments dropped.
/// Why:
///  Every rule below is about a call in code,
///  and a file explains in a comment what it may not do as often as it says what it does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const linesHolding = (source, needles) => source.split('\n').filter((line) => !line.trimStart().startsWith('//')).filter((line) => needles.some((needle) => line.includes(needle)));
/// ```
fn lines_holding<'a>(
    source: &'a str,
    needles: &[&str],
) -> Vec<&'a str> {
    let mut found: Vec<&'a str> = Vec::new();
    for line in source.lines() {
        if line.trim_start().starts_with("//") {
            continue;
        }
        if !needles.iter().any(|needle| return line.contains(*needle)) {
            continue;
        }
        found.push(line);
    }
    return found;
}

/// What:
///  No signal in the wait's files takes a number that may already belong to another process.
/// Why:
///  A control cannot wait for a process number to be reused,
///  so this is a control over the source:
///  the command is signalled through a pinned process,
///  the group is signalled only after a `WNOWAIT` look and before the command is reaped,
///  and no other signal call exists in any file the split made of the wait.
#[test]
fn no_signal_in_the_wait_takes_a_number_its_child_no_longer_holds() {
    let left: Vec<String> = files_that_left_the_wait(THE_WAIT);
    assert!(
        left.is_empty(),
        "a file of the wait no longer holds what it is about, so this ban reads less than it says: move the \
         entry to the file that holds it now, or delete it:\n{}",
        left.join("\n")
    );
    for (name, source, _about) in THE_WAIT {
        assert!(
            lines_holding(source, &["killpg("]).is_empty(),
            "{name}: a whole group is signalled through its leader with `kill(-leader)`, never through `killpg`"
        );
        for line in lines_holding(source, &["libc::kill("]) {
            let after: &str = &line[line
                .find("libc::kill(")
                .expect("the line holds the call it was found by")..];
            assert!(
                after.starts_with("libc::kill(-leader, libc::SIGKILL)"),
                "{name}: a signal may name only a group its leader still pins, and this call is {after}"
            );
        }
    }
    let bound: &str = THE_WAIT
        .iter()
        .find(|(file, _source, _about)| return *file == THE_BOUNDED_WAIT)
        .map(|(_file, source, _about)| return *source)
        .expect("the bounded wait is one of the wait's files");
    let checked: usize = bound
        .find("WNOWAIT | libc::WNOHANG")
        .expect("the wait looks before it signals, without reaping");
    let signalled: usize = bound
        .find("libc::kill(-leader, libc::SIGKILL)")
        .expect("the group is signalled at the bound");
    assert!(
        checked < signalled,
        "the wait looks at the child without reaping it before it signals, so the number it names is still the child's"
    );
    assert!(
        bound.contains("pin.signal(libc::SIGKILL)"),
        "the command itself is signalled through its pin, never by a number"
    );
}

/// What:
///  A command that ends by itself keeps its own ending,
///  whatever that ending was.
/// Why:
///  This wait names the bound's ending only for a death by `SIGKILL` at or after the bound the command's own line states:
///  an `exit 124`,
///  an `exit 137`,
///  a death by `SIGPIPE`,
///  and a command's own `SIGKILL` before its line's bound are all its own,
///  and only a command the bound ended may be reported as the bound's doing.
#[test]
fn a_command_that_ends_by_itself_keeps_its_own_status_and_signal() {
    let made: Fixture = fixture("observe-endings");
    let cases: [(&str, Option<i32>); 5] = [
        ("exit 0", Some(0)),
        ("exit 124", Some(124)),
        ("exit 137", Some(137)),
        ("kill -PIPE $$", None),
        ("kill -KILL $$", None),
    ];
    for (index, (body, code)) in cases.iter().enumerate() {
        let name: String = format!("ending-{index}");
        let command: PathBuf = script(
            &made,
            name.as_str(),
            format!("#!/bin/sh\n{body}\n").as_str(),
        );
        let mut wrapped: Command = bounded_script(&made, command.as_path(), AMPLE_SECONDS);
        let seen = observe(&mut wrapped, b"");
        println!("{body}: the wait answered code {:?}", seen.code);
        assert_eq!(
            seen.code, *code,
            "{body}: the command ended by itself, so its own status is the answer"
        );
    }
    remove(&made);
}

/// What:
///  A command that never ends is ended by this wait at the bound its own line states,
///  and the answer says this wait ended it.
/// Why:
///  Nothing waits without a bound,
///  a test included:
///  a command that waits for ever reports nothing and a suite that waits in one control reports nothing either.
///  The sentence must name the bound this wait kept,
///  because no status the command came back with can say it.
#[test]
fn a_command_that_never_ends_is_ended_at_its_bound() {
    let made: Fixture = fixture("observe-bound");
    let command: PathBuf = script(
        &made,
        "waits-for-ever",
        format!("#!/bin/sh\nexec /usr/bin/sleep {HOLDS_SECONDS}\n").as_str(),
    );
    let began: Instant = Instant::now();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut wrapped: Command = bounded_script(&made, command.as_path(), ONE_SECOND);
        return observe(&mut wrapped, b"");
    }));
    let elapsed: Duration = began.elapsed();
    let payload = outcome.expect_err(
        "a command this wait ended is reported, never answered as one that ended by itself",
    );
    let message: String = message_of(payload);
    println!("the wait returned after {elapsed:?}, and said: {message}");
    assert!(
        message.contains("was ended at its 1s wall-clock bound"),
        "the sentence says what the bound did: {message}"
    );
    assert!(
        elapsed < PATIENCE,
        "the wait kept its one-second bound, and returned after {elapsed:?}"
    );
    remove(&made);
}

/// What:
///  A descendant in a session of its own that holds one of the command's pipes cannot hold the call past the bound,
///  and the sentence says that the pipe is still held.
/// Why:
///  This is the measured defect:
///  with a child whose descendant held its output pipe,
///  the wait returned after 8,005 ms against a bound of one
///  second,
///  and a descendant that never ends holds such a wait for ever.
///  The descendant is released by this control before it returns.
#[test]
fn a_command_whose_descendant_holds_its_output_is_ended_at_its_bound() {
    let made: Fixture = fixture("observe-descendant");
    let release: PathBuf = made.root.join("release");
    let gone: PathBuf = made.root.join("descendant-gone");
    named_pipe(release.as_path());
    let body: String = format!(
        "#!/bin/sh\n{}exec /usr/bin/sleep {HOLDS_SECONDS}\n",
        descendant((release.as_path(), gone.as_path()))
    );
    let command: PathBuf = script(&made, "starts-a-descendant", body.as_str());
    let began: Instant = Instant::now();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut wrapped: Command = bounded_script(&made, command.as_path(), ONE_SECOND);
        return observe(&mut wrapped, b"");
    }));
    let elapsed: Duration = began.elapsed();
    // The descendant is this control's own to release, whatever the wait answered.
    released((release.as_path(), gone.as_path()));
    let payload = outcome.expect_err("a command that never ends is ended by this wait");
    let message: String = message_of(payload);
    println!(
        "the wait returned after {elapsed:?} against its one-second bound, and said: {message}"
    );
    assert!(
        message.contains("was ended at its 1s wall-clock bound"),
        "the sentence says what the bound did: {message}"
    );
    assert!(
        message.contains("still holds its standard output"),
        "the sentence names the pipe a descendant still holds: {message}"
    );
    assert!(
        elapsed < PATIENCE,
        "the wait kept its bound while a descendant held the pipe, and returned after {elapsed:?}"
    );
    remove(&made);
}

/// What:
///  A command that ends by itself says that a descendant still holds its output.
/// Why:
///  The wait takes what the pipes gave under one grace and leaves the rest to whoever holds it,
///  so an output that a process outliving the command keeps open may be short of what the command wrote;
///  silence about that would turn a missing byte into a wrong comparison,
///  and a control would read a short answer as the whole one.
///  The descendant is released by this control before it returns.
#[test]
fn a_command_that_ends_by_itself_says_that_a_descendant_still_holds_its_output() {
    let made: Fixture = fixture("observe-descendant-holds");
    let release: PathBuf = made.root.join("release");
    let gone: PathBuf = made.root.join("descendant-gone");
    named_pipe(release.as_path());
    let body: String = format!(
        "#!/bin/sh\n{}exit 0\n",
        descendant((release.as_path(), gone.as_path()))
    );
    let command: PathBuf = script(&made, "leaves-a-descendant", body.as_str());
    let began: Instant = Instant::now();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut wrapped: Command = bounded_script(&made, command.as_path(), AMPLE_SECONDS);
        return observe(&mut wrapped, b"");
    }));
    let elapsed: Duration = began.elapsed();
    // The descendant is this control's own to release, whatever the wait answered.
    released((release.as_path(), gone.as_path()));
    let payload = outcome.expect_err(
        "an output a descendant may still be writing is never answered as the whole one",
    );
    let message: String = message_of(payload);
    println!(
        "the wait returned after {elapsed:?} against its bound of {AMPLE_SECONDS} s, and said: {message}"
    );
    assert!(
        message.contains("ended by itself"),
        "the sentence says the command ended by itself: {message}"
    );
    assert!(
        message.contains("still holds its standard output"),
        "the sentence names the pipe a descendant still holds: {message}"
    );
    assert!(
        elapsed < PATIENCE,
        "the wait gave its one grace and no more, and returned after {elapsed:?}"
    );
    remove(&made);
}

/// What:
///  A command that writes more than a pipe holds keeps every byte it wrote.
/// Why:
///  Both pipes are drained from the spawn,
///  so a command that fills its output before it reads its input is not deadlocked,
///  and what it wrote arrives whole rather than up to the length of one pipe buffer.
#[test]
fn a_command_that_writes_more_than_a_pipe_holds_keeps_all_of_it() {
    let made: Fixture = fixture("observe-many-bytes");
    // One mebibyte, several times what a pipe holds.
    let written: usize = 1 << 20;
    let blocks: usize = written / 4096;
    let body: String =
        format!("#!/bin/sh\nexec /usr/bin/dd if=/dev/zero bs=4096 count={blocks} 2>/dev/null\n");
    let command: PathBuf = script(&made, "writes-much", body.as_str());
    let mut wrapped: Command = bounded_script(&made, command.as_path(), AMPLE_SECONDS);
    let seen = observe(&mut wrapped, b"");
    assert_eq!(
        seen.code,
        Some(0),
        "the command that writes much ends by itself"
    );
    assert_eq!(
        seen.stdout.len(),
        written,
        "every byte the command wrote is in the answer"
    );
    remove(&made);
}

/// One move a control's step makes before a numbered read of the wait.
/// Why a table and not a rule:
///  the wait's window is a record moving between two of its reads,
/// so a control has to say which read each move comes
///  before;
/// the same table then holds for any order the wait reads its folders in and for any number of samples.
struct AMove {
    /// The ordinal of the read it comes before,
    /// counted from one across every sample of one wait.
    at: usize,
    /// The folder the record must stand in,
    /// or the move is not made.
    from: &'static str,
    /// The folder it is moved into.
    to: &'static str,
}

thread_local! {
    /// The queue a control's step moves one record in,
    /// and the record's name,
    /// which no step of an ordinary run holds.
    static THE_RECORD_A_STEP_MOVES: std::cell::RefCell<Option<(PathBuf, String)>> = const { std::cell::RefCell::new(None) };
    /// The moves that control makes,
    /// by the read they come before.
    static THE_MOVES: std::cell::RefCell<&'static [AMove]> = const { std::cell::RefCell::new(&[]) };
    /// How many reads of the wait the control's step has been given,
    /// counted from one.
    static THE_READ_ORDINAL: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// Whether a read of a sample saw that record in one of the folders it is in flight in,
    /// which a step that runs immediately before that read knows.
    static A_READ_SAW_THE_RECORD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// What:
///  Make the control's moves for the read about to happen,
///  say whether that read is one that sees the record,
///  and settle the record once a read has seen it.
/// Why:
///  `take_entry`,
/// `recover` and `settle_named` move a record between the three folders the wait reads,
///  and the wait's own window is a record moving between two of those reads.
///  A record standing in the folder this step is given is one the read that follows sees,
///  so `A_READ_SAW_THE_RECORD` is that read's answer;
///  a record a read has seen is settled before the next read,
/// so the wait ends rather than holding a record for ever.
fn make_the_controlled_moves(folder: &str) {
    let held: Option<(PathBuf, String)> = THE_RECORD_A_STEP_MOVES.with(|step| {
        return step.borrow().clone();
    });
    let Some((work, name)) = held else {
        return;
    };
    let ordinal: usize = THE_READ_ORDINAL.with(|at| {
        let now = at.get() + 1;
        at.set(now);
        return now;
    });
    let moves: &'static [AMove] = THE_MOVES.with(|tabled| {
        return *tabled.borrow();
    });
    for one in moves.iter().filter(|one| {
        return one.at == ordinal;
    }) {
        let standing: PathBuf = work.join(one.from).join(name.as_str());
        if !standing.is_file() {
            continue;
        }
        let _moved = std::fs::rename(standing.as_path(), work.join(one.to).join(name.as_str()));
    }
    let standing: PathBuf = work.join(folder).join(name.as_str());
    let seen_before: bool = A_READ_SAW_THE_RECORD.with(std::cell::Cell::get);
    if standing.is_file() {
        A_READ_SAW_THE_RECORD.with(|seen| {
            seen.set(true);
        });
    }
    if seen_before {
        for from in [PENDING, RECOVERING, CLAIMED] {
            let path: PathBuf = work.join(from).join(name.as_str());
            if path.is_file() {
                let _settled = std::fs::rename(path.as_path(), work.join(DONE).join(name.as_str()));
                break;
            }
        }
    }
}

/// What:
///  Run the wait once over a queue holding one record in `starts_in`,
/// with `moves` made between its reads,
///  and answer whether a read saw the record,
/// with the queue's listing.
/// Why:
///  Each control of the wait differs only in where its record starts and which reads its moves come before,
///  so one driver states the scene once and each control states its own window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { seen, listing } = await waitWith([{ at: 2, from: 'claimed', to: 'recovering' }], 'pending');
/// ```
fn wait_with_moves(
    moves: &'static [AMove],
    starts_in: &str,
    label: &str,
    what: &str,
) {
    let made: Fixture = fixture(label);
    let repository: PathBuf = made.root.join("repo");
    let git_dir: PathBuf = repository.join(".git");
    std::fs::create_dir_all(git_dir.as_path()).expect("the Git directory");
    let work: PathBuf = git_policy_cli::service_work::work_directory(git_dir.as_path());
    for state in [PENDING, RECOVERING, CLAIMED, DONE, FAILED] {
        std::fs::create_dir_all(work.join(state)).expect("a state folder");
    }
    let name: String = format!("subvolume-delete-{}", "0".repeat(40));
    std::fs::write(
        work.join(starts_in).join(name.as_str()),
        "a record a control moves\n",
    )
    .expect("the record");
    THE_RECORD_A_STEP_MOVES.with(|step| {
        *step.borrow_mut() = Some((work.clone(), name.clone()));
    });
    THE_MOVES.with(|held| {
        *held.borrow_mut() = moves;
    });
    THE_READ_ORDINAL.with(|at| {
        at.set(0);
    });
    A_READ_SAW_THE_RECORD.with(|seen| {
        seen.set(false);
    });
    crate::support_observation::run_before_each_folder_read(Some(make_the_controlled_moves));
    crate::support_observation::settle_service_work(repository.as_path());
    crate::support_observation::run_before_each_folder_read(None);
    THE_RECORD_A_STEP_MOVES.with(|step| {
        *step.borrow_mut() = None;
    });
    THE_MOVES.with(|held| {
        *held.borrow_mut() = &[];
    });
    let seen: bool = A_READ_SAW_THE_RECORD.with(std::cell::Cell::get);
    let listing: String = crate::support::work_listing(work.as_path());
    // The control's own sentence comes first, so a red says what the wait missed rather than only that it returned;
    // the settle check after it says the wait did end, which a wait that never returned would otherwise hide.
    assert!(
        seen,
        "a read of the wait sees the record while {what}, and none did: {listing}"
    );
    assert!(
        work.join(DONE).join(name.as_str()).is_file(),
        "the record the control moves ends where the wait reads it as settled: {listing}"
    );
    remove(&made);
}

/// What:
///  A read in the order a record moves catches a record that a claim and a recovery move on between its reads.
/// Why:
///  This is the order's own guard,
/// and it is a record claimed,
/// recovered and claimed again across three samples:
///  the moves come before the first,
/// third,
/// fifth,
/// sixth,
/// seventh and ninth reads,
/// so at every read of the old order
///  (`pending`,
/// `claimed`,
/// `recovering`) the record stands in a folder that sample has already read,
///  and the wait returns with it live in `claimed`.
///  Read in the order a record moves,
/// the second read of the first sample meets it in `recovering`.
///  Put the order back with the three samples kept,
/// and this control fails while the others pass.
#[test]
fn a_read_in_the_order_a_record_moves_catches_what_a_claim_and_a_recovery_move_between_its_reads() {
    wait_with_moves(
        &[
            AMove {
                at: 1,
                from: PENDING,
                to: RECOVERING,
            },
            AMove {
                at: 3,
                from: RECOVERING,
                to: CLAIMED,
            },
            AMove {
                at: 5,
                from: CLAIMED,
                to: RECOVERING,
            },
            AMove {
                at: 6,
                from: RECOVERING,
                to: PENDING,
            },
            AMove {
                at: 7,
                from: PENDING,
                to: RECOVERING,
            },
            AMove {
                at: 9,
                from: RECOVERING,
                to: CLAIMED,
            },
        ],
        PENDING,
        "observe-wait-the-order-holds-alone",
        "a claim and a recovery move it between the reads",
    );
}

/// What:
///  A second sample catches what a recovery takes back between two reads of the first one.
/// Why:
///  This is the second sample's own guard:
/// the record leaves `claimed` and comes back into `pending` before the one
///  read that could have seen it,
/// so a wait with one sample returns with the record live in `pending`,
/// and the second
///  sample reads it there.
#[test]
fn a_second_sample_catches_what_a_recovery_takes_back_between_two_reads() {
    wait_with_moves(
        &[
            AMove {
                at: 3,
                from: CLAIMED,
                to: RECOVERING,
            },
            AMove {
                at: 3,
                from: RECOVERING,
                to: PENDING,
            },
        ],
        CLAIMED,
        "observe-wait-a-second-sample-holds-alone",
        "a recovery takes it back between two reads",
    );
}

/// What:
///  A third sample catches a recovery whose two renames straddle two samples.
/// Why:
///  This is the third sample's own guard,
/// and it is the window the second integration pass made on the unplanted
///  code:
/// `recover` takes an expired claim from `claimed` into `recovering` before the first sample read `claimed`,
///  and moves it on into `pending` before the second sample read `recovering`,
/// so two samples return with the record
///  live in `pending` and the third reads it there.
#[test]
fn a_third_sample_catches_a_recovery_whose_two_renames_straddle_two_samples() {
    wait_with_moves(
        &[
            AMove {
                at: 3,
                from: CLAIMED,
                to: RECOVERING,
            },
            AMove {
                at: 5,
                from: RECOVERING,
                to: PENDING,
            },
        ],
        CLAIMED,
        "observe-wait-a-third-sample-holds-alone",
        "a recovery's two renames straddle two samples",
    );
}

/// What:
///  A claim published after the folder it lands in was read is missed by a wait of one sample in the old order.
/// Why:
///  This is the pair of halves' guard,
/// and the second move's read is what makes it one:
/// the record leaves `pending` before the first read and reaches `claimed` before the third,
/// so the order `pending`,
/// `claimed`,
/// `recovering` reads `pending` after the record left it and `claimed` before it
/// arrived,
/// and one sample returns with the record live in `claimed`.
/// Read in the order a record moves the third read is `claimed` and meets it,
/// and with that order reverted a second sample meets it in `recovering`,
/// so neither half alone is enough for this scene (`a_read_in_the_order...` and `a_second_sample...` hold each
/// half on a scene of its own).
/// With the second move at the second read instead,
/// the old order's second read is `claimed` and sees the record
/// at once,
/// and this control passes on every wait,
/// which the third integration pass measured.
#[test]
fn a_record_that_moves_between_two_reads_is_not_a_settled_queue() {
    wait_with_moves(
        &[
            AMove {
                at: 1,
                from: PENDING,
                to: RECOVERING,
            },
            AMove {
                at: 3,
                from: RECOVERING,
                to: CLAIMED,
            },
        ],
        PENDING,
        "observe-wait-a-record-between-reads",
        "it moves between the folders a claim passes through",
    );
}
