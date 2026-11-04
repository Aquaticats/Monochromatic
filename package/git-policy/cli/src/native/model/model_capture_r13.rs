//! What:
//!  Three small models of what a capture and an install may take from the index and the worktree when another request, a deferred
//!  merge install or the person's later edit is in flight: capture against a withdrawn logical base (W1 step 4, W2 step 2, C1),
//!  the merge install that finds later staging (MG2 and MG3 with the publication-time entry `E`), and the capture of a file that a
//!  merge changed while the file still lags (the three-way merge of the original `O`, the worktree `W` and the merge result `M`).
//! Why:
//!  Revision 12 left a withdrawn request in the capture base until the lane applied the withdrawal, so a request that captured in
//!  between omitted a path that nobody would now commit; replaced a path the person staged during the install's lag with the merge
//!  result; and captured a lagging file whole, reverting the merge's change. Revision 13 makes W1 wait for W2 and C1 refuse a base that
//!  still holds a marked request, records `E` at publication so an entry that differs is left alone and reported as `later-staging`,
//!  and captures the three-way merge. Each model checks one local property under stated abstractions: the first three are single-transition lemmas (the document and the marks are read in one step, the line composition is stipulated, the staging ghost sees one overwrite); `Sampled` (revision 14) has C1's observations as separate steps; `Rules::Twelve` there is a deliberately broken variant, not a claim about what revision 12 executed.
//!  The person's own staging and edits are environment steps; the protocol's steps never assume them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type Base = { rules; r1InDocument; marked; w1Done; w2Done; yStaged; captured: number | undefined };
//! // type Install = { rules; entry: 'E' | 'Z' | 'M'; mg3Done; reported };
//! // type Lines = { rules; a: 'O' | 'M'; b: 'O' | 'P'; captured: [string, string] | undefined };
//! ```

use crate::model_explore::{Limits, Report, explore, push_step};

/// The rule under test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 12 (a broken variant of the mark rule: the mark is read once, with the base).
    Twelve,
    /// Revision 13 (C1 validates the document generation only).
    Thirteen,
    /// Revision 14 (C1 reads the marks twice: a mark that appeared between the samples repeats the capture).
    Fourteen,
}

/// The path `x` that the withdrawn request had consumed.
pub const X: u8 = 1;
/// The path `y` the person stages later.
pub const Y: u8 = 2;

/// Capture against a withdrawn logical base.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Base {
    /// The rule under test.
    pub rules: Rules,
    /// R1 is still in the lane's document (its consumption entry hides `x` from a capture).
    pub r1_in_document: bool,
    /// R1's withdrawal mark exists.
    pub marked: bool,
    /// The command of R1 exited.
    pub w1_done: bool,
    /// W2 applied the withdrawal.
    pub w2_done: bool,
    /// The person staged `y`.
    pub y_staged: bool,
    /// R2's capture, as a set of paths, once taken.
    pub captured: Option<u8>,
    /// Ghost: R1's command had already exited when R2 captured (a capture before the mark is re-based by W2 and loses nothing).
    pub captured_after_exit: bool,
}

/// The start state: R1 is pending with `x` consumed, the index holds `x`.
pub fn base_initial(rules: Rules) -> Base {
    return Base { rules, r1_in_document: true, marked: false, w1_done: false, w2_done: false, y_staged: false, captured: None, captured_after_exit: false };
}

/// Every step enabled in a base state.
pub fn base_steps(state: &Base) -> Vec<(String, Base)> {
    let mut found: Vec<(String, Base)> = Vec::new();
    if !state.marked {
        let mut after: Base = state.clone();
        after.marked = true;
        push_step(&mut found, "W1: creates the withdrawal mark of R1", after);
    }
    if state.marked && !state.w1_done && (state.rules == Rules::Twelve || state.w2_done) {
        let mut after: Base = state.clone();
        after.w1_done = true;
        push_step(&mut found, "W1: R1's command exits", after);
    }
    if state.marked && !state.w1_done && state.rules == Rules::Thirteen && !state.w2_done {
        let mut after: Base = state.clone();
        after.w1_done = true;
        push_step(&mut found, "W1: WAIT_WITHDRAW passed; R1's command exits and C1 keeps refusing", after);
    }
    if state.marked && state.r1_in_document {
        let mut after: Base = state.clone();
        after.r1_in_document = false;
        after.w2_done = true;
        push_step(&mut found, "owner: W2 applies the withdrawal and re-bases the descendants", after);
    }
    if !state.y_staged {
        let mut after: Base = state.clone();
        after.y_staged = true;
        push_step(&mut found, "environment: the person stages y", after);
    }
    let refuses: bool = state.rules == Rules::Thirteen && state.marked && state.r1_in_document;
    if state.y_staged && state.captured.is_none() && !refuses {
        let mut after: Base = state.clone();
        let hidden_x: bool = state.r1_in_document;
        after.captured = Some(if hidden_x { Y } else { X | Y });
        after.captured_after_exit = state.w1_done;
        push_step(&mut found, "client: C1 captures for R2", after);
    }
    return found;
}

