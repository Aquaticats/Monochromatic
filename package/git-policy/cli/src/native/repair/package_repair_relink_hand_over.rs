//! What:
//!  Handing the relinking of a new worktree's copied package state to the commit service,
//! and doing it here when no service takes the record:
//! which tools the worktree needs,
//! the record and the wake that ask the service for them,
//! and the line the add answers when the service holds them.
//! Why:
//!  Both tools rewrite link layouts in place,
//! and a traced add spent 49.6 s in pnpm's relink and 11.8 s in mpm-shadow's install,
//! so they are the service's work exactly as the Cargo build-state clone is.
//! A worktree that needs neither leaves no record,
//! one whose tool is missing gets the line that names the command,
//! and one whose record no service took is relinked here,
//! so the person's next command in the worktree never reads links that name the source.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const links = await handTheLinksOver(request); // { pnpm: Finding[]; toolchain: Finding[] }
//! ```

/// Import the timing record the hand-over and each tool's run are kept in.
use crate::add_timing::{
    begin_work,
    end_work,
};
/// Import the refusal a job of the blocking pool answers with.
use crate::blocking_job::BlockingJobError;
/// Import the runner that does a blocking job on the pool or in this thread.
use crate::blocking_jobs_within_the_budget::run_a_job_or_in_turn;
/// Import the one diagnostic log this crate writes.
use crate::diagnostic_log::debug;
/// Import what the pnpm repair needs and the repair itself.
use crate::package_repair_pnpm::{
    PnpmNeed,
    repair_pnpm_awaiting,
};
/// Import the directory the repository's Git answers with,
/// which a record is written beside.
use crate::package_repair_prewarm::common_directory_awaiting;
/// Import the relink request,
/// its tools,
/// the record a service is given,
/// and how the record is written and woken.
use crate::package_repair_relink::{
    NotTaken,
    QueuedRelink,
    RelinkEntry,
    RelinkTool,
    SearchPathOfTheAdd,
    hand_over_in_a_job,
};
/// Import what a repair reports.
use crate::package_repair_report::{
    Finding,
    Outcome,
    Tool,
};
/// Import what a repair reads and is asked.
use crate::package_repair_request::RepairRequest;
/// Import what the toolchain repair needs,
/// the repair itself,
/// and the command a person runs instead.
use crate::package_repair_toolchain::{
    MANUAL_COMMAND as TOOLCHAIN_MANUAL_COMMAND,
    ToolchainNeed,
    repair_toolchain_awaiting,
};
/// Import the borrowed and owned filesystem paths a record and a line name.
use std::path::{
    Path,
    PathBuf,
};
/// Import the clock that times each tool's run.
use std::time::Instant;
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
pub(super) struct LinksHandedOver {
    /// pnpm's findings.
    pub(super) pnpm: Vec<Finding>,
    /// mpm-shadow's findings.
    pub(super) toolchain: Vec<Finding>,
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
fn handed_over(
    tool: RelinkTool,
    millis: u128,
    destination: &Path,
) -> Finding {
    let (as_report, detail): (Tool, &str) = match tool {
        RelinkTool::Pnpm => (Tool::Pnpm, "pnpm will relink the copied node_modules here"),
        RelinkTool::Toolchain => (
            Tool::Toolchain,
            "mpm-shadow will rebuild the toolchain links here",
        ),
    };
    debug(
        TAG,
        format!(
            "{}: {} is the service's work",
            destination.display(),
            tool.name()
        )
        .as_str(),
    );
    return Finding {
        tool: as_report,
        outcome: Outcome::Repaired {
            detail: format!(
                "{detail}, because the commit service runs it after this command returns"
            ),
            millis,
        },
    };
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
    Stays {
        reason: String,
        queued: Option<QueuedRelink>,
    },
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
async fn hand_over_the_record(
    request: &RepairRequest<'_>,
    entry: &RelinkEntry,
) -> Told {
    let began: Instant = begin_work();
    let common_dir: Result<PathBuf, String> =
        common_directory_awaiting(request.git, request.destination).await;
    let told: Result<Result<(), NotTaken>, BlockingJobError> = match common_dir {
        Ok(common) => {
            let state: crate::package_repair_relink::TheHandOver = (
                request.git.clone(),
                common,
                request.destination.to_path_buf(),
                entry.clone(),
                request.environment.start_directory.clone(),
            );
            run_a_job_or_in_turn(state, hand_over_in_a_job).await
        }
        Err(reason) => Ok(Err(NotTaken {
            reason,
            queued: None,
        })),
    };
    end_work("relinkHandOver", began);
    let millis: u128 = began.elapsed().as_millis();
    return match told {
        Ok(Ok(())) => Told::Taken(millis),
        Ok(Err(not_taken)) => Told::Stays {
            reason: not_taken.reason,
            queued: not_taken.queued,
        },
        Err(refusal) => Told::Stays {
            reason: format!("{refusal}"),
            queued: None,
        },
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
async fn relink_here(
    request: &RepairRequest<'_>,
    tools: &[RelinkTool],
) -> LinksHandedOver {
    let mut pnpm: Vec<Finding> = Vec::new();
    let mut toolchain: Vec<Finding> = Vec::new();
    for tool in tools {
        match tool {
            RelinkTool::Pnpm => {
                let began: Instant = begin_work();
                let found: Option<Finding> =
                    repair_pnpm_awaiting(request.destination, request.environment).await;
                end_work("repairPnpm", began);
                pnpm.extend(found);
            }
            RelinkTool::Toolchain => {
                let began: Instant = begin_work();
                let found: Option<Finding> =
                    repair_toolchain_awaiting(request.destination, request.environment).await;
                end_work("repairToolchain", began);
                toolchain.extend(found);
            }
        }
    }
    return LinksHandedOver {
        pnpm,
        toolchain,
    };
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
pub(super) async fn hand_the_links_over(request: &RepairRequest<'_>) -> LinksHandedOver {
    let mut findings: LinksHandedOver = LinksHandedOver {
        pnpm: Vec::new(),
        toolchain: Vec::new(),
    };
    let mut tools: Vec<RelinkTool> = Vec::new();
    match crate::package_repair_pnpm::need_of(request.destination, request.environment) {
        PnpmNeed::Nothing => {}
        PnpmNeed::NoProgram(finding) => findings.pnpm.push(finding),
        PnpmNeed::Needed => tools.push(RelinkTool::Pnpm),
    }
    match crate::package_repair_toolchain::need_within_the_budget(
        request.destination,
        request.environment,
    )
    .await
    {
        Ok(ToolchainNeed::Nothing) => {}
        Ok(ToolchainNeed::NoProgram(finding)) => findings.toolchain.push(finding),
        Ok(ToolchainNeed::Needed) => tools.push(RelinkTool::Toolchain),
        Err(refusal) => findings.toolchain.push(Finding {
            tool: Tool::Toolchain,
            outcome: Outcome::Failed {
                reason: format!(
                    "the launcher records of the worktree could not be read ({refusal})"
                ),
                run_instead: String::from(TOOLCHAIN_MANUAL_COMMAND),
            },
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
    let entry: RelinkEntry = RelinkEntry {
        root: request.destination.to_path_buf(),
        search_path,
        tools: tools.clone(),
    };
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
        Told::Stays {
            reason,
            queued,
        } => {
            debug(
                TAG,
                format!(
                    "{}: no service took the relink ({reason}), so it runs here",
                    request.destination.display()
                )
                .as_str(),
            );
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

/// Tag used for diagnostics from this module.
pub(super) const TAG: &str = "packageRepair";
