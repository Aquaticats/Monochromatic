//! The analysis indicator's state machine:
//!  where it is in its life,
//!  which view goes to the bus and when,
//!  and what each input ends.
//!
//! What:
//! `Machine` holds the phase,
//!  the pause flag,
//!  the last sent view and the view the rate limit held back,
//!  and every input
//! (a sweep report,
//!  a tick,
//!  a pause request,
//!  a dismissal,
//!  a failure or a dead bus) returns the effects it causes.
//! Why:
//! deciding and carrying out are separate:
//!  this file decides,
//!  the D-Bus layer carries the effects out,
//!  and a phase machine that only reads views and plain `Instant` values is testable
//! without a clock,
//! a thread or a bus.
//! `analysis_notification_logic.rs` keeps the view and effect vocabulary this speaks.

/// What:
///      `use std::time::Instant;`.
///  A monotonic point in time,
///  with no calendar meaning,
///  compared only with other `Instant`s.
/// Why:
///       Every method takes the current time as an argument,
///  so the rate limit is decided by
///  arithmetic instead of a clock read.
///       The siblings are `SystemTime`,
///  the wall clock that can jump both ways,
///  and `Duration`,
///  a span rather than a point.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // performance.now() sampled by the caller and passed in as a number
/// ```
use std::time::Instant;

/// What:
///      `use super::{COMPLETE_PERCENT, Effect, IndicatorView, MIN_UPDATE_INTERVAL, Outcome, PERCENT_WHILE_RUNNING_MAX};`.
///  The vocabulary the machine answers in,
///  from the module that declares it.
/// Why:
///       One state machine speaks one set of types;
///  naming them from the owner module keeps this file
///  about the transitions alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Effect, IndicatorView, Outcome, MIN_UPDATE_INTERVAL } from "./analysis_notification_logic";
/// ```
use super::{
    COMPLETE_PERCENT,
    Effect,
    IndicatorView,
    MIN_UPDATE_INTERVAL,
    Outcome,
    PERCENT_WHILE_RUNNING_MAX,
};
/// What:
///      `use crate::scan_indicator::scan_indicator_count_text;`.
///  The app's one digit-grouping formatter for a count.
/// Why:
///       The count line under the bar is spelled exactly as the former in-app banner spelled it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { scanIndicatorCountText } from "./scan_indicator";
/// ```
use crate::scan_indicator::scan_indicator_count_text;
/// What:
/// `Phase` is where the indicator is in its life.
/// Why:
/// the same report means different things before the open,
/// while open,
/// after a dismissal and after a failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    /// No indicator exists;
    /// the next unfinished report opens one.
    Idle,
    /// The indicator is open.
    Active,
    /// The user dismissed the indicator;
    /// nothing is shown until this sweep ends.
    Dismissed,
    /// The bus failed or no service answered;
    /// nothing is attempted for the rest of the run.
    Disabled,
}

/// What:
/// `Sent` is the last view that went to the bus and when.
/// Why:
/// the dedupe compares against the view and the rate limit measures from the time.
#[derive(Clone, Debug)]
struct Sent {
    /// The view that went to the bus.
    view: IndicatorView,
    /// When that view was sent,
    /// which the rate limit measures from.
    at: Instant,
}

/// What:
/// `Machine` is the indicator's state machine.
/// Why:
/// one value holds the phase,
/// the pause flag,
/// the last sent view and the held-back view,
/// and every input
/// returns the effects it causes.
#[derive(Debug)]
pub struct Machine {
    /// Where the indicator is in its life:
    /// idle,
    /// active,
    /// dismissed or disabled.
    phase: Phase,
    /// Whether the app has a pause to offer;
    /// without one the machine never asks for a pause.
    suspendable: bool,
    /// Whether the analysis is paused now.
    paused: bool,
    /// The last view that went to the bus,
    /// which the dedupe and the rate limit compare against.
    sent: Option<Sent>,
    /// The newest view that the rate limit held back,
    /// sent when the interval has passed.
    pending: Option<IndicatorView>,
}

/// What:
/// `to_count` converts a track count to the signed width the shared count formatter takes.
/// Why:
/// a count above the signed range is clamped rather than wrapped,
/// as the former banner did.
fn to_count(value: usize) -> i32 {
    return i32::try_from(value).unwrap_or(i32::MAX);
}

/// What:
/// `count_text` formats `done` and `total` as `412 of 1,218` with the app's grouping helper.
/// Why:
/// the shared helper keeps the grouping rule in one place.
pub fn count_text(
    done: usize,
    total: usize,
) -> String {
    let done_text = scan_indicator_count_text(to_count(done));
    let total_text = scan_indicator_count_text(to_count(total));
    return format!("{done_text} of {total_text}");
}

