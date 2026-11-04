//! What:
//!  Two models of revision 13's late observers. First, `attach-watch` and MG6c: a native `git switch` that finishes after the merge's
//!  files were installed, with the measured span, the retained obligation, the leftover test and the restore that must never run over
//!  Git's own result. Second, T1's registry comparison around the connectivity check: a baseline listing `L0`, the probe of the next
//!  consecutive reservation number, a non-atomic second listing `L1`, and a restart after a crash.
//! Why:
//!  Revision 12 dropped the watch when a timer ran out and told the person a next check would come, and restored the pinned original
//!  over whatever the switch had written. It also restarted T1 at step (2) with the baseline lost in the crash, and compared two
//!  directory listings that a concurrent `link` can fall between. Revision 13 retains the obligation for the span and states the
//!  residue (a switch finishing later leaves the merge's files in the new branch's worktree), tests a leftover by index, file and the
//!  new tree's entry, restarts T1 from its Pre with a fresh `L0`, and probes the next number as an exact test.
//!  A timer is never evidence that a switch finished; the span only bounds how long the wrapper keeps watching.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type Watch = { rules; head; switch; file: 'R' | 'G' | 'N'; clock; watching; promisesNextCheck; gitResultOverwritten };
//! // type Retire = { rules; step; l0: number | undefined; reservations; deletedAfterCheck; released };
//! ```

use crate::model_explore::{Limits, Report, explore, push_step};

/// The span of the watch, in ticks (a model unit; the shipped constant `ATTACH_WATCH_SPAN` is measured, owner question 7b).
pub const SPAN: u8 = 2;

/// The owner's answer to question 7a: may the cleanup act on a by-value test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Authority {
    /// Option A: yes, with the index-unchanged test and the span.
    ByValueWithIndexTest,
    /// Option B: report the leftover and the command that restores it; restore nothing.
    AskFirst,
    /// Option C: no cleanup; the leftover is reported once.
    ReportOnly,
}

/// The owner's answer to question 7b, "A measured span" (the only option kept; B and C were rejected): how long the cleanup authority lasts.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Span {
    /// Option A: a measured span after the merge.
    Measured,
}

/// The owner's answer to question 7c: what happens after the watch ends.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Residue {
    /// Option A: accept the leftover and print one line at the merge.
    PrintOnce,
    /// Option B: keep a reminder record that the next command prints.
    Reminder,
}

/// The three answers, each a parameter.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Policy {
    /// Question 7a.
    pub authority: Authority,
    /// Question 7b.
    pub span: Span,
    /// Question 7c.
    pub residue: Residue,
}

/// The rule under test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 12.
    Twelve,
    /// Revision 13: the leftover test is by value (the file is still the installed one); no index-unchanged test.
    Thirteen,
    /// Revision 14, with the owner's answers to questions 7a, 7b and 7c as the parameter (they are not yet answered).
    Fourteen(Policy),
    /// Revision 15 with the owner's answers: 7a "Clean up automatically" (by value, with the index fingerprint test), 7b "A measured span"
    /// (the window starts at MG3's installation; at its end no new cleanup starts and one already started finishes), 7c "Keep a reminder"
    /// (dismissing deletes no saved copy). The violation this rule shows is the accepted concession of 7a, stated as such.
    Fifteen,
}

/// Where `HEAD` is.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Head {
    /// Attached to the merge's target.
    Target,
    /// Moved by a native switch.
    Elsewhere,
}

/// What a worktree file holds.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum File {
    /// The installed merge result `R`.
    Merged,
    /// What a native switch wrote (`G`, Git's own result).
    Git,
    /// The pinned original `N`.
    Original,
}

/// Where a switch is.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Switch {
    /// None running.
    None,
    /// Began before the install and has not finished.
    Paused,
    /// Finished.
    Done,
}

