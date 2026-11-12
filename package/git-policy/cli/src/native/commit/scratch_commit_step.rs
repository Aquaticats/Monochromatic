//! What:
//!  What the waiting notice of the scratch `git commit` saw while the step ran,
//! one entry per build,
//! kept until the commit's timing record takes them.
//! Why:
//!  The record holds one figure for the step,
//! and that figure names no part of it:
//! Git and the signing program run in one process tree with no sub-step figure.
//!  So a commit whose step ran 36,651 ms under a 3,006 ms ceiling on 2026-11-08 (release 0.6.1-alpha.4,
//! landed)
//! could not be explained,
//! and neither could the 171,383 and 26,256 ms steps of the day before,
//! which the record's own steps only place on a retry.
//!  The owner's answer (2026-11-08,
//! "About 16 seconds"):
//! "If we can't figure it out,
//! then we have a defect and should log more."
//!  What is logged here is what the notice thread already reads:
//! whether a ceiling was armed at all (a commit that may open an editor keeps none,
//! and a thread the system refuses keeps none either),
//! how long the notice's own end took,
//! how long the writer spent on the lines it finished,
//! and how many lines it had to drop because the reader stopped taking them,
//! how many looks it made and how many found the step,
//! which programs other than Git ran below the step and for how long,
//! the processor time everything below had used at the last look,
//! and how many looks found one of them in uninterruptible sleep (`D`),
//! which is a stalled disk or a processor it is not given rather than work,
//! how many looks could not be made at all,
//! which is a `/proc` that refused the read rather than a step that ended,
//! and the moment a ceiling stood aside for a step the kernel held,
//! with the bound the step then ran under,
//! so a commit whose step was slow and landed is told from one whose ceiling ended it.
//!  The time a program below the step is credited is measured at the look's interval,
//! so a program that starts and ends between two looks is not seen at all:
//! the figure is a lower bound of the time it ran.
//!  Nothing here holds a message,
//! an argument,
//! a path or an environment value:
//! the names are the kernel's `comm`,
//! cut to the first 15 characters.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts: ScratchStepFacts[] = [];
//! export const noteScratchStep = (f) => facts.push(f);
//! export const drainScratchSteps = () => facts.splice(0);
//! ```

/// Escaping of a word inside a JSON string.
use crate::json_record::quote;
/// A lock around the shared list.
use std::sync::Mutex;
/// Time spans and the ceiling of the step.
use std::time::Duration;

/// The most programs one entry names.
/// A step runs Git and,
/// when the commit is signed,
/// one signing program,
/// so one or two names are the usual number;
/// the step runs with `core.hooksPath` at a directory that cannot exist and every commit event switched off
/// (`commit_preparation.rs`),
/// so no hook adds a name.
/// The list is bounded so a record never grows with the work,
/// and a step that starts more programs than this names the first ones only.
pub const MAX_BELOW_NAMES: usize = 8;

/// One program that ran below the step,
/// and how long the same process was seen.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeenProgram {
    /// The kernel's name for it.
    pub name: String,
    /// The time it was seen below the step,
    /// measured at the notice's look interval,
    /// and only in the intervals in which the same process number and start time were there at both ends.
    /// Zero is a program the notice saw and could not credit with any interval.
    pub seen: Duration,
}