/// A capture taken after R1's command left must hold `x`, which no request will now commit.
pub fn base_invariant(state: &Base) -> Option<String> {
    if let Some(paths) = state.captured
        && state.captured_after_exit
        && paths & X == 0
    {
        return Some("R2's capture omits x although R1 was withdrawn and its command returned".to_string());
    }
    return None;
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 10_000, max_depth: 20 };
}

/// Explores the base model.
pub fn check_base(rules: Rules) -> Report {
    return explore(vec![base_initial(rules)], base_steps, base_invariant, |_| { return None; }, limits());
}

/// The entry of a path in the index.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Entry {
    /// The entry the merge's publication recorded (`E`, the staged version at that time).
    Published,
    /// The person staged something else during the install's lag (`z`).
    Later,
    /// The merge result (`M`).
    Merged,
}

/// The deferred merge install.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Install {
    /// The rule under test.
    pub rules: Rules,
    /// The path's index entry.
    pub entry: Entry,
    /// MG3 ran.
    pub mg3_done: bool,
    /// MG3 reported `later-staging`.
    pub reported: bool,
}

/// The start state: the merge published; the install is deferred.
pub fn install_initial(rules: Rules) -> Install {
    return Install { rules, entry: Entry::Published, mg3_done: false, reported: false };
}

/// Every step enabled in an install state.
pub fn install_steps(state: &Install) -> Vec<(String, Install)> {
    let mut found: Vec<(String, Install)> = Vec::new();
    if state.entry == Entry::Published {
        let mut after: Install = state.clone();
        after.entry = Entry::Later;
        push_step(&mut found, "environment: the person stages z during the lag", after);
    }
    if !state.mg3_done {
        let mut after: Install = state.clone();
        after.mg3_done = true;
        if state.rules == Rules::Twelve || state.entry == Entry::Published {
            after.entry = Entry::Merged;
        } else {
            after.reported = true;
        }
        push_step(&mut found, "owner: MG3 runs the install", after);
    }
    return found;
}

/// The person's later staging is never replaced, and an entry left alone is reported.
pub fn install_invariant(state: &Install) -> Option<String> {
    if state.mg3_done && state.entry == Entry::Merged && state.reported {
        return Some("an entry was both replaced and reported as left".to_string());
    }
    if state.mg3_done && state.entry == Entry::Later && !state.reported {
        return Some("an entry the person staged was left and not reported".to_string());
    }
    return None;
}

/// A staged path replaced check that needs the ghost knowledge of whether the person staged: the entry went through `Later`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Staging {
    /// The install state.
    pub install: Install,
    /// The person staged `z` at some point (ghost).
    pub person_staged: bool,
}

/// Steps of the install model with the ghost.
pub fn staging_steps(state: &Staging) -> Vec<(String, Staging)> {
    let mut found: Vec<(String, Staging)> = Vec::new();
    for (label, install) in install_steps(&state.install) {
        let staged: bool = state.person_staged || label.contains("stages z");
        found.push((label, Staging { install, person_staged: staged }));
    }
    return found;
}

/// The person's staging survives the install in every schedule.
pub fn staging_invariant(state: &Staging) -> Option<String> {
    if state.person_staged && state.install.mg3_done && state.install.entry == Entry::Merged {
        return Some("the merge result replaced the version the person staged during the lag".to_string());
    }
    return install_invariant(&state.install);
}

/// Explores the install model.
pub fn check_install(rules: Rules) -> Report {
    return explore(vec![Staging { install: install_initial(rules), person_staged: false }], staging_steps, staging_invariant, |_| { return None; }, limits());
}

/// A file with two lines the merge or the person may change.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Lines {
    /// The rule under test.
    pub rules: Rules,
    /// The merge changed line a in the index and the worktree file still lags (line a is `O` there).
    pub person_edited_b: bool,
    /// The capture, once taken: (line a, line b) as captured; `a` true means the merge's version, `b` true means the person's edit.
    pub captured: Option<(bool, bool)>,
}

