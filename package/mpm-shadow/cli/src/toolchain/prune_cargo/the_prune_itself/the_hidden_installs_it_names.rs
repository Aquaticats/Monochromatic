//! What:
//! The hidden installs `prune --toolchain` removes,
//! and the ones it cannot judge and only names.
//! Why:
//! A hidden name holds bytes of a tool no version scan lists,
//! so a removal here can cost the only finished copy of a version;
//! the line it prints for a name it cannot judge therefore says whether that directory holds one.
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
/// The line `prune --toolchain` prints for a hidden name whose installer it cannot judge,
/// for a directory that holds a finished copy and for one that holds none.
/// Why:
/// Deleting such a directory by hand is the only way a person clears it,
/// and when it holds a finished copy that deletion costs the version its only finished bytes;
/// the line has to say which case it is,
/// or a person deletes the copy the version needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theLineForANameItCannotJudge(id: string, version: string, path: string, holdsAFinishedCopy: boolean): string {
///   const start = `left ${path}: the installer that made it cannot be judged, so it is not removed`;
///   return holdsAFinishedCopy
///     ? `${start}; it holds a finished copy of ${id} ${version}, and deleting it by hand loses that copy, which nothing publishes while its installer cannot be judged`
///     : `${start}; it holds no finished copy, so deleting it by hand loses no finished copy, once no install uses it`;
/// }
/// ```
fn the_line_for_a_name_it_cannot_judge(
    id: &str,
    version: &str,
    path: &Path,
    holds_a_finished_copy: bool,
) -> String {
    if holds_a_finished_copy {
        return format!(
            "left {}: the installer that made it cannot be judged, so it is not removed; it holds a finished copy of {id} {version}, and deleting it by hand loses that copy, which nothing publishes while its installer cannot be judged",
            path.display()
        );
    }
    return format!(
        "left {}: the installer that made it cannot be judged, so it is not removed; it holds no finished copy, so deleting it by hand loses no finished copy, once no install uses it",
        path.display()
    );
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
/// and the person is told which path it is and whether it holds a finished copy.
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
                    let holds_a_finished_copy: bool = is_finished(&sibling.path, first_bin);
                    eprintln!(
                        "mpm-shadow: {}",
                        the_line_for_a_name_it_cannot_judge(
                            &id,
                            &sibling.version,
                            &sibling.path,
                            holds_a_finished_copy
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
