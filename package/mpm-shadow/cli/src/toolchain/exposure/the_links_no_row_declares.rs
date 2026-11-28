//! What:
//! The symbolic links a rebuild of the exposure directory removes:
//! one link per name no row of the lock declares,
//! and nothing else in that directory.
//! Why:
//! A name an earlier row declared and this lock does not would go on standing first on `PATH`,
//! and the sentence one removal prints belongs beside the rule that decides it;
//! `exposure.rs` keeps the linking and this file the taking away.
//!
//! Which names stay,
//! and why:
//! a name a row of the lock declares is the plan's;
//! the program's own link is the one a shell outside a checkout reaches `mpm-shadow` through,
//! and it stays while it resolves into an installed copy (`exposure.rs`,
//! `the_link_the_farm_keeps`);
//! a temporary link an install that may still run holds is that install's and stays,
//! because taking it would fail the rename that gives the link its name
//! (`install_support`,
//! `the_temporary_link_of_an_installer`);
//! and everything else at such a name is not a symbolic link,
//! so a file,
//! a directory or a hard link is a person's.
//! A name this run has just linked is never removed for a third reason:
//! it is either a name the lock declares or the program's own link,
//! which the two rules above already keep.
//! A link another run removed or renamed in the instant before this one reaches it is no failure:
//! the removal reads an entry's own metadata,
//! then the link's target,
//! and then removes it,
//! and a name that any of those three calls no longer finds is gone,
//! which is what this run wanted,
//! and the run that took it named it.
//! A temporary link this run cannot judge is kept and named once,
//! up to the point the pass stops:
//! it may belong to an install that still runs and that no run here can see,
//! so the report says which name stayed and what deleting it by hand would cost,
//! as `prune --toolchain` says of a hidden copy it cannot judge.
//! A removal that fails stops the pass,
//! and so does a read of an entry,
//! its metadata or its target,
//! so a rebuild that stops there names the names it reached before that failure
//! and the next install names the ones after it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function removeTheLinksNoRowDeclares(ctx: Context): { sentences: string[]; error?: Error };
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports `PathBuf` from `std::path`,
/// which is the owned path type (its sibling `Path` is the borrowed one).
/// Why:
/// A sentence holds the link's target past the directory entry it was read from,
/// which a borrowed path could not outlive.
use std::path::PathBuf;

/// Imports what an install that still runs says about a temporary link,
/// which is the one question that tells such a link from a leftover.
use super::super::archive_staging::Liveness;
/// Imports `Context` from `super::super::context`.
use super::super::context::Context;
/// Imports `ToolchainError` from `super::super::error`.
use super::super::error::ToolchainError;
/// Imports the question what a temporary link's name says about the install that holds it.
use super::super::install_support::liveness_of_the_installer;
/// Imports the three instants of one removal a control can plant at,
/// which a shipped build never holds.
use super::a_control_plants_a_removal::{
    at_the_instant_before_a_removal,
    at_the_instant_before_the_link_is_read,
    at_the_instant_before_the_target_is_read,
};
/// Imports the order a control can give the folder's entries,
/// which a shipped build answers with the directory's own.
use super::a_control_plants_the_order_the_folder_is_read_in::in_the_order_it_is_read;
/// Imports `declares_bin` from the module that links the farm.
use super::declares_bin;
/// Imports the question whether a link is the farm's own link to an installed program.
use super::the_link_the_farm_keeps;

/// What:
/// The links one pass removed and the error that stopped it,
/// if any:
/// the links removed before the error still left the folder and are named.
/// Also the temporary links it could not judge and left where they are.
/// Why:
/// A sentence is built when its link is removed,
/// so a removal that fails later cannot lose the names of the ones already gone,
/// and a kept name has to be said once or nothing tells a person the folder holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Removals = { sentences: string[]; left: string[]; error?: Error };
/// ```
#[derive(Debug, Default)]
pub(super) struct Removals {
    /// One sentence per link removed,
    /// in name order.
    pub(super) sentences: Vec<String>,
    /// One sentence per temporary link this pass could not judge,
    /// in name order.
    pub(super) left: Vec<String>,
    /// The error that ended the pass.
    pub(super) error: Option<ToolchainError>,
}

/// What:
/// One link the pass took out of the folder,
/// with where it pointed and whether the name was a held link's the run could show an ended install had left.
/// Why:
/// The sentence says which of the two a name was,
/// and both are read before the removal,
/// when the link still answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ARemoval = { name: string; target: string; temporary: boolean };
/// ```
struct ARemoval {
    /// The name the link had in the folder.
    name: String,
    /// Where the link pointed.
    target: PathBuf,
    /// Whether the name was a temporary link's.
    temporary: bool,
}

