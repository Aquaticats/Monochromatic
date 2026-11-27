//! What:
//! `update` and `outdated`:
//! look every tool of the plan up again and write the lock when anything moved,
//! or report what would move without writing.
//! Why:
//! This is the only command that asks the registries for versions,
//! so the report it answers with,
//! the resolution of one tool,
//! the auto-sync of a missing one,
//! and the text a person reads are all here.
//! The work is one file per step,
//! named for what it holds:
//! what the run did (`what_an_update_did`),
//! resolving one tool (`resolving_one_tool`),
//! the update itself (`the_update_itself`),
//! the text a person reads (`the_update_report`),
//! and the entry points a caller runs (`the_update_entry_points`).
//! A caller imports from this file and never from a part.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update.ts
//! export function runUpdate(scope, env, runner, http, host, check): UpdateReport
//! export function updateLoaded(loaded, scope, env, runner, http, host, check, only?: Set<string>): UpdateReport
//! export function formatUpdateReport(report: UpdateReport, check: boolean): string
//! export * from './update/what_an_update_did.ts';
//! export * from './update/resolving_one_tool.ts';
//! export * from './update/the_update_itself.ts';
//! export * from './update/the_update_report.ts';
//! export * from './update/the_update_entry_points.ts';
//! ```

/// The resolution of one plan tool.
mod resolving_one_tool;
/// The functions a command calls to run `update` or `outdated`.
mod the_update_entry_points;
/// The update over every tool,
/// and the auto-sync before a second lookup.
mod the_update_itself;
/// The text `update` and `outdated` print.
mod the_update_report;
/// What happened to one tool,
/// and the report of a run.
mod what_an_update_did;

/// Re-exports the entry points a command runs `update` through.
pub use self::the_update_entry_points::{
    run_update,
    run_update_for,
};
/// Re-exports the update over files that are already read.
pub use self::the_update_itself::update_loaded;
/// Re-exports the text a person reads.
pub use self::the_update_report::format_update_report;
/// Re-exports what happened to one tool,
/// one line of the report,
/// and the report itself.
pub use self::what_an_update_did::{
    Outcome,
    OutcomeKind,
    UpdateReport,
};
