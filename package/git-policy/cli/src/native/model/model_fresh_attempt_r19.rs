//! What:
//!  Model of decision 19: after a power loss that leaves no decisive positive authority for a request whose command had already returned
//!  success, recovery makes a fresh attempt and stores an outcome-specific notice that the next wrapper command prints. The facts are the
//!  branch ref write, the client's receipt and the `commit: ok` line, each visible and durable apart (`model_observe::World`); the notice
//!  is a file with a visible and a durable content; power loss, write-back, the attempt's outcomes and the person's next command are steps.
//! Why:
//!  The earlier row for question 8 was a stale owner-question matrix. The owner chose option A, so what remains is its behaviour with the
//!  actual observations and no atomic recovery: a fresh attempt only without durable positive authority; the notice durable before the
//!  attempt starts, so another crash still leaves a notice; the notice follows the attempt's outcome; it is printed on the next command and
//!  is not discarded before the outcome has been delivered. Each clause has a single-defect control. The accepted risk (a deliberate removal
//!  can be published again, and a second crash can repeat the ambiguity) is a reachable state here, not a violation.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // recover(durable) = positive(durable) ? settled : (writeNotice('scheduled'), force(notice), freshAttempt())
//! ```

use crate::model_explore::{Limits, Report, explore, push_step};
use crate::model_observe::{World, make_visible, power_loss, sync};

/// The branch ref holds the commit.
pub const REF: u32 = 1;
/// `lane/receipt/<id>` exists.
pub const RECEIPT: u32 = 2;
/// `commit: ok` was recorded on the attempt's `out`.
pub const OUT_OK: u32 = 4;
/// Every fact that is positive authority that the request landed.
pub const POSITIVE: u32 = REF | RECEIPT | OUT_OK;

/// How many power losses the search may inject.
pub const MAX_LOSSES: u8 = 2;

/// The rule under test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Decision 19 as the owner approved it.
    Nineteen,
    /// Control: recovery makes a fresh attempt even when durable positive authority survives.
    FreshDespitePositive,
    /// Control: the notice is written only after the attempt, so a crash during the attempt leaves a published commit and no notice.
    NoticeAfterAttempt,
    /// Control: the notice keeps saying that an attempt is scheduled after the attempt has ended.
    StaleNotice,
    /// Control: printing the notice discards it, even while the attempt may still run again.
    DiscardedOnFirstPrint,
    /// Control: the request's recovery data is reclaimed before the outcome notice has been delivered.
    ReclaimBeforeDelivery,
}

/// What the stored notice says.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Notice {
    /// No notice file.
    None,
    /// The attempt is scheduled or running.
    Scheduled,
    /// Published, with the identifier of the commit actually made and the command to drop it.
    Published,
    /// Conflicted, refused or signing failed, with the retained inputs and what to do next.
    Failed,
}

/// Where the request is.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Phase {
    /// The command returned success; nothing has been lost yet.
    Returned,
    /// Power was lost; recovery has not run.
    Down,
    /// Recovery found durable positive authority: no attempt, no notice.
    Settled,
    /// Recovery chose a fresh attempt and is writing the notice.
    Scheduling,
    /// The notice is stored; the attempt starts next.
    Ready,
    /// The fresh attempt runs.
    Running,
    /// The attempt ended with this outcome (`true` published, `false` failed).
    Ended(bool),
    /// The request's recovery data was reclaimed.
    Reclaimed,
}

/// The model's state.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// The facts of the request's landing.
    pub world: World,
    /// The notice file's visible content.
    pub notice_v: Notice,
    /// The notice file's durable content.
    pub notice_d: Notice,
    /// Where the request is.
    pub phase: Phase,
    /// Power losses so far.
    pub losses: u8,
    /// Commits made for the request (the first one counts).
    pub commits: u8,
    /// Recovery found durable positive authority at its last run.
    pub positive_at_recovery: bool,
    /// The last content a wrapper command printed.
    pub printed: Notice,
    /// The notice for the final outcome was delivered (printed and removed).
    pub delivered: bool,
}

/// The start state: the command returned success; the landing facts are visible and not yet durable.
pub fn initial(rules: Rules) -> State {
    let world: World = make_visible(make_visible(make_visible(World { visible: 0, durable: 0 }, REF), RECEIPT), OUT_OK);
    return State {
        rules,
        world,
        notice_v: Notice::None,
        notice_d: Notice::None,
        phase: Phase::Returned,
        losses: 0,
        commits: 1,
        positive_at_recovery: false,
        printed: Notice::None,
        delivered: false,
    };
}

