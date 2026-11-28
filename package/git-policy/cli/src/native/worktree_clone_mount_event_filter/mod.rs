//! What:
//!  The event filter of plan step 19:
//!  the names one ZED delivery carries,
//!  the names the route's clone parent has,
//!  the decision about one delivery,
//!  and the rule that decides it.
//! Why:
//!  ZED forks the installed zedlet for every history event of every pool,
//!  and every such fork would otherwise start a pass over every managed clone,
//!  so the handler reads the delivery it was started for and drops everything
//!  that is not about its own clone parent or a dataset below it
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "The event filter,
//! in brief";
//!  veto entry V2 of the same file,
//!  accepted with the other sixteen choices in
//!  `doc/decision/cli-git-zfs-lost-event-triggers-hook-nudge-and-cost-cap.md`,
//!  "The design is accepted").
//!  The filter is cost and never safety:
//!  what it drops could have been the accidental recovery of a clone whose own
//!  event was lost,
//!  which the design states as its price (limitation L5 there).
//!  Nothing here is reached by a command yet:
//!  plan step 19's wiring calls the filter at the head of the handler's
//!  dispatch,
//!  before the compatibility gate and before any mode,
//!  and that wiring is the round of step 19's own
//!  (`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`,
//!  "A piece built ahead merges as code no person's command reaches").
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `event_identity.rs`:
//!   the pool and dataset one ZED delivery carries,
//!   read from the environment ZED builds for a zedlet.
//! - `parent_scope.rs`:
//!   the pool and dataset the route's clone parent has,
//!   read from the one dataset name the configuration carries.
//! - `event_filter_decision.rs`:
//!   the answer for one delivery,
//!   the reason it gives and the one line each answer is written as.
//! - `event_filter_scope.rs`:
//!   the rule over those names,
//!   which reads and opens nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const decision = decideAboutTheEvent(EventIdentity.live(), ParentScope.ofDataset(configuredParent));
//! ```

/// The two names one ZED delivery carries.
pub mod event_identity;

/// The two names the route's clone parent has.
pub mod parent_scope;

/// What the filter decided,
/// why,
/// and the line each answer is written as.
pub mod event_filter_decision;

/// The rule that decides one delivery.
pub mod event_filter_scope;

/// What the filter decided and the line it is written as.
pub use event_filter_decision::{
    DroppedWhy,
    FilterDecision,
    PassedWhy,
};
/// The rule over one delivery's names.
pub use event_filter_scope::decide_about_the_event;
/// The absent name a delivery can carry.
pub use event_identity::EventIdentity;
/// The names the route's clone parent has.
pub use parent_scope::ParentScope;
