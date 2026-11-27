//! What:
//!  The accounting of one add's steps,
//!  and the JSON line that is written for it.
//! Why:
//!  The accounting is separate from the clock so that a control can drive it with chosen times:
//!  a step inside another is taken out of the outer one,
//!  so the steps are disjoint and what none covers is reported as `otherMs`.
//!  The line is written once when the add ends,
//!  in one `write`,
//!  and a word with a quote or a newline is escaped,
//!  so the lines of concurrent adds never interleave.
//!  This file is one part of the module whose name it carries,
//!  split because that module was over the line limit:
//!  `add_timing_steps.rs` holds the recording of one thread and the steps it publishes,
//!  and `add_timing.rs` holds the clock, the work of other threads and the file the line goes to.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const recorder = new Recorder(commonDir); recorder.enter('walkSource', at); recorder.leave(later);
//! const line = recorder.line(total, exitCode);
//! ```

use crate::iso_time::now_iso;
use crate::json_record::quote;
use crate::quiet_ledger::object_json;
use crate::service_protocol::RELEASE;
use std::path::{
    Path,
    PathBuf,
};
use std::time::Duration;

/// The version of the record's fields;
/// it changes only when a field's meaning changes,
/// never when a field is added.
pub const ADD_TIMING_SCHEMA_VERSION: u32 = 1;

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
fn add_spent(
    list: &mut Vec<Spent>,
    name: &str,
    own: Duration,
) {
    for known in list.iter_mut() {
        if known.name == name {
            known.own += own;
            known.calls += 1;
            return;
        }
    }
    list.push(Spent {
        name: name.to_string(),
        own,
        calls: 1,
    });
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
    return format!(
        "{}:{{\"ms\":{},\"calls\":{}}}",
        quote(each.name.as_str()),
        micros_to_ms(each.own),
        each.calls
    );
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
    pub(crate) quiet: Vec<(String, Duration)>,
}

impl Recorder {
    /// A recorder for an add in the repository whose common Git directory is `common_dir`.
    pub fn new(common_dir: PathBuf) -> Self {
        // A clock before the Unix epoch gives zero, as `unwrap_or_default` did.
        let start_millis: u128 =
            match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                Ok(elapsed) => elapsed.as_millis(),
                Err(_before_epoch) => 0,
            };
        return Self {
            common_dir,
            open: Vec::new(),
            spent: Vec::new(),
            notes: Vec::new(),
            start_millis,
            work: Vec::new(),
            quiet: Vec::new(),
        };
    }

    /// Enter the step `name` at `at`,
    /// a time since the recorder began.
    pub fn enter(
        &mut self,
        name: &str,
        at: Duration,
    ) {
        self.open.push(Frame {
            name: name.to_string(),
            entered: at,
            inside: Duration::ZERO,
        });
    }

    /// Leave the innermost step at `at`;
    /// its own time is what the steps inside it left,
    /// and the outer step loses the whole.
    pub fn leave(
        &mut self,
        at: Duration,
    ) {
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
    pub fn note(
        &mut self,
        key: &str,
        value: &str,
    ) {
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
    pub fn add_work(
        &mut self,
        name: &str,
        own: Duration,
    ) {
        add_spent(&mut self.work, name, own);
    }

    /// The record as one line of JSON ending in a newline;
    /// `total` is the add's whole time.
    ///
    /// One `write` per line keeps the lines of concurrent adds from interleaving;
    /// a word with a quote or a newline is escaped
    /// by `quote`,
    /// so the line stays one line.
    pub fn line(
        &self,
        total: Duration,
        exit_code: i32,
    ) -> String {
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

    /// The innermost step the recorder has entered and not left,
    /// or nothing between two steps.
    pub(crate) fn innermost(&self) -> Option<String> {
        let frame: &Frame = self.open.last()?;
        return Some(frame.name.clone());
    }

    /// The directory that holds the record's file.
    pub fn common_dir(&self) -> &Path {
        return self.common_dir.as_path();
    }
}
