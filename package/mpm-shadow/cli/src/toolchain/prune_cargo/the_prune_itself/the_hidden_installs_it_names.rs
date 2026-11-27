//! What:
//! The hidden installs `prune --toolchain` removes,
//! and the ones it cannot judge and only names.
//! Why:
//! A hidden name holds bytes of a tool no version scan lists,
//! so a removal here can cost the only finished copy of a version;
//! the line it prints for a name it cannot judge therefore says what it read there
//! and what deleting that directory would cost.
//! It sits in a file of its own because the prune next to it is longer than a person reads at once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // the-hidden-installs-it-names.ts
//! export function pruneHiddenSiblings(scope: Scope, every: Map<string, NamedArchive>, dryRun: boolean, removed: string[]): void;
//! ```

/// Imports the sorted collection the plan of deletions is held in.
use std::collections::BTreeMap;
/// Imports `fs` from `std`.
use std::fs;
/// Imports `ErrorKind` from `std::io`.
use std::io::ErrorKind;
/// Imports what this file uses from `std::path`.
use std::path::{
    Path,
    PathBuf,
};

/// Imports the hidden names this tool leaves,
/// the question whether their installer ended,
/// and the marker that says whether a copy can be published.
use super::super::super::archive_staging::{
    Liveness,
    has_vouched_marker,
    hidden_siblings,
};
/// Imports `ToolchainError` from the toolchain error module.
use super::super::super::error::ToolchainError;
/// Imports the release-archive tools a lock names.
use super::super::super::published_archive::{
    NamedArchive,
    is_finished,
};
/// Imports what this file uses from the toolchain scope.
use super::super::super::scope::Scope;
/// Imports the words a failed run leaves behind.
use super::super::what_a_failed_run_says_it_removed::after_removing;
/// Imports every tool directory under the toolchain root,
/// which the prune above this file lists once for both passes.
use super::tool_directories;

/// What:
/// What a `prune --toolchain` run read at a hidden name whose installer it cannot judge:
/// no finished copy at all,
/// no completion marker where no program name is known,
/// a completion marker where no program name is known,
/// a finished copy where the version stands finished at its own name,
/// or a finished copy that is the version's only one here.
/// Why:
/// The line says which of these it read,
/// because a marker alone is not a working tool,
/// a version that is finished at its name needs no hidden copy,
/// and only the last of them is a copy whose deletion costs the version its bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhatItRead = 'no-finished-copy' | 'no-completion-marker' | 'a-completion-marker-only' | 'a-second-finished-copy' | 'the-versions-only-finished-copy';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WhatItRead {
    /// The directory holds no finished copy,
    /// and a program name is known to look for.
    NoFinishedCopy,
    /// The directory holds no completion marker,
    /// and no known lock names a program to look for.
    NoCompletionMarker,
    /// The directory holds a completion marker,
    /// and no known lock names the program that would finish the copy.
    ACompletionMarkerOnly,
    /// The directory holds a finished copy of the version,
    /// and the version stands finished at its own name.
    ASecondFinishedCopy,
    /// The directory holds a finished copy of the version,
    /// and no finished copy of the version is at its own name.
    TheVersionsOnlyFinishedCopy,
}

