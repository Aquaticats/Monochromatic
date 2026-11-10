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
/// so a control runs fixture tools and reaches no real one.
/// Why:
///   The tools' programs and bounds come from the process a service runs in,
/// which a control cannot choose for the test binary.
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
    let mut failures: Vec<String> = Vec::new();
    let mut repaired: usize = 0;
    for tool in &entry.tools {
        match relink_one(*tool, entry.root.as_path(), environment).await {
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

/// The job's controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_relink_job_tests.rs"]
mod tests;
