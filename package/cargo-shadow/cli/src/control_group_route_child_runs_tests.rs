//! What:
//! Controls for the grace a killed short program is given to become reapable:
//! that the wait the discovery makes notices a killed program within it,
//! and that the grace is long enough for the polls the code makes.
//! Why:
//! The doc beside `KILL_REAP_GRACE` states what the polling does,
//! and the statement used to name a poll interval the code never used,
//! so the control reads the wait the code makes rather than a copy of its idea of it.
//! A handful of kills is what a measurement of this needs;
//! a loop of dozens would be the synthetic load the machine's rules ban.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('the reap grace', () => { it('reaps a killed program within the grace', ...) })
//! ```

/// What:
/// `use super::{KILL_REAP_GRACE, Reap, reap_within};` imports the grace under test,
/// what the wait answers,
/// and the wait itself.
/// Why:
/// The control judges the same wait the step that ends a run's unit uses,
/// not a second implementation of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { KILL_REAP_GRACE, reapWithin } from './control-group-route-child-runs'
/// ```
use super::{
    KILL_REAP_GRACE,
    Reap,
    reap_within,
};
/// What:
/// `use crate::control_group_route_discovery::DISCOVERY_POLL;` imports the interval at which the wait asks whether the
/// killed program ended.
/// Why:
/// The grace is worth what that interval makes it worth,
/// so the control relates the two rather than reading either alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DISCOVERY_POLL } from './control-group-route-discovery'
/// ```
use crate::control_group_route_discovery::DISCOVERY_POLL;
/// What:
/// `use std::process::Command;` imports the command builder.
/// Why:
/// The program the control kills is a real `sleep`,
/// which costs nothing and cannot be mistaken for the wait under test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { spawn } from 'node:child_process';
/// ```
use std::process::Command;
/// What:
/// `use std::time::{Duration, Instant};` imports a span of time and a point on the clock (its sibling `SystemTime` is the wall
/// clock, which can jump).
/// Why:
/// Each wait is measured from the kill to the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Date.now() and a number of milliseconds
/// ```
use std::time::{
    Duration,
    Instant,
};

/// How many killed programs the control waits for.
/// A handful is what the grace needs to be judged against the polling,
/// and every kill adds a process start to a suite that already runs hundreds of them.
const KILLS: usize = 5;

/// The fewest polls the grace has to afford.
/// A child ends just after one poll as often as just before it,
/// and a wakeup delayed by the machine's own load costs polls,
/// so a grace that is worth only one or two of them would report a program as unreapable after a delay that says nothing
/// about the program.
const POLLS_THE_GRACE_AFFORDS: u32 = 10;

/// The control of the grace the doc beside `KILL_REAP_GRACE` states:
/// a killed `sleep 30` is reaped within it,
/// the wait takes as long as the code's polling makes it take,
/// and the grace is worth several of those polls.
/// The measured waits are printed so the sentence beside the constant can be checked against a run.
#[test]
fn a_killed_program_is_reaped_within_the_grace_at_the_polls_the_code_makes() {
    let mut waits: Vec<Duration> = Vec::with_capacity(KILLS);
    for _ in 0..KILLS {
        let mut child = Command::new("sleep")
            .arg("30")
            .spawn()
            .expect("a program to kill");
        child.kill().expect("the program is killed");
        let began: Instant = Instant::now();
        let answered: Reap = reap_within(&mut child, began + KILL_REAP_GRACE);
        let waited: Duration = began.elapsed();
        // What:     `if !matches!(answered, Reap::Reaped)` runs for a wait that did not reap; it ends the child and waits for it,
        //           so a failed assertion leaves no process and no zombie behind.
        // Why:      A control that fails still has to leave the machine as it found it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!reaped) { child.kill(); await child.wait(); }
        // ```
        if !matches!(answered, Reap::Reaped) {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "a killed `sleep 30` was not reaped within the grace of {KILL_REAP_GRACE:?}: the wait answered after {waited:?}"
            );
        }
        waits.push(waited);
    }

    let mut sorted: Vec<Duration> = waits.clone();
    sorted.sort();
    let median: Duration = sorted[sorted.len() / 2];
    let longest: Duration = *sorted.last().expect("at least one kill");
    for waited in &sorted {
        eprintln!("reap_within: a killed `sleep 30` was reaped after {waited:?}");
    }
    eprintln!(
        "reap_within: {KILLS} kills, median {median:?}, longest {longest:?}, poll {DISCOVERY_POLL:?}, grace {KILL_REAP_GRACE:?}"
    );

    assert!(
        longest < KILL_REAP_GRACE,
        "the longest wait, {longest:?}, is not inside the grace of {KILL_REAP_GRACE:?} that the note promises"
    );
    assert!(
        KILL_REAP_GRACE >= DISCOVERY_POLL * POLLS_THE_GRACE_AFFORDS,
        "the grace of {KILL_REAP_GRACE:?} is worth fewer than {POLLS_THE_GRACE_AFFORDS} polls of {DISCOVERY_POLL:?}, so a wakeup delayed by the machine's own load would report a program as unreapable"
    );
}