/// The start state.
pub fn lines_initial(rules: Rules) -> Lines {
    return Lines { rules, person_edited_b: false, captured: None };
}

/// Every step enabled in a lines state.
pub fn lines_steps(state: &Lines) -> Vec<(String, Lines)> {
    let mut found: Vec<(String, Lines)> = Vec::new();
    if !state.person_edited_b && state.captured.is_none() {
        let mut after: Lines = state.clone();
        after.person_edited_b = true;
        push_step(&mut found, "environment: the person edits line b of the lagging file", after);
    }
    if state.captured.is_none() {
        let mut after: Lines = state.clone();
        after.captured = Some(match state.rules {
            // Revision 12: the whole worktree file, which still has `O` on line a.
            Rules::Twelve => (false, state.person_edited_b),
            // Revision 13: the three-way merge of O, W and M.
            Rules::Thirteen | Rules::Fourteen => (true, state.person_edited_b),
        });
        push_step(&mut found, "client: commit -a captures the file", after);
    }
    return found;
}

/// The capture holds the merge's change on line a and the person's edit on line b when there was one.
pub fn lines_invariant(state: &Lines) -> Option<String> {
    if let Some((merge_kept, edit_kept)) = state.captured {
        if !merge_kept {
            return Some("the full-file capture reverted the merge's change to line a".to_string());
        }
        if edit_kept != state.person_edited_b {
            return Some("the capture lost the person's edit to line b".to_string());
        }
    }
    return None;
}

/// Explores the lines model.
pub fn check_lines(rules: Rules) -> Report {
    return explore(vec![lines_initial(rules)], lines_steps, lines_invariant, |_| { return None; }, limits());
}

/// F9 reopened against the state log: a request consumes staging, is refused, and a plain commit captures before any later cleanup.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Consumption {
    /// The rule under test.
    pub rules: Rules,
    /// The lane's document still holds R1's consumption entry for `x`.
    pub consumes_x: bool,
    /// R1 ended as refused (the person's `pre-commit` style refusal or a hook refusal).
    pub refused: bool,
    /// A plain commit captured; its value is whether the capture held `x`.
    pub captured_with_x: Option<bool>,
    /// Ghost: the capture happened after the refusal (a capture before it is rightly without `x`).
    pub captured_after_refusal: bool,
}

/// The start state: R1 holds `x` in its consumption entry and is pending.
pub fn consumption_initial(rules: Rules) -> Consumption {
    return Consumption { rules, consumes_x: true, refused: false, captured_with_x: None, captured_after_refusal: false };
}

/// Every step enabled in a consumption state.
pub fn consumption_steps(state: &Consumption) -> Vec<(String, Consumption)> {
    let mut found: Vec<(String, Consumption)> = Vec::new();
    if !state.refused {
        let mut after: Consumption = state.clone();
        after.refused = true;
        if state.rules == Rules::Thirteen {
            // One state-log record holds the whole document: the refusal and the dropped consumption are one append.
            after.consumes_x = false;
        }
        push_step(&mut found, "owner: R1 is refused and the document is replaced", after);
    }
    if state.refused && state.consumes_x {
        let mut after: Consumption = state.clone();
        after.consumes_x = false;
        push_step(&mut found, "owner: a later cleanup drops R1's consumption entry", after);
    }
    if state.captured_with_x.is_none() {
        let mut after: Consumption = state.clone();
        after.captured_with_x = Some(!state.consumes_x);
        after.captured_after_refusal = state.refused;
        push_step(&mut found, "client: a plain commit captures", after);
    }
    return found;
}

/// A capture taken after the refusal holds the staging the refused request had consumed.
pub fn consumption_invariant(state: &Consumption) -> Option<String> {
    if state.captured_with_x == Some(false) && state.captured_after_refusal {
        return Some("a capture after the refusal omitted the staging the refused request had consumed".to_string());
    }
    return None;
}

/// Explores the consumption model.
pub fn check_consumption(rules: Rules) -> Report {
    return explore(vec![consumption_initial(rules)], consumption_steps, consumption_invariant, |_| { return None; }, limits());
}

/// What C1 has observed so far (revision 14, R13-7): the document and the marks first, the index second, then the validation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Phase {
    /// Nothing observed.
    Idle,
    /// Sample one taken: whether the base held R1 and whether R1's mark existed.
    Sampled { base_has_r1: bool, mark_seen: bool },
    /// The index was read too; the delta omits `x` when the base held R1.
    Read { base_has_r1: bool, mark_seen: bool },
    /// The capture was accepted with the paths it holds.
    Accepted { holds_x: bool },
}

