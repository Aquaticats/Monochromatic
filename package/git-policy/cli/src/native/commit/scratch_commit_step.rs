//! What:
//!  What the waiting notice of the scratch `git commit` saw while the step ran,
//! one entry per build,
//! kept until the commit's timing record takes them.
//! Why:
//!  The record holds one figure for the step,
//! and that figure names no part of it:
//! Git and the signing program run in one process tree with no sub-step figure,
//! and the wrapper's own time after the step (the notice thread's join,
//! which waits for its line to be written)
//! sat inside the same figure.
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
//! how long the notice's own join took,
//! how long its lines took to write,
//! how many looks it made and how many found the step,
//! which programs other than Git ran below the step and for how long,
//! the processor time everything below had used at the last look,
//! and how many looks found one of them in uninterruptible sleep (`D`),
//! which is a stalled disk or a processor it is not given rather than work.
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
/// Measured on 2026-11-09 with `strace` over ten commits of the shaped repository:
/// a step runs Git and,
/// when the commit is signed,
/// one signing program,
/// so two names are the usual number;
/// a hook would add one,
/// and the step runs with every hook off.
pub const MAX_BELOW_NAMES: usize = 8;

/// One program other than Git that ran below the step,
/// and how long it was seen.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeenProgram {
    /// The kernel's name for it.
    pub name: String,
    /// The time it was seen below the step,
    /// measured at the notice's look interval.
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
    /// and for one whose thread the system refused.
    pub armed: bool,
    /// How long the step had lasted when the notice ended it,
    /// and nothing when the step ended by itself.
    pub stopped_after: Option<Duration>,
    /// How long the notice's own join took after the step ended:
    /// its lines are written from its thread,
    /// so a writing that waits for its reader is this figure and not the step's.
    pub notice: Duration,
    /// How long the notice spent writing its lines.
    pub wrote: Duration,
    /// How many looks the notice made for the step.
    pub looks: u32,
    /// How many of those found at least one process below the command.
    pub looks_found: u32,
    /// The programs other than Git seen below the step,
    /// in the order first seen.
    pub below: Vec<SeenProgram>,
    /// The processor time everything below the step had used at the last look.
    pub below_cpu: Duration,
    /// How many looks found a process below the step in uninterruptible sleep.
    pub blocked_looks: u32,
}

impl ScratchStepFacts {
    /// What:
    ///  The facts of a step whose ceiling no thread enforced.
    /// Why:
    ///   A commit that may open an editor keeps no ceiling,
    /// and one whose thread could not be started keeps none either;
    /// the record says which of the two it was.
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
            notice: Duration::ZERO,
            wrote: Duration::ZERO,
            looks: 0,
            looks_found: 0,
            below: Vec::new(),
            below_cpu: Duration::ZERO,
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
/// JSON.stringify({ ceilingMs, armed, stoppedAfterMs, noticeMs, wroteMs, looks, looksFound, below, belowCpuMs, blockedLooks })
/// ```
fn facts_json(facts: &ScratchStepFacts) -> String {
    let names: Vec<String> = facts.below.iter().map(seen_json).collect();
    return format!(
        "{{\"ceilingMs\":{},\"armed\":{},\"stoppedAfterMs\":{},\"noticeMs\":{},\"wroteMs\":{},\"looks\":{},\"looksFound\":{},\"below\":[{}],\"belowCpuMs\":{},\"blockedLooks\":{}}}",
        millis(facts.ceiling),
        facts.armed,
        millis(facts.stopped_after),
        facts.notice.as_millis(),
        facts.wrote.as_millis(),
        facts.looks,
        facts.looks_found,
        names.join(","),
        facts.below_cpu.as_millis(),
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
