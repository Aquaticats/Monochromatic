//! What:
//!  The record of where one wrapped `git worktree add` spent its time,
//! step by step,
//! appended to a size-capped file in the
//!  common Git directory beside the commit timing record.
//! Why:
//!  An add that took thirteen seconds could not be split afterwards without a `strace` that distorted it.
//! Each step is a
//!  stretch of the command (a walk of the source,
//! a Git run,
//! the snapshot call,
//! a durable write),
//! timed on the monotonic
//!  clock;
//! a step inside another is taken out of the outer one,
//! so the steps are disjoint and what none covers is reported
//!  as `otherMs`.
//! The record goes to the file only,
//! in one append when the command ends,
//! with no flush to disk,
//! and it
//!  never changes what the add does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! begin(commonDir);
//! const entered = beginStep('walkSource');
//! const value = walk();
//! endStep(entered);
//! finish(exitCode); // appends JSON.stringify(record) + '\n' to join(commonDir, 'monochromatic-git-policy-cli-worktree-add-timing.jsonl')
//! ```

use crate::wrapper_names_old_and_new::WORKTREE_ADD_TIMING_LOG;
use crate::diagnostic_log::debug;
use crate::log_generations_kept_to_a_bound::UNPINNED_LOG;
use crate::log_lines_appended_without_a_lock::append_line;
use crate::iso_time::now_iso;
use crate::json_record::quote;
use crate::quiet_ledger::{drain, object_json};
use crate::service_protocol::RELEASE;
use std::cell::RefCell;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// The file inside the common Git directory that collects the add timing records of every worktree of the repository.
pub const ADD_TIMING_FILE: &str = WORKTREE_ADD_TIMING_LOG.current;

/// The version of the record's fields;
/// it changes only when a field's meaning changes,
/// never when a field is added.
pub const ADD_TIMING_SCHEMA_VERSION: u32 = 1;

/// Module tag for diagnostics.
const TAG: &str = "addTiming";

/// A step that has been entered and not yet left.
#[derive(Debug)]
struct Frame {
    /// The step's name.
    name: String,
    /// When the step was entered,
    /// since the recorder began.
    entered: Duration,
    /// The time of the steps entered and left inside this one.
    inside: Duration,
}

/// What one step name has spent so far.
#[derive(Debug)]
struct Spent {
    /// The step's name.
    name: String,
    /// The time of the step itself,
    /// the steps inside it taken out.
    own: Duration,
    /// How many times the step was entered.
    calls: usize,
}

/// What:
///  Add `own` to the time of the step named `name`,
/// as one more call,
/// or start the name's entry at the end of `list`.
/// Why:
///   Steps and worker-thread work both sum their time per name,
/// in the order the names first appeared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const known = list.find((each) => each.name === name);
/// if (known) { known.own += own; known.calls += 1; } else { list.push({ name, own, calls: 1 }); }
/// ```
fn add_spent(list: &mut Vec<Spent>, name: &str, own: Duration) {
    for known in list.iter_mut() {
        if known.name == name {
            known.own += own;
            known.calls += 1;
            return;
        }
    }
    list.push(Spent { name: name.to_string(), own, calls: 1 });
}

/// What:
///  A duration as milliseconds with three decimals,
/// the spelling of the record's `ms` fields.
/// Why:
///   Microsecond resolution matters for steps that take less than a millisecond.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (micros / 1000).toFixed(3)
/// ```
fn micros_to_ms(time: Duration) -> String {
    return format!("{:.3}", time.as_micros() as f64 / 1000.0);
}

/// What:
///  One step or work entry as its JSON member value `"name":{"ms":..,"calls":..}`.
/// Why:
///   The record lists steps and work in the same shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${quote(each.name)}:{"ms":${ms(each.own)},"calls":${each.calls}}`
/// ```
fn spent_json(each: &Spent) -> String {
    return format!("{}:{{\"ms\":{},\"calls\":{}}}", quote(each.name.as_str()), micros_to_ms(each.own), each.calls);
}

/// What:
///  One note as its JSON member `"key":"value"`.
/// Why:
///   Notes are words,
/// so both halves are quoted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${quote(key)}:${quote(value)}`
/// ```
fn note_json((key, value): &(String, String)) -> String {
    return format!("{}:{}", quote(key.as_str()), quote(value.as_str()));
}