/// The watch.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Watch {
    /// The rule under test.
    pub rules: Rules,
    /// Where `HEAD` is.
    pub head: Head,
    /// The state of a native switch.
    pub switch: Switch,
    /// The file.
    pub file: File,
    /// The merge's install happened (the entries are `attach-watch`).
    pub installed: bool,
    /// Ticks since the install.
    pub clock: u8,
    /// The lane still holds the `attach-watch` obligation.
    pub watching: bool,
    /// The message told the person that a next check would come.
    pub promises_next_check: bool,
    /// MG6c restored the original over a file Git had written.
    pub git_result_overwritten: bool,
    /// A leftover remained after the obligation ended (the residue revision 13 states).
    pub residue: bool,
    /// A command wrote the index between the merge and the late switch (`git add`, a refresh).
    pub index_rewritten: bool,
    /// The person kept the merge's content staged on purpose: `reset --soft`, then a switch that carries it without rewriting the file.
    pub deliberate: bool,
    /// MG6c restored a state the person kept on purpose (ghost).
    pub undone_deliberate: bool,
    /// A reminder record is kept for the leftover (question 7c option B).
    pub reminder: bool,
    /// The leftover was reported to the person (question 7a options B and C).
    pub reported: bool,
    /// The index's fingerprint (size and checksum) differs from the one recorded at installation. A byte-identical rewrite, or a change
    /// away and back, leaves it equal while `index_rewritten` is true: the wrapper sees the fingerprint and never the write.
    pub fingerprint_changed: bool,
    /// MG6c holds the index lock and has begun to exchange the files.
    pub cleanup_started: bool,
    /// The saved copies of the merge's pre-state exist (installation wrote them; only the person's explicit disposal deletes them).
    pub saved_copy: bool,
    /// The person restored the files with the printed command.
    pub restored_by_person: bool,
    /// The person dismissed the reminder (a decision of the person's; the saved copies stay).
    pub dismissed: bool,
}

/// The start state: the merge is published.
pub fn watch_initial(rules: Rules) -> Watch {
    return Watch {
        rules,
        head: Head::Target,
        switch: Switch::None,
        file: File::Merged,
        installed: false,
        clock: 0,
        watching: false,
        promises_next_check: false,
        git_result_overwritten: false,
        residue: false,
        index_rewritten: false,
        deliberate: false,
        undone_deliberate: false,
        reminder: false,
        reported: false,
        fingerprint_changed: false,
        cleanup_started: false,
        saved_copy: true,
        restored_by_person: false,
        dismissed: false,
    };
}

fn policy_of(rules: Rules) -> Option<Policy> {
    return match rules {
        Rules::Fourteen(policy) => Some(policy),
        Rules::Fifteen => Some(Policy { authority: Authority::ByValueWithIndexTest, span: Span::Measured, residue: Residue::Reminder }),
        Rules::Twelve | Rules::Thirteen => None,
    };
}

