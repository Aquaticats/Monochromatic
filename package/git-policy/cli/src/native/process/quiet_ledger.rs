//! What:
//!  The longest quiet gap of every Git step and every tool this process ran, kept until a timing record takes it.
//!  A quiet gap is a stretch in which the step showed no sign of work: no bytes moved, no processor time, no task running or
//!  waiting for the disk (the signs of `push_progress.rs`).
//! Why:
//!  A step ends only when it has been quiet for a window, so the window must be set above the longest gap a healthy step
//!  shows (`doc/planning/cli-git-shipped-limits-audit.md`, "Time limits on Git child steps of a commit").
//!  Nothing logged a gap before this, only whole times, and a whole time is an upper bound of the gap, never the gap.
//!  The timing records (`commit_timing.rs`, `add_timing.rs`) take the figures when they are written, so a measurement
//!  comes from the runs the person's own work makes. A step that was ended for being quiet is kept apart from the steps
//!  that ended by themselves, because only the steps that ended by themselves show how quiet a healthy step gets.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const gaps = new Map<string, number>();
//! export const note = (step: string, gapMs: number, ended: boolean) => gaps.set(key(step, ended), Math.max(gaps.get(key(step, ended)) ?? 0, gapMs));
//! export const drain = () => { const all = [...gaps]; gaps.clear(); return all; };
//! ```

/// Escaping of a step name inside a JSON string.
use crate::json_record::quote;
/// A lock around the shared list.
use std::sync::Mutex;
/// A span of time.
use std::time::Duration;

/// The suffix of a step name whose run was ended for being quiet, so it never mixes with the steps that ended by themselves.
pub const ENDED_SUFFIX: &str = " [ended for being quiet]";

/// The longest gap of each step name so far; the names are the Git subcommands and the tool names, a small fixed set.
static GAPS: Mutex<Vec<(String, Duration)>> = Mutex::new(Vec::new());

/// What:
///  Keep `gap` as the longest quiet gap of `step` when it is longer than the one kept.
/// Why:
///   A step runs many times in one command; the longest of them is the figure an interval has to clear.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// note('git rev-parse', 1200, false);
/// ```
pub fn note(step: &str, gap: Duration, ended_for_being_quiet: bool) {
    let name: String = if ended_for_being_quiet { format!("{step}{ENDED_SUFFIX}") } else { step.to_owned() };
    // A poisoned lock means another thread panicked while holding it; the list is still a list of gaps.
    let mut gaps = GAPS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    match gaps.iter_mut().find(|(known, _)| { return *known == name; }) {
        Some((_, longest)) => *longest = (*longest).max(gap),
        None => gaps.push((name, gap)),
    }
}

/// What:
///  Take every figure kept and forget them, in the order of the step names.
/// Why:
///   One record takes the figures of the command it describes; the next command starts from nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const figures = drain();
/// ```
pub fn drain() -> Vec<(String, Duration)> {
    let mut gaps = GAPS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut taken: Vec<(String, Duration)> = std::mem::take(&mut *gaps);
    taken.sort();
    return taken;
}

/// The milliseconds of a gap with three decimals, the way the timing records write times.
fn milliseconds(gap: Duration) -> String {
    return format!("{:.3}", gap.as_micros() as f64 / 1000.0);
}

/// What:
///  The figures as a JSON object from step name to milliseconds.
/// Why:
///   Both timing records carry it under one key, so a reader asks one question of both files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify(Object.fromEntries(figures.map(([step, gap]) => [step, gap])))
/// ```
pub fn object_json(figures: &[(String, Duration)]) -> String {
    let members: Vec<String> = figures.iter().map(|(step, gap)| { return format!("{}:{}", quote(step.as_str()), milliseconds(*gap)); }).collect();
    return format!("{{{}}}", members.join(","));
}

/// What:
///  The longest gap among the steps that ended by themselves, in milliseconds, or zero when none ran.
/// Why:
///   One number answers "how quiet did a healthy step get in this command" without reading the object.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Math.max(0, ...figures.filter(([step]) => !step.endsWith(ENDED_SUFFIX)).map(([, gap]) => gap))
/// ```
pub fn longest_healthy(figures: &[(String, Duration)]) -> String {
    let longest: Duration = figures
        .iter()
        .filter(|(step, _)| { return !step.ends_with(ENDED_SUFFIX); })
        .map(|(_, gap)| { return *gap; })
        .max()
        .unwrap_or(Duration::ZERO);
    return milliseconds(longest);
}

/// The figures are kept, drained and written as controls in the test module.
#[cfg(test)]
#[path = "quiet_ledger_tests.rs"]
mod tests;
