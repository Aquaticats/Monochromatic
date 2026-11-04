//! Historical control, not approval evidence for revision 13: compare and unlink are one transition and M1 content evidence is not modelled.
//!
//! What:
//!  Model of the revision 8 finding R8-8 for M1: the process that materializes a published request into the index is killed
//!  at each boundary, and the next owner recovers.
//! Why:
//!  Revision 9 (`doc/planning/cli-git-commits-never-fail.md`, M1) builds an unnamed file, journals its identity, takes
//!  `index.lock` by `linkat`, sets the marker, renames, applies the per-path outcomes and writes even; L4 recovers by identity.
//!  Revision 8 relied on the dead process's exit handling to remove its lock, which a killed process does not run.
//!  A foreign program can hold or replace `index.lock` and the index at any point; it must never be touched.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type State = { rules; step; dead; lock: 'none'|'ours'|'foreign'; index: 'old'|'ours'|'foreign'; ... };
//! ```

use crate::model_explore::{Limits, Report, explore, push_step};

/// What:
///  `enum Rules { ... }`: the rule under test (TS: a union of string literals).
/// Why:
///  The rule lives in the state so the explorer's function pointers need no captured variable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Rules = 'eight' | 'nine' | 'ten' | 'nine-unlinks-any';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 8: a lock of the dead process is removed by its exit handling; L4 repeats M1 from the journal.
    Eight,
    /// Revision 9: L4 unlinks `index.lock` when it is the journaled inode, finishes (7) when the index is the journaled inode,
    /// leaves any other inode alone, and otherwise repeats M1 from (1).
    Nine,
    /// Revision 10, rule N: a name shared with other programs is never removed on the strength of an inode comparison (inode
    /// numbers are reused and a pathname can be replaced between the comparison and the unlink); L4 reports the lock.
    Ten,
    /// Revision 9 with the identity test left out (a mutation that must stay red).
    NineUnlinksAny,
}

/// What:
///  `struct State { ... }`: the progress of M1 and what is on disk.
///  `step` counts M1's completed boundaries (0 to 7). `lock` is 0 no file, 1 our linked file, 2 a foreign file; `index` is 0
///  the old file, 1 our file, 2 a foreign replacement.
/// Why:
///  Flat `Copy` fields hash cheaply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = { rules: Rules; step: number; dead: boolean; lock: number; index: number; ... };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// How many boundaries of M1 are done.
    pub step: u8,
    /// The materializing process was killed.
    pub dead: bool,
    /// The state of `index.lock`.
    pub lock: u8,
    /// The state of the index file.
    pub index: u8,
    /// The identity of our unnamed file was journaled.
    pub journaled: bool,
    /// The marker `materializing` (the odd generation) is set.
    pub marker_odd: bool,
    /// The consumption entries of the installed paths were released.
    pub released: bool,
    /// A foreign program took `index.lock` once.
    pub foreign_took: bool,
    /// Recovery ran.
    pub recovered: bool,
    /// The file system gave a foreign `index.lock` the inode number of our dead unnamed file.
    pub reuse: bool,
    /// L4 reported a lock it did not remove.
    pub reported: bool,
    /// Harm: a foreign `index.lock` or index was removed or replaced by the wrapper.
    pub foreign_touched: bool,
}

/// The start state.
pub fn initial(rules: Rules) -> State {
    return State {
        rules,
        step: 0,
        dead: false,
        lock: 0,
        index: 0,
        journaled: false,
        marker_odd: false,
        released: false,
        foreign_took: false,
        recovered: false,
        reuse: false,
        reported: false,
        foreign_touched: false,
    };
}