/// What the notice of one build of the scratch commit saw.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScratchStepFacts {
    /// The ceiling the step ran under,
    /// and nothing when the commit may open the person's editor and has no ceiling.
    pub ceiling: Option<Duration>,
    /// Whether a notice thread ran to enforce the ceiling:
    /// false for a commit with an editor,
    /// and for one whose thread the system refused,
    /// which the wait that owns the step ends instead,
    /// and whose stop then says `true`.
    ///  A thread that ran but did not answer within the notice's bound is `true` with `looks` at zero,
    /// which no thread that ran can otherwise report:
    /// every one of them looks before it can end.
    pub armed: bool,
    /// How long the step had lasted when the notice ended it,
    /// and nothing when the step ended by itself.
    pub stopped_after: Option<Duration>,
    /// How long the step had lasted when its ceiling came for a step the kernel held in a disk wait,
    /// and nothing when no ceiling stood aside for it.
    ///  A step the kernel holds is slow and not stuck,
    ///  so its ceiling stands aside once and the step runs under `stall_ceiling` instead.
    pub held_after_ceiling: Option<Duration>,
    /// The bound the step ran under once its ceiling stood aside for a disk wait:
    /// `wait_ceilings::SCRATCH_COMMIT_STEP_STALL_CEILING`,
    /// and nothing when the ceiling never stood aside.
    pub stall_ceiling: Option<Duration>,
    /// How long the notice's own join took after the step ended:
    /// its lines are written from its thread,
    /// so a writing that waits for its reader is this figure and not the step's.
    pub notice: Duration,
    /// How long the writer spent on the lines it finished.
    ///  A write still in progress when the commit ended is not in it,
    /// so this is a lower bound of the time a reader cost.
    pub wrote: Duration,
    /// How many lines were dropped because the writer still held the one before them,
    /// which is a reader that stopped taking them.
    pub lines_dropped: u32,
    /// How many looks the notice made for the step.
    pub looks: u32,
    /// How many of those found at least one process below the command.
    pub looks_found: u32,
    /// The programs other than Git seen below the step,
    /// in the order first seen.
    pub below: Vec<SeenProgram>,
    /// The processor time everything below the step had used at the last look.
    pub below_cpu: Duration,
    /// Whether the ceiling could not list the programs below the step when it ended it,
    /// so one of them may still be running.
    pub below_unlisted: bool,
    /// How many programs below the step were not ended:
    /// one the kernel refused a descriptor for,
    /// or one it refused the signal to.
    /// Such a program may still be running,
    /// which the stop's words say (round six of the review).
    pub below_unended: u32,
    /// How many looks could not be made at all,
    /// because `/proc` could not be read
    /// (a descriptor or memory limit,
    /// a `/proc` that is not there).
    pub blind_looks: u32,
    /// How many looks found a process below the step in uninterruptible sleep.
    pub blocked_looks: u32,
}

impl ScratchStepFacts {
    /// What:
    ///  The facts of a step whose ceiling no thread enforced.
    /// Why:
    ///   A commit that may open an editor keeps no ceiling;
    /// one whose thread could not be started keeps a ceiling that the wait owning the step enforces,
    /// and the stop it makes says so.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const unarmed = (ceiling?: number): ScratchStepFacts => ({ ceiling, armed: false, stoppedAfter: undefined, notice: 0, wrote: 0, looks: 0, looksFound: 0, below: [], belowCpu: 0, blockedLooks: 0 });
    /// ```
    pub fn unarmed(ceiling: Option<Duration>) -> Self {
        return Self {
            ceiling,
            armed: false,
            stopped_after: None,
            held_after_ceiling: None,
            stall_ceiling: None,
            notice: Duration::ZERO,
            wrote: Duration::ZERO,
            lines_dropped: 0,
            looks: 0,
            looks_found: 0,
            below: Vec::new(),
            below_cpu: Duration::ZERO,
            below_unlisted: false,
            below_unended: 0,
            blind_looks: 0,
            blocked_looks: 0,
        };
    }

    /// What:
    ///  Keep the time one program was seen below the step,
    /// adding to a name already there.
    /// Why:
    ///   The notice credits each look's interval to every program it saw at both ends of it,
    /// so one program that runs and ends and starts again is one entry.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const entry = facts.below.find((b) => b.name === name); if (entry) entry.seen += interval; else if (facts.below.length < MAX) facts.below.push({ name, seen: interval });
    /// ```
    pub fn credit(&mut self, name: &str, interval: Duration) {
        for program in &mut self.below {
            if program.name == name {
                program.seen = program.seen.saturating_add(interval);
                return;
            }
        }
        if self.below.len() >= MAX_BELOW_NAMES {
            return;
        }
        self.below.push(SeenProgram { name: name.to_string(), seen: interval });
    }
}

