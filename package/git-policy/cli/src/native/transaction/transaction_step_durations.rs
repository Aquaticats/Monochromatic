//! What:
//!  The Git steps a command or a landing started,
//! each with the program and subcommand,
//! when it began and how long it took,
//! kept until a timing record takes them.
//! Why:
//!  A timing record that names only phases cannot say which step spent a slow phase:
//! on 2026-11-07 a commit spent 241 s in its
//! build phase,
//! which runs real Git in a scratch repository,
//! and the record held no entry for that step (the quiet ledger keeps
//! the longest quiet gap of a step,
//! not how long it ran).
//! The name is the program and the subcommand (`git commit`),
//! never a path,
//! a message,
//! an option value or the environment,
//! so a
//! record carries nothing the person typed.
//! The list is bounded (`MAX_STEP_RUNS`) and says how many entries it dropped,
//! so a record never grows with the work.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const runs: StepRun[] = [];
//! export const noteStep = (name: string, began: number, took: number) => { if (runs.length < MAX) runs.push({ name, began, took }); else dropped++; };
//! export const drainSteps = (origin: number) => ({ runs: runs.splice(0).sort(byBegan).map(offsetFrom(origin)), dropped });
//! ```

/// Escaping of a step name inside a JSON string.
use crate::json_record::quote;
/// A lock around the shared list.
use std::sync::Mutex;
/// A moment and a span of time.
use std::time::{Duration, Instant};

/// The most steps one record lists.
///
/// Measured on 2026-11-07 with `strace` over ten commits of the shaped repository (`native:perf --trace-directory`):
/// a commit starts
/// 10 Git processes in the client and 7 in the service,
/// 17 in all.
/// A landing that loses its branch is prepared again at most
/// `LANDING_ATTEMPTS` (3) times,
/// so the most one commit starts is 3 times 17,
/// 51;
/// 64 is the next power of two above it.
/// The record
/// names how many steps went unlisted,
/// so a longer run is counted,
/// not lost silently.
/// At about 70 bytes an entry,
/// 64 entries are
/// under 4.5 KiB,
/// a small share of the one mebibyte the timing file holds.
pub const MAX_STEP_RUNS: usize = 64;

/// One step that ran.
#[derive(Clone, Debug)]
pub struct StepRun {
    /// The program and the subcommand,
    /// for example `git commit`.
    pub name: String,
    /// When the step started.
    pub began: Instant,
    /// How long it ran.
    pub took: Duration,
}

/// The steps kept and the number that did not fit.
struct Ledger {
    /// The steps kept,
    /// in the order they ended.
    runs: Vec<StepRun>,
    /// How many steps came after the list was full.
    dropped: usize,
}

/// The steps of this process since a record last took them.
static STEPS: Mutex<Ledger> = Mutex::new(Ledger { runs: Vec::new(), dropped: 0 });

/// What:
///  Keeps one step that has just ended.
/// Why:
///   The Git runner calls it after every step it ran,
/// whatever the step's outcome.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noteStep('git commit', began, Date.now() - began);
/// ```
pub fn note_step(name: &str, began: Instant, took: Duration) {
    // A poisoned lock means another thread panicked while holding it; the list is still a list of steps.
    let mut ledger = STEPS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if ledger.runs.len() >= MAX_STEP_RUNS {
        ledger.dropped += 1;
        return;
    }
    ledger.runs.push(StepRun { name: plain_name(name), began, took });
}

/// What:
///  How much of a stretch of the command the Git steps it started took,
///  clipped to the stretch's end.
/// Why:
///   A preparation's stretch is measured from seam to seam,
///  and the Git child that ran inside it is recorded apart;
///  taking it out here is what makes a stretch's figure the time it spent waiting
///  rather than the time of a step another reading already names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const children = runs.filter((run) => run.began >= from && run.began <= to).reduce((sum, run) => sum + Math.min(run.took, to - run.began), 0);
/// ```
pub fn step_time_between(from: Instant, to: Instant) -> Duration {
    let ledger = STEPS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut taken: Duration = Duration::ZERO;
    for run in &ledger.runs {
        if run.began < from || run.began > to {
            continue;
        }
        let room: Duration = to.saturating_duration_since(run.began);
        taken += run.took.min(room);
    }
    return taken;
}

