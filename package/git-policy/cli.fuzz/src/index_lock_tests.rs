//! What: Controls proving the PID file and stat generators reach both outcomes, and that the
//!       invariants hold on every generated input.
//! Why: An invariant that no input reaches proves nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(outcomesReachedBy(generatedPidText)).toEqual(['accepted', 'refused']);
//! ```

/// Import the generators and invariants under control.
use super::{
    check_linux_stat, check_pid_file_text, generated_linux_stat, generated_pid_text,
};
/// The generated inputs the owner-record controls already share.
use crate::owners::tests::inputs;

/// Generated PID files are accepted with a value and refused, and every input holds the invariants.
#[test]
fn pid_files_reach_accepted_and_refused_outcomes() {
    let mut accepted: usize = 0;
    let mut refused: usize = 0;
    for data in inputs() {
        check_pid_file_text(data.as_slice());
        match generated_pid_text(data.as_slice()).1 {
            Some(_) => accepted += 1,
            None => refused += 1,
        }
    }
    assert!(accepted > 100, "{accepted}");
    assert!(refused > 100, "{refused}");
}

/// Generated stat lines are read to a state and ticks and refused, and every input holds the invariants.
#[test]
fn stat_lines_reach_accepted_and_refused_outcomes() {
    let mut accepted: usize = 0;
    let mut refused: usize = 0;
    for data in inputs() {
        check_linux_stat(data.as_slice());
        match generated_linux_stat(data.as_slice()).1 {
            Some(_) => accepted += 1,
            None => refused += 1,
        }
    }
    assert!(accepted > 100, "{accepted}");
    assert!(refused > 100, "{refused}");
}

/// Known spellings from the incumbent's controls hold the invariants.
#[test]
fn known_pid_and_stat_spellings_hold_the_invariants() {
    for text in ["pid 42\n", "pid 007", "pid 0\n", "pid 9007199254740992", "PID 42", "pid 4x", ""] {
        check_pid_file_text(text.as_bytes());
    }
    for line in [
        "4242 (git) S 1 4242 4242 0 -1 4194560 100 0 0 0 1 2 0 0 20 0 1 0 4200 1000 100 0\n",
        "4242 (a) b (c) R 1 4242 4242 0 -1 4194560 100 0 0 0 1 2 0 0 20 0 1 0 7 1000 100 0\n",
        "4242 (git) Z 1 4242 4242 0 -1 4194560 100 0 0 0 1 2 0 0 20 0 1 0 9007199254740992\n",
        "4242 git S 1 2 3",
    ] {
        check_linux_stat(line.as_bytes());
    }
}
