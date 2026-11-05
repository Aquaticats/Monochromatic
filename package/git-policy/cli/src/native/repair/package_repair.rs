//! What:
//!  The repair of a new worktree's package-manager state: after a rung filled the worktree and the ignored files were
//!  copied, run each tool's own offline repair so the state it keeps for the old location works at the new one.
//! Why:
//!  A copy keeps links and build state exactly as they were. pnpm's links climb out of the tree by a fixed number of
//!  levels, Cargo's build state bakes in the source's paths, and the toolchain launchers name the source's lock. Each tool
//!  knows how to rebuild its own layout, so the wrapper starts it (the owner's exemption, 2026-11-03) and never edits
//!  its files itself. A repair never fails the add: it ends in a line that names the command to run, and a tool the
//!  environment does not provide is a verdict, not an error. Another process changing the files while they are copied or
//!  repaired is the person's risk and is not defended against.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! repairWorktree({ git, source, destination, switch: { quiet, noCheckout }, environment }, process.stderr);
//! ```

use crate::scoped_start::start_or_run;
use crate::add_timing::work;
use crate::diagnostic_log::debug;
use crate::package_repair_cargo::repair_cargo;
use crate::package_repair_pnpm::repair_pnpm;
/// What the repairs read, who asks, and the switches of the person; kept in a file of their own and named here so the paths
///  outside this folder stay the same.
pub use crate::package_repair_request::{RepairEnvironment, RepairRequest, RepairSwitch, workers_for};
use crate::package_repair_report::{Finding, render};
use crate::package_repair_toolchain::repair_toolchain;
use std::io::Write;

/// Tag used for diagnostics from this module.
const TAG: &str = "packageRepair";

/// What:
///  A finished tool thread's finding, or no finding when the tool panicked.
///  `std::thread::Result<Option<Finding>>` is "what the thread returned, or the panic it died of".
/// Why:
///   A tool that panicked has no finding; the add goes on and the worktree stays as the copy left it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function settled(joined: Finding | undefined | Error, tool: string): Finding | undefined;
/// ```
fn settled(joined: std::thread::Result<Option<Finding>>, tool: &str) -> Option<Finding> {
    return match joined {
        Ok(finding) => finding,
        Err(_panic) => {
            debug(TAG, format!("the {tool} repair stopped unexpectedly and reports nothing").as_str());
            None
        }
    };
}

/// What:
///  Run the pnpm, toolchain and Cargo repairs side by side inside one thread scope and return their findings.
/// Why:
///   The tools are independent, and each borrows the request for the length of the scope, which a named function
///  cannot express. A thread that cannot start runs its tool in turn (`start_or_run`); the Cargo repair runs on the
///  calling thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [pnpm, cargo, toolchain] = await Promise.all([repairPnpm(request), repairCargo(request), repairToolchain(request)]);
/// ```
#[cfg_attr(
    dylint_lib = "monochromatic_lint_closure",
    expect(
        avoidable_closure,
        reason = "std::thread::scope, start_or_run and work (add_timing.rs) take closures that borrow the request for the scope only, not 'static; see doc/planning/cli-git-closure-lint-suppressions.md"
    )
)]
fn repair_side_by_side(request: &RepairRequest<'_>) -> (Option<Finding>, Vec<Finding>, Option<Finding>) {
    return std::thread::scope(|scope| {
        let pnpm = start_or_run(scope, || { return work("repairPnpm", || { return repair_pnpm(request.destination, request.environment); }); });
        let toolchain = start_or_run(scope, || { return work("repairToolchain", || { return repair_toolchain(request.destination, request.environment); }); });
        let cargo: Vec<Finding> = work("repairCargo", || { return repair_cargo(request); });
        return (settled(pnpm.join(), "pnpm"), cargo, settled(toolchain.join(), "toolchain"));
    });
}

/// What:
///  Repair the package-manager state of one new worktree and print what the person needs to know.
/// Why:
///   The tools are independent, so they run side by side; the add ends when the slowest does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await repairWorktree(request, stderr);
/// ```
pub fn repair_worktree(request: &RepairRequest<'_>, notices: &mut dyn Write) -> Vec<Finding> {
    if request.switch.no_checkout {
        debug(TAG, format!("{}: --no-checkout, so there is nothing to repair", request.destination.display()).as_str());
        return Vec::new();
    }
    let started: std::time::Instant = std::time::Instant::now();
    let (pnpm, cargo, toolchain): (Option<Finding>, Vec<Finding>, Option<Finding>) = repair_side_by_side(request);
    let findings: Vec<Finding> = pnpm.into_iter().chain(cargo).chain(toolchain).collect();
    debug(TAG, format!("{}: {} findings in {} ms", request.destination.display(), findings.len(), started.elapsed().as_millis()).as_str());
    let text: String = render(request.destination, findings.as_slice(), request.switch.quiet);
    // A closed standard error leaves nowhere to say it; the worktree is whole either way.
    let _written = notices.write_all(text.as_bytes());
    return findings;
}

/// Repair controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_tests.rs"]
mod tests;
