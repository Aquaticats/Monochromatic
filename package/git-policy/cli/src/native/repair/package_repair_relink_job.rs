//! What:
//!  Relink the copied package state of a new worktree inside the commit service,
//! after the add that queued it has returned.
//! Why:
//!  The add writes what the worktree needs and returns
//! (`package_repair_relink.rs`),
//! so the two tools that rewrite link layouts in place are the service's work:
//! pnpm's own `install --offline --frozen-lockfile` for the new depth,
//! and mpm-shadow's `install --frozen-lockfile` for the new checkout.
//! Both runs keep every guarantee the add gave them:
//! each tool's own command,
//! the same quiet window and the same ceiling,
//! and a run that fails leaves the record failed and names the tool.
//! A tool the service cannot find is a failure that says so,
//! as a missing tool was a line of the add and never its failure.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await relinkAwaiting(item); // for each tool of the record: run its own frozen install in the worktree
//! ```

use crate::diagnostic_log::debug;
use crate::package_repair_pnpm::repair_pnpm_awaiting;
use crate::package_repair_relink::{RelinkEntry, RelinkTool, decode};
use crate::package_repair_report::{Finding, Outcome};
use crate::package_repair_request::RepairEnvironment;
use crate::package_repair_toolchain::repair_toolchain_awaiting;
use crate::service_work_handlers_replanning_and_acceptance::Verdict;
use crate::work_item::WorkItem;

/// Tag used for diagnostics from this module.
const TAG: &str = "packageRepairRelinkJob";

/// What:
///  Run one tool's repair in the worktree,
/// awaited by the service's round.
/// Why:
///   The two tools are one call each,
/// and which one runs is the record's choice,
/// so the match stands where the record is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const run = (tool, root, environment) => tool === 'pnpm' ? repairPnpmAwaiting(root, environment) : repairToolchainAwaiting(root, environment);
/// ```
async fn relink_one(tool: RelinkTool, root: &std::path::Path, environment: &RepairEnvironment) -> Option<Finding> {
    return match tool {
        RelinkTool::Pnpm => repair_pnpm_awaiting(root, environment).await,
        RelinkTool::Toolchain => repair_toolchain_awaiting(root, environment).await,
    };
}

/// What:
///  Relink every tool one record names,
/// awaited by the service's round.
/// Why:
///   The add no longer does this work;
/// the record says which worktree and which tools,
/// so no decision of the add is repeated here.
/// One tool that fails does not stop the next,
/// and the verdict names every tool that failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function relinkAwaiting(item): Promise<Verdict>;
/// ```
pub async fn relink_awaiting(item: &WorkItem) -> Verdict {
    let environment: RepairEnvironment = RepairEnvironment::from_process();
    return relink_with(item, &environment).await;
}

/// What:
///  `relink_awaiting` with what the tools read from the process injected,
/// and the add's own search path taken from the record.
/// Why:
///   The tools' programs and bounds come from the process a service runs in,
/// which a control cannot choose for the test binary;
/// the search path is the add's when it recorded one,
/// because that is the path the owner decided the repair resolves on,
/// and the service's own path need hold no pnpm or another one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function relinkWith(item, environment): Promise<Verdict>;
/// ```
async fn relink_with(item: &WorkItem, environment: &RepairEnvironment) -> Verdict {
    let entry: RelinkEntry = match decode(item.parent.as_str()) {
        Ok(decoded) => decoded,
        Err(reason) => return Verdict::Failed(reason),
    };
    if let Some(refusal) = refusal_of_a_record_that_is_not_about_its_own_directory(item, &entry) {
        return Verdict::Skipped(refusal);
    }
    let as_the_add_left_it: RepairEnvironment = environment_as_the_add_left_it(&entry, environment);
    let mut failures: Vec<String> = Vec::new();
    let mut repaired: usize = 0;
    for tool in &entry.tools {
        match relink_one(*tool, entry.root.as_path(), &as_the_add_left_it).await {
            // The worktree no longer needs this tool, which a person's own command may have seen to.
            None => debug(TAG, format!("{}: {} had nothing to relink", entry.root.display(), tool.name()).as_str()),
            Some(Finding { outcome: Outcome::Repaired { .. }, .. }) => repaired += 1,
            Some(Finding { outcome: Outcome::Failed { reason, .. }, .. }) => failures.push(format!("{}: {reason}", tool.name())),
        }
    }
    debug(TAG, format!("{}: {repaired} relinked, {} failed", entry.root.display(), failures.len()).as_str());
    return match failures.first() {
        None => Verdict::Done,
        Some(first_failure) => Verdict::Failed(first_failure.clone()),
    };
}

/// What:
///  Why this record is not acted on:
/// its two names of the directory disagree,
/// or the directory is no longer this repository's worktree.
/// Why:
///   A record outlives the worktree it was written for:
/// a service that died holds a claim,
/// a claim that ran out is queued again,
/// and a person may remove the worktree and put their own directory at that path meanwhile.
/// The tools rewrite link layouts in place,
/// so the directory is proved to be one of this repository's registrations before any of them starts,
/// and the answer names what was left exactly as it is.
/// A worktree removed and added again at the same path before this record was drained is a registration
/// of this repository too,
/// and each tool decides its need again from the directory it finds,
/// so the run there is the one that worktree's own record asks for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = (item, entry) => entry.root !== item.repository ? 'the record names two directories' : registrationOf(item.gitDir, entry.root).error;
/// ```
fn refusal_of_a_record_that_is_not_about_its_own_directory(item: &WorkItem, entry: &RelinkEntry) -> Option<String> {
    if entry.root != item.repository {
        return Some(format!(
            "the record names {} as its worktree and {} in its payload, so neither was touched",
            item.repository.display(),
            entry.root.display()
        ));
    }
    return match crate::worktree_added_by_the_wrapper::registration_admin(item.git_dir.as_path(), entry.root.as_path()) {
        Ok(_registration) => None,
        Err(reason) => Some(format!("this directory is no worktree of the repository any more ({reason})")),
    };
}

/// What:
///  The environment the tools run in:
/// the service's own bounds and workers,
/// with the search path the add recorded when it recorded one.
/// Why:
///   The bounds are the service's own and are not the add's to set,
/// while the search path is the person's:
/// it decides which pnpm and mpm-shadow run,
/// and their own children read it too,
/// so a record without one (written before this release carried it) keeps the service's path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const environmentAsTheAddLeftIt = (entry, environment) => entry.searchPath === undefined ? environment : { ...environment, searchPath: entry.searchPath };
/// ```
fn environment_as_the_add_left_it(entry: &RelinkEntry, environment: &RepairEnvironment) -> RepairEnvironment {
    return match entry.search_path.as_ref() {
        None => environment.clone(),
        Some(search_path) => RepairEnvironment { search_path: Some(search_path.clone()), ..environment.clone() },
    };
}

/// The job's controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_relink_job_tests.rs"]
mod tests;
