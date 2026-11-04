//! What:
//!  The record of where one wrapped `git worktree add` spent its time, step by step, appended to a size-capped file in the
//!  common Git directory beside the commit timing record.
//! Why:
//!  An add that took thirteen seconds could not be split afterwards without a `strace` that distorted it. Each step is a
//!  stretch of the command (a walk of the source, a Git run, the snapshot call, a durable write), timed on the monotonic
//!  clock; a step inside another is taken out of the outer one, so the steps are disjoint and what none covers is reported
//!  as `otherMs`. The record goes to the file only, in one append when the command ends, with no flush to disk, and it
//!  never changes what the add does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! begin(commonDir);
//! const value = step('walkSource', () => walk());
//! finish(exitCode); // appends JSON.stringify(record) + '\n' to join(commonDir, 'cli-git-worktree-add-timing.jsonl')
//! ```

use crate::diagnostic_log::debug;
use crate::discard_audit::append_line;
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
pub const ADD_TIMING_FILE: &str = "cli-git-worktree-add-timing.jsonl";

/// The version of the record's fields; it changes only when a field's meaning changes, never when a field is added.
pub const ADD_TIMING_SCHEMA_VERSION: u32 = 1;

/// Module tag for diagnostics.
const TAG: &str = "addTiming";

/// A step that has been entered and not yet left.
#[derive(Debug)]
struct Frame {
    /// The step's name.
    name: String,
    /// When the step was entered, since the recorder began.
    entered: Duration,
    /// The time of the steps entered and left inside this one.
    inside: Duration,
}

/// What one step name has spent so far.
#[derive(Debug)]
struct Spent {
    /// The step's name.
    name: String,
    /// The time of the step itself, the steps inside it taken out.
    own: Duration,
    /// How many times the step was entered.
    calls: usize,
}

/// What:
///  The steps of one add, filled as the add runs and written when it ends.
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
    /// The steps entered and not left, the innermost last.
    open: Vec<Frame>,
    /// The time per step name, in the order the names first appeared.
    spent: Vec<Spent>,
    /// Facts about the add that are words, not times (the route taken, why not the snapshot), in the order they were noted.
    notes: Vec<(String, String)>,
    /// Milliseconds since the Unix epoch at the start, for finding the add among other logs.
    start_millis: u128,
    /// Time spent on worker threads by name, summed over the threads; threads overlap, so it is not part of the sum rule.
    work: Vec<Spent>,
    /// The longest quiet gap of every Git step and tool the add ran, by name, taken from the ledger when the record is written.
    quiet: Vec<(String, Duration)>,
}

