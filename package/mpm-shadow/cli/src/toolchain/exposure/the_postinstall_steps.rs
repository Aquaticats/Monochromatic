//! What:
//! The steps a sync runs after its installers:
//! the postinstall links (`mpm-shadow` linked as `mpm`),
//! and the postinstall commands.
//! Why:
//! A plan row may name one link to make beside the farm and one command to run,
//! and both read the programs the installers put on disk the same way the farm does;
//! they live beside the farm and not inside it,
//! so the farm's own rules stay in one file (`exposure.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // the-postinstall-steps.ts
//! export function runPostinstallLinks(ctx: Context, tools: LockedTool[]): string[]
//! export function runPostinstallCommands(ctx: Context, tools: LockedTool[]): CommandReport
//! ```

/// Imports `BTreeMap` from `std::collections`.
use std::collections::BTreeMap;
/// Imports what this module uses from `std::path`.
use std::path::{
    Path,
    PathBuf,
};

/// Imports `Context` from `super::super::context`.
use super::super::context::Context;
/// Imports `ToolchainError` from `super::super::error`.
use super::super::error::ToolchainError;
/// Imports `atomic_symlink` from `super::super::install_support`.
use super::super::install_support::atomic_symlink;
/// Imports `find_tool_bin` from `super::super::layout`.
use super::super::layout::find_tool_bin;
/// Imports what this module uses from `super::super::model`.
use super::super::model::{
    LockedTool,
    Postinstall,
};
/// Imports what this module uses from `super::super::runner`.
use super::super::runner::{
    RunOutcome,
    RunRequest,
};
/// Imports `Scope` from `super::super::scope`.
use super::super::scope::Scope;
/// Imports the farm's own answer to where a declared bin's link points,
/// and the directories its program is searched in.
use super::{
    exposed_target,
    extra_dirs,
};

/// The directory a postinstall link goes in:
/// the link's own `linkDir` (absolute,
/// or relative to the scope root),
/// or the
/// scope's link directory.
pub fn link_directory(
    scope: &Scope,
    link_dir: &Option<String>,
) -> PathBuf {
    return match link_dir {
        None => scope.link_dir.clone(),
        Some(directory) => {
            let path: &Path = Path::new(directory);
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                scope.root.join(path)
            }
        }
    };
}

/// One postinstall link that was just made.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MadeLink {
    /// The file name of the link.
    pub name: String,
    /// The full path of the link.
    pub path: PathBuf,
    /// What the link points at.
    pub target: PathBuf,
}

/// The links a pass made and the error that stopped it,
/// if any:
/// the links made before the error still exist and must be recorded.
#[derive(Debug, Default)]
pub struct MadeLinks {
    /// The links made,
    /// in plan order.
    pub made: Vec<MadeLink>,
    /// The error that ended the pass.
    pub error: Option<ToolchainError>,
}

/// What:
/// Makes each postinstall link of `tools`:
/// `<link dir>/<link name>` pointing at the tool's `sourceBin`,
/// and reports every link made even when a later one failed.
/// Why:
/// The cleanup of links the lock no longer declares needs the full path and target of each link made.
pub fn make_postinstall_links(
    ctx: &Context,
    tools: &[&LockedTool],
) -> MadeLinks {
    let mut outcome: MadeLinks = MadeLinks::default();
    for tool in tools {
        let Some(Postinstall::Link(link)) = &tool.declaration.options.postinstall else {
            continue;
        };
        let Some(found) =
            find_tool_bin(tool, &link.source_bin, ctx.scope, ctx.env, &extra_dirs(ctx))
        else {
            continue;
        };
        let target: PathBuf = match exposed_target(ctx.scope, tool, &link.source_bin, &found) {
            Ok(target) => target,
            Err(error) => {
                outcome.error = Some(error);
                return outcome;
            }
        };
        let path: PathBuf = link_directory(ctx.scope, &link.link_dir).join(&link.link_name);
        if let Err(error) = atomic_symlink(&path, &target) {
            outcome.error = Some(error);
            return outcome;
        }
        outcome.made.push(MadeLink {
            name: link.link_name.clone(),
            path,
            target,
        });
    }
    return outcome;
}