/// Every step enabled in a watch state.
pub fn watch_steps(state: &Watch) -> Vec<(String, Watch)> {
    let mut found: Vec<(String, Watch)> = Vec::new();
    let policy: Option<Policy> = policy_of(state.rules);
    if !state.installed {
        let mut after: Watch = state.clone();
        after.installed = true;
        after.watching = true;
        // Revision 12 told the person that a next check would come; revision 13 states the residue.
        after.promises_next_check = state.rules == Rules::Twelve;
        push_step(&mut found, "owner: MG3 installs the merge's files and records attach-watch", after);
        if state.switch == Switch::None && state.head == Head::Target {
            let mut begun: Watch = state.clone();
            begun.switch = Switch::Paused;
            push_step(&mut found, "environment: a native switch begins before the install and pauses", begun);
        }
    }
    if state.installed && state.switch == Switch::Paused {
        let mut after: Watch = state.clone();
        after.switch = Switch::Done;
        after.head = Head::Elsewhere;
        push_step(&mut found, "environment: the paused switch completes; HEAD leaves the target and the merge's files stay", after);
    }
    if state.installed && state.switch == Switch::None && state.head == Head::Target {
        let mut after: Watch = state.clone();
        after.switch = Switch::Done;
        after.head = Head::Elsewhere;
        after.file = File::Git;
        push_step(&mut found, "environment: a switch that began after the install rewrites the file itself", after);
        let mut kept: Watch = state.clone();
        kept.switch = Switch::Done;
        kept.head = Head::Elsewhere;
        kept.deliberate = true;
        push_step(&mut found, "environment: reset --soft, then a switch that carries the staged merge content and leaves the file alone", kept);
    }
    if state.installed && !state.index_rewritten {
        let mut after: Watch = state.clone();
        after.index_rewritten = true;
        after.fingerprint_changed = true;
        push_step(&mut found, "environment: a command writes the index (git add or a refresh)", after);
        if state.rules == Rules::Fifteen {
            // The wrapper compares a fingerprint, so a rewrite with identical bytes, or a change away from the recorded bytes and back,
            // is a write it cannot see.
            let mut same: Watch = state.clone();
            same.index_rewritten = true;
            push_step(&mut found, "environment: a command rewrites the index with identical bytes, or changes it away and back", same);
        }
    }
    if state.watching && state.clock < SPAN + 1 {
        let mut after: Watch = state.clone();
        after.clock = state.clock + 1;
        push_step(&mut found, "timer: one tick of the span", after);
    }
    if state.rules == Rules::Fifteen {
        let by_value: bool = state.file == File::Merged;
        let fingerprint_equal: bool = !state.fingerprint_changed;
        if state.watching && state.head == Head::Elsewhere && !state.cleanup_started && state.clock <= SPAN && by_value && fingerprint_equal {
            let mut after: Watch = state.clone();
            after.cleanup_started = true;
            push_step(&mut found, "owner: MG6c takes index.lock and begins the cleanup", after);
        }
        if state.cleanup_started {
            // A cleanup already begun finishes, whatever the clock says; the files are exchanged back from the saved copies.
            let mut after: Watch = state.clone();
            after.git_result_overwritten = state.file == File::Git;
            after.undone_deliberate = state.deliberate;
            after.file = File::Original;
            after.cleanup_started = false;
            after.watching = false;
            push_step(&mut found, "owner: MG6c finishes the cleanup", after);
        }
        if state.watching && state.head == Head::Elsewhere && !state.cleanup_started && (state.clock > SPAN || !by_value || !fingerprint_equal) {
            // No new cleanup starts after the span, or when the by-value test or the fingerprint fails: the leftover is named and a
            // reminder record is kept (7c), and nothing is restored.
            let mut after: Watch = state.clone();
            after.watching = false;
            after.reported = true;
            after.reminder = by_value;
            push_step(&mut found, "owner: no cleanup starts; the leftover is named and a reminder is kept", after);
        }
        if state.reminder {
            let mut after: Watch = state.clone();
            after.reminder = false;
            after.dismissed = true;
            // Dismissing the notification disposes of the notification only; the saved copies stay.
            push_step(&mut found, "person: dismisses the reminder", after);
        }
        if state.saved_copy && state.head == Head::Elsewhere && state.file == File::Merged && !state.cleanup_started {
            let mut after: Watch = state.clone();
            after.file = File::Original;
            after.restored_by_person = true;
            after.reminder = false;
            push_step(&mut found, "person: runs the printed restore command", after);
        }
    } else if state.watching && state.head == Head::Elsewhere {
        let mut after: Watch = state.clone();
        let by_value: bool = state.file == File::Merged;
        let index_test: bool = match policy {
            Some(chosen) => chosen.authority == Authority::ByValueWithIndexTest && !state.index_rewritten,
            None => true,
        };
        let acts: bool = match state.rules {
            Rules::Twelve => true,
            Rules::Thirteen => by_value,
            Rules::Fourteen(_) | Rules::Fifteen => by_value && index_test,
        };
        if acts {
            after.git_result_overwritten = state.file == File::Git;
            after.undone_deliberate = state.deliberate;
            after.file = File::Original;
        } else if by_value {
            // Question 7a options B and C, or the index changed: the leftover is named, nothing is restored, and under the reminder answer
            // a reminder record is kept.
            after.reported = true;
            after.reminder = policy.is_some_and(|chosen| { return chosen.residue == Residue::Reminder; });
        }
        after.watching = false;
        push_step(&mut found, "owner: MG6c cleans up after the late switch", after);
    }
    // The end of the authority is observed, not assumed: a check that finds HEAD elsewhere dispatches the cleanup above and never ends
    // the watch silently. Only a check that finds HEAD still attached to the target may end it.
    let expires: bool = state.clock > SPAN;
    if state.watching && expires && state.head == Head::Target {
        let mut after: Watch = state.clone();
        after.watching = false;
        after.residue = state.switch == Switch::Paused;
        if after.residue && policy.is_some_and(|chosen| { return chosen.residue == Residue::Reminder; }) {
            after.reminder = true;
        }
        push_step(&mut found, "owner: the end of the authority and the check found HEAD still attached to the target", after);
    }
    if !state.watching && state.installed && state.switch == Switch::Paused && state.head == Head::Target {
        let mut after: Watch = state.clone();
        after.switch = Switch::Done;
        after.head = Head::Elsewhere;
        after.residue = true;
        push_step(&mut found, "environment: the paused switch completes after the watch ended", after);
    }
    return found;
}

