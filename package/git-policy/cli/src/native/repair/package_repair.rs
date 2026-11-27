//! What:
//!  The repair of a new worktree's package-manager state:
//! after a rung filled the worktree and the ignored files were
//!  copied,
//! run each tool's own offline repair so the state it keeps for the old location works at the new one.
//! Why:
//!  A copy keeps links and build state exactly as they were.
//! pnpm's links climb out of the tree by a fixed number of
//!  levels,
//! Cargo's build state bakes in the source's paths,
//! and the toolchain launchers name the source's lock.
//! Each tool
//!  knows how to rebuild its own layout,
//! so the wrapper starts it (the owner's exemption,
//! 2026-11-03) and never edits
//!  its files itself.
//! A repair never fails the add:
//! it ends in a line that names the command to run,
//! and a tool the
//!  environment does not provide is a verdict,
//! not an error.
//! The three repairs are tasks of the process's runtime,
//! one lane each,
//! so the add waits for the
//!  slowest of them and a runtime worker is never blocked by one
//! (`doc/planning/git-policy-cli-full-tokio.md`,
//! "The order of the move").
//! Another process changing the files while they are copied or
//!  repaired is the person's risk and is not defended against.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! repairWorktree({ git, source, destination, switch: { quiet, noCheckout }, environment }, process.stderr);
//! ```
//!
//! Files in this module,
//! each using only the files listed before it:
//!
//! - `package_repair_relink_hand_over.rs`:
//!   the hand-over of a worktree's relink to the commit service,
//!   and the relink the add does when no service takes it.

/// Import the timing record of each repair's run.
use crate::add_timing::{
    begin_work,
    end_work,
};
/// Import the call that turns a panicked task into a value.
use crate::call_catching_panics_awaiting::call_catching_panics_awaiting;
/// Import the one diagnostic log this crate writes.
use crate::diagnostic_log::debug;
/// Import the process's runtime,
/// which the synchronous face enters.
use crate::lazy_runtime::run_on_runtime;
/// Import the Cargo repair,
/// both faces of it.
use crate::package_repair_cargo::{
    repair_cargo,
    repair_cargo_awaiting,
};
/// Import the pnpm repair,
/// both faces of it.
use crate::package_repair_pnpm::repair_pnpm;
/// Import the hand-over of the relink,
/// what it came to,
/// and the tag every line of this repair's diagnostic log carries.
use crate::package_repair_relink_hand_over::{
    LinksHandedOver,
    TAG,
    hand_the_links_over,
};
/// Import what a repair reports and the words it prints.
use crate::package_repair_report::{
    Finding,
    render,
};
/// What the repairs read,
/// who asks,
/// and the switches of the person;
/// kept in a file of their own and named here so the paths
///  outside this folder stay the same.
pub use crate::package_repair_request::{
    RepairEnvironment,
    RepairRequest,
    RepairSwitch,
    workers_for,
};
/// Import the toolchain repair,
/// both faces of it.
use crate::package_repair_toolchain::repair_toolchain;
/// Import the stream the repair's lines are written to.
use std::io::Write;
/// Import the borrowed and owned filesystem paths the repair serves and names.
use std::path::{
    Path,
    PathBuf,
};
/// Import the shared environment the repairs read from.
use std::sync::Arc;
/// Import the clock that times the whole repair.
use std::time::Instant;
/// What:
///  What one repair reads,
/// owned,
/// so that the waits of the repairs may outlive the caller's borrows.
/// Why:
///   The repairs are futures the runtime drives,
/// and one of them holds its facts across the waits of a task;
/// the four facts are a Git context and two paths,
/// which are cheap to hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwnedRequest = { git: GitContext; source: string; destination: string; switch: RepairSwitch; environment: RepairEnvironment };
/// ```
struct OwnedRequest {
    /// The real Git and its overlay.
    git: crate::transaction_git::GitContext,
    /// The checkout the ignored files came from.
    source: PathBuf,
    /// The new worktree.
    destination: PathBuf,
    /// The person's switches.
    switch: RepairSwitch,
    /// What the repairs read from the process.
    environment: Arc<RepairEnvironment>,
}

impl OwnedRequest {
    /// Take what the caller lent for the length of the repairs.
    fn of_the_caller(request: &RepairRequest<'_>) -> OwnedRequest {
        return OwnedRequest {
            git: request.git.clone(),
            source: request.source.to_path_buf(),
            destination: request.destination.to_path_buf(),
            switch: request.switch,
            environment: Arc::new(request.environment.clone()),
        };
    }

    /// What one repair reads,
    /// lent for the length of its own work.
    fn lent(&self) -> RepairRequest<'_> {
        return RepairRequest {
            git: &self.git,
            source: self.source.as_path(),
            destination: self.destination.as_path(),
            switch: self.switch,
            environment: self.environment.as_ref(),
        };
    }
}

/// What:
///  The Cargo repair on the runtime,
/// with a panic of its work reported as no findings for it.
/// Why:
///   A workspace that stopped unexpectedly reports nothing and the other lanes still report,
/// as a scoped thread's panic did before
///  the move;
/// the panic is turned into a value so it cannot end the task that runs every repair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function repairCargoOne(request: RepairRequest): Promise<Finding[]>;
/// ```
async fn repair_cargo_one(request: RepairRequest<'_>) -> Vec<Finding> {
    let began: Instant = begin_work();
    let repaired: Result<Vec<Finding>, String> =
        call_catching_panics_awaiting(repair_cargo_awaiting(&request)).await;
    end_work("repairCargo", began);
    return match repaired {
        Ok(findings) => findings,
        Err(_words) => {
            debug(
                TAG,
                "the cargo repair stopped unexpectedly and reports nothing",
            );
            Vec::new()
        }
    };
}