/// What:
/// What this run read at the hidden name at `path`,
/// which belongs to `version` of the tool at `directory`.
/// Why:
/// The two questions differ by what the run could read:
/// a program is looked for only when a known lock names one,
/// and the version's own name is read only when the hidden name holds a finished copy,
/// because only then does the answer decide what a deletion costs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whatItRead(directory: string, path: string, version: string, firstBin: string): WhatItRead {
///   const finished = isFinished(path, firstBin);
///   if (firstBin === '') return finished ? 'a-completion-marker-only' : 'no-completion-marker';
///   if (!finished) return 'no-finished-copy';
///   return isFinished(join(directory, version), firstBin) ? 'a-second-finished-copy' : 'the-versions-only-finished-copy';
/// }
/// ```
fn what_it_read(
    directory: &Path,
    path: &Path,
    version: &str,
    first_bin: &str,
) -> WhatItRead {
    // What: the question that tells a finished install from an unfinished one, asked of the hidden name.
    // Why: with no program name it reads the completion marker alone, which is all any read of that name can say.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const finished = isFinished(path, firstBin);
    // ```
    let finished: bool = is_finished(path, first_bin);
    if first_bin.is_empty() {
        // What: no known lock names a program, so the marker is the whole read.
        // Why: the line then says `a completion marker`, never `a finished copy`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return finished ? 'a-completion-marker-only' : 'no-completion-marker';
        // ```
        return if finished {
            WhatItRead::ACompletionMarkerOnly
        } else {
            WhatItRead::NoCompletionMarker
        };
    }
    if !finished {
        // What: no finished copy under the hidden name, which the line says as today.
        // Why: nothing is lost with a directory that holds no finished copy.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return 'no-finished-copy';
        // ```
        return WhatItRead::NoFinishedCopy;
    }
    // What: whether the version's own name holds a finished copy, read beside the hidden one.
    // Why: a version that is finished at its name does not need what a deletion would take.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return isFinished(join(directory, version), firstBin) ? 'a-second-finished-copy' : 'the-versions-only-finished-copy';
    // ```
    return if is_finished(&directory.join(version), first_bin) {
        WhatItRead::ASecondFinishedCopy
    } else {
        WhatItRead::TheVersionsOnlyFinishedCopy
    };
}

/// What:
/// The line `prune --toolchain` prints for a hidden name whose installer it cannot judge,
/// for each of the things that run can have read there.
/// Why:
/// Deleting such a directory by hand is the only way a person clears it,
/// and the line has to say what the deletion costs without claiming a read the run did not make:
/// a marker is not a finished copy,
/// and a copy beside a version that stands finished at its name costs the version nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theLineForANameItCannotJudge(id: string, version: string, path: string, read: WhatItRead): string {
///   const start = `left ${path}: the installer that made it cannot be judged, so it is not removed`;
///   switch (read) {
///     case 'no-finished-copy': return `${start}; it holds no finished copy, so deleting it by hand loses no finished copy, once no install uses it`;
///     case 'no-completion-marker': return `${start}; it holds no completion marker, so deleting it by hand loses no finished copy, once no install uses it`;
///     case 'a-completion-marker-only': return `${start}; it holds a completion marker, but no known lock names the program that would finish a copy of ${id} ${version}, so deleting it by hand loses whatever that name holds, which nothing publishes while its installer cannot be judged`;
///     case 'a-second-finished-copy': return `${start}; it holds a second finished copy of ${id} ${version}, because ${version} is finished at its name, and deleting it by hand loses only that second copy`;
///     case 'the-versions-only-finished-copy': return `${start}; it holds a finished copy of ${id} ${version}, and deleting it by hand loses that copy, which nothing publishes while its installer cannot be judged`;
///   }
/// }
/// ```
fn the_line_for_a_name_it_cannot_judge(
    id: &str,
    version: &str,
    path: &Path,
    read: WhatItRead,
) -> String {
    // What: the two clauses every one of these lines shares.
    // Why: the name, why it stays, and the version differ per line, so only this much is common.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const start = `left ${path}: the installer that made it cannot be judged, so it is not removed`;
    // ```
    let start: String = format!(
        "left {}: the installer that made it cannot be judged, so it is not removed",
        path.display()
    );
    // What: `match` on the five answers, each of which is a different read of the same directory.
    // Why: a sentence that names a copy the run did not read is the fault this wording exists to avoid.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (read) { ... }
    // ```
    return match read {
        WhatItRead::NoFinishedCopy => format!(
            "{start}; it holds no finished copy, so deleting it by hand loses no finished copy, once no install uses it"
        ),
        WhatItRead::NoCompletionMarker => format!(
            "{start}; it holds no completion marker, so deleting it by hand loses no finished copy, once no install uses it"
        ),
        WhatItRead::ACompletionMarkerOnly => format!(
            "{start}; it holds a completion marker, but no known lock names the program that would finish a copy of {id} {version}, so deleting it by hand loses whatever that name holds, which nothing publishes while its installer cannot be judged"
        ),
        WhatItRead::ASecondFinishedCopy => format!(
            "{start}; it holds a second finished copy of {id} {version}, because {version} is finished at its name, and deleting it by hand loses only that second copy"
        ),
        WhatItRead::TheVersionsOnlyFinishedCopy => format!(
            "{start}; it holds a finished copy of {id} {version}, and deleting it by hand loses that copy, which nothing publishes while its installer cannot be judged"
        ),
    };
}

