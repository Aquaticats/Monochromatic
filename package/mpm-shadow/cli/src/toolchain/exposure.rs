//! What:
//! The exposure directory (a farm of symlinks,
//! one per declared binary,
//! that the scope puts first on
//!           `PATH`),
//! the postinstall links (`mpm-shadow` linked as `mpm`),
//! and the postinstall commands.
//! Why:
//! Installers put binaries wherever their package manager decides;
//! the farm gives every declared name one
//!           stable place,
//! and `bin` reports it to the IDE.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // exposure.ts
//! export function rebuildExposure(ctx: Context, tools: LockedTool[]): { linked: string[]; missing: string[] }
//! export function runPostinstallLinks(ctx: Context, tools: LockedTool[]): string[]
//! export function runPostinstallCommands(ctx: Context, tools: LockedTool[]): { succeeded: string[]; failed: [string, string][] }
//! export function unresolvableBins(scope: Scope, tools: LockedTool[]): string[]
//! ```

/// Imports `BTreeMap` from `std::collections`.
use std::collections::BTreeMap;
/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `is_repository_published` from `super::cargo_installs`.
use super::cargo_installs::is_repository_published;
/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `atomic_symlink` from `super::install_support`.
use super::install_support::atomic_symlink;
/// Imports `ensure_launcher` from `super::launcher`.
use super::launcher::ensure_launcher;
/// Imports what this module uses from `super::launcher_source`.
use super::launcher_source::{newest_installed_shadow, program_name, running_program};
/// Imports `find_tool_bin` from `super::layout`.
use super::layout::find_tool_bin;
/// Imports what this module uses from `super::model`.
use super::model::{LockedTool, Postinstall};
/// Imports what this module uses from `super::runner`.
use super::runner::{RunOutcome, RunRequest};
/// Imports `Scope` from `super::scope`.
use super::scope::Scope;

/// The result of rebuilding the farm:
/// the names now linked and the names whose program was not found.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExposureReport {
    /// Names linked in the exposure directory.
    pub linked: Vec<String>,
    /// Declared names no installed program was found for.
    pub missing: Vec<String>,
}

/// The directories searched in addition to a route's own:
/// the scope's bin directory.
fn extra_dirs(ctx: &Context) -> Vec<PathBuf> {
    return vec![ctx.scope.toolchain_bin_dir.clone()];
}

/// What:
/// What an exposed link points at:
/// the real program,
/// or (for a repository-published tool) this tree's launcher for it.
/// Why:
/// The launcher is what warns on every run when the tree's lock pins an older version than the one that runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function exposedTarget(scope: Scope, tool: LockedTool, name: string, found: string): string
/// ```
fn exposed_target(scope: &Scope, tool: &LockedTool, name: &str, found: &Path) -> Result<PathBuf, ToolchainError> {
    if is_repository_published(tool) {
        return ensure_launcher(scope, &tool.declaration.id, name, found);
    }
    return Ok(found.to_path_buf());
}

/// What:
/// Links every declared bin of `tools` in the exposure directory:
/// the program is searched under its route's
///           directories,
/// aliases link their primary's program,
/// and the running `mpm-shadow` is linked under its own name when no tool declares it.
/// Why:
/// Called after the installers ran,
/// so freshly installed programs are found;
/// a name with no program stays
///           missing and is reported,
/// never linked to nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function rebuildExposure(ctx: Context, tools: LockedTool[]): ExposureReport
/// ```
pub fn rebuild_exposure(ctx: &Context, tools: &[&LockedTool]) -> Result<ExposureReport, ToolchainError> {
    if let Err(error) = fs::create_dir_all(&ctx.scope.exposure_dir) {
        return Err(ToolchainError::failure(format!("cannot create {}: {error}", ctx.scope.exposure_dir.display())));
    }
    let mut report: ExposureReport = ExposureReport::default();
    for tool in tools {
        let aliases: Vec<(String, String)> = tool.declaration.options.bin_aliases.clone().unwrap_or_default();
        for name in &tool.declaration.bins {
            let mut primary: String = name.clone();
            for (alias, primary_name) in &aliases {
                if alias == name {
                    primary = primary_name.clone();
                    break;
                }
            }
            match find_tool_bin(tool, &primary, ctx.scope, ctx.env, &extra_dirs(ctx)) {
                Some(found) => {
                    let target: PathBuf = exposed_target(ctx.scope, tool, name, &found)?;
                    atomic_symlink(&ctx.scope.exposure_dir.join(name), &target)?;
                    report.linked.push(name.clone());
                }
                None => report.missing.push(name.clone()),
            }
        }
    }
    if let Some(name) = expose_running_shadow(ctx, &running_program()?)? {
        report.linked.push(name);
    }
    return Ok(report);
}

