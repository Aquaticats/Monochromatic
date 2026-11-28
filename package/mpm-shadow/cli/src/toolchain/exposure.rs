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

/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{
    Path,
    PathBuf,
};

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
/// Imports the question which install of either toolchain root a program belongs to.
use super::launcher_source::install_below_root;
/// Imports what this module uses from `super::launcher_source`.
use super::launcher_source::{
    newest_installed_shadow,
    program_name,
    running_program,
};
/// Imports `LockedTool` from `super::model`.
use super::model::LockedTool;
/// Imports `Scope` from `super::scope`.
use super::scope::Scope;
/// Imports `file_name_text` from `super::util`.
use super::util::file_name_text;

/// What:
/// The removal of the farm's links whose names no row of the lock declares,
/// which a rebuild runs after it links the declared names.
/// Why:
/// The rule that decides a removal and the sentence it prints belong beside each other,
/// and this file keeps the linking.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { removeTheLinksNoRowDeclares } from './exposure/the-links-no-row-declares.ts';
/// ```
mod the_links_no_row_declares;
/// Imports the removal of the farm's links no row of the lock declares,
/// and what it removed and where it stopped.
use self::the_links_no_row_declares::{
    Removals,
    remove_the_links_no_row_declares,
};

/// What:
/// The instant before one removal of the farm's rebuild,
/// which a build made for controls can plant:
/// another run's removal of the same link,
/// or an `unlink` the kernel refuses.
/// Why:
/// A control has to show what a run prints and leaves in those two states,
/// and neither can be made to happen on demand by a disposable fixture.
/// A shipped build holds no plant at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { atTheInstantBeforeARemoval } from './exposure/a-control-plants-a-removal.ts';
/// ```
mod a_control_plants_a_removal;
/// Re-exports the plants a control makes for its own thread.
#[cfg(test)]
pub(crate) use self::a_control_plants_a_removal::{
    What,
    planting_at_the_removal_of,
};

/// What:
/// The order the same rebuild reads the folder's entries in,
/// which a build made for controls can turn into the reverse of name order.
/// Why:
/// A control has to be able to make the directory's order differ from name order,
/// or the sort that makes a report stable is held by the file system's own hash order and not by a
/// failing assertion.
/// A shipped build holds no plant at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readingTheFolderInReverseNameOrder } from './exposure/a-control-plants-the-order-the-folder-is-read-in.ts';
/// ```
mod a_control_plants_the_order_the_folder_is_read_in;
/// Re-exports the plant a control makes for its own thread,
/// and the two orders it can choose.
#[cfg(test)]
pub(crate) use self::a_control_plants_the_order_the_folder_is_read_in::{
    TheFoldersOrder,
    reading_the_folder_in,
};

/// The result of rebuilding the farm:
/// the names now linked,
/// the names whose program was not found,
/// and one sentence per link the rebuild removed because no row of the lock declares it.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExposureReport {
    /// Names linked in the exposure directory.
    pub linked: Vec<String>,
    /// Declared names no installed program was found for.
    pub missing: Vec<String>,
    /// What the rebuild removed from the directory,
    /// one sentence per symbolic link,
    /// naming the link,
    /// where it pointed,
    /// and why the rebuild took it away:
    /// that no row of the lock declares it,
    /// or that it was a temporary link the run could show an ended install had left.
    pub links_removed: Vec<String>,
    /// What the rebuild left in the directory,
    /// one sentence per temporary link it could not judge,
    /// naming the link and what deleting it by hand would risk.
    pub links_left: Vec<String>,
}

/// What:
/// The farm a rebuild left and the error that stopped it,
/// if any.
/// Why:
/// The links a rebuild removed before a later removal failed still left the folder a person's `PATH` reads,
/// so the run that failed must name them,
/// and an error alone would lose them (`MadeLinks` and `CommandReport` record their partial work the same way).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ExposureOutcome = { report: ExposureReport; error?: ToolchainError };
/// ```
#[derive(Debug, Default)]
pub struct ExposureOutcome {
    /// The names linked,
    /// the names missing,
    /// and the removals this run made and named.
    pub report: ExposureReport,
    /// The error that ended the rebuild,
    /// after the removals it named.
    pub error: Option<ToolchainError>,
}