/// What:
/// Removes (or,
/// with `dry_run`,
/// lists) every hidden install whose installer has provably ended,
/// except a finished copy whose marker records a digest,
/// for a version some known lock names,
/// and names the ones it cannot judge.
/// Why:
/// A finished copy of a version a lock names can still be published by that version's own install,
/// while an unfinished copy,
/// one whose marker records no digest,
/// or one of a version no lock names will never be used;
/// leaving it would keep a whole tool's bytes for ever.
/// A name whose installer cannot be judged is never removed,
/// and the person is told which path it is,
/// what the run read there,
/// and what deleting it would cost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pruneHiddenSiblings(scope: Scope, every: Map<string, NamedArchive>, dryRun: boolean, removed: string[]): void
/// ```
pub(super) fn prune_hidden_siblings(
    scope: &Scope,
    every: &BTreeMap<String, NamedArchive>,
    dry_run: bool,
    removed: &mut Vec<PathBuf>,
) -> Result<(), ToolchainError> {
    for (id, directory) in tool_directories(&scope.toolchain_root)? {
        let siblings =
            hidden_siblings(&directory).map_err(|error| return after_removing(error, removed))?;
        for sibling in siblings {
            let archive: Option<&NamedArchive> = every.get(&id);
            let named_version: bool = archive.is_some_and(|named| {
                return named.versions.contains(&sibling.version);
            });
            let first_bin: &str = archive
                .map(|named| return named.first_bin.as_str())
                .unwrap_or("");
            match sibling.liveness() {
                Liveness::Running => continue,
                Liveness::Unknown => {
                    // What: what this run read at the name, from the hidden directory and the version's own name.
                    // Why: the line says what was read, and a tool no known lock names gives no program to look for.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // const read = whatItRead(directory, sibling.path, sibling.version, firstBin);
                    // ```
                    let read: WhatItRead =
                        what_it_read(&directory, &sibling.path, &sibling.version, first_bin);
                    eprintln!(
                        "mpm-shadow: {}",
                        the_line_for_a_name_it_cannot_judge(
                            &id,
                            &sibling.version,
                            &sibling.path,
                            read
                        )
                    );
                    continue;
                }
                Liveness::Gone => {}
            }
            let keep: bool = named_version
                && is_finished(&sibling.path, first_bin)
                && has_vouched_marker(&sibling.path);
            if keep {
                continue;
            }
            if !dry_run {
                let result: std::io::Result<()> = if sibling.path.is_dir() {
                    fs::remove_dir_all(&sibling.path)
                } else {
                    fs::remove_file(&sibling.path)
                };
                match result {
                    Ok(()) => {}
                    Err(error) if error.kind() == ErrorKind::NotFound => {}
                    Err(error) => {
                        return Err(after_removing(
                            ToolchainError::failure(format!(
                                "cannot remove the hidden install at {}: {error}",
                                sibling.path.display()
                            )),
                            removed,
                        ));
                    }
                }
            }
            removed.push(sibling.path);
        }
    }
    return Ok(());
}
