//! What:
//! The symbolic links a rebuild of the exposure directory removes:
//! one link per name no row of the lock declares,
//! and nothing else in that directory.
//! Why:
//! A name an earlier row declared and this lock does not would go on standing first on `PATH`,
//! and the sentence one removal prints belongs beside the rule that decides it;
//! `exposure.rs` keeps the linking and this file the taking away.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function removeTheLinksNoRowDeclares(ctx: Context, alsoDeclared: string[]): string[]  // throws
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports `PathBuf` from `std::path`,
/// which is the owned path type (its sibling `Path` is the borrowed one).
/// Why:
/// A sentence holds the link's target past the directory entry it was read from,
/// which a borrowed path could not outlive.
use std::path::PathBuf;

/// Imports `Context` from `super::super::context`.
use super::super::context::Context;
/// Imports `ToolchainError` from `super::super::error`.
use super::super::error::ToolchainError;
/// Imports `declares_bin` from the module that links the farm.
use super::declares_bin;
/// Imports the instant a control can plant at one removal,
/// which a shipped build never holds.
use super::a_control_plants_a_removal::at_the_instant_before_a_removal;

/// What:
/// Removes every symbolic link in the exposure directory whose name no row of the lock declares,
/// and answers one sentence per link it removed,
/// naming the link and where it pointed.
/// Why:
/// A row that stops declaring a bin,
/// as the `rust` row stopped declaring `rustup` (issue 6626),
/// left its link in the farm;
/// the link then dangled once the program behind it went,
/// `list` said the toolchain was in sync,
/// and nothing said the name was still there (issue 6634's Low point 3).
/// A regular file,
/// a directory or anything else at such a name belongs to a person and stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removeTheLinksNoRowDeclares(ctx: Context, alsoDeclared: string[]): string[]  // throws
/// ```
pub(super) fn remove_the_links_no_row_declares(
    ctx: &Context,
    also_declared: &[String],
) -> Result<Vec<String>, ToolchainError> {
    // What:
    // the directory's entries, or the error naming the directory that could not be read.
    // Why:
    // A farm that cannot be listed leaves every stale link in place,
    // and the run has to say so rather than report a clean pass.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const entries = readdirSync(ctx.scope.exposureDir);
    // ```
    let entries = match fs::read_dir(&ctx.scope.exposure_dir) {
        Ok(entries) => entries,
        Err(error) => {
            return Err(ToolchainError::failure(format!(
                "cannot read {}: {error}",
                ctx.scope.exposure_dir.display()
            )));
        }
    };
    // What:
    // the name and target of each link to remove, collected before any sentence is built.
    // Why:
    // A directory's entries come in no order,
    // and the report reads the same on every run only if the sentences are sorted by name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const orphans: [string, string][] = [];
    // ```
    let mut orphans: Vec<(String, PathBuf)> = Vec::new();
    for listed in entries {
        // What:
        // one entry of the directory, or the error naming the directory.
        // Why:
        // An error here is the same unreadable-directory state as above.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const entry = entries.next().value;
        // ```
        let entry = match listed {
            Ok(entry) => entry,
            Err(error) => {
                return Err(ToolchainError::failure(format!(
                    "cannot read {}: {error}",
                    ctx.scope.exposure_dir.display()
                )));
            }
        };
        // What:
        // the entry's own name as text.
        // Why:
        // The name is what the lock's rows are compared against.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const name = entry.name;
        // ```
        let name: String = entry.file_name().to_string_lossy().to_string();
        if declares_bin(ctx.tools, &name)
            || also_declared
                .iter()
                .any(|declared| return declared == &name)
        {
            continue;
        }
        // What:
        // the entry's own metadata, read without following the link.
        // Why:
        // Only a symbolic link is this tool's to remove;
        // a file or a directory there is a person's.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const metadata = lstatSync(entry.path);
        // ```
        let metadata = match fs::symlink_metadata(entry.path()) {
            Ok(metadata) => metadata,
            Err(error) => {
                return Err(ToolchainError::failure(format!(
                    "cannot read {}: {error}",
                    entry.path().display()
                )));
            }
        };
        if !metadata.file_type().is_symlink() {
            continue;
        }
        // What:
        // where the link points,
        // read before it is removed so the sentence can say.
        // Why:
        // A dangling link still records its target,
        // and that text is how a person recognises what left it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const target = readlinkSync(entry.path);
        // ```
        let target: PathBuf = match fs::read_link(entry.path()) {
            Ok(target) => target,
            Err(error) => {
                return Err(ToolchainError::failure(format!(
                    "cannot read the link {}: {error}",
                    entry.path().display()
                )));
            }
        };
        // What:
        // the step a control plants where the kernel's own `unlink` answers,
        // or the `unlink` itself in a shipped build.
        // Why:
        // A control shows what the rebuild leaves when the link it came to remove is already gone,
        // and when one removal fails after earlier ones succeeded.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const planted = atTheInstantBeforeARemoval(name, entry.path());
        // ```
        let planted: Option<std::io::Error> =
            at_the_instant_before_a_removal(&name, &entry.path());
        // What:
        // the removal's own answer: the planted error, or the `unlink`'s.
        // Why:
        // Both are the same failure for the caller,
        // so the failure arm below is written once.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const removal = planted ?? unlinkSync(entry.path());
        // ```
        let removal: std::io::Result<()> = match planted {
            Some(error) => Err(error),
            None => fs::remove_file(entry.path()),
        };
        if let Err(error) = removal {
            return Err(ToolchainError::failure(format!(
                "cannot remove the exposure link {}: {error}",
                entry.path().display()
            )));
        }
        orphans.push((name, target));
    }
    orphans.sort();
    // What:
    // one sentence per removed link, in name order.
    // Why:
    // The install's report says which names it took out of the folder a person's `PATH` reads.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return orphans.sort().map(([name, target]) => `removed the exposure link ${name} (it pointed at ${target}), which no row of the lock declares`);
    // ```
    let mut sentences: Vec<String> = Vec::new();
    for (name, target) in orphans {
        sentences.push(format!(
            "removed the exposure link {name} (it pointed at {}), which no row of the lock declares",
            target.display()
        ));
    }
    return Ok(sentences);
}
