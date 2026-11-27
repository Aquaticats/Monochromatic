//! What:
//!  The recording one thread runs for its add:
//!  the recorder it holds,
//!  the notice that runs beside it,
//!  and the steps it publishes as it enters and leaves them.
//! Why:
//!  The steps are recorded on the thread that runs the add,
//!  and the notice is printed on a thread of its own,
//!  so a step entered and left is published here for that notice to name.
//!  A step entered with no recording begun is left alone,
//!  so a caller that never began one pays nothing,
//!  and the work of worker threads is summed by name beside it.
//!  This file is one part of the module whose name it carries,
//!  split because that module was over the line limit:
//!  `add_timing_record.rs` holds the accounting and the JSON line,
//!  and `add_timing.rs` holds the clock,
//! the work of other threads and the file the line goes to.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! beginOnRoute(commonDir, quiet, plain);
//! const entered = beginStep('walkSource');
//! endStep(entered);
//! ```

use crate::add_timing_record::Recorder;
use std::cell::RefCell;
use std::ffi::{
    OsStr,
    OsString,
};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{
    Duration,
    Instant,
};

/// Module tag for diagnostics.
pub(crate) const TAG: &str = "addTiming";

/// The work of worker threads while an add is recorded:
/// none outside a recording,
/// so a stray call keeps nothing.
pub(crate) static WORK: Mutex<Option<Vec<(String, Duration)>>> = Mutex::new(None);

thread_local! {
    /// The recorder of the add this thread is running,
    /// and when it began;
    /// none outside an add.
    pub(crate) static ACTIVE: RefCell<Option<(Instant, Recorder)>> = const { RefCell::new(None) };
    /// The notice this thread's add prints while it runs,
    /// which the recording's own end drops;
    /// none outside an add.
    pub(crate) static NOTICE: RefCell<Option<crate::add_standing_notice::StandingNotice>> = const { RefCell::new(None) };
}

/// Begin recording the add of the repository whose common Git directory is `common_dir`,
/// replacing any earlier recording
///  of this thread.
pub fn begin(
    common_dir: PathBuf,
    quiet: bool,
) {
    begin_on_route(common_dir, quiet, None);
}

/// What:
///  Begin recording an add,
///  saying why it runs as a plain checkout when the wrapper does not serve it.
/// Why:
///   An add the ladder does not serve still takes time a person waits through,
/// and it is the same add to
///  the recording:
/// its whole wait is one step nothing of the ladder surrounds,
/// the note says which
///  fact put it on the plain route,
/// and the notice says the same while it runs,
/// so a plain add is
///  never silent and never leaves the record of an add that says nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// beginOnRoute(commonDir, quiet, plain); // plain: why the ladder refused, or undefined
/// ```
pub fn begin_on_route(
    common_dir: PathBuf,
    quiet: bool,
    plain: Option<String>,
) {
    let began: Instant = Instant::now();
    ACTIVE.set(Some((began, Recorder::new(common_dir))));
    // An add that is still running after a few seconds says which step it is in,
    // on a thread of its own,
    // so the wait is never
    //  silent and the add never waits for the line (owner:
    // "nothing should ever wait forever or indefinitely").
    // The steps published below are what it
    //  names, so the recording is what starts and ends it.
    NOTICE.set(Some(crate::add_standing_notice::begin_for(quiet)));
    if let Ok(mut work) = WORK.lock() {
        *work = Some(Vec::new());
    }
    if let Some(clause) = plain {
        // The route is noted so the record says why the ladder stepped aside, and the step is published
        //  once, because a plain add has one stretch of work from here to its answer.
        note("route", "plain");
        note("whyPlain", clause.as_str());
        crate::add_standing_notice::the_plain_add_began(clause.as_str(), began);
    }
}