impl Recorder {
    /// A recorder for an add in the repository whose common Git directory is `common_dir`.
    pub fn new(common_dir: PathBuf) -> Self {
        let start_millis: u128 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|elapsed: Duration| { return elapsed.as_millis(); }).unwrap_or_default();
        return Self { common_dir, open: Vec::new(), spent: Vec::new(), notes: Vec::new(), start_millis, work: Vec::new(), quiet: Vec::new() };
    }

    /// Enter the step `name` at `at`, a time since the recorder began.
    pub fn enter(&mut self, name: &str, at: Duration) {
        self.open.push(Frame { name: name.to_string(), entered: at, inside: Duration::ZERO });
    }

    /// Leave the innermost step at `at`; its own time is what the steps inside it left, and the outer step loses the whole.
    pub fn leave(&mut self, at: Duration) {
        let Some(frame) = self.open.pop() else { return };
        let whole: Duration = at.saturating_sub(frame.entered);
        let own: Duration = whole.saturating_sub(frame.inside);
        if let Some(outer) = self.open.last_mut() {
            outer.inside += whole;
        }
        match self.spent.iter_mut().find(|known: &&mut Spent| { return known.name == frame.name; }) {
            Some(known) => {
                known.own += own;
                known.calls += 1;
            }
            None => self.spent.push(Spent { name: frame.name, own, calls: 1 }),
        }
    }

    /// Note a fact about the add; a later note of the same key replaces the earlier one.
    pub fn note(&mut self, key: &str, value: &str) {
        match self.notes.iter_mut().find(|known: &&mut (String, String)| { return known.0 == key; }) {
            Some(known) => known.1 = value.to_string(),
            None => self.notes.push((key.to_string(), value.to_string())),
        }
    }

    /// Add `own` to the work named `name`, as one call.
    pub fn add_work(&mut self, name: &str, own: Duration) {
        match self.work.iter_mut().find(|known: &&mut Spent| { return known.name == name; }) {
            Some(known) => {
                known.own += own;
                known.calls += 1;
            }
            None => self.work.push(Spent { name: name.to_string(), own, calls: 1 }),
        }
    }

    /// The record as one line of JSON ending in a newline; `total` is the add's whole time.
    ///
    /// One `write` per line keeps the lines of concurrent adds from interleaving; a word with a quote or a newline is escaped
    /// by `quote`, so the line stays one line.
    pub fn line(&self, total: Duration, exit_code: i32) -> String {
        let micros_to_ms = |time: Duration| -> String { return format!("{:.3}", time.as_micros() as f64 / 1000.0); };
        let covered: Duration = self.spent.iter().map(|each: &Spent| { return each.own; }).sum();
        let steps: Vec<String> = self.spent.iter().map(|each: &Spent| { return format!("{}:{{\"ms\":{},\"calls\":{}}}", quote(each.name.as_str()), micros_to_ms(each.own), each.calls); }).collect();
        let work: Vec<String> = self.work.iter().map(|each: &Spent| { return format!("{}:{{\"ms\":{},\"calls\":{}}}", quote(each.name.as_str()), micros_to_ms(each.own), each.calls); }).collect();
        let notes: Vec<String> = self.notes.iter().map(|(key, value): &(String, String)| { return format!("{}:{}", quote(key.as_str()), quote(value.as_str())); }).collect();
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

/// The work of worker threads while an add is recorded: none outside a recording, so a stray call keeps nothing.
static WORK: Mutex<Option<Vec<(String, Duration)>>> = Mutex::new(None);

thread_local! {
    /// The recorder of the add this thread is running, and when it began; none outside an add.
    static ACTIVE: RefCell<Option<(Instant, Recorder)>> = const { RefCell::new(None) };
}

/// Begin recording the add of the repository whose common Git directory is `common_dir`, replacing any earlier recording
///  of this thread.
pub fn begin(common_dir: PathBuf) {
    ACTIVE.with(|active: &RefCell<Option<(Instant, Recorder)>>| { *active.borrow_mut() = Some((Instant::now(), Recorder::new(common_dir))); });
    if let Ok(mut work) = WORK.lock() {
        *work = Some(Vec::new());
    }
}

/// Run `body` as the step `name`; with no recording begun, `body` runs alone.
pub fn step<T>(name: &str, body: impl FnOnce() -> T) -> T {
    let entered: bool = ACTIVE.with(|active: &RefCell<Option<(Instant, Recorder)>>| {
        return match active.borrow_mut().as_mut() {
            Some((began, recorder)) => {
                recorder.enter(name, began.elapsed());
                true
            }
            None => false,
        };
    });
    let value: T = body();
    if entered {
        ACTIVE.with(|active: &RefCell<Option<(Instant, Recorder)>>| {
            if let Some((began, recorder)) = active.borrow_mut().as_mut() {
                recorder.leave(began.elapsed());
            }
        });
    }
    return value;
}

/// What:
///  The step name of one Git run: `git ` and the subcommand, found past the global options that take a value (`-C`, `-c`,
///  `--git-dir`, `--work-tree`) and the flags that do not.
/// Why:
///   The record says how many Git processes an add started and how long each kind took, which names the avoidable ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git_step_label(['-C', dir, '--no-optional-locks', 'status']) // 'git status'
/// ```
pub fn git_step_label(arguments: &[OsString]) -> String {
    let takes_value = |word: &OsStr| -> bool { return ["-C", "-c", "--git-dir", "--work-tree", "--namespace", "--exec-path"].iter().any(|known: &&str| { return word == OsStr::new(known); }); };
    let mut words = arguments.iter().map(OsString::as_os_str);
    while let Some(word) = words.next() {
        if takes_value(word) {
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
///  Run `body` as work named `name` on any thread, adding its time to the name's sum; with no recording begun, `body` runs alone.
/// Why:
///   The repair of package managers runs its tools on worker threads, which the thread's own steps cannot see; the sum over
///  threads says how much work a step held, and the step that waited for it says how long the add waited.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = work('clone', () => cloneTree());
/// ```
pub fn work<T>(name: &str, body: impl FnOnce() -> T) -> T {
    let began: Instant = Instant::now();
    let value: T = body();
    let spent: Duration = began.elapsed();
    if let Ok(mut work) = WORK.lock()
        && let Some(list) = work.as_mut()
    {
        list.push((name.to_string(), spent));
    }
    return value;
}

/// Note a fact about the add being recorded; nothing outside a recording.
pub fn note(key: &str, value: &str) {
    ACTIVE.with(|active: &RefCell<Option<(Instant, Recorder)>>| {
        if let Some((_began, recorder)) = active.borrow_mut().as_mut() {
            recorder.note(key, value);
        }
    });
}

/// What:
///  End the recording and append its record to the repository's add timing file, best effort.
/// Why:
///   The command's result never depends on the record: a file that cannot be written is only a debug line. The directory is
///  never created and nothing is flushed to disk, so the record costs one `write`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// finish(exitCode);
/// ```
pub fn finish(exit_code: i32) {
    let taken: Option<(Instant, Recorder)> = ACTIVE.with(|active: &RefCell<Option<(Instant, Recorder)>>| { return active.borrow_mut().take(); });
    let drained: Option<Vec<(String, Duration)>> = WORK.lock().ok().and_then(|mut work: std::sync::MutexGuard<'_, Option<Vec<(String, Duration)>>>| { return work.take(); });
    let Some((began, mut recorder)) = taken else { return };
    for (name, spent) in drained.unwrap_or_default() {
        recorder.add_work(name.as_str(), spent);
    }
    recorder.quiet = drain();
    let target: PathBuf = recorder.common_dir().join(ADD_TIMING_FILE);
    let line: String = recorder.line(began.elapsed(), exit_code);
    if let Err(error) = append_line(target.as_path(), line.as_bytes()) {
        debug(TAG, &format!("the add timing record could not be written to {}: {error}", target.display()));
    }
}

/// The accounting, the line and the writer.
#[cfg(test)]
#[path = "add_timing_tests.rs"]
mod tests;
