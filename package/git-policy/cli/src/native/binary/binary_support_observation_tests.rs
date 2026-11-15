//! What:
//!  The controls of the wait in `binary_support_observation_bound.rs`
//!  and of its faces in `binary_support_observation.rs`:
//!  the program a bounded line names,
//!  the bound counted from the command's own start,
//!  a command that ends by itself with its own status or signal,
//!  one this wait ends at its bound,
//!  a descendant in a session of its own that holds one of the command's pipes,
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
/// The wait itself and what it answers,
/// for the controls that give a wait a child of their own.
use crate::support_observation_bound::{Ending, wait_under_bound};
/// The disposable fixture directory,
/// the one writer of executable fixture files,
/// and its removal.
use crate::support::{Fixture, executable, fixture, remove};
/// `process_group` puts a command of a control in a group of its own.
use std::os::unix::process::CommandExt as _;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// The command builder and the child's pipes.
use std::process::{Child, Command, Stdio};
/// The clock a control measures its wait with,
/// and the sleeps of its own waits.
use std::time::{Duration, Instant};

/// The program whose form a bound takes in this harness (`binary_support.rs`, `bounded_for`).
const TIMEOUT: &str = "/usr/bin/timeout";

/// The shell every scripted command of these controls is.
const SHELL: &str = "/bin/sh";

/// The bound these controls give a command this wait must end,
/// in seconds,
/// as `timeout` takes it.
const ONE_SECOND: &str = "1";

/// The bound these controls give a command that ends by itself,
/// in seconds:
///  ample for a shell that exits at once,
///  and the bound the command's own line is read for.
const AMPLE_SECONDS: &str = "30";

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

/// How long a wait may take when the bound it is given has already passed,
/// in seconds.
/// Why a fifth of a second:
///  the wait has nothing left to wait for,
///  so this covers a signal and a reap and no more;
///  a wait that counts its bound from the moment it is called takes the whole bound instead.
const ALREADY_SPENT_PATIENCE: Duration = Duration::from_millis(200);

/// What:
///  A command in the harness's own bounded form:
///  `timeout --signal=KILL <seconds> <script>`.
/// Why:
///  Every command of the binary controls carries its bound in its own line,
///  and the bound this wait keeps is the one that line states.
fn bounded_script(script: &Path, seconds: &str) -> Command {
    let mut command: Command = Command::new(TIMEOUT);
    command.arg("--signal=KILL").arg(seconds).arg(script);
    return command;
}