/// What:
/// Makes each postinstall link of `tools`:
/// `<link dir>/<link name>` pointing at the tool's `sourceBin`.
/// Why:
/// This is how `mpm-shadow` becomes `mpm` in a directory that comes first on `PATH`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runPostinstallLinks(ctx: Context, tools: LockedTool[]): string[]
/// ```
pub fn run_postinstall_links(
    ctx: &Context,
    tools: &[&LockedTool],
) -> Result<Vec<String>, ToolchainError> {
    let outcome: MadeLinks = make_postinstall_links(ctx, tools);
    if let Some(error) = outcome.error {
        return Err(error);
    }
    let mut names: Vec<String> = Vec::with_capacity(outcome.made.len());
    for link in outcome.made {
        names.push(link.name);
    }
    return Ok(names);
}

/// The postinstall commands' outcome:
/// tool ids that succeeded,
/// and tool id with the failure text for those that did not.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CommandReport {
    /// Tools whose command exited 0.
    pub succeeded: Vec<String>,
    /// Tools whose command failed,
    /// with what it printed.
    pub failed: Vec<(String, String)>,
}

/// What:
/// The `PATH` a postinstall command runs with:
/// the exposure directory and the toolchain bin directory first,
/// then the inherited `PATH`.
/// Why:
/// A directory name holding the path separator cannot be joined into `PATH`,
/// so the failure is returned for the caller to report instead of being thrown away.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function postinstallPath(exposure: string, toolchainBin: string, inherited?: string): string  // throws
/// ```
pub(crate) fn postinstall_path(
    exposure: &Path,
    toolchain_bin: &Path,
    inherited: Option<&str>,
) -> Result<String, std::env::JoinPathsError> {
    let mut directories: Vec<PathBuf> = vec![exposure.to_path_buf(), toolchain_bin.to_path_buf()];
    if let Some(existing) = inherited {
        directories.extend(std::env::split_paths(existing));
    }
    return std::env::join_paths(&directories).map(path_text);
}

/// One joined `PATH` entry as text.
/// A named function and not a closure:
/// nothing is captured,
/// and a reader sees where the text comes from.
fn path_text(path: std::ffi::OsString) -> String {
    return path.to_string_lossy().to_string();
}

/// What:
/// Runs each postinstall command of `tools` from the scope root,
/// with the exposure and toolchain bin
///           directories first on `PATH`.
/// Why:
/// A command such as `search-shadow link` decides its own links from the tools it finds on `PATH`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runPostinstallCommands(ctx: Context, tools: LockedTool[]): CommandReport
/// ```
pub fn run_postinstall_commands(
    ctx: &Context,
    tools: &[&LockedTool],
) -> CommandReport {
    let mut report: CommandReport = CommandReport::default();
    let joined: String = match postinstall_path(
        &ctx.scope.exposure_dir,
        &ctx.scope.toolchain_bin_dir,
        ctx.env.get("PATH"),
    ) {
        Ok(path) => path,
        Err(error) => {
            // Refusing beats running the command on the inherited PATH alone, where it would miss the tools the
            // exposure and toolchain directories link.
            for tool in tools {
                if let Some(Postinstall::Run(_)) = &tool.declaration.options.postinstall {
                    report.failed.push((
                        tool.declaration.id.clone(),
                        format!(
                            "the postinstall command was not run: the exposure and toolchain directories cannot be put on PATH ({error})"
                        ),
                    ));
                }
            }
            return report;
        }
    };
    let mut env: BTreeMap<String, String> = ctx.env.all().clone();
    env.insert("PATH".to_string(), joined);
    for tool in tools {
        let Some(Postinstall::Run(step)) = &tool.declaration.options.postinstall else {
            continue;
        };
        let Some(target) =
            find_tool_bin(tool, &step.source_bin, ctx.scope, ctx.env, &extra_dirs(ctx))
        else {
            continue;
        };
        let request: RunRequest = RunRequest {
            command: target.to_string_lossy().to_string(),
            args: step.args.clone(),
            env: Some(env.clone()),
            cwd: Some(ctx.scope.root.clone()),
        };
        let outcome: RunOutcome = ctx.run(&request);
        if outcome.succeeded() {
            report.succeeded.push(tool.declaration.id.clone());
        } else {
            report.failed.push((
                tool.declaration.id.clone(),
                format!(
                    "postinstall command failed with exit code {}: {}\n{}{}",
                    outcome.code,
                    request.line(),
                    outcome.stdout,
                    outcome.stderr
                ),
            ));
        }
    }
    return report;
}