/// What:
///  The steps of one add,
/// filled as the add runs and written when it ends.
/// Why:
///   The accounting is separate from the clock so that the controls can drive it with chosen times.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Recorder = { commonDir: string; open: Frame[]; spent: Spent[]; notes: [string, string][] };
/// ```
#[derive(Debug)]
pub struct Recorder {
    /// The common Git directory that holds the file.
    common_dir: PathBuf,
    /// The steps entered and not left,
    /// the innermost last.
    open: Vec<Frame>,
    /// The time per step name,
    /// in the order the names first appeared.
    spent: Vec<Spent>,
    /// Facts about the add that are words,
    /// not times (the route taken,
    /// why not the snapshot),
    /// in the order they were noted.
    notes: Vec<(String, String)>,
    /// Milliseconds since the Unix epoch at the start,
    /// for finding the add among other logs.
    start_millis: u128,
    /// Time spent on worker threads by name,
    /// summed over the threads;
    /// threads overlap,
    /// so it is not part of the sum rule.
    work: Vec<Spent>,
    /// The longest quiet gap of every Git step and tool the add ran,
    /// by name,
    /// taken from the ledger when the record is written.
    quiet: Vec<(String, Duration)>,
}

impl Recorder {
    /// A recorder for an add in the repository whose common Git directory is `common_dir`.
    pub fn new(common_dir: PathBuf) -> Self {
        // A clock before the Unix epoch gives zero, as `unwrap_or_default` did.
        let start_millis: u128 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(elapsed) => elapsed.as_millis(),
            Err(_before_epoch) => 0,
        };
        return Self { common_dir, open: Vec::new(), spent: Vec::new(), notes: Vec::new(), start_millis, work: Vec::new(), quiet: Vec::new() };
    }

    /// Enter the step `name` at `at`,
    /// a time since the recorder began.
    pub fn enter(&mut self, name: &str, at: Duration) {
        self.open.push(Frame { name: name.to_string(), entered: at, inside: Duration::ZERO });
    }

    /// Leave the innermost step at `at`;
    /// its own time is what the steps inside it left,
    /// and the outer step loses the whole.
    pub fn leave(&mut self, at: Duration) {
        let Some(frame) = self.open.pop() else { return };
        let whole: Duration = at.saturating_sub(frame.entered);
        let own: Duration = whole.saturating_sub(frame.inside);
        if let Some(outer) = self.open.last_mut() {
            outer.inside += whole;
        }
        add_spent(&mut self.spent, frame.name.as_str(), own);
    }

    /// Note a fact about the add;
    /// a later note of the same key replaces the earlier one.
    pub fn note(&mut self, key: &str, value: &str) {
        // Replace the value of a key that is already noted; the loop stops at the first match, as `find` did.
        for known in &mut self.notes {
            if known.0 == key {
                known.1 = value.to_string();
                return;
            }
        }
        self.notes.push((key.to_string(), value.to_string()));
    }

    /// Add `own` to the work named `name`,
    /// as one call.
    pub fn add_work(&mut self, name: &str, own: Duration) {
        add_spent(&mut self.work, name, own);
    }

    /// The record as one line of JSON ending in a newline;
    /// `total` is the add's whole time.
    ///
    /// One `write` per line keeps the lines of concurrent adds from interleaving;
    /// a word with a quote or a newline is escaped
    /// by `quote`,
    /// so the line stays one line.
    pub fn line(&self, total: Duration, exit_code: i32) -> String {
        let mut covered: Duration = Duration::ZERO;
        for each in &self.spent {
            covered += each.own;
        }
        let steps: Vec<String> = self.spent.iter().map(spent_json).collect();
        let work: Vec<String> = self.work.iter().map(spent_json).collect();
        let notes: Vec<String> = self.notes.iter().map(note_json).collect();
        return format!(
            "{{\"schemaVersion\":{ADD_TIMING_SCHEMA_VERSION},\"time\":{},\"startMillis\":{},\"pid\":{},\"release\":{},\"exitCode\":{exit_code},\"totalMs\":{},\"steps\":{{{}}},\"otherMs\":{},\"work\":{{{}}},\"notes\":{{{}}},\"quiet\":{}}}\n",
            quote(now_iso().as_str()),
            self.start_millis,
            std::process::id(),
            quote(RELEASE),
            micros_to_ms(total),
            steps.join(","),
            micros_to_ms(total.saturating_sub(covered)),
            work.join(","),
            notes.join(","),
            object_json(self.quiet.as_slice()),
        );
    }

    /// The directory that holds the record's file.
    pub fn common_dir(&self) -> &Path {
        return self.common_dir.as_path();
    }
}