/// What:
///  Enter the step `name` and say whether a recording took it;
/// the first half of a timed step.
/// Why:
///   The body runs in place between `begin_step` and `end_step`,
/// so it reads the caller's variables without a closure to
///  carry them.
/// The answer goes to `end_step` unchanged;
/// with no recording begun,
/// nothing is entered and `end_step` does nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entered = beginStep('walkSource');
/// const value = walk();
/// endStep(entered);
/// ```
pub fn begin_step(name: &str) -> bool {
    // The recording is taken out of its cell while it is updated and put back at once, so the body never runs with the cell borrowed.
    let mut recording: Option<(Instant, Recorder)> = ACTIVE.take();
    let mut standing: Option<(Instant, Option<String>)> = None;
    let entered: bool = match recording.as_mut() {
        Some((began, recorder)) => {
            recorder.enter(name, began.elapsed());
            standing = Some((*began, innermost_step(recorder)));
            true
        }
        None => false,
    };
    ACTIVE.set(recording);
    // The step is published for the notice a long wait prints, so the add's own thread never holds a stream for it.
    if let Some((began, innermost)) = standing {
        crate::add_standing_notice::a_step_began(innermost.as_deref(), began);
    }
    return entered;
}

/// What:
///  Leave the step that `begin_step` entered,
/// when it entered one.
/// Why:
///   The second half of a timed step:
/// it ends the step's clock at the moment the body is done,
/// whether the body succeeded or not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// endStep(entered);
/// ```
pub fn end_step(entered: bool) {
    if entered {
        let mut recording: Option<(Instant, Recorder)> = ACTIVE.take();
        let mut standing: Option<(Instant, Option<String>)> = None;
        if let Some((began, recorder)) = recording.as_mut() {
            recorder.leave(began.elapsed());
            standing = Some((*began, innermost_step(recorder)));
        }
        ACTIVE.set(recording);
        if let Some((began, innermost)) = standing {
            crate::add_standing_notice::a_step_began(innermost.as_deref(), began);
        }
    }
}

/// What:
///  The innermost step the recorder has entered and not left,
/// or nothing between two steps.
/// Why:
///   The step a long wait is in is read by the notice on another thread,
/// so the recorded stack is read through one function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const innermostStep = (recorder) => recorder.open.at(-1)?.name;
/// ```
fn innermost_step(recorder: &Recorder) -> Option<String> {
    return recorder.innermost();
}

/// What:
///  Whether a global Git option is followed by its value as the next word.
/// Why:
///   The subcommand is the first word that is neither an option nor the value of one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['-C', '-c', '--git-dir', '--work-tree', '--namespace', '--exec-path'].includes(word)
/// ```
pub(crate) fn option_takes_value(word: &OsStr) -> bool {
    for known in [
        "-C",
        "-c",
        "--git-dir",
        "--work-tree",
        "--namespace",
        "--exec-path",
    ] {
        if word == OsStr::new(known) {
            return true;
        }
    }
    return false;
}

/// Note a fact about the add being recorded;
/// nothing outside a recording.
pub fn note(
    key: &str,
    value: &str,
) {
    let mut recording: Option<(Instant, Recorder)> = ACTIVE.take();
    if let Some((_began, recorder)) = recording.as_mut() {
        recorder.note(key, value);
    }
    ACTIVE.set(recording);
}

/// What:
///  The step name of one Git run:
/// `git ` and the subcommand,
/// found past the global options that take a value (`-C`,
/// `-c`,
///  `--git-dir`,
/// `--work-tree`) and the flags that do not.
/// Why:
///   The record says how many Git processes an add started and how long each kind took,
/// which names the avoidable ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git_step_label(['-C', dir, '--no-optional-locks', 'status']) // 'git status'
/// ```
pub fn git_step_label(arguments: &[OsString]) -> String {
    let mut words = arguments.iter().map(OsString::as_os_str);
    while let Some(word) = words.next() {
        if option_takes_value(word) {
            words.next();
            continue;
        }
        if word.to_string_lossy().starts_with('-') {
            continue;
        }
        return format!("git {}", word.to_string_lossy());
    }
    return String::from("git");
}
