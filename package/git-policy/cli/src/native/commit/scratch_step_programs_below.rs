//! What:
//!  The programs one look of the waiting notice saw below the step of the scratch `git commit`,
//!  and the time each was credited with.
//! Why:
//!  The record's `below` list says which programs ran under the step and for how long,
//!  so a commit whose step ran 36,651 ms can be told from one whose signing program waited
//! (the owner,
//! 2026-11-08:
//! "About 36 seconds" of work it could not explain,
//! and
//! "If we can't figure it out,
//! then we have a defect and should log more").
//!  The kernel records no start and no end of a short program the notice could read,
//! so what is kept is a lower bound,
//!  and only the same process at two looks may be credited:
//! a name is not an identity,
//!  and two programs that carry one name are not one program
//! (P2 of the 2026-11-09 review).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const seen = seenBelow(look); creditBetweenLooks(facts, seen, nextLook, interval);
//! ```

/// What the record keeps of the programs below the step.
use crate::scratch_commit_step::ScratchStepFacts;
/// The kernel's name,
/// state and processor time of one process.
use crate::process_tree::ProcessUse;
/// Time spans,
/// for the interval one look credits.
use std::time::Duration;

/// What:
///  One process a look saw below the step,
/// as the next look needs it:
/// the number and the start time that prove it is the same process,
/// and the name the record keeps.
/// Why:
///   Two programs can carry one name,
/// so the name alone cannot say that the program a look credited was below the step at both ends of the interval.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SeenBelow = { pid: number; started: number; name: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeenBelow {
    /// The process number.
    pid: i32,
    /// The start time the kernel recorded for it.
    started: u64,
    /// The kernel's name for it.
    name: String,
}

/// What:
///  The processes one look saw below the step,
/// numbered once each.
/// Why:
///   The next look credits a program with the interval only when it was there at both ends,
/// so this is what the look is remembered by.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const seen = below.map(([pid, used]) => ({ pid, started: used.started, name: used.name }));
/// ```
pub fn seen_below(below: &[(i32, ProcessUse)]) -> Vec<SeenBelow> {
    let mut seen: Vec<SeenBelow> = Vec::with_capacity(below.len());
    for (pid, used) in below {
        seen.push(SeenBelow { pid: *pid, started: used.started, name: used.name.clone() });
    }
    return seen;
}

/// Whether the same program was below the step at both looks:
/// the same number,
/// the same start time,
/// which is what proves the number names that process and not a later one,
/// and the same name,
/// because an `exec` keeps both of the first two while the program behind them changes
/// (finding 7 of the second review).
fn seen_at_both_looks(before: &[SeenBelow], pid: i32, started: u64, name: &str) -> bool {
    for known in before {
        if known.pid == pid && known.started == started && known.name == name {
            return true;
        }
    }
    return false;
}

/// What:
///  Credit the time since the previous look to every program that was below the step at both looks,
/// and keep every program the look saw that cannot be credited at all.
/// Why:
///   The kernel records no start and no end of a short program that the notice could read,
/// so it measures what it can:
/// a program that starts and ends between two looks is not seen at all,
/// and the time an entry holds is a lower bound of the time that program ran.
///  Only the same process at both looks is credited:
/// a name is not an identity,
/// and a program that took another's name between the looks ran in neither interval it would be credited with.
///  A program seen once is kept with a zero lower bound,
/// so the record says it was below the step even when no interval can be measured for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [pid, used] of now) facts.credit(used.name, seenAtBothLooks(before, pid, used.started) ? sinceLastLook : 0);
/// ```
pub fn credit_between_looks(facts: &mut ScratchStepFacts, before: &[SeenBelow], now: &[(i32, ProcessUse)], interval: Duration) {
    let mut credited: Vec<&str> = Vec::new();
    for (pid, used) in now {
        if credited.contains(&used.name.as_str()) {
            continue;
        }
        let at_both_looks: bool = seen_at_both_looks(before, *pid, used.started, used.name.as_str());
        facts.credit(used.name.as_str(), if at_both_looks { interval } else { Duration::ZERO });
        credited.push(used.name.as_str());
    }
}