/// Git's own result is never overwritten, a residue is never promised a next check, an unrecorded residue is not accepted under the
/// reminder answer, and (the property that depends on the answer to 7a and 7b) a state the person kept on purpose is not undone.
pub fn watch_invariant(state: &Watch) -> Option<String> {
    if state.git_result_overwritten {
        return Some("MG6c restored the original over the file a native switch had written".to_string());
    }
    if state.residue && state.promises_next_check {
        return Some("a leftover remained after the watch ended and the message promised a next check".to_string());
    }
    if state.undone_deliberate {
        return Some("MG6c undid a state the person kept on purpose".to_string());
    }
    // The obligation is derived from the modelled state (a leftover that may remain with no cleanup under way, no reminder and no
    // restoration), not from a flag the transitions maintain.
    let wants_reminder: bool = policy_of(state.rules).is_some_and(|chosen| { return chosen.residue == Residue::Reminder; });
    let possible_leftover: bool = state.head == Head::Elsewhere
        && state.file == File::Merged
        && !state.watching
        && !state.cleanup_started
        && !state.reminder
        && !state.restored_by_person
        && !state.dismissed
        && !state.deliberate;
    if wants_reminder && possible_leftover && (state.residue || state.reported) {
        return Some("a leftover remained after the watch ended and no reminder record is kept".to_string());
    }
    if state.rules == Rules::Fifteen && !state.saved_copy {
        return Some("the saved copies of the merge's pre-state are gone".to_string());
    }
    return None;
}

/// The invariant without the accepted concession of 7a: used to show that the concession is the only violation revision 15 has.
pub fn watch_invariant_except_concession(state: &Watch) -> Option<String> {
    if state.undone_deliberate {
        let mut relaxed: Watch = state.clone();
        relaxed.undone_deliberate = false;
        return watch_invariant(&relaxed);
    }
    return watch_invariant(state);
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 50_000, max_depth: 30 };
}

/// Explores the watch with the invariant that leaves out the accepted concession.
pub fn check_watch_except_concession(rules: Rules) -> Report {
    return explore(vec![watch_initial(rules)], watch_steps, watch_invariant_except_concession, |_| { return None; }, limits());
}

/// Explores the watch.
pub fn check_watch(rules: Rules) -> Report {
    return explore(vec![watch_initial(rules)], watch_steps, watch_invariant, |_| { return None; }, limits());
}