/// The work of worker threads while an add is recorded:
/// none outside a recording,
/// so a stray call keeps nothing.
static WORK: Mutex<Option<Vec<(String, Duration)>>> = Mutex::new(None);

thread_local! {
    /// The recorder of the add this thread is running,
    /// and when it began;
    /// none outside an add.
    static ACTIVE: RefCell<Option<(Instant, Recorder)>> = const { RefCell::new(None) };
}

/// Begin recording the add of the repository whose common Git directory is `common_dir`,
/// replacing any earlier recording
///  of this thread.
pub fn begin(common_dir: PathBuf) {
    ACTIVE.set(Some((Instant::now(), Recorder::new(common_dir))));
    if let Ok(mut work) = WORK.lock() {
        *work = Some(Vec::new());
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
    let entered: bool = match recording.as_mut() {
        Some((began, recorder)) => {
            recorder.enter(name, began.elapsed());
            true
        }
        None => false,
    };
    ACTIVE.set(recording);
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
        if let Some((began, recorder)) = recording.as_mut() {
            recorder.leave(began.elapsed());
        }
        ACTIVE.set(recording);
    }
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
fn option_takes_value(word: &OsStr) -> bool {
    for known in ["-C", "-c", "--git-dir", "--work-tree", "--namespace", "--exec-path"] {
        if word == OsStr::new(known) {
            return true;
        }
    }
    return false;
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

/// What:
///  The clock reading that starts a stretch of work on any thread,
/// added to the name's sum when `end_work` is called.
/// Why:
///   The repair of package managers runs its tools on worker threads,
/// which the thread's own steps cannot see;
/// the sum over
///  threads says how much work a step held,
/// and the step that waited for it says how long the add waited.
/// The body runs in
///  place between `begin_work` and `end_work`,
/// so it reads the caller's variables without a closure to carry them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const began = beginWork();
/// const value = scan();
/// endWork('untrackedScan', began);
/// ```
pub fn begin_work() -> Instant {
    return Instant::now();
}

/// What:
///  Add the time since `began` to the work named `name`.
/// Why:
///   The second half of a stretch of work:
/// the sum per name says how much work a step held across threads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// endWork('untrackedScan', began);
/// ```
pub fn end_work(name: &str, began: Instant) {
    let spent: Duration = began.elapsed();
    if let Ok(mut work) = WORK.lock()
        && let Some(list) = work.as_mut()
    {
        list.push((name.to_string(), spent));
    }
}

/// Note a fact about the add being recorded;
/// nothing outside a recording.
pub fn note(key: &str, value: &str) {
    let mut recording: Option<(Instant, Recorder)> = ACTIVE.take();
    if let Some((_began, recorder)) = recording.as_mut() {
        recorder.note(key, value);
    }
    ACTIVE.set(recording);
}

/// What:
///  End the recording and append its record to the repository's add timing file,
/// best effort.
/// Why:
///   The command's result never depends on the record:
/// a file that cannot be written is only a debug line.
/// The directory is
///  never created and nothing is flushed to disk,
/// so the record costs one `write`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// finish(exitCode);
/// ```
pub fn finish(exit_code: i32) {
    let taken: Option<(Instant, Recorder)> = ACTIVE.take();
    let drained: Option<Vec<(String, Duration)>> = match WORK.lock() {
        Ok(mut work) => work.take(),
        Err(_poisoned) => None,
    };
    let Some((began, mut recorder)) = taken else { return };
    for (name, spent) in drained.unwrap_or_default() {
        recorder.add_work(name.as_str(), spent);
    }
    recorder.quiet = drain();
    let target: PathBuf = recorder.common_dir().join(ADD_TIMING_FILE);
    let line: String = recorder.line(began.elapsed(), exit_code);
    if let Err(error) = append_line(target.as_path(), line.as_bytes(), &UNPINNED_LOG) {
        debug(TAG, &format!("the add timing record could not be written to {}: {error}", target.display()));
    }
}

/// The accounting,
/// the line and the writer.
#[cfg(test)]
#[path = "add_timing_tests.rs"]
mod tests;