/// One executable script in the fixture's directory.
fn script(made: &Fixture, name: &str, body: &str) -> PathBuf {
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
fn descendant(made: &Fixture, release_and_gone: (&Path, &Path)) -> String {
    let (release, gone) = release_and_gone;
    return format!(
        "/usr/bin/setsid {SHELL} -c 'read -r line < \"{}\"; printf gone > \"{}\"' &\n",
        release.display(),
        gone.display()
    );
}

/// A named pipe the fixture can release a descendant through.
fn named_pipe(path: &Path) {
    let named: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("a fixture path holds no NUL");
    // SAFETY: `mkfifo` reads the NUL-terminated path it is given and touches no memory of this process.
    let made: libc::c_int = unsafe { libc::mkfifo(named.as_ptr(), 0o600) };
    assert_eq!(made, 0, "the fixture makes the pipe {}: {}", path.display(), std::io::Error::last_os_error());
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
        assert!(began.elapsed() < PATIENCE, "the descendant released through {} did not say it was gone within {PATIENCE:?}", release.display());
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
    let group: libc::pid_t = libc::pid_t::try_from(child.id()).expect("a process ID fits the pid type");
    return (child, group);
}

/// What:
///  The wait starts the program a bounded line names itself,
///  and nothing else keeps a timer on the command.
/// Why:
///  Two timers with one deadline cannot tell which of them ended a command,
///  so the wait starts the program the line names in the group it makes;
///  then the command's own parent is this test process,
///  and no `timeout` stands between them to fire first.
#[test]
fn the_wait_starts_the_program_the_line_names_with_no_timer_in_front_of_it() {
    let made: Fixture = fixture("observe-no-timer");
    let parent: PathBuf = made.root.join("parent");
    let body: String = format!("#!/bin/sh\nprintf '%s' \"$PPID\" > \"{}\"\nexit 0\n", parent.display());
    let command: PathBuf = script(&made, "says-its-parent", body.as_str());
    let mut wrapped: Command = bounded_script(command.as_path(), AMPLE_SECONDS);
    wrapped.current_dir(made.root.as_path());
    let seen = observe(&mut wrapped, b"");
    assert_eq!(seen.code, Some(0), "the command ends by itself: {}", String::from_utf8_lossy(seen.stderr.as_slice()));
    let written: String = std::fs::read_to_string(parent.as_path()).expect("the script wrote its parent");
    assert_eq!(
        written.trim().parse::<u32>().expect("a process number"),
        std::process::id(),
        "the command the wait started is this test process's own child, and no `timeout` stands between them"
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
    let command: PathBuf = script(&made, "runs-past-its-bound", format!("#!/bin/sh\nexec /usr/bin/sleep {HOLDS_SECONDS}\n").as_str());
    let mut started: Command = bounded_script(command.as_path(), ONE_SECOND);
    started.current_dir(made.root.as_path());
    let (child, group): (Child, libc::pid_t) = spawned_beside_the_control(&mut started);
    // The command runs for longer than its own bound before the wait is called at all.
    std::thread::sleep(Duration::from_millis(1_200));
    let began: Instant = Instant::now();
    let answer = wait_under_bound(child, Some(group), Duration::from_secs(1), b"");
    let elapsed: Duration = began.elapsed();
    println!("the wait answered {:?} after {elapsed:?}", answer.ending);
    assert_eq!(answer.ending, Ending::AtTheBound, "a command past its bound is ended by this wait, not by itself");
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
    let command: PathBuf = script(&made, "kept-alive", format!("#!/bin/sh\nexec /usr/bin/sleep {HOLDS_SECONDS}\n").as_str());
    let mut wrapped: Command = bounded_script(command.as_path(), ONE_SECOND);
    wrapped.current_dir(made.root.as_path());
    let (child, group): (Child, libc::pid_t) = spawned_beside_the_control(&mut wrapped);
    // The permitted interleaving the review reproduced: this process does other work before it waits.
    std::thread::sleep(Duration::from_millis(100));
    let answer = wait_under_bound(child, Some(group), Duration::from_secs(1), b"");
    println!("the wait answered {:?} with status {:?}", answer.ending, answer.status);
    assert_ne!(
        answer.ending,
        Ending::Itself,
        "the child's own `timeout` fired inside the bound this wait states, so this wait must not call that ending the child's own"
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
    let command: PathBuf = script(&made, "waits-for-ever", format!("#!/bin/sh\nexec /usr/bin/sleep {HOLDS_SECONDS}\n").as_str());
    let mut started: Command = bounded_script(command.as_path(), ONE_SECOND);
    started.current_dir(made.root.as_path());
    let (child, group): (Child, libc::pid_t) = spawned_beside_the_control(&mut started);
    // A second child of this control's own, in a group of its own, that no signal may reach.
    let mut beside: Command = Command::new("/usr/bin/sleep");
    beside.arg(HOLDS_SECONDS.to_string());
    beside.current_dir(made.root.as_path());
    let mut untouched: Child = beside.process_group(0).spawn().expect("the second child starts");
    let answer = wait_under_bound(child, Some(group), Duration::from_secs(1), b"");
    let stayed: bool = untouched.try_wait().expect("the second child's state").is_none();
    let _ended: std::io::Result<()> = untouched.kill();
    let _waited: std::io::Result<std::process::ExitStatus> = untouched.wait();
    assert_eq!(answer.ending, Ending::AtTheBound, "the wait ended the command at its bound");
    assert_eq!(
        std::os::unix::process::ExitStatusExt::signal(&answer.status),
        Some(libc::SIGKILL),
        "a command whose status is a `SIGKILL` death was alive when that signal went out: {:?}",
        answer.status
    );
    assert!(stayed, "the signal reached only the command, and the second child of this control still ran");
    remove(&made);
}

/// What:
///  No signal in the two files of the wait takes a number that may already belong to another process.
/// Why:
///  A control cannot wait for a process number to be reused,
///  so this is a control over the source:
///  the command is signalled through a pinned process,
///  the group is signalled only after a `WNOWAIT` look and before the command is reaped,
///  and no other signal call exists in either file.
#[test]
fn no_signal_in_the_wait_takes_a_number_its_child_no_longer_holds() {
    let files: [(&str, &str); 2] = [
        ("binary_support_observation.rs", include_str!("binary_support_observation.rs")),
        ("binary_support_observation_bound.rs", include_str!("binary_support_observation_bound.rs")),
    ];
    for (name, text) in files {
        assert!(!text.contains("killpg("), "{name}: a whole group is signalled through its leader with `kill(-leader)`, never through `killpg`");
        for (index, _) in text.match_indices("libc::kill(") {
            let after: &str = &text[index..];
            assert!(
                after.starts_with("libc::kill(-leader, libc::SIGKILL)"),
                "{name}: a signal may name only a group its leader still pins, and this call is {}",
                after.lines().next().unwrap_or_default()
            );
        }
    }
    let bound: &str = include_str!("binary_support_observation_bound.rs");
    let checked: usize = bound.find("WNOWAIT | libc::WNOHANG").expect("the wait looks before it signals, without reaping");
    let signalled: usize = bound.find("libc::kill(-leader, libc::SIGKILL)").expect("the group is signalled at the bound");
    assert!(
        checked < signalled,
        "the wait looks at the child without reaping it before it signals, so the number it names is still the child's"
    );
    assert!(bound.contains("pin.signal(libc::SIGKILL)"), "the command itself is signalled through its pin, never by a number");
}

/// What:
///  A command that ends by itself keeps its own ending,
///  whatever that ending was.
/// Why:
///  This wait reads nothing back from a status to learn who ended a command:
///  an `exit 124`,
///  an `exit 137`,
///  a death by `SIGPIPE` and a command's own `SIGKILL` are all its own,
///  and only a command this wait ended may be reported as the bound's doing.
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
        let command: PathBuf = script(&made, name.as_str(), format!("#!/bin/sh\n{body}\n").as_str());
        let mut wrapped: Command = bounded_script(command.as_path(), AMPLE_SECONDS);
        wrapped.current_dir(made.root.as_path());
        let seen = observe(&mut wrapped, b"");
        println!("{body}: the wait answered code {:?}", seen.code);
        assert_eq!(seen.code, *code, "{body}: the command ended by itself, so its own status is the answer");
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
    let command: PathBuf = script(&made, "waits-for-ever", format!("#!/bin/sh\nexec /usr/bin/sleep {HOLDS_SECONDS}\n").as_str());
    let began: Instant = Instant::now();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut wrapped: Command = bounded_script(command.as_path(), ONE_SECOND);
        wrapped.current_dir(made.root.as_path());
        return observe(&mut wrapped, b"");
    }));
    let elapsed: Duration = began.elapsed();
    let payload = outcome.expect_err("a command this wait ended is reported, never answered as one that ended by itself");
    let message: String = message_of(payload);
    println!("the wait returned after {elapsed:?}, and said: {message}");
    assert!(message.contains("ended the command at its 1s"), "the sentence says what this wait did: {message}");
    assert!(elapsed < PATIENCE, "the wait kept its one-second bound, and returned after {elapsed:?}");
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
        descendant(&made, (release.as_path(), gone.as_path()))
    );
    let command: PathBuf = script(&made, "starts-a-descendant", body.as_str());
    let began: Instant = Instant::now();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut wrapped: Command = bounded_script(command.as_path(), ONE_SECOND);
        wrapped.current_dir(made.root.as_path());
        return observe(&mut wrapped, b"");
    }));
    let elapsed: Duration = began.elapsed();
    // The descendant is this control's own to release, whatever the wait answered.
    released((release.as_path(), gone.as_path()));
    let payload = outcome.expect_err("a command that never ends is ended by this wait");
    let message: String = message_of(payload);
    println!("the wait returned after {elapsed:?} against its one-second bound, and said: {message}");
    assert!(message.contains("ended the command at its 1s"), "the sentence says what this wait did: {message}");
    assert!(
        message.contains("still holds its standard output"),
        "the sentence names the pipe a descendant still holds: {message}"
    );
    assert!(elapsed < PATIENCE, "the wait kept its bound while a descendant held the pipe, and returned after {elapsed:?}");
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
    let body: String = format!("#!/bin/sh\n{}exit 0\n", descendant(&made, (release.as_path(), gone.as_path())));
    let command: PathBuf = script(&made, "leaves-a-descendant", body.as_str());
    let began: Instant = Instant::now();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut wrapped: Command = bounded_script(command.as_path(), AMPLE_SECONDS);
        wrapped.current_dir(made.root.as_path());
        return observe(&mut wrapped, b"");
    }));
    let elapsed: Duration = began.elapsed();
    // The descendant is this control's own to release, whatever the wait answered.
    released((release.as_path(), gone.as_path()));
    let payload = outcome.expect_err("an output a descendant may still be writing is never answered as the whole one");
    let message: String = message_of(payload);
    println!("the wait returned after {elapsed:?} against its bound of {AMPLE_SECONDS} s, and said: {message}");
    assert!(message.contains("ended by itself"), "the sentence says the command ended by itself: {message}");
    assert!(
        message.contains("still holds its standard output"),
        "the sentence names the pipe a descendant still holds: {message}"
    );
    assert!(elapsed < PATIENCE, "the wait gave its one grace and no more, and returned after {elapsed:?}");
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
    let body: String = format!("#!/bin/sh\nexec /usr/bin/dd if=/dev/zero bs=4096 count={blocks} 2>/dev/null\n");
    let command: PathBuf = script(&made, "writes-much", body.as_str());
    let mut wrapped: Command = bounded_script(command.as_path(), AMPLE_SECONDS);
    wrapped.current_dir(made.root.as_path());
    let seen = observe(&mut wrapped, b"");
    assert_eq!(seen.code, Some(0), "the command that writes much ends by itself");
    assert_eq!(seen.stdout.len(), written, "every byte the command wrote is in the answer");
    remove(&made);
}