/// Every step enabled in `state`.
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    for (fact, name) in [(REF, "the branch ref"), (RECEIPT, "the receipt"), (OUT_OK, "commit: ok")] {
        if state.world.visible & fact != 0 && state.world.durable & fact == 0 {
            let mut after: State = state.clone();
            after.world = sync(after.world, fact);
            push_step(&mut found, &format!("environment: write-back persists {}", name), after);
        }
    }
    if state.notice_v != state.notice_d {
        let mut after: State = state.clone();
        after.notice_d = state.notice_v;
        push_step(&mut found, "environment: write-back persists the notice", after);
    }
    if state.losses < MAX_LOSSES && !matches!(state.phase, Phase::Down | Phase::Reclaimed) {
        let mut after: State = state.clone();
        after.world = power_loss(after.world);
        after.notice_v = after.notice_d;
        after.phase = Phase::Down;
        after.losses += 1;
        push_step(&mut found, "environment: power is lost", after);
    }
    match state.phase {
        Phase::Down => {
            let positive: bool = state.world.durable & POSITIVE != 0;
            let mut after: State = state.clone();
            after.positive_at_recovery = positive;
            if positive && state.rules != Rules::FreshDespitePositive {
                after.phase = Phase::Settled;
                push_step(&mut found, "owner: recovery sees durable positive authority and settles the request", after);
            } else {
                after.phase = Phase::Scheduling;
                push_step(&mut found, "owner: recovery sees no decisive positive authority and chooses a fresh attempt", after);
            }
        }
        Phase::Scheduling => {
            let mut after: State = state.clone();
            if state.rules == Rules::NoticeAfterAttempt {
                after.phase = Phase::Ready;
                push_step(&mut found, "owner: recovery goes straight to the attempt", after);
            } else if state.notice_v != Notice::Scheduled {
                after.notice_v = Notice::Scheduled;
                push_step(&mut found, "owner: recovery writes the notice that an attempt is scheduled", after);
            } else if state.notice_d == Notice::Scheduled {
                after.phase = Phase::Ready;
                push_step(&mut found, "owner: recovery has forced the notice and proceeds", after);
            } else {
                after.notice_d = Notice::Scheduled;
                push_step(&mut found, "owner: recovery forces the notice", after);
            }
        }
        Phase::Ready => {
            let mut after: State = state.clone();
            after.phase = Phase::Running;
            push_step(&mut found, "owner: the fresh attempt starts", after);
        }
        Phase::Running => {
            let mut published: State = state.clone();
            published.commits += 1;
            published.world = make_visible(make_visible(make_visible(published.world, REF), RECEIPT), OUT_OK);
            published.phase = Phase::Ended(true);
            push_step(&mut found, "owner: the attempt publishes a commit", published);
            let mut failed: State = state.clone();
            failed.phase = Phase::Ended(false);
            push_step(&mut found, "owner: the attempt ends conflicted, refused or signing failed", failed);
        }
        Phase::Ended(published) => {
            let content: Notice = if published { Notice::Published } else { Notice::Failed };
            if state.rules != Rules::StaleNotice && state.notice_v != content {
                let mut after: State = state.clone();
                after.notice_v = content;
                push_step(&mut found, "owner: the notice is updated to the attempt's outcome", after);
            } else if state.rules != Rules::StaleNotice && state.notice_d != content {
                let mut after: State = state.clone();
                after.notice_d = content;
                push_step(&mut found, "owner: the outcome notice is forced", after);
            }
            let delivered_or_early: bool = state.delivered || state.rules == Rules::ReclaimBeforeDelivery;
            if delivered_or_early {
                let mut after: State = state.clone();
                after.phase = Phase::Reclaimed;
                push_step(&mut found, "owner: the request's recovery data is reclaimed", after);
            }
        }
        Phase::Returned | Phase::Settled | Phase::Reclaimed => {}
    }
    // The person's next wrapper command in the worktree prints the stored notice. A notice of the final outcome is delivered by that print
    // (removed, and the removal forced); a notice that an attempt is scheduled stays until the outcome replaces it.
    if state.notice_v != Notice::None && state.printed != state.notice_v || (state.notice_v != Notice::None && matches!(state.phase, Phase::Ended(_))) {
        let mut after: State = state.clone();
        after.printed = state.notice_v;
        let final_outcome: bool = matches!(state.notice_v, Notice::Published | Notice::Failed);
        if final_outcome || state.rules == Rules::DiscardedOnFirstPrint {
            after.notice_v = Notice::None;
            after.notice_d = Notice::None;
            after.delivered = final_outcome;
        }
        push_step(&mut found, "environment: the next wrapper command prints the notice", after);
    }
    return found;
}

/// The properties.
pub fn invariant(state: &State) -> Option<String> {
    if state.phase == Phase::Running && state.positive_at_recovery {
        return Some("a fresh attempt ran although durable positive authority survived".to_string());
    }
    if state.phase == Phase::Running && state.notice_d != Notice::Scheduled {
        return Some("a fresh attempt runs and no durable notice names it".to_string());
    }
    if state.phase == Phase::Reclaimed && !state.delivered {
        return Some("the request's recovery data was reclaimed before the outcome notice was delivered".to_string());
    }
    return None;
}

/// A quiescent state is acceptable only if the notice says what the attempt did.
pub fn terminal(state: &State) -> Option<String> {
    if let Phase::Ended(published) = state.phase {
        let wanted: Notice = if published { Notice::Published } else { Notice::Failed };
        if state.notice_v == Notice::Scheduled || (state.notice_v != wanted && state.notice_v != Notice::None) {
            return Some("the stored notice does not say what the attempt did".to_string());
        }
    }
    return None;
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 200_000, max_depth: 60 };
}

/// Explores every schedule under `rules`.
pub fn check(rules: Rules) -> Report {
    return explore(vec![initial(rules)], steps, invariant, terminal, limits());
}
