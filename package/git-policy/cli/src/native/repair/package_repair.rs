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

use crate::add_timing::{begin_work, end_work};
use crate::blocking_job::BlockingJobError;
use crate::blocking_jobs_within_the_budget::run_a_job_or_in_turn;
use crate::call_catching_panics_awaiting::call_catching_panics_awaiting;
use crate::diagnostic_log::debug;
use crate::lazy_runtime::run_on_runtime;
use crate::package_repair_cargo::{repair_cargo, repair_cargo_awaiting};
use crate::package_repair_pnpm::{PnpmNeed, repair_pnpm, repair_pnpm_awaiting};
use crate::package_repair_prewarm::common_directory_awaiting;
use crate::package_repair_relink::{NotTaken, QueuedRelink, RelinkEntry, RelinkTool, SearchPathOfTheAdd, hand_over_in_a_job};
/// What the repairs read,
/// who asks,
/// and the switches of the person;
/// kept in a file of their own and named here so the paths
///  outside this folder stay the same.
pub use crate::package_repair_request::{RepairEnvironment, RepairRequest, RepairSwitch, workers_for};
use crate::package_repair_report::{Finding, Outcome, Tool, render};
use crate::package_repair_toolchain::{MANUAL_COMMAND as TOOLCHAIN_MANUAL_COMMAND, ToolchainNeed, repair_toolchain, repair_toolchain_awaiting};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

/// Tag used for diagnostics from this module.
const TAG: &str = "packageRepair";

