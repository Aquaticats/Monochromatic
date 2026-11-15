//! What:
//!  The controls of the shared wait in `binary_support_observation.rs`:
//!  a command that ends by itself with its own status or signal,
//!  one that never ends and is ended at the bound its own line states,
//!  a descendant in a session of its own that holds one of the command's pipes,
//!  and a command that writes more than a pipe holds.
//! Why:
//!  The bound is this wait's own promise and the ending is its own report:
//!  a command that ended by itself must come back with its own status or signal,
//!  a command this wait ended must say so,
//!  and a pipe another process holds must not hold the call past the bound.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const seen = observe(boundedScript('kill -KILL $$', 30)); // answered, never reported as the bound's doing
//! ```

/// The shared wait these controls are of.
use super::observe;
/// The disposable fixture directory, the one writer of executable fixture files, and its removal.
use crate::support::{Fixture, executable, fixture, remove};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// The command builder every control runs a script under.
use std::process::Command;
/// The clock a control measures its wait with, and the sleeps of its own waits.
use std::time::{Duration, Instant};

/// The program whose form a bound takes in this harness (`binary_support.rs`, `bounded_for`).
const TIMEOUT: &str = "/usr/bin/timeout";

/// The shell every scripted command of these controls is.
const SHELL: &str = "/bin/sh";

/// The bound these controls give a command this wait must end, in seconds, as `timeout` takes it.
const ONE_SECOND: &str = "1";

/// The bound these controls give a command that ends by itself, in seconds:
///  ample for a shell that exits at once, and the bound the command's own line is read for.
const AMPLE_SECONDS: &str = "30";

/// How long a descendant of a control holds the pipe it inherited, in seconds.
/// Why longer than every bound here and short of the runner's five-minute stop:
///  a wait that reads a pipe to its end waits for this descendant, so the control shows that wait,
///  and a wait that keeps its bound answers long before it.
const HOLDS_SECONDS: u64 = 30;

/// How long a control lets its wait take before it says the wait lost its bound.
/// Why five seconds:
///  the one-second bound these controls state, the grace the wait gives a held pipe, and room for the scheduler
///  of a loaded host;
///  the descendant these controls start holds its pipe for `HOLDS_SECONDS`, so a wait that waits for it shows itself
///  far past this window.
const PATIENCE: Duration = Duration::from_secs(5);

/// What:
///  A command in the harness's own bounded form: `timeout --signal=KILL <seconds> <script>`.
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

/// The process number a script wrote for the descendant it started,
/// once that number is there.
fn process_number(recorded: &Path) -> libc::pid_t {
    let began: Instant = Instant::now();
    loop {
        if let Ok(text) = std::fs::read_to_string(recorded)
            && let Ok(number) = text.trim().parse::<libc::pid_t>()
        {
            return number;
        }
        assert!(began.elapsed() < PATIENCE, "the script wrote its descendant's process number within {PATIENCE:?}");
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// Ends a descendant a control started,
/// so nothing of the control outlives it.
fn end(number: libc::pid_t) {
    // SAFETY: `kill` reads the two numbers it is given and touches no memory of this process.
    let _signalled: libc::c_int = unsafe { libc::kill(number, libc::SIGKILL) };
}

/// What:
///  A command that ends by itself keeps its own ending,
///  whatever that ending was.
/// Why:
///  This wait reads nothing back from a status to learn who ended a command:
///  an `exit 124`, an `exit 137`, a death by `SIGPIPE` and a command's own `SIGKILL` are all its own,
///  and only a command this wait ended may be reported as the bound's doing.
///  A command that kills itself with `SIGKILL` was reported as having hit the wall-clock bound before this
///  control existed, because the signal `timeout` re-raises is the same number.
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
        let seen = observe(&mut bounded_script(command.as_path(), AMPLE_SECONDS), b"");
        println!("{body}: the wait answered code {:?}", seen.code);
        assert_eq!(seen.code, *code, "{body}: the command ended by itself, so its own status is the answer");
    }
    remove(&made);
}

/// What:
///  A command that never ends is ended by this wait at the bound its own line states,
///  and the answer says this wait ended it.
/// Why:
///  Nothing waits without a bound, a test included:
///  a command that waits for ever reports nothing and a suite that waits in one control reports nothing either.
///  The sentence must name the bound this wait kept, because no status the command came back with can say it.
#[test]
fn a_command_that_never_ends_is_ended_at_its_bound() {
    let made: Fixture = fixture("observe-bound");
    let command: PathBuf = script(&made, "waits-for-ever", format!("#!/bin/sh\nexec /usr/bin/sleep {HOLDS_SECONDS}\n").as_str());
    let began: Instant = Instant::now();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        return observe(&mut bounded_script(command.as_path(), ONE_SECOND), b"");
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
///  with a child whose descendant held its output pipe, the wait returned after 8,005 ms against a bound of one
///  second, and a descendant that never ends holds such a wait for ever.
///  The descendant is ended by this control before it returns.
#[test]
fn a_command_whose_descendant_holds_its_output_is_ended_at_its_bound() {
    let made: Fixture = fixture("observe-descendant");
    let recorded: PathBuf = made.root.join("descendant");
    let body: String = format!(
        "#!/bin/sh\n/usr/bin/setsid {SHELL} -c 'echo $$ > \"{}\"; exec /usr/bin/sleep {HOLDS_SECONDS}' &\nexec /usr/bin/sleep {HOLDS_SECONDS}\n",
        recorded.display()
    );
    let command: PathBuf = script(&made, "starts-a-descendant", body.as_str());
    let began: Instant = Instant::now();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        return observe(&mut bounded_script(command.as_path(), ONE_SECOND), b"");
    }));
    let elapsed: Duration = began.elapsed();
    // The descendant is this control's own to end, whatever the wait answered.
    end(process_number(recorded.as_path()));
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
///  The descendant is ended by this control before it returns.
#[test]
fn a_command_that_ends_by_itself_says_that_a_descendant_still_holds_its_output() {
    let made: Fixture = fixture("observe-descendant-holds");
    let recorded: PathBuf = made.root.join("descendant");
    let body: String = format!(
        "#!/bin/sh\n/usr/bin/setsid {SHELL} -c 'echo $$ > \"{}\"; exec /usr/bin/sleep {HOLDS_SECONDS}' &\nexit 0\n",
        recorded.display()
    );
    let command: PathBuf = script(&made, "leaves-a-descendant", body.as_str());
    let began: Instant = Instant::now();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        return observe(&mut bounded_script(command.as_path(), AMPLE_SECONDS), b"");
    }));
    let elapsed: Duration = began.elapsed();
    // The descendant is this control's own to end, whatever the wait answered.
    end(process_number(recorded.as_path()));
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
    let seen = observe(&mut bounded_script(command.as_path(), AMPLE_SECONDS), b"");
    assert_eq!(seen.code, Some(0), "the command that writes much ends by itself");
    assert_eq!(seen.stdout.len(), written, "every byte the command wrote is in the answer");
    remove(&made);
}