/// What:
///  Every step enabled in `state`: M1's boundaries, the kill, the foreign program's, and recovery.
/// Why:
///  The explorer takes them in every order, so the kill falls at each boundary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function steps(s: State): [string, State][];
/// ```
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let alive: bool = !state.dead && state.step < 7;
    if alive {
        let mut after: State = state.clone();
        match state.step {
            0 => {
                after.step = 1;
                push_step(&mut found, "M1 (1): builds the new index in an unnamed file and fsyncs it", after);
            }
            1 => {
                after.step = 2;
                after.journaled = true;
                push_step(&mut found, "M1 (2): journals the identity of the file and the intent", after);
            }
            2 => {
                if state.lock == 0 {
                    after.step = 3;
                    after.lock = 1;
                    push_step(&mut found, "M1 (3): links the file to index.lock, the single try", after);
                } else {
                    after.step = 7;
                    push_step(&mut found, "M1 (3): the link fails because a foreign program holds index.lock, the entries are kept", after);
                }
            }
            3 => {
                after.step = 4;
                after.marker_odd = true;
                push_step(&mut found, "M1 (5): sets the marker materializing", after);
            }
            4 => {
                after.step = 5;
                after.lock = 0;
                after.index = 1;
                push_step(&mut found, "M1 (6): renames index.lock over the index", after);
            }
            5 => {
                after.step = 6;
                after.released = true;
                push_step(&mut found, "M1 (7): applies the per-path outcomes and releases the entries", after);
            }
            _ => {
                after.step = 7;
                after.marker_odd = false;
                push_step(&mut found, "M1: writes the generation even", after);
            }
        }
    }
    if alive && state.step >= 1 {
        let mut died: State = state.clone();
        died.dead = true;
        push_step(&mut found, "environment: the process is killed with SIGKILL, no exit handler runs", died);
    }
    if !state.foreign_took && state.lock == 0 {
        let mut after: State = state.clone();
        after.lock = 2;
        after.foreign_took = true;
        push_step(&mut found, "foreign: takes index.lock", after);
    }
    if state.lock == 2 {
        let mut after: State = state.clone();
        after.lock = 0;
        after.index = 2;
        push_step(&mut found, "foreign: replaces the index and releases index.lock", after);
    }
    if state.dead && !state.recovered {
        found.push(recovery(state));
    }
    return found;
}

/// What:
///  The step the next owner takes at L4 for a dead M1, under the rule.
/// Why:
///  This is where revision 8 (repeat from the journal) and revision 9 (identity rules) differ.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recovery(s: State): [string, State];
/// ```
fn recovery(state: &State) -> (String, State) {
    let mut after: State = state.clone();
    after.recovered = true;
    after.dead = false;
    after.step = 7;
    if state.rules == Rules::Ten {
        after.reported = state.lock != 0;
        if state.index == 1 {
            after.released = true;
        }
        after.marker_odd = false;
        return ("L4 (rule N): reports index.lock and removes nothing that another program may share".to_string(), after);
    }
    if state.rules == Rules::Eight {
        // The repeat's single try fails when our stranded file is at index.lock, and the file is taken for a foreign one.
        if state.index == 1 && !state.released {
            after.released = true;
        }
        after.marker_odd = false;
        return ("L4: repeats M1 from the journal".to_string(), after);
    }
    let remove_any: bool = state.rules == Rules::NineUnlinksAny && state.lock != 0;
    // The comparison is by inode number, which a foreign file can carry after the number was reused.
    let same_number: bool = state.lock == 1 || (state.lock == 2 && state.reuse);
    if same_number && state.journaled || remove_any {
        if state.lock == 2 {
            after.foreign_touched = true;
        }
        after.lock = 0;
    }
    if state.index == 1 {
        after.released = true;
    }
    after.marker_odd = false;
    return ("L4: unlinks index.lock only if it is the journaled inode, finishes (7) if the index is the journaled inode".to_string(), after);
}

/// The safety check: a foreign file is never touched.
pub fn invariant(state: &State) -> Option<String> {
    if state.foreign_touched {
        return Some("a foreign index.lock was removed by the wrapper".to_string());
    }
    return None;
}

/// The check at a state where only the environment can move: no stranded lock of ours, no installed index with unreleased entries.
pub fn end_state(state: &State) -> Option<String> {
    if state.lock == 1 && !state.reported {
        return Some("a stranded index.lock of our own file is left for good".to_string());
    }
    if state.index == 1 && !state.released {
        return Some("an installed index has unreleased consumption entries".to_string());
    }
    if state.marker_odd && !state.dead {
        return Some("the generation stays odd with no owner running".to_string());
    }
    return None;
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 10_000, max_depth: 40 };
}

/// Explores every schedule under `rules`.
pub fn initial_reuse(rules: Rules) -> State {
    let mut state: State = initial(rules);
    state.reuse = true;
    return state;
}

/// Explores every schedule under `rules`, also with a foreign lock that carries a reused inode number.
pub fn check(rules: Rules) -> Report {
    return explore(vec![initial(rules), initial_reuse(rules)], steps, invariant, end_state, limits());
}

/// Explores every schedule under `rules` where inode numbers are never reused (the assumption revision 9 made).
pub fn check_without_reuse(rules: Rules) -> Report {
    return explore(vec![initial(rules)], steps, invariant, end_state, limits());
}