/// C1 with its constituent observations.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Sampled {
    /// The rule under test.
    pub rules: Rules,
    /// R1 is in the lane's document (the generation `g` is unchanged until W2 removes it).
    pub r1_in_document: bool,
    /// R1's withdrawal mark exists.
    pub marked: bool,
    /// R1's command returned (after `WAIT_WITHDRAW` expired, without W2).
    pub w1_returned: bool,
    /// The person staged `y` (the index changes without the document's generation changing).
    pub y_staged: bool,
    /// C1's progress.
    pub phase: Phase,
    /// Ghost: the capture was accepted after R1's command had returned.
    pub accepted_after_return: bool,
}

/// The start state: R1 pending with `x` consumed; the index holds `x`.
pub fn sampled_initial(rules: Rules) -> Sampled {
    return Sampled { rules, r1_in_document: true, marked: false, w1_returned: false, y_staged: false, phase: Phase::Idle, accepted_after_return: false };
}

/// Every step enabled.
pub fn sampled_steps(state: &Sampled) -> Vec<(String, Sampled)> {
    let mut found: Vec<(String, Sampled)> = Vec::new();
    if !state.marked {
        let mut after: Sampled = state.clone();
        after.marked = true;
        push_step(&mut found, "W1: creates the withdrawal mark of R1", after);
    }
    if state.marked && !state.w1_returned {
        let mut after: Sampled = state.clone();
        after.w1_returned = true;
        push_step(&mut found, "W1: WAIT_WITHDRAW expires and R1's command returns without W2", after);
    }
    if state.marked && state.r1_in_document {
        let mut after: Sampled = state.clone();
        after.r1_in_document = false;
        push_step(&mut found, "owner: W2 removes R1 from the document", after);
    }
    if !state.y_staged {
        let mut after: Sampled = state.clone();
        after.y_staged = true;
        push_step(&mut found, "environment: the person stages y", after);
    }
    match state.phase {
        Phase::Idle => {
            if state.rules != Rules::Twelve && state.marked && state.r1_in_document {
                // C1 refuses a base that still holds a marked request and waits for W2 (revisions 13 and 14).
                push_step(&mut found, "client: C1 refuses a base that still holds a marked request", state.clone());
            } else {
                let mut after: Sampled = state.clone();
                after.phase = Phase::Sampled { base_has_r1: state.r1_in_document, mark_seen: state.marked };
                push_step(&mut found, "client: C1 reads the document generation and the marks (sample one)", after);
            }
        }
        Phase::Sampled { base_has_r1, mark_seen } => {
            let mut after: Sampled = state.clone();
            after.phase = Phase::Read { base_has_r1, mark_seen };
            push_step(&mut found, "client: C1 reads the index", after);
        }
        Phase::Read { base_has_r1, mark_seen } => {
            // The generation is unchanged while W2 has not run, whatever happened to the marks and the index.
            let generation_unchanged: bool = state.r1_in_document == base_has_r1;
            if generation_unchanged {
                // Revision 14's second sample of the marks; revisions 12 and 13 validate the generation only.
                let mark_appeared: bool = state.marked && !mark_seen;
                if state.rules != Rules::Fourteen || !mark_appeared {
                    let mut after: Sampled = state.clone();
                    after.phase = Phase::Accepted { holds_x: !base_has_r1 };
                    after.accepted_after_return = state.w1_returned;
                    push_step(&mut found, "client: C1 validates the generation and accepts", after);
                } else {
                    let mut after: Sampled = state.clone();
                    after.phase = Phase::Idle;
                    push_step(&mut found, "client: C1 samples the marks again, sees a mark that appeared, and repeats the capture", after);
                }
            } else {
                let mut after: Sampled = state.clone();
                after.phase = Phase::Idle;
                push_step(&mut found, "client: C1 sees the generation change and repeats the capture", after);
            }
        }
        Phase::Accepted { .. } => {}
    }
    return found;
}

/// An accepted capture taken after R1's command returned, against a base that held R1, omits the path nobody else will now commit.
pub fn sampled_invariant(state: &Sampled) -> Option<String> {
    if let Phase::Accepted { holds_x: false } = state.phase
        && state.marked
        && state.accepted_after_return
    {
        return Some("R2's accepted capture omits x although R1 was withdrawn and its command had returned".to_string());
    }
    return None;
}

/// Explores C1's observations.
pub fn check_sampled(rules: Rules) -> Report {
    return explore(vec![sampled_initial(rules)], sampled_steps, sampled_invariant, |_| { return None; }, limits());
}