/// What:
/// `percent_for` is the whole percent of `done` in `total`,
/// rounded down and never 100 while work remains.
/// Why:
/// rounding down never overstates progress,
/// and the wide multiply cannot overflow on any count.
pub fn percent_for(
    done: usize,
    total: usize,
) -> u32 {
    if total == 0 {
        return 0;
    }
    let scaled = (done as u128).saturating_mul(100) / (total as u128);
    let capped = u32::try_from(scaled).unwrap_or(PERCENT_WHILE_RUNNING_MAX);
    return capped.min(PERCENT_WHILE_RUNNING_MAX);
}

/// What:
/// `view_for` builds the running view for a report.
/// Why:
/// the percent and the text derive from the same two counts,
/// and the paused flag comes from the machine.
fn view_for(
    done: usize,
    total: usize,
    paused: bool,
) -> IndicatorView {
    return IndicatorView {
        percent: percent_for(done, total),
        text: count_text(done, total),
        paused,
    };
}

/// What:
/// `final_view_for` builds the last view of a sweep:
/// full bar and `total of total`.
/// Why:
/// the user sees the bar complete before it goes away,
/// and an update that skipped the last reports still lands.
fn final_view_for(total: usize) -> IndicatorView {
    return IndicatorView {
        percent: COMPLETE_PERCENT,
        text: count_text(total, total),
        paused: false,
    };
}

/// Methods of `Machine`.
impl Machine {
    /// What:
    /// `new` builds an idle machine.
    /// Why:
    /// `suspendable` is false when the app has no pause to offer,
    /// so stray pause requests are ignored and no
    /// pause is advertised.
    pub fn new(suspendable: bool) -> Machine {
        tracing::debug!(suspendable, "analysis indicator machine created");
        return Machine {
            phase: Phase::Idle,
            suspendable,
            paused: false,
            sent: None,
            pending: None,
        };
    }

    /// What:
    /// `is_disabled` is true after the bus failed or no service answered.
    /// Why:
    /// the bus layer stops connecting once the machine will never ask for anything again.
    pub fn is_disabled(&self) -> bool {
        return self.phase == Phase::Disabled;
    }

    /// What:
    /// `interval_elapsed` is true when an ordinary update may be sent at `now`.
    /// Why:
    /// this is the one place the rate limit is decided.
    fn interval_elapsed(
        &self,
        now: Instant,
    ) -> bool {
        let Some(sent) = self.sent.as_ref() else {
            return true;
        };
        return now.saturating_duration_since(sent.at) >= MIN_UPDATE_INTERVAL;
    }

    /// What:
    /// `end_session` returns the machine to idle and releases a pause.
    /// Why:
    /// whatever ended the indicator,
    /// the engine must not stay paused with no control left to resume it.
    fn end_session(
        &mut self,
        next: Phase,
    ) -> Vec<Effect> {
        self.phase = next;
        self.sent = None;
        self.pending = None;
        if self.paused {
            self.paused = false;
            tracing::info!("analysis indicator ended while paused; asking the engine to resume");
            return vec![Effect::SetEnginePause(false)];
        }
        return Vec::new();
    }

    /// What:
    /// `offer` sends `view` now when it differs from the last sent one and the limit or `force` allows it,
    /// otherwise holds it.
    /// Why:
    /// a repeat is skipped,
    /// an early change waits for `deadline`,
    /// and a forced change goes at once.
    fn offer(
        &mut self,
        now: Instant,
        view: IndicatorView,
        force: bool,
    ) -> Vec<Effect> {
        let unchanged = match self.sent.as_ref() {
            Some(sent) => sent.view == view,
            None => false,
        };
        if unchanged {
            tracing::trace!(
                percent = view.percent,
                "analysis indicator update skipped: view unchanged"
            );
            self.pending = None;
            return Vec::new();
        }
        if force || self.interval_elapsed(now) {
            self.sent = Some(Sent {
                view: view.clone(),
                at: now,
            });
            self.pending = None;
            tracing::debug!(percent = view.percent, text = %view.text, force, "analysis indicator update sent");
            return vec![Effect::Update(view)];
        }
        tracing::trace!(
            percent = view.percent,
            "analysis indicator update held for the rate limit"
        );
        self.pending = Some(view);
        return Vec::new();
    }

    /// What:
    /// `progress` takes one report of tracks visited (`done`) of all tracks (`total`).
    /// Why:
    /// this is the main input;
    /// it opens the indicator,
    /// updates it,
    /// or finishes it.
    pub fn progress(
        &mut self,
        now: Instant,
        done: usize,
        total: usize,
    ) -> Vec<Effect> {
        if total == 0 {
            tracing::trace!("analysis report ignored: the total is not known yet");
            return Vec::new();
        }
        let finished = done >= total;
        return match self.phase {
            Phase::Disabled => Vec::new(),
            Phase::Idle if finished => {
                tracing::trace!("analysis report ignored: nothing to show for a finished sweep");
                Vec::new()
            }
            Phase::Idle => {
                let opening = view_for(done, total, self.paused);
                self.phase = Phase::Active;
                self.sent = Some(Sent {
                    view: opening.clone(),
                    at: now,
                });
                self.pending = None;
                tracing::info!(done, total, "analysis indicator opens");
                vec![Effect::Open(opening)]
            }
            Phase::Dismissed => {
                if finished {
                    tracing::info!(
                        "sweep ended after the user dismissed the analysis indicator; it may open again"
                    );
                    self.phase = Phase::Idle;
                }
                Vec::new()
            }
            Phase::Active if finished => {
                let last = final_view_for(total);
                let mut effects = self.offer(now, last, true);
                effects.push(Effect::Close(Outcome::Finished));
                tracing::info!(total, "analysis indicator closes: the sweep finished");
                effects.extend(self.end_session(Phase::Idle));
                effects
            }
            Phase::Active => {
                let running = view_for(done, total, self.paused);
                self.offer(now, running, false)
            }
        };
    }

