//! The two independent pieces of startup work,
//! run side by side without a closure:
//! compiling the rules and listing the files of the working tree.
//!
//! Both take time and neither needs the other's result,
//! so `--all` runs them in parallel.
//! Each piece is a value of [`StartupWork`] and one named function runs it,
//! so the parallel call names a function and carries its state as data.

/// Imports the error constructor and the result type of the two pieces of work.
use anyhow::{anyhow, Result};
/// Imports the parallel-iterator traits that run the pieces side by side.
use rayon::prelude::*;

/// Imports the rules loader and the compiled rules it returns.
use crate::frx_load::{load_listed, LoadedRules};
/// Imports the entry type of the rules list.
use crate::rules_list::RulesEntry;
/// Imports the working-tree walk and what it lists.
use crate::walk::{list_files, WorkingTreeListing};

/// What the two pieces of startup work need to know.
pub(crate) struct StartupRequest<'entries> {
    /// The rules files to compile,
    /// in list order.
    pub(crate) rules_entries: &'entries [RulesEntry],
    /// Whether the embedded precompiled baseline joins the compiled rules.
    pub(crate) builtin_rules: bool,
    /// Whether a missing optional rules file is logged quietly,
    /// which only the implicit default rules file asks for.
    pub(crate) quiet_absence: bool,
    /// Whether the files of the working tree are listed as well (`--all`).
    pub(crate) list_working_tree: bool,
}

/// One piece of startup work.
enum StartupWork<'request, 'entries> {
    /// Compile the rules the request names.
    LoadRules(
        /// What the rules compilation needs.
        &'request StartupRequest<'entries>,
    ),
    /// List the files of the working tree.
    ListFiles,
}

/// What one piece of startup work produced.
enum StartupOutcome {
    /// The compiled rules,
    /// or why they could not be compiled.
    Rules(
        /// The compiled rules,
        /// or the redacted reason they could not be built.
        Result<LoadedRules>,
    ),
    /// The listed files,
    /// or why the walk failed.
    Files(
        /// The listed paths and any tracked-file union gap,
        /// or the reason the walk failed.
        Result<WorkingTreeListing>,
    ),
}

/// Runs one piece of startup work.
fn run_work(work: StartupWork<'_, '_>) -> StartupOutcome {
    return match work {
        StartupWork::LoadRules(request) => StartupOutcome::Rules(load_listed(
            request.rules_entries,
            request.builtin_rules,
            request.quiet_absence,
            // The standalone command is the owner of this user's cache, so it publishes what it compiles.
            crate::runtime_cache::CacheAccess::ReadWrite,
            crate::BUILTIN_PRECOMPILED,
            crate::BUILTIN_NAMES,
        )),
        StartupWork::ListFiles => StartupOutcome::Files(list_files(".")),
    };
}

/// Compiles the rules and,
/// when asked,
/// lists the working tree,
/// both at once.
///
/// Returns the rules result and the listing result;
/// the listing is `None` when it was not asked for.
pub(crate) fn run_startup_work(
    request: &StartupRequest<'_>,
) -> (Result<LoadedRules>, Option<Result<WorkingTreeListing>>) {
    let mut work: Vec<StartupWork<'_, '_>> = vec![StartupWork::LoadRules(request)];
    if request.list_working_tree {
        work.push(StartupWork::ListFiles);
    }
    let outcomes: Vec<StartupOutcome> = work.into_par_iter().map(run_work).collect();
    let mut rules_result: Option<Result<LoadedRules>> = None;
    let mut files_result: Option<Result<WorkingTreeListing>> = None;
    for outcome in outcomes {
        match outcome {
            StartupOutcome::Rules(result) => rules_result = Some(result),
            StartupOutcome::Files(result) => files_result = Some(result),
        }
    }
    let Some(rules) = rules_result else {
        return (Err(anyhow!("startup work returned no rules result")), files_result);
    };
    return (rules, files_result);
}