/// What:
/// Removes every symbolic link in the exposure directory whose name no row of the lock declares,
/// answers one sentence per link it removed while it removes them,
/// names each temporary link it could not judge and leaves that one,
/// and stops at the first removal that fails.
/// Why:
/// A row that stops declaring a bin,
/// as the `rust` row stopped declaring `rustup` (issue 6626),
/// left its link in the farm;
/// the link then dangled once the program behind it went,
/// `list` said the toolchain was in sync,
/// and nothing said the name was still there (issue 6634's Low point 3).
/// A regular file,
/// a directory or anything else at such a name belongs to a person and stays,
/// and a temporary link an install that may still run holds is that install's and stays too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removeTheLinksNoRowDeclares(ctx: Context): Removals
/// ```
pub(super) fn remove_the_links_no_row_declares(ctx: &Context) -> Removals {
    // What:
    // the directory's entries, collected so a control can give them an order of its own,
    // or the error naming the directory that could not be read.
    // Why:
    // A farm that cannot be listed leaves every stale link in place,
    // and the run has to say so rather than report a clean pass;
    // the entries are collected because the order a report is stable in is asserted by a control whose
    // folder has to disagree with name order, which a lazy read of the directory cannot be given.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const entries = readdirSync(ctx.scope.exposureDir).map(Ok);
    // ```
    let entries = match fs::read_dir(&ctx.scope.exposure_dir) {
        Ok(read) => read.collect::<Vec<std::io::Result<fs::DirEntry>>>(),
        Err(error) => {
            return Removals {
                sentences: Vec::new(),
                left: Vec::new(),
                error: Some(ToolchainError::failure(format!(
                    "cannot read {}: {error}",
                    ctx.scope.exposure_dir.display()
                ))),
            };
        }
    };
    // What:
    // the same entries in the order this run reads them.
    // Why:
    // A control's folder comes in the reverse of name order and a shipped build's as the directory
    // lists it, so the loop below needs one answer for both.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const ordered = inTheOrderItIsRead(entries);
    // ```
    let ordered = in_the_order_it_is_read(entries);
    // What:
    // the links removed so far, each recorded as it is removed.
    // Why:
    // The report reads the same on every run only if the sentences are sorted by name,
    // and a failure after some removals must still name them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const removed: ARemoval[] = [];
    // ```
    let mut removed: Vec<ARemoval> = Vec::new();
    // What:
    // the temporary links this pass could not judge, which stay in the folder and are named once.
    // Why:
    // Such a link may belong to an install that still runs in a process namespace this run cannot see,
    // so leaving it silently would keep a name in a folder a person reads and say nothing about it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const left: string[] = [];
    // ```
    let mut left: Vec<String> = Vec::new();
    // What:
    // the error that ended the pass,
    // which is answered after the removals already made are named.
    // Why:
    // One failure ends the pass, and the sentences built before it are what a person reads.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let error: Error | undefined;
    // ```
    let mut error: Option<ToolchainError> = None;
    for listed in ordered {
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
            Err(failure) => {
                error = Some(ToolchainError::failure(format!(
                    "cannot read {}: {failure}",
                    ctx.scope.exposure_dir.display()
                )));
                break;
            }
        };
        // What:
        // the entry's own name as text.
        // Why:
        // The name is what the lock's rows and the temporary shape are compared against.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const name = entry.name;
        // ```
        let name: String = entry.file_name().to_string_lossy().to_string();
        if declares_bin(ctx.tools, &name) {
            continue;
        }
        // What:
        // the step a control plants where the kernel's own `lstat` answers,
        // or nothing in a shipped build.
        // Why:
        // Another install may take the name between this run's listing of the folder and this read,
        // and a control has to be able to put that instant here rather than only before the removal.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // atTheInstantBeforeTheLinkIsRead(name, entry.path);
        // ```
        at_the_instant_before_the_link_is_read(&name, &entry.path());
        // What:
        // the entry's own metadata, read without following the link.
        // Why:
        // Only a symbolic link is this tool's to remove;
        // a file or a directory there is a person's,
        // and a name another run just took is already gone rather than a failure.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const metadata = lstatSync(entry.path, { throwIfNoEntry: false });
        // ```
        let metadata = match fs::symlink_metadata(entry.path()) {
            Ok(metadata) => metadata,
            Err(failure) if failure.kind() == std::io::ErrorKind::NotFound => continue,
            Err(failure) => {
                error = Some(ToolchainError::failure(format!(
                    "cannot read {}: {failure}",
                    entry.path().display()
                )));
                break;
            }
        };
        if !metadata.file_type().is_symlink() {
            continue;
        }
        // What:
        // what the name says about the install that holds it, when the name is a temporary link's.
        // Why:
        // A second install rebuilding the farm at this instant must not take a link an install that still runs is
        // about to rename,
        // and a name nothing can be concluded about may belong to such an install,
        // so only a name the run can show an ended install had held is removed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const installer = livenessOfTheInstaller(name);
        // if (installer === 'running') continue;
        // if (installer === 'unknown') { left.push(name); continue; }
        // ```
        let installer: Option<Liveness> = liveness_of_the_installer(&name);
        if installer == Some(Liveness::Running) {
            continue;
        }
        if installer == Some(Liveness::Unknown) {
            left.push(name.clone());
            continue;
        }
        // What:
        // the step a control plants where the kernel's own `readlink` answers,
        // or nothing in a shipped build.
        // Why:
        // The metadata read and this one are two calls,
        // so another install can take the name in the gap between them,
        // and the control has to reach that gap.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // atTheInstantBeforeTheTargetIsRead(name, entry.path);
        // ```
        at_the_instant_before_the_target_is_read(&name, &entry.path());
        // What:
        // where the link points,
        // read before it is removed so the sentence can say.
        // Why:
        // A dangling link still records its target,
        // and that text is how a person recognises what left it;
        // a name another run took in the gap is gone rather than a failure.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const target = readlinkSync(entry.path);
        // ```
        let target: PathBuf = match fs::read_link(entry.path()) {
            Ok(target) => target,
            Err(failure) if failure.kind() == std::io::ErrorKind::NotFound => continue,
            Err(failure) => {
                error = Some(ToolchainError::failure(format!(
                    "cannot read the link {}: {failure}",
                    entry.path().display()
                )));
                break;
            }
        };
        if the_link_the_farm_keeps(ctx.scope, &name, &entry.path()) {
            continue;
        }
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
        let planted: Option<std::io::Error> = at_the_instant_before_a_removal(&name, &entry.path());
        // What:
        // the removal's own answer: the planted error, or the `unlink`'s.
        // Why:
        // Both are the same answer for the caller,
        // so the arms below are written once.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const removal = planted ?? unlinkSync(entry.path());
        // ```
        let removal: std::io::Result<()> = match planted {
            Some(planted_error) => Err(planted_error),
            None => fs::remove_file(entry.path()),
        };
        match removal {
            // What: the link is gone, so the removal happened and is recorded with what it pointed at.
            // Why: the sentence is built from this record whatever happens to a later removal.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // removed.push({ name, target, temporary: installer === 'gone' });
            // ```
            Ok(()) => removed.push(ARemoval {
                name,
                target,
                temporary: installer == Some(Liveness::Gone),
            }),
            // What: the name is gone already, which another run's removal did in the instant before this one.
            // Why: the link left the folder, which is what this run wanted,
            //  and the run that took it is the one that names it.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (error.code !== 'ENOENT') throw error;
            // ```
            Err(failure) if failure.kind() == std::io::ErrorKind::NotFound => {}
            Err(failure) => {
                error = Some(ToolchainError::failure(format!(
                    "cannot remove the exposure link {}: {failure}",
                    entry.path().display()
                )));
                break;
            }
        }
    }
    // What:
    // the removals in name order, one sentence each.
    // Why:
    // A directory's entries come in no order,
    // and the report reads the same on every run only if the sentences are sorted by name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // removed.sort((left, right) => left.name.localeCompare(right.name));
    // return { sentences: removed.map(theSentenceOf), error };
    // ```
    removed.sort_by(|left_one, right_one| return left_one.name.cmp(&right_one.name));
    let mut sentences: Vec<String> = Vec::with_capacity(removed.len());
    for one in &removed {
        sentences.push(the_sentence_of(one));
    }
    // What:
    // the kept temporary links, one sentence each in name order.
    // Why:
    // The folder's entries come in no order,
    // and the report reads the same on every run only if the names are sorted.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // left.sort((a, b) => a.localeCompare(b));
    // ```
    left.sort();
    let mut left_sentences: Vec<String> = Vec::with_capacity(left.len());
    for name in &left {
        left_sentences.push(the_kept_sentence_of(name));
    }
    return Removals {
        sentences,
        left: left_sentences,
        error,
    };
}