/// What:
///  Say what the repairs found and the lines that name what a person can run instead.
/// Why:
///   Both faces report the same findings the same way,
/// whatever waited for them,
/// and a repair never fails the add.
///  This write is the add's own line on the person's stream,
/// as every other line the add prints is,
/// and it is not bounded:
/// nothing the repair owns is still running when it is written,
/// so a reader that stopped draining waits for the command's own output,
/// and the Git this command forwarded writes that stream itself.
/// The one line of the repair that is bounded is the notice of a tool's wait,
/// which is written by the loop that ends the tool (`REPAIR_NOTICE_WRITE_CEILING`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function report(destination: string, findings: Finding[], quiet: boolean, notices: Write, started: Instant): Finding[];
/// ```
fn report(
    destination: &Path,
    findings: Vec<Finding>,
    quiet: bool,
    notices: &mut dyn Write,
    started: Instant,
) -> Vec<Finding> {
    debug(
        TAG,
        format!(
            "{}: {} findings in {} ms",
            destination.display(),
            findings.len(),
            started.elapsed().as_millis()
        )
        .as_str(),
    );
    let text: String = render(destination, findings.as_slice(), quiet);
    // A closed standard error leaves nowhere to say it; the worktree is whole either way.
    let _written = notices.write_all(text.as_bytes());
    return findings;
}

/// What:
///  Repair the package-manager state of one new worktree on the runtime and print what the person needs to know.
/// Why:
///   Cargo's repair and the hand-over of the other two tools are independent,
/// so the two waits run beside each other on one task of the runtime:
/// the add ends when the slowest does,
/// and no worker is held while either runs,
/// because every suspension is an await.
/// The worktree's relink is the commit service's work when a service answers,
/// and the add's own when none does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await repairWorktreeAwaiting(request, stderr);
/// ```
pub(crate) async fn repair_worktree_awaiting(
    request: &RepairRequest<'_>,
    notices: &mut dyn Write,
) -> Vec<Finding> {
    if request.switch.no_checkout {
        debug(
            TAG,
            format!(
                "{}: --no-checkout, so there is nothing to repair",
                request.destination.display()
            )
            .as_str(),
        );
        return Vec::new();
    }
    let started: Instant = Instant::now();
    let owned: OwnedRequest = OwnedRequest::of_the_caller(request);
    // One wait for Cargo's repair and one for the relink, driven together on this task: both suspend rather than hold
    //  a thread, so they overlap without a task of their own.
    let links_request: RepairRequest<'_> = owned.lent();
    let cargo_request: RepairRequest<'_> = owned.lent();
    let (links, cargo): (LinksHandedOver, Vec<Finding>) = tokio::join!(
        hand_the_links_over(&links_request),
        repair_cargo_one(cargo_request)
    );
    // The report's order has always been pnpm's findings, then Cargo's, then the toolchain's.
    let findings: Vec<Finding> = links
        .pnpm
        .into_iter()
        .chain(cargo)
        .chain(links.toolchain)
        .collect();
    return report(
        request.destination,
        findings,
        request.switch.quiet,
        notices,
        started,
    );
}

/// What:
///  The three repairs one after another on the calling thread,
/// for the state where no runtime can be built.
/// Why:
///   The owner's rule is "Commits should never fail.",
/// so an add still reports each tool's answer where the process cannot build the
///  runtime its waits need;
/// each tool then reports that in its own words,
/// and the report and the findings are the ones the three waits give.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repairWithoutTheRuntime(request, stderr) { return report(pnpm(request), toolchain(request), cargo(request)); }
/// ```
fn repair_without_the_runtime(
    request: &RepairRequest<'_>,
    notices: &mut dyn Write,
) -> Vec<Finding> {
    let started: Instant = Instant::now();
    let pnpm: Option<Finding> = repair_pnpm(request.destination, request.environment);
    let toolchain: Option<Finding> = repair_toolchain(request.destination, request.environment);
    let cargo: Vec<Finding> = repair_cargo(request);
    let findings: Vec<Finding> = pnpm.into_iter().chain(cargo).chain(toolchain).collect();
    return report(
        request.destination,
        findings,
        request.switch.quiet,
        notices,
        started,
    );
}

/// What:
///  Repair the package-manager state of one new worktree and print what the person needs to know.
/// Why:
///   The tools are independent,
/// so they run side by side;
/// the add ends when the slowest does.
/// This face enters the
///  process's runtime once and waits there,
/// and where no runtime can be built the three repairs run in turn,
/// so the add
///  reports every tool and changes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const findings = repairWorktree(request, stderr);
/// ```
pub fn repair_worktree(
    request: &RepairRequest<'_>,
    notices: &mut dyn Write,
) -> Vec<Finding> {
    if request.switch.no_checkout {
        debug(
            TAG,
            format!(
                "{}: --no-checkout, so there is nothing to repair",
                request.destination.display()
            )
            .as_str(),
        );
        return Vec::new();
    }
    return match run_on_runtime(repair_worktree_awaiting(request, notices)) {
        Ok(findings) => findings,
        // No runtime could be built, so the tools run in turn and each says in its own words that it could not run.
        Err(refusal) => {
            debug(
                TAG,
                format!(
                    "{}: the repairs run in turn ({refusal})",
                    request.destination.display()
                )
                .as_str(),
            );
            repair_without_the_runtime(request, notices)
        }
    };
}

/// Repair controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_tests.rs"]
mod tests;