/// The name with every character that is not a letter,
/// a digit,
/// a space,
/// a hyphen or an underscore replaced by `?`,
/// so a name can
///  never carry a path,
/// an option value or a separator of the log line.
/// The note of a failed step (`git_step_failure.rs`) is written with it too,
/// so both records name a step the same way.
pub(crate) fn plain_name(name: &str) -> String {
    return name.chars().map(plain_character).collect();
}

/// One character of a name,
/// or `?` when it is none of the allowed ones.
fn plain_character(character: char) -> char {
    return if character.is_ascii_alphanumeric() || matches!(character, ' ' | '-' | '_') { character } else { '?' };
}

/// One step as its record knows it:
/// offset from the record's origin and duration,
/// in whole milliseconds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepFigure {
    /// The program and the subcommand.
    pub name: String,
    /// Milliseconds from the origin to the step's start.
    pub offset_ms: u64,
    /// Milliseconds the step ran.
    pub duration_ms: u64,
}

/// What:
///  Every step kept,
/// ordered by when it began and measured from `origin`,
/// with the number dropped;
/// the list is emptied.
/// Why:
///   One record takes the steps of the command it describes;
/// the next starts from nothing.
/// Threads add steps in the order they end,
///  so the order is made here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { figures, dropped } = drainSteps(commandStartedAt);
/// ```
pub fn drain_steps(origin: Instant) -> (Vec<StepFigure>, usize) {
    let mut ledger = STEPS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut taken: Vec<StepRun> = std::mem::take(&mut ledger.runs);
    let dropped: usize = std::mem::take(&mut ledger.dropped);
    drop(ledger);
    taken.sort_by_key(began_of);
    let mut figures: Vec<StepFigure> = Vec::new();
    for run in &taken {
        figures.push(figure_of(run, origin));
    }
    return (figures, dropped);
}

/// When a step started,
/// the key the steps are ordered by.
fn began_of(run: &StepRun) -> Instant {
    return run.began;
}

/// The figure of one step against an origin;
/// a step that began before the origin starts at zero.
fn figure_of(run: &StepRun, origin: Instant) -> StepFigure {
    return StepFigure {
        name: run.name.clone(),
        offset_ms: u64::try_from(run.began.saturating_duration_since(origin).as_millis()).unwrap_or(u64::MAX),
        duration_ms: u64::try_from(run.took.as_millis()).unwrap_or(u64::MAX),
    };
}

/// What:
///  One figure as a JSON object:
/// `{"name":"git commit","offsetMs":12,"durationMs":1004}`.
/// Why:
///   `steps_json` joins them into the record's list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify({ name, offsetMs, durationMs })
/// ```
fn figure_json(figure: &StepFigure) -> String {
    return format!("{{\"name\":{},\"offsetMs\":{},\"durationMs\":{}}}", quote(figure.name.as_str()), figure.offset_ms, figure.duration_ms);
}

/// What:
///  The steps as a JSON array,
/// in the order given.
/// Why:
///   The client's timing record carries it under `steps`,
/// with the number dropped under `stepsDropped`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify(figures)
/// ```
pub fn steps_json(figures: &[StepFigure]) -> String {
    let members: Vec<String> = figures.iter().map(figure_json).collect();
    return format!("[{}]", members.join(","));
}

/// What:
///  The steps as one word for the service's log line:
/// `name@offset+duration` joined by semicolons.
/// Why:
///   The service's lines are `key=value` words;
/// the name has no semicolon,
/// at sign or plus because it is a program and a
///  subcommand of Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// figures.map((f) => `${f.name}@${f.offsetMs}+${f.durationMs}`).join(';')
/// ```
pub fn steps_word(figures: &[StepFigure]) -> String {
    let words: Vec<String> = figures.iter().map(step_word).collect();
    return words.join(";");
}

/// One step as `name@offset+duration`.
fn step_word(figure: &StepFigure) -> String {
    return format!("{}@{}+{}", figure.name, figure.offset_ms, figure.duration_ms);
}

/// The ledger is kept,
/// bounded,
/// ordered and written as controls in the test module.
#[cfg(test)]
#[path = "transaction_step_durations_tests.rs"]
mod tests;