/// Where T1 is.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Step {
    /// Not begun.
    Fresh,
    /// The release intent is written and the baseline listing `L0` taken.
    Baselined,
    /// The connectivity check ran.
    Checked,
    /// The comparison passed and the spool was released.
    Released,
    /// The comparison failed and the intent was withdrawn.
    Withdrawn,
}

/// T1 against a maintenance run.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Retire {
    /// The rule under test.
    pub rules: Rules,
    /// T1's step.
    pub step: Step,
    /// The baseline: how many reservations `L0` held; none when lost in a crash.
    pub l0: Option<u8>,
    /// Reservations linked so far (0 or 1: one run is enough).
    pub reservations: u8,
    /// T1 crashed once between the check and the comparison.
    pub restarted: bool,
}

/// The start state.
pub fn retire_initial(rules: Rules) -> Retire {
    return Retire { rules, step: Step::Fresh, l0: None, reservations: 0, restarted: false };
}

/// Every step enabled in a retire state.
pub fn retire_steps(state: &Retire) -> Vec<(String, Retire)> {
    let mut found: Vec<(String, Retire)> = Vec::new();
    if state.reservations == 0 && !matches!(state.step, Step::Released | Step::Withdrawn) {
        let mut after: Retire = state.clone();
        after.reservations = 1;
        push_step(&mut found, "environment: a maintenance run is admitted and its reservation linked", after);
    }
    if state.step == Step::Fresh {
        let mut after: Retire = state.clone();
        after.step = Step::Baselined;
        after.l0 = Some(state.reservations);
        push_step(&mut found, "owner: T1 writes the intent and lists the registry as L0", after);
    }
    if state.step == Step::Baselined {
        let mut after: Retire = state.clone();
        after.step = Step::Checked;
        push_step(&mut found, "owner: T1 runs the connectivity check", after);
    }
    if state.step == Step::Checked && !state.restarted {
        let mut after: Retire = state.clone();
        after.restarted = true;
        if state.rules == Rules::Twelve {
            // Revision 12 repeated from the check with the baseline lost in the crash.
            after.l0 = None;
        } else {
            after.step = Step::Fresh;
            after.l0 = None;
        }
        push_step(&mut found, "environment: the owner dies after the check and restarts", after);
    }
    if state.step == Step::Checked && state.rules != Rules::Twelve {
        // The probe of the next number is an exact test; `L1` is read too and both are compared with the fresh baseline.
        let extra: bool = state.reservations > state.l0.unwrap_or(0);
        let mut after: Retire = state.clone();
        after.step = if extra { Step::Withdrawn } else { Step::Released };
        push_step(&mut found, "owner: T1 probes the next number, lists L1, compares with L0, and releases or withdraws", after);
    }
    if state.step == Step::Checked && state.rules == Rules::Twelve {
        // Revision 12 compared two directory listings; a baseline lost in a crash compares as equal.
        let extra: bool = state.l0.is_some_and(|baseline| { return state.reservations > baseline; });
        let mut sees: Retire = state.clone();
        sees.step = if extra { Step::Withdrawn } else { Step::Released };
        push_step(&mut found, "owner: T1 lists L1 and compares it with L0", sees);
        if state.reservations > 0 {
            let mut misses: Retire = state.clone();
            misses.step = Step::Released;
            push_step(&mut found, "owner: T1 lists L1 and a link that falls between the reads of the listing is missed", misses);
        }
    }
    return found;
}

/// The spool is released only when no run was admitted after the baseline and before the comparison.
pub fn retire_invariant(state: &Retire) -> Option<String> {
    if state.step == Step::Released && state.reservations > 0 && state.l0 != Some(1) {
        return Some("T1 released the spool although a run was admitted after its baseline".to_string());
    }
    return None;
}

/// Explores T1.
pub fn check_retire(rules: Rules) -> Report {
    return explore(vec![retire_initial(rules)], retire_steps, retire_invariant, |_| { return None; }, limits());
}