/// What:
///  What the add's hand-over of the copied package state came to for each tool.
/// Why:
///   The report has always named pnpm's findings first,
/// then Cargo's,
/// then the toolchain's,
/// and the two tools of the hand-over are read separately from the Cargo repair,
/// so each keeps its own list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LinksHandedOver = { pnpm: Finding[]; toolchain: Finding[] };
/// ```
struct LinksHandedOver {
    /// pnpm's findings.
    pnpm: Vec<Finding>,
    /// mpm-shadow's findings.
    toolchain: Vec<Finding>,
}

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
        return RepairRequest { git: &self.git, source: self.source.as_path(), destination: self.destination.as_path(), switch: self.switch, environment: self.environment.as_ref() };
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
    let repaired: Result<Vec<Finding>, String> = call_catching_panics_awaiting(repair_cargo_awaiting(&request)).await;
    end_work("repairCargo", began);
    return match repaired {
        Ok(findings) => findings,
        Err(_words) => {
            debug(TAG, "the cargo repair stopped unexpectedly and reports nothing");
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function report(destination: string, findings: Finding[], quiet: boolean, notices: Write, started: Instant): Finding[];
/// ```
fn report(destination: &Path, findings: Vec<Finding>, quiet: bool, notices: &mut dyn Write, started: Instant) -> Vec<Finding> {
    debug(TAG, format!("{}: {} findings in {} ms", destination.display(), findings.len(), started.elapsed().as_millis()).as_str());
    let text: String = render(destination, findings.as_slice(), quiet);
    // A closed standard error leaves nowhere to say it; the worktree is whole either way.
    let _written = notices.write_all(text.as_bytes());
    return findings;
}

/// What:
///  What the add tells the person about one tool it handed over:
/// the tool's line when the record reached the service.
/// Why:
///   The record is the service's work,
/// so the line says the relinking happens after this command returns;
/// a hand-over no service took is the add's own run,
/// and each tool reports it as it always did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handedOver = (tool, millis, destination) => Finding
/// ```
fn handed_over(tool: RelinkTool, millis: u128, destination: &Path) -> Finding {
    let (as_report, detail): (Tool, &str) = match tool {
        RelinkTool::Pnpm => (Tool::Pnpm, "pnpm will relink the copied node_modules here"),
        RelinkTool::Toolchain => (Tool::Toolchain, "mpm-shadow will rebuild the toolchain links here"),
    };
    debug(TAG, format!("{}: {} is the service's work", destination.display(), tool.name()).as_str());
    return Finding { tool: as_report, outcome: Outcome::Repaired { detail: format!("{detail}, because the commit service runs it after this command returns"), millis } };
}

/// What:
///  How far the hand-over of a worktree's relink got.
/// Why:
///   The three answers differ in what the add does next:
/// a service that took the record means the tools run there,
/// and any other answer means the add runs them here,
/// because a worktree whose copied links were never relinked is wrong for the person's next command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Told = { kind: 'taken'; millis: number } | { kind: 'stays'; reason: string; queued?: QueuedRelink };
/// ```
enum Told {
    /// The record reached the service,
    /// so the tools are its work;
    /// the time is what the record and the wake cost.
    Taken(u128),
    /// No service took the record,
    /// so the add relinks the worktree here;
    /// the text says why the hand-over failed,
    /// and the record still queued is taken back after this run has finished.
    Stays { reason: String, queued: Option<QueuedRelink> },
}

/// What:
///  Write the relink's record and wake the service that runs it,
/// timed as the step `relinkHandOver`,
/// and say whether the tools are the service's work or the add's.
/// Why:
///   The record and the wake are storage and socket waits,
/// so they run as a job of the blocking pool and never on the task that also runs the Cargo repair;
/// a hand-over no service took takes its record back,
/// and the add relinks the worktree itself,
/// so no record waits for a directory that may hold other work later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function handOverTheRecord(request, entry): Promise<Told>
/// ```
async fn hand_over_the_record(request: &RepairRequest<'_>, entry: &RelinkEntry) -> Told {
    let began: Instant = begin_work();
    let common_dir: Result<PathBuf, String> = common_directory_awaiting(request.git, request.destination).await;
    let told: Result<Result<(), NotTaken>, BlockingJobError> = match common_dir {
        Ok(common) => {
            let state: crate::package_repair_relink::TheHandOver =
                (request.git.clone(), common, request.destination.to_path_buf(), entry.clone());
            run_a_job_or_in_turn(state, hand_over_in_a_job).await
        }
        Err(reason) => Ok(Err(NotTaken { reason, queued: None })),
    };
    end_work("relinkHandOver", began);
    let millis: u128 = began.elapsed().as_millis();
    return match told {
        Ok(Ok(())) => Told::Taken(millis),
        Ok(Err(not_taken)) => Told::Stays { reason: not_taken.reason, queued: not_taken.queued },
        Err(refusal) => Told::Stays { reason: format!("{refusal}"), queued: None },
    };
}

/// What:
///  Relink the worktree here,
/// for a hand-over no service took,
/// and report each tool's own line.
/// Why:
///   The copied links must work at the new location before the person's next command there,
/// so a session that cannot reach a service does what the add always did,
/// and the record that hand-over wrote is taken back by then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function relinkHere(request, tools): Promise<LinksHandedOver>
/// ```
async fn relink_here(request: &RepairRequest<'_>, tools: &[RelinkTool]) -> LinksHandedOver {
    let mut pnpm: Vec<Finding> = Vec::new();
    let mut toolchain: Vec<Finding> = Vec::new();
    for tool in tools {
        match tool {
            RelinkTool::Pnpm => {
                let began: Instant = begin_work();
                let found: Option<Finding> = repair_pnpm_awaiting(request.destination, request.environment).await;
                end_work("repairPnpm", began);
                pnpm.extend(found);
            }
            RelinkTool::Toolchain => {
                let began: Instant = begin_work();
                let found: Option<Finding> = repair_toolchain_awaiting(request.destination, request.environment).await;
                end_work("repairToolchain", began);
                toolchain.extend(found);
            }
        }
    }
    return LinksHandedOver { pnpm, toolchain };
}

/// What:
///  Take the relinking of the worktree's copied package state off the add's waiting path,
/// or do it here when no service can be told.
/// Why:
///   Both tools rewrite link layouts in place,
/// and a traced add spent 49.6 s in pnpm's relink and 11.8 s in mpm-shadow's install
/// (`doc/planning/cli-git-worktree-add-package-repair.md`),
/// so they are the service's work exactly as the Cargo build-state clone is.
/// A worktree that needs neither leaves no record,
/// and one whose tool is missing gets the line that names the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function handTheLinksOver(request): Promise<LinksHandedOver>
/// ```
async fn hand_the_links_over(request: &RepairRequest<'_>) -> LinksHandedOver {
    let mut findings: LinksHandedOver = LinksHandedOver { pnpm: Vec::new(), toolchain: Vec::new() };
    let mut tools: Vec<RelinkTool> = Vec::new();
    match crate::package_repair_pnpm::need_of(request.destination, request.environment) {
        PnpmNeed::Nothing => {}
        PnpmNeed::NoProgram(finding) => findings.pnpm.push(finding),
        PnpmNeed::Needed => tools.push(RelinkTool::Pnpm),
    }
    match crate::package_repair_toolchain::need_within_the_budget(request.destination, request.environment).await {
        Ok(ToolchainNeed::Nothing) => {}
        Ok(ToolchainNeed::NoProgram(finding)) => findings.toolchain.push(finding),
        Ok(ToolchainNeed::Needed) => tools.push(RelinkTool::Toolchain),
        Err(refusal) => findings.toolchain.push(Finding {
            tool: Tool::Toolchain,
            outcome: Outcome::Failed { reason: format!("the launcher records of the worktree could not be read ({refusal})"), run_instead: String::from(TOOLCHAIN_MANUAL_COMMAND) },
        }),
    }
    if tools.is_empty() {
        return findings;
    }
    // The add's environment travels as the three answers it can be: a path it held, no path at all, or (for a record this
    //  release reads and writes again) none recorded.
    let search_path: SearchPathOfTheAdd = match request.environment.search_path.as_ref() {
        None => SearchPathOfTheAdd::Unset,
        Some(path) => SearchPathOfTheAdd::Set(path.clone()),
    };
    let entry: RelinkEntry = RelinkEntry { root: request.destination.to_path_buf(), search_path, tools: tools.clone() };
    let told: Told = hand_over_the_record(request, &entry).await;
    match told {
        Told::Taken(millis) => {
            for tool in tools {
                let finding: Finding = handed_over(tool, millis, request.destination);
                match tool {
                    RelinkTool::Pnpm => findings.pnpm.push(finding),
                    RelinkTool::Toolchain => findings.toolchain.push(finding),
                }
            }
        }
        Told::Stays { reason, queued } => {
            debug(TAG, format!("{}: no service took the relink ({reason}), so it runs here", request.destination.display()).as_str());
            let here: LinksHandedOver = relink_here(request, tools.as_slice()).await;
            // The record goes only now: a kill before this run finished leaves it queued, and a service then relinks the
            //  worktree this record names rather than nothing at all.
            if let Some(still_queued) = queued {
                still_queued.take_back();
            }
            findings.pnpm.extend(here.pnpm);
            findings.toolchain.extend(here.toolchain);
        }
    }
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
pub(crate) async fn repair_worktree_awaiting(request: &RepairRequest<'_>, notices: &mut dyn Write) -> Vec<Finding> {
    if request.switch.no_checkout {
        debug(TAG, format!("{}: --no-checkout, so there is nothing to repair", request.destination.display()).as_str());
        return Vec::new();
    }
    let started: Instant = Instant::now();
    let owned: OwnedRequest = OwnedRequest::of_the_caller(request);
    // One wait for Cargo's repair and one for the relink, driven together on this task: both suspend rather than hold
    //  a thread, so they overlap without a task of their own.
    let links_request: RepairRequest<'_> = owned.lent();
    let cargo_request: RepairRequest<'_> = owned.lent();
    let (links, cargo): (LinksHandedOver, Vec<Finding>) = tokio::join!(hand_the_links_over(&links_request), repair_cargo_one(cargo_request));
    // The report's order has always been pnpm's findings, then Cargo's, then the toolchain's.
    let findings: Vec<Finding> = links.pnpm.into_iter().chain(cargo).chain(links.toolchain).collect();
    return report(request.destination, findings, request.switch.quiet, notices, started);
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
fn repair_without_the_runtime(request: &RepairRequest<'_>, notices: &mut dyn Write) -> Vec<Finding> {
    let started: Instant = Instant::now();
    let pnpm: Option<Finding> = repair_pnpm(request.destination, request.environment);
    let toolchain: Option<Finding> = repair_toolchain(request.destination, request.environment);
    let cargo: Vec<Finding> = repair_cargo(request);
    let findings: Vec<Finding> = pnpm.into_iter().chain(cargo).chain(toolchain).collect();
    return report(request.destination, findings, request.switch.quiet, notices, started);
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
pub fn repair_worktree(request: &RepairRequest<'_>, notices: &mut dyn Write) -> Vec<Finding> {
    if request.switch.no_checkout {
        debug(TAG, format!("{}: --no-checkout, so there is nothing to repair", request.destination.display()).as_str());
        return Vec::new();
    }
    return match run_on_runtime(repair_worktree_awaiting(request, notices)) {
        Ok(findings) => findings,
        // No runtime could be built, so the tools run in turn and each says in its own words that it could not run.
        Err(refusal) => {
            debug(TAG, format!("{}: the repairs run in turn ({refusal})", request.destination.display()).as_str());
            repair_without_the_runtime(request, notices)
        }
    };
}

/// Repair controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_tests.rs"]
mod tests;