/// The facts of this process since a record last took them,
/// in the order the builds ended.
static FACTS: Mutex<Vec<ScratchStepFacts>> = Mutex::new(Vec::new());

/// What:
///  Keep the facts of one build of the scratch commit.
/// Why:
///   A commit whose branch moved builds again,
/// so one command can run the step more than once,
/// and the record lists them in the order they ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noteScratchStep(facts);
/// ```
pub fn note_scratch_step(facts: ScratchStepFacts) {
    // A poisoned lock means another thread panicked while holding it; the list is still a list of facts.
    let mut kept = FACTS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    kept.push(facts);
}

/// What:
///  Take every build's facts and forget them.
/// Why:
///   One record takes the builds of the command it describes;
/// the next command starts from nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = drainScratchSteps();
/// ```
pub fn drain_scratch_steps() -> Vec<ScratchStepFacts> {
    let mut kept = FACTS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    return std::mem::take(&mut *kept);
}

/// One figure as the record states it:
/// a number of milliseconds,
/// or `null`.
fn millis(word: Option<Duration>) -> String {
    return match word {
        Some(span) => span.as_millis().to_string(),
        None => String::from("null"),
    };
}

/// What:
///  One program below the step as a JSON object:
/// `{"name":"ssh-keygen","seenMs":1200}`.
/// Why:
///   `scratch_steps_json` joins them into the entry's list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify({ name, seenMs: seen })
/// ```
fn seen_json(program: &SeenProgram) -> String {
    return format!("{{\"name\":{},\"seenMs\":{}}}", quote(program.name.as_str()), program.seen.as_millis());
}

/// What:
///  One build's facts as a JSON object.
/// Why:
///   The client's timing record carries one entry per build under `scratchSteps`,
/// and an entry says of every figure what it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify({ ceilingMs, armed, stoppedAfterMs, heldAfterCeilingMs, stallCeilingMs, noticeMs, wroteMs, linesDropped, looks, looksFound, below, belowCpuMs, belowUnlisted, belowUnended, blindLooks, blockedLooks })
/// ```
fn facts_json(facts: &ScratchStepFacts) -> String {
    let names: Vec<String> = facts.below.iter().map(seen_json).collect();
    return format!(
        "{{\"ceilingMs\":{},\"armed\":{},\"stoppedAfterMs\":{},\"heldAfterCeilingMs\":{},\"stallCeilingMs\":{},\"noticeMs\":{},\"wroteMs\":{},\"linesDropped\":{},\"looks\":{},\"looksFound\":{},\"below\":[{}],\"belowCpuMs\":{},\"belowUnlisted\":{},\"belowUnended\":{},\"blindLooks\":{},\"blockedLooks\":{}}}",
        millis(facts.ceiling),
        facts.armed,
        millis(facts.stopped_after),
        millis(facts.held_after_ceiling),
        millis(facts.stall_ceiling),
        facts.notice.as_millis(),
        facts.wrote.as_millis(),
        facts.lines_dropped,
        facts.looks,
        facts.looks_found,
        names.join(","),
        facts.below_cpu.as_millis(),
        facts.below_unlisted,
        facts.below_unended,
        facts.blind_looks,
        facts.blocked_looks,
    );
}

/// What:
///  The entries as a JSON array,
/// in the order given.
/// Why:
///   The record carries it under `scratchSteps`,
/// which is empty for a commit that ran no scratch `git commit`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify(entries)
/// ```
pub fn scratch_steps_json(entries: &[ScratchStepFacts]) -> String {
    let members: Vec<String> = entries.iter().map(facts_json).collect();
    return format!("[{}]", members.join(","));
}

/// The ledger,
/// its order and its escaping are kept as controls in the test module beside it.
#[cfg(test)]
#[path = "scratch_commit_step_tests.rs"]
mod tests;
