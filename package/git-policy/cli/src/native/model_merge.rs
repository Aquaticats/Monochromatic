//! Historical control, not approval evidence for revision 13: its edit-loss clause cannot fire (the edit flag is never cleared) and the pre-publication rollback it checks is retired by publish-first.
//!
//! What:
//!  Model of the revision 8 finding R8-8 for merges: a merge is checked out before it is published, then its hook refuses, or
//!  the swap is lost, while a person keeps editing a merged file.
//! Why:
//!  Revision 9 (MG1 to MG6, `doc/planning/cli-git-commits-never-fail.md`) restores the worktree on a refusal by running the
//!  carry-forward in reverse (MG6) and recomputes the merge on the new tip after a lost swap (MG5); revision 8 had neither.
//!  The model tracks one merged file's content, a person's edit made after the checkout, and the request's stage.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type State = { rules; stage; worktree: 'old'|'merge'|'retargeted'; personEdit; editInWorktree; editSaved; tipMoved };
//! ```

use crate::model_explore::{Limits, Report, explore, push_step};

/// What:
///  `enum Rules { ... }`: the rule under test (TS: a union of string literals).
/// Why:
///  The rule lives in the state so the explorer's function pointers need no captured variable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Rules = 'eight' | 'nine';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 8 draft: the merge is checked out, then swapped; a refusal ends the request and a lost swap leaves it pending.
    Eight,
    /// Revision 9: MG5 retargets after a lost swap and MG6 restores after a refusal.
    Nine,
}

/// What:
///  `struct State { ... }`: the stage, the worktree and the edit.
///  `stage`: 0 adopted, 1 merge-ready, 2 checked out, 3 published, 4 refused, 5 restored, 6 pending after a lost swap.
///  `worktree`: 0 the old tip's content, 1 the merge `M`, 2 the merge `M'` recomputed on the new tip.
/// Why:
///  Flat `Copy` fields hash cheaply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = { rules: Rules; stage: number; worktree: number; ... };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// How far the request got.
    pub stage: u8,
    /// The content the worktree holds for the merged path.
    pub worktree: u8,
    /// A person edited the file after the checkout.
    pub person_edit: bool,
    /// The edit is in the worktree file.
    pub edit_in_worktree: bool,
    /// The edit is kept in a saved file.
    pub edit_saved: bool,
    /// A foreign program moved the tip after the merge was computed.
    pub tip_moved: bool,
}

/// The start state.
pub fn initial(rules: Rules) -> State {
    return State { rules, stage: 0, worktree: 0, person_edit: false, edit_in_worktree: false, edit_saved: false, tip_moved: false };
}

/// What:
///  Every step enabled in `state`.
/// Why:
///  The explorer takes them in every order, so the edit and the tip move fall at every point.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function steps(s: State): [string, State][];
/// ```
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let nine: bool = state.rules == Rules::Nine;
    if state.stage == 0 {
        let mut after: State = state.clone();
        after.stage = 1;
        push_step(&mut found, "MG1 compute: computes the merge M", after);
    }
    if state.stage == 1 {
        let mut after: State = state.clone();
        after.stage = 2;
        after.worktree = 1;
        push_step(&mut found, "MG3 check out: installs M in the worktree under the marker", after);
    }
    if state.stage == 2 && !state.person_edit {
        let mut after: State = state.clone();
        after.person_edit = true;
        after.edit_in_worktree = true;
        push_step(&mut found, "person: saves an edit to the merged file after the checkout", after);
    }
    if state.stage >= 1 && state.stage <= 2 && !state.tip_moved {
        let mut after: State = state.clone();
        after.tip_moved = true;
        push_step(&mut found, "foreign: moves the tip", after);
    }
    if state.stage == 2 && !state.tip_moved {
        let mut after: State = state.clone();
        after.stage = 3;
        push_step(&mut found, "MG4 swap: the swap succeeds", after);
    }
    if state.stage == 2 && state.tip_moved {
        let mut after: State = state.clone();
        after.stage = 6;
        push_step(&mut found, "MG4 swap: the swap is lost because the tip moved, P4", after);
    }
    if state.stage == 2 {
        let mut after: State = state.clone();
        after.stage = 4;
        push_step(&mut found, "environment: a person's hook refuses the swap", after);
    }
    if nine && state.stage == 4 {
        let mut after: State = state.clone();
        after.stage = 5;
        after.worktree = 0;
        if state.person_edit {
            // The old tip's content is installed with M as the old side: the edit merges in, and the displaced inode is kept.
            after.edit_saved = true;
        }
        push_step(&mut found, "MG6 restore: installs the old tip's content with M as the old side and keeps every displaced inode", after);
    }
    if nine && state.stage == 6 {
        let mut after: State = state.clone();
        after.stage = 2;
        after.worktree = 2;
        after.tip_moved = false;
        push_step(&mut found, "MG5 retarget: recomputes M' on the new tip and installs the difference with M as the old side", after);
    }
    return found;
}

/// The safety checks: a refusal changes nothing, and a concurrent edit is never lost.
pub fn invariant(state: &State) -> Option<String> {
    if state.stage == 4 && state.rules == Rules::Eight {
        return Some("a refused merge left its checkout in the worktree".to_string());
    }
    if state.stage == 5 && state.worktree != 0 {
        return Some("a restored merge left the merge content in the worktree".to_string());
    }
    if state.person_edit && !state.edit_in_worktree && !state.edit_saved {
        return Some("a concurrent edit was lost".to_string());
    }
    return None;
}

/// The check where only the environment can move: no checkout is left for an old base with no retarget.
pub fn end_state(state: &State) -> Option<String> {
    if state.stage == 6 {
        return Some("a lost swap left the checkout of M for a base that is no longer the tip, with no retarget".to_string());
    }
    return None;
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 10_000, max_depth: 40 };
}

/// Explores every schedule under `rules`.
pub fn check(rules: Rules) -> Report {
    return explore(vec![initial(rules)], steps, invariant, end_state, limits());
}