/// The directories searched in addition to a route's own:
/// the scope's bin directory.
pub(super) fn extra_dirs(ctx: &Context) -> Vec<PathBuf> {
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
pub(super) fn exposed_target(
    scope: &Scope,
    tool: &LockedTool,
    name: &str,
    found: &Path,
) -> Result<PathBuf, ToolchainError> {
    if is_repository_published(tool) {
        return ensure_launcher(scope, &tool.declaration.id, name, found);
    }
    return Ok(found.to_path_buf());
}

/// What:
/// Rebuilds the farm:
/// links every declared bin of `tools`,
/// then removes the links no row of the lock declares and names each removal as it makes it.
/// Why:
/// Called after the installers ran,
/// so freshly installed programs are found,
/// and the run must be able to say which links left the folder even when a later removal fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function rebuildExposure(ctx: Context, tools: LockedTool[]): ExposureOutcome
/// ```
pub fn rebuild_exposure(
    ctx: &Context,
    tools: &[&LockedTool],
) -> ExposureOutcome {
    let mut report: ExposureReport = ExposureReport::default();
    if let Err(error) = link_the_farm(ctx, tools, &mut report) {
        return ExposureOutcome {
            report,
            error: Some(error),
        };
    }
    // What:
    // the links the farm held for names no row of the lock declares, removed and each named as it is removed.
    // Why:
    // The farm is `PATH`'s first directory, so a name an earlier row declared and this lock does not
    // would go on standing in front of the machine's own programs;
    // each removal is named where it is made, so a failure after it keeps the names of the ones already gone.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const removals = removeTheLinksNoRowDeclares(ctx);
    // report.linksRemoved = removals.sentences;
    // report.linksLeft = removals.left;
    // return { report, error: removals.error };
    // ```
    let removals: Removals = remove_the_links_no_row_declares(ctx);
    report.links_removed = removals.sentences;
    report.links_left = removals.left;
    return ExposureOutcome {
        report,
        error: removals.error,
    };
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
/// never linked to nothing,
/// and the first failure ends the pass so the report says which bin could not be linked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function linkTheFarm(ctx: Context, tools: LockedTool[], report: ExposureReport): void  // throws
/// ```
fn link_the_farm(
    ctx: &Context,
    tools: &[&LockedTool],
    report: &mut ExposureReport,
) -> Result<(), ToolchainError> {
    if let Err(error) = fs::create_dir_all(&ctx.scope.exposure_dir) {
        return Err(ToolchainError::failure(format!(
            "cannot create {}: {error}",
            ctx.scope.exposure_dir.display()
        )));
    }
    for tool in tools {
        let aliases: Vec<(String, String)> = tool
            .declaration
            .options
            .bin_aliases
            .clone()
            .unwrap_or_default();
        // What:
        // the toolchain `rustup` says this row's channel is, asked once for the whole row.
        // Why:
        // Every program of a `rustup-toolchain` row is read from that one directory,
        // so one question per row is enough, and it is `None` for every other route.
        let channel_bin: Option<PathBuf> = ctx.the_toolchains_bin_dir(tool);
        for name in &tool.declaration.bins {
            let mut primary: String = name.clone();
            for (alias, primary_name) in &aliases {
                if alias == name {
                    primary = primary_name.clone();
                    break;
                }
            }
            match ctx.find_bin_of_in(tool, &primary, &extra_dirs(ctx), channel_bin.as_deref()) {
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
    return Ok(());
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
pub fn expose_running_shadow(
    ctx: &Context,
    running: &Path,
) -> Result<Option<String>, ToolchainError> {
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

/// What:
/// The name the farm links an installed `mpm-shadow` under:
/// the program's own file name,
/// which every sentence this tool prints tells a person to type.
/// Why:
/// The farm keeps that one name while an install of the program exists,
/// so the name the rule asks about lives beside the rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_PROGRAMS_OWN_NAME = 'mpm-shadow';
/// ```
const THE_PROGRAMS_OWN_NAME: &str = "mpm-shadow";

/// What:
/// The crate this program is published as,
/// which names the directory a cargo-route install of it sits in:
/// `<toolchain root>/cargo/<crate>`.
/// Why:
/// The farm's own-name link is kept only while it resolves into an install of this program,
/// and a cargo install of it is named after the crate while a release-archive one is named after the tool,
/// so the rule compares the install's own directory against both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_PROGRAMS_CRATE = 'monochromatic-mpm-shadow';
/// ```
const THE_PROGRAMS_CRATE: &str = env!("CARGO_PKG_NAME");

/// What:
/// Whether a farm link is the one the program's own name carries and it resolves into an install of `mpm-shadow`
/// in either toolchain root.
/// Why:
/// A shell outside a checkout reaches the program only through this link,
/// so a rebuild run by a binary that belongs to no install must leave it:
/// the release removed it although an installed copy existed and the link resolved,
/// and the name then came and went with whichever binary ran `install`.
/// A link whose target no longer resolves is not kept:
/// with nothing installed there the name reaches nothing,
/// and a scope that has the program installed nowhere removes it as a link no row declares.
/// Neither is a link that resolves into another tool's install directory:
/// that is not an install of this program,
/// and the name is meant for the one that is.
/// The question asked below is which install directory the resolved file sits in,
/// never which version of the program it is,
/// so a link that resolves onto an older version of this program,
/// or onto a copy under a hidden staging or replaced name inside that install directory,
/// is kept while it resolves:
/// the name reaches this program either way,
/// the newest-only rule removes the older version when a newer one is installed,
/// and the link then resolves nowhere and the next rebuild removes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theLinkTheFarmKeeps(scope: Scope, name: string, linkPath: string): boolean
/// ```
fn the_link_the_farm_keeps(
    scope: &Scope,
    name: &str,
    link_path: &Path,
) -> bool {
    if name != THE_PROGRAMS_OWN_NAME {
        return false;
    }
    // What: where the link resolves, following every link on the way.
    // Why: the farm's link goes through the tool's `current`, which a later version move rewrites.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const resolved = canonicalize(linkPath);  // throws when the link dangles
    // ```
    let Ok(resolved) = fs::canonicalize(link_path) else {
        return false;
    };
    // What: whether the program the link resolves to is the program's own name, as a file.
    // Why: an alias a row stopped declaring resolves to the same program under another name, and must go.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (fileNameText(resolved) !== name || !resolved.isFile()) return false;
    // ```
    if file_name_text(&resolved) != name || !resolved.is_file() {
        return false;
    }
    // What: the directory of the install the resolved program belongs to, as the path below a toolchain root.
    // Why: a file of the same name that belongs to no install is not the program's own link,
    //  and neither is one inside another tool's install directory.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const installation = installBelowRoot(scope, resolved);
    // if (installation === undefined) return false;
    // ```
    let Some(installation): Option<PathBuf> = install_below_root(scope, &resolved) else {
        return false;
    };
    // What: whether that install is this program's own:
    // the tool's own directory for a release archive, or the cargo route's `cargo/<crate>`.
    // Why: the link is meant for the scope's installed program, and any other tool's install directory is not it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return installation === THE_PROGRAMS_OWN_NAME || installation === `cargo/${THE_PROGRAMS_CRATE}`;
    // ```
    return installation == Path::new(THE_PROGRAMS_OWN_NAME)
        || installation == Path::new("cargo").join(THE_PROGRAMS_CRATE);
}

/// Whether a tool of the scope's lock declares `name` as one of its bins.
fn declares_bin(
    tools: &[LockedTool],
    name: &str,
) -> bool {
    for tool in tools {
        for bin in &tool.declaration.bins {
            if bin == name {
                return true;
            }
        }
    }
    return false;
}

/// What:
/// The steps a sync runs after its installers:
/// the postinstall links and the postinstall commands.
/// Why:
/// They read the programs the installers put on disk the way the farm does,
/// and they belong beside the farm rather than inside its rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { runPostinstallLinks, runPostinstallCommands } from './exposure/the-postinstall-steps.ts';
/// ```
mod the_postinstall_steps;
/// Re-exports the postinstall step every plan row may name:
/// its link,
/// its command,
/// and what each one reported.
pub use self::the_postinstall_steps::{
    CommandReport,
    MadeLink,
    MadeLinks,
    link_directory,
    make_postinstall_links,
    run_postinstall_commands,
    run_postinstall_links,
};
/// The declared binary names of `tools` whose exposure link does not resolve.
pub fn unresolvable_bins(
    scope: &Scope,
    tools: &[&LockedTool],
) -> Vec<String> {
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