    /// What:
    /// `deadline` is when `tick` has something to send,
    /// or `None` when nothing is held back.
    /// Why:
    /// the bus thread sleeps until then,
    /// so the last view of a stalled sweep still reaches the user.
    pub fn deadline(&self) -> Option<Instant> {
        if self.phase != Phase::Active || self.pending.is_none() {
            return None;
        }
        let sent = self.sent.as_ref()?;
        return Some(sent.at + MIN_UPDATE_INTERVAL);
    }

    /// What:
    /// `tick` sends the held view when the interval has passed at `now`.
    /// Why:
    /// without it,
    /// a sweep that stalls after a throttled report would leave an old count on screen.
    pub fn tick(
        &mut self,
        now: Instant,
    ) -> Vec<Effect> {
        if self.phase != Phase::Active || !self.interval_elapsed(now) {
            return Vec::new();
        }
        let Some(held) = self.pending.take() else {
            return Vec::new();
        };
        return self.offer(now, held, true);
    }

    /// What:
    /// `set_paused` takes a pause (`true`) or resume (`false`) request from the indicator's button.
    /// Why:
    /// the engine is told,
    /// and the button flips at once,
    /// because Plasma and notification servers only flip it
    /// when the app reports the new state.
    pub fn set_paused(
        &mut self,
        now: Instant,
        paused: bool,
    ) -> Vec<Effect> {
        if !self.suspendable || self.phase != Phase::Active || self.paused == paused {
            tracing::debug!(paused, phase = ?self.phase, "pause request ignored");
            return Vec::new();
        }
        self.paused = paused;
        tracing::info!(paused, "analysis pause changed from the indicator");
        let mut held: Option<IndicatorView> = self.pending.clone();
        if held.is_none()
            && let Some(sent) = self.sent.as_ref()
        {
            held = Some(sent.view.clone());
        }
        let Some(base) = held else {
            return vec![Effect::SetEnginePause(paused)];
        };
        let flipped = IndicatorView {
            paused,
            ..base
        };
        let mut effects = vec![Effect::SetEnginePause(paused)];
        effects.extend(self.offer(now, flipped, true));
        return effects;
    }

    /// What:
    /// `dismissed` records that the user removed the indicator.
    /// Why:
    /// it must not be brought back for this sweep;
    /// a pause is released so the sweep cannot stay stuck.
    pub fn dismissed(&mut self) -> Vec<Effect> {
        if self.phase != Phase::Active {
            return Vec::new();
        }
        tracing::info!("analysis indicator dismissed by the user");
        return self.end_session(Phase::Dismissed);
    }

    /// What:
    /// `failed` ends the indicator with an engine failure message.
    /// Why:
    /// the user is told why the analysis stopped;
    /// D84 gives them no other way to find out.
    pub fn failed(
        &mut self,
        message: String,
    ) -> Vec<Effect> {
        if self.phase != Phase::Active {
            return Vec::new();
        }
        tracing::warn!(message = %message, "analysis indicator closes: the engine failed");
        let mut effects = vec![Effect::Close(Outcome::Failed(message))];
        effects.extend(self.end_session(Phase::Idle));
        return effects;
    }

    /// What:
    /// `stop` closes an open indicator without a result.
    /// Why:
    /// used at shutdown so no job or notification outlives the app.
    pub fn stop(&mut self) -> Vec<Effect> {
        if self.phase != Phase::Active {
            return Vec::new();
        }
        tracing::info!("analysis indicator closes: the app is stopping it");
        let mut effects = vec![Effect::Close(Outcome::Stopped)];
        effects.extend(self.end_session(Phase::Idle));
        return effects;
    }

    /// What:
    /// `bus_failed` ends all use of the bus for the rest of the run.
    /// Why:
    /// no service answered or a send failed;
    /// retrying on every report would be a retry storm,
    /// and a pause is
    /// released because its button is gone.
    pub fn bus_failed(&mut self) -> Vec<Effect> {
        if self.phase == Phase::Disabled {
            return Vec::new();
        }
        tracing::info!("analysis indicator disabled for the rest of the run");
        return self.end_session(Phase::Disabled);
    }
}
