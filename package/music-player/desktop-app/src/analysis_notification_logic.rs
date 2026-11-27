//! The pure decision logic of the true-peak analysis indicator (D107):
//!  which view to show,
//! when to send it,
//! and when to open,
//! pause or close it.
//!
//! D107 replaces the in-app banner with a system indicator:
//!  a Plasma job on KDE,
//! a freedesktop notification elsewhere.
//! Both are reached over D-Bus,
//!  so the rules that are independent of the bus live here and take time as an argument,
//!  which makes them testable without a clock,
//! a thread or a bus:
//!
//! - The view is the integer percent (never 100 while work remains),
//!   the count text and the paused flag.
//! - A view equal to the last sent one is skipped.
//! - Ordinary updates are at most one per `MIN_UPDATE_INTERVAL` (two per second);
//!   the newest skipped view is kept and sent at `deadline`.
//! - The open,
//!   the final state,
//!   the close and a pause change are never delayed.
//! - A user dismissal,
//!   an engine failure or a dead bus end the indicator and always release a pause.
//!
//! The state machine itself lives in `analysis_notification_logic/machine.rs`;
//! this file holds the words it
//! answers in and the rate it may answer at.

/// What:
/// `mod machine;` the indicator's state machine (`src/analysis_notification_logic/machine.rs`).
/// Why:
/// the transitions are the bulk of the logic and the tests read them one phase at a time,
/// so
/// they sit apart from the view,
/// outcome and effect types this module hands to the bus layer.
mod machine;

/// Re-exports the machine and the two view builders the tests and the bus layer name,
/// so
/// `analysis_notification_logic::Machine` keeps working.
pub use self::machine::{
    Machine,
    count_text,
    percent_for,
};

/// Imports the span type with which the machine rate-limits updates.
use std::time::Duration;

/// What:
/// `MIN_UPDATE_INTERVAL` is the shortest gap between two ordinary updates sent to the bus.
/// Why:
/// D107 asks for at most two updates a second;
/// the sweep reports four times a second at most,
/// and a bar
/// that moves in half-second steps reads as smooth while sparing the notification server.
pub const MIN_UPDATE_INTERVAL: Duration = Duration::from_millis(500);

/// What:
/// `PERCENT_WHILE_RUNNING_MAX` is the highest percent shown while any track remains.
/// Why:
/// a bar that reads 100 while work remains looks finished;
/// 100 appears only in the final view.
const PERCENT_WHILE_RUNNING_MAX: u32 = 99;

/// What:
/// `COMPLETE_PERCENT` is the percent of the final view.
/// Why:
/// the last update before the close fills the bar,
/// as the compress-to-sqfs job does before it terminates.
const COMPLETE_PERCENT: u32 = 100;

/// What:
/// `IndicatorView` is everything the indicator shows:
/// the percent,
/// the count text and the paused flag.
/// Why:
/// comparing two views is the "nothing changed" test,
/// so the paused flag is part of it and a pause change is
/// never mistaken for a repeat.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndicatorView {
    /// Whole percent from 0 to 100.
    pub percent: u32,
    /// The count line,
    /// for example `412 of 1,218`.
    pub text: String,
    /// True while the user has paused the analysis.
    pub paused: bool,
}

/// What:
/// `Outcome` says why the indicator is being closed.
/// Why:
/// a Plasma job is ended with a different code for success,
/// a stop and a failure,
/// and only a failure shows text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Every track was visited.
    Finished,
    /// The app stopped the indicator before the sweep ended (shutdown);
    /// nothing is reported to the user.
    Stopped,
    /// The engine failed;
    /// the message is shown to the user.
    Failed(
        /// The plain sentence that tells the user what failed.
        String,
    ),
}

/// What:
/// `Effect` is one thing the bus layer must do,
/// in the order returned.
/// Why:
/// the machine decides and the bus layer only carries out,
/// so the layer holds no rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Create the indicator showing this view.
    Open(
        /// The view to show.
        IndicatorView,
    ),
    /// Change the open indicator to this view.
    Update(
        /// The view to show instead.
        IndicatorView,
    ),
    /// Remove the indicator for this reason.
    Close(
        /// Why the indicator closes.
        Outcome,
    ),
    /// Tell the engine to pause (`true`) or resume (`false`) the analysis.
    SetEnginePause(
        /// `true` to pause,
        /// `false` to resume.
        bool,
    ),
}

/// Unit tests in `analysis_notification_logic_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "analysis_notification_logic_tests.rs"]
mod tests;