/// What:
/// The farm link that makes the running `mpm-shadow` reachable under its own name,
/// or `None` when there is nothing to link.
/// Why:
/// The sentences `mpm-shadow` prints name it as the program a person runs:
/// the machine-global older-lock sentence (`mpm-shadow update -g <tool id>`),
/// the launcher's advice when a program under `current` is missing (`mpm-shadow install -g`),
/// and `mpm-shadow --help` in the shim's help.
/// A shell outside a checkout reaches only this scope's farm,
/// so a machine whose global plan declares no `mpm-shadow` entry left every one of those commands unrunnable
/// (measured 2026-11-11 in a home laid out like this machine's:
/// `/bin/sh: line 1: mpm-shadow: command not found`,
/// exit status 127).
/// The link names the installed program,
/// never the running file,
/// so a run from a build folder puts no build on the machine's `PATH`,
/// and it goes through `current`,
/// so a later install that removes the version it names never leaves it dangling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function exposeRunningShadow(ctx: Context, running: string): string | undefined
/// ```
pub fn expose_running_shadow(ctx: &Context, running: &Path) -> Result<Option<String>, ToolchainError> {
    let Some(installed) = newest_installed_shadow(ctx.scope, running) else {
        return Ok(None);
    };
    let Some(program) = program_name(&installed) else {
        return Ok(None);
    };
    let name: String = program.to_string_lossy().to_string();
    // A plan row that declares this bin owns the farm link and the launcher record behind it,
    // which names that tool's id; a second link here would take the name over with a record no lock row knows.
    if declares_bin(ctx.tools, &name) {
        return Ok(None);
    }
    atomic_symlink(&ctx.scope.exposure_dir.join(&name), &installed)?;
    return Ok(Some(name));
}

/// Whether a tool of the scope's lock declares `name` as one of its bins.
fn declares_bin(tools: &[LockedTool], name: &str) -> bool {
    for tool in tools {
        for bin in &tool.declaration.bins {
            if bin == name {
                return true;
            }
        }
    }
    return false;
}

/// The directory a postinstall link goes in:
/// the link's own `linkDir` (absolute,
/// or relative to the scope root),
/// or the
/// scope's link directory.
pub fn link_directory(scope: &Scope, link_dir: &Option<String>) -> PathBuf {
    return match link_dir {
        None => scope.link_dir.clone(),
        Some(directory) => {
            let path: &Path = Path::new(directory);
            if path.is_absolute() { path.to_path_buf() } else { scope.root.join(path) }
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
pub fn make_postinstall_links(ctx: &Context, tools: &[&LockedTool]) -> MadeLinks {
    let mut outcome: MadeLinks = MadeLinks::default();
    for tool in tools {
        let Some(Postinstall::Link(link)) = &tool.declaration.options.postinstall else {
            continue;
        };
        let Some(found) = find_tool_bin(tool, &link.source_bin, ctx.scope, ctx.env, &extra_dirs(ctx)) else {
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
        outcome.made.push(MadeLink { name: link.link_name.clone(), path, target });
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
pub fn run_postinstall_links(ctx: &Context, tools: &[&LockedTool]) -> Result<Vec<String>, ToolchainError> {
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
pub fn run_postinstall_commands(ctx: &Context, tools: &[&LockedTool]) -> CommandReport {
    let mut report: CommandReport = CommandReport::default();
    let joined: String = match postinstall_path(&ctx.scope.exposure_dir, &ctx.scope.toolchain_bin_dir, ctx.env.get("PATH")) {
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
        let Some(target) = find_tool_bin(tool, &step.source_bin, ctx.scope, ctx.env, &extra_dirs(ctx)) else {
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

/// The declared binary names of `tools` whose exposure link does not resolve.
pub fn unresolvable_bins(scope: &Scope, tools: &[&LockedTool]) -> Vec<String> {
    let mut unresolved: Vec<String> = Vec::new();
    for tool in tools {
        for name in &tool.declaration.bins {
            if !scope.exposure_dir.join(name).exists() {
                unresolved.push(name.clone());
            }
        }
    }
    return unresolved;
}