/// What:
/// The one line a removal prints:
/// the name that left the folder,
/// where it pointed,
/// and why the rebuild took it.
/// Why:
/// A name no row declares and a temporary link are told apart in the sentence,
/// because a person reading it has to know whether a plan stopped declaring a bin or a run was killed;
/// and the temporary link's sentence says only what the name is and what the run could show of the process it
/// records,
/// never that an install wrote it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theSentenceOf(one: ARemoval): string
/// ```
fn the_sentence_of(one: &ARemoval) -> String {
    if one.temporary {
        return format!(
            "removed the temporary link {} (it pointed at {}), a name of the shape an install writes while it holds a link, and the process that name records is not running",
            one.name,
            one.target.display()
        );
    }
    return format!(
        "removed the exposure link {} (it pointed at {}), which no row of the lock declares",
        one.name,
        one.target.display()
    );
}

/// What:
/// The one line a rebuild prints for a temporary link it could not judge and left where it is:
/// the name,
/// why nothing was removed,
/// and what deleting it by hand would risk.
/// Why:
/// The name records the install that may still hold the link,
/// and a process in another process namespace cannot be judged from here,
/// so the line says no more than that and names the cost of a person's own deletion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theKeptSentenceOf(name: string): string
/// ```
fn the_kept_sentence_of(name: &str) -> String {
    return format!(
        "left the temporary link {name}: this run cannot judge the install its name records, so it is not removed, and deleting it by hand may make that install fail where it gives the link its name"
    );
}
