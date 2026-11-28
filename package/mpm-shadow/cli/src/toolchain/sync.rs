//! What:
//! `install`:
//! install every locked tool that is not already at its locked version,
//! add the components and targets the lock declares to a Rust toolchain that is,
//! then make the postinstall links,
//! rebuild the exposure directory and run the postinstall commands.
//! Why:
//! This is the installer.
//! It reads only the lock (which `install` first proves agrees with the plan),
//! probes before installing so a second run changes nothing,
//! and keeps going after one tool fails so the report names every failure.
//! The work is one file per step,
//! named for what it holds:
//! the order the tools are installed in (`the_install_order`),
//! what one run did (`what_the_sync_did`),
//! the install itself (`the_sync_itself`),
//! the frozen policy and its refusal (`the_frozen_policy`),
//! the lock preparation (`preparing_the_lock`),
//! and the entry points a caller runs (`the_install_entry_points`).
//! A caller imports from this file and never from a part.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // sync.ts
//! export function syncOrder(tools: LockedTool[]): LockedTool[]
//! export function syncTools(ctx: Context, only?: string[]): SyncReport
//! export function runSync(scope, env, runner, http, host): SyncReport  // throws when plan and lock disagree
//! export * from './sync/the_install_order.ts';
//! export * from './sync/what_the_sync_did.ts';
//! export * from './sync/the_sync_itself.ts';
//! export * from './sync/the_frozen_policy.ts';
//! export * from './sync/preparing_the_lock.ts';
//! export * from './sync/the_install_entry_points.ts';
//! ```

/// The one instant a recovery's newest-only rule has,
/// which a control stands in with a start of the tool and may end a run at.
mod a_control_holds_the_instant_between_the_two_steps;
/// Bringing the lock up to date with the plan without installing anything.
mod preparing_the_lock;
/// Whether a plan ahead of the lock is resolved or refused.
mod the_frozen_policy;
/// The functions a command calls to run `install`.
mod the_install_entry_points;
/// The order the tools are installed in.
mod the_install_order;
/// The install of every selected tool,
/// and the steps that follow it.
mod the_sync_itself;
/// What one sync did,
/// the warning for a replaced version,
/// and the sentence for a version a killed install left unfinished,
/// which this crate's own report reads (not a caller).
mod what_the_sync_did;

/// Re-exports the hold a control plants at the newest-only rule's one instant,
/// which only a test build reads (`a_control_holds_the_instant_between_the_two_steps`).
#[cfg(test)]
pub(crate) use self::a_control_holds_the_instant_between_the_two_steps::{
    Hold,
    holding_the_next_instant,
};
/// Re-exports what a lock preparation wrote.
pub use self::preparing_the_lock::{
    LockPreparation,
    prepare_lock,
};
/// Re-exports the lock policy and the reason a frozen one names.
pub use self::the_frozen_policy::{
    FrozenReason,
    LockPolicy,
};
/// Re-exports the entry points a command runs `install` through.
pub use self::the_install_entry_points::{
    run_lockfile_only,
    run_sync,
    run_sync_with,
    run_sync_with_named,
};
/// Re-exports the tools installed first,
/// and the order the lock's tools are worked through.
pub use self::the_install_order::{
    STAGING_IDS,
    sync_order,
};
/// Re-exports the install itself.
pub use self::the_sync_itself::sync_tools;
/// Re-exports what one sync did,
/// and the warning for an install that replaces another version.
pub use self::what_the_sync_did::{
    SyncReport,
    replacement_warning,
};
