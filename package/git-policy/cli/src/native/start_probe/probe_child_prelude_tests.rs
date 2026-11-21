//! What:
//!  Controls of the prelude a child runs:
//!  this process can join a group of its own,
//!  ask for a parent-death signal,
//!  and read the parent it belongs to.
//! Why:
//!  A child whose first calls refuse must not go on to its steps,
//!  and these three are the ones every child makes before anything else.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! setpgid(0, 0); prctl(PR_SET_PDEATHSIG, SIGKILL); getppid();
//! ```

use super::{join_own_process_group, parent_death_signal, parent_id};

/// A process can join a group of its own,
/// ask for a parent-death signal,
/// and name the parent it belongs to.
#[test]
fn the_prelude_calls_answer() {
    join_own_process_group().expect("the process joins its own group");
    parent_death_signal().expect("the parent-death signal is set");
    assert!(parent_id() > 0);
}
