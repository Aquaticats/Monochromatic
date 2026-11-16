//! What:
//!  The temporary file one write of one record makes before its rename,
//!  named after the process that writes it,
//!  and the sweep that removes the temporaries of writers the kernel proves gone.
//! Why:
//!   A write publishes a record by renaming a file it made beside the record's own name,
//!  so a process killed between the write and the rename leaves a file nothing names,
//!  and a reader that lists the folder finds it beside the records.
//! The sweep may remove such a file only when no live writer is about to rename it,
//!  because removing it makes that writer's rename fail;
//!  an age is no proof of that,
//!  since a writer on a slow disk, a suspended host or a stopped debugger is old and alive.
//! So the writer's process number is part of the file's name,
//!  and the sweep removes the file only when the kernel answers that no process of that number exists:
//!  a live writer's own number cannot answer that.
//! A name that carries no writer is one a release before this one wrote;
//! it can prove nothing, so the sweep removes it only once it is far older than any write takes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeFile(nameOf(record), bytes); await rename(nameOf(record), record); // or: sweepTemporaries(folder)
//! ```

use std::path::{Path, PathBuf};

/// The ends of the names the package writes a record under before its rename,
/// each with the writer's process number in the field before it.
/// Why:
///   The sweep reads a name by its end,
///  so every temporary of every writer ends in one of these.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TEMPORARY_SUFFIXES = ['.tmp', '.partial']
/// ```
pub const TEMPORARY_SUFFIXES: [&str; 2] = [".tmp", ".partial"];

/// What:
///  The separator between a temporary's own name and the writer's process number.
/// Why:
///   A record's name may hold dots,
///  so the reader takes the field before the suffix and not the first field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WRITER_SEPARATOR = '.'
/// ```
const WRITER_SEPARATOR: char = '.';

/// What:
///  The separator a unit file's staging name carries before the writer's number,
///  in the names the release before this one wrote,
///  and the same one its link's name carries after the word `link`.
/// Why:
///   Those names were made by replacing a unit file's extension,
///  so they end with the marker rather than with [`TEMPORARY_SUFFIXES`],
///  and they still name their writer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TEMPORARY_SEPARATOR = '.tmp-'
/// const LINK_TEMPORARY_SEPARATOR = '.linktmp-'
/// ```
const TEMPORARY_SEPARATOR: &str = ".tmp-";

/// The separator of the staging name of a link the release before this one made.
const LINK_TEMPORARY_SEPARATOR: &str = ".linktmp-";

/// How many writes of this process have made a temporary.
/// Why:
///   Two threads of one process may write the same record at once,
///  and one temporary per record and process would let the second thread's rename find nothing
///  because the first published it:
///  the count makes each write's own file, and the writer's number still comes first in the field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let writes = 0
/// ```
static WRITES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// What:
///  The name of the file a write of `target` makes before its rename:
///  `<target>.<pid>-<write>.tmp`.
/// Why:
///   The writer's number is what the sweep judges,
///  and it is appended to the record's whole name,
///  so every byte of that name keeps its place.
/// The write's own count follows it,
/// so two writes of one process never share a file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nameOf = (target) => `${target}.${process.pid}-${writes += 1}.tmp`
/// ```
pub fn name_of(target: &Path) -> PathBuf {
    let write: u64 = WRITES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut temporary: std::ffi::OsString = target.as_os_str().to_owned();
    temporary.push(format!("{WRITER_SEPARATOR}{}-{write}.tmp", std::process::id()));
    return PathBuf::from(temporary);
}

/// What:
///  The process number a temporary file's name carries,
///  and nothing for a name that carries none.
/// Why:
///  A writer outside a process cannot be named,
///  so a file of no writer cannot be proven abandoned by its number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writerOf = (name) => /^(.*)\.(\d+)(-\d+)?\.(tmp|partial)$/.exec(name)?.[2]
/// ```
pub fn writer_of(name: &str) -> Option<u32> {
    if let Some(stem) = TEMPORARY_SUFFIXES.iter().find_map(|suffix| { return name.strip_suffix(suffix); }) {
        let (_, field) = stem.rsplit_once(WRITER_SEPARATOR)?;
        // The writer's number comes first in the field, and the count of this write follows it after a dash.
        return field.split('-').next()?.parse().ok();
    }
    // A unit file and the link beside it were staged by the release before this one as `<name>.tmp-<pid>-<count>` and
    //  `<name>.linktmp-<pid>-<count>`, whose writer is the field after the marker.
    let rest: &str = match name.rsplit_once(LINK_TEMPORARY_SEPARATOR) {
        Some((_, after)) => after,
        None => name.rsplit_once(TEMPORARY_SEPARATOR)?.1,
    };
    return rest.split('-').next()?.parse().ok();
}

/// What:
///  Whether the process a temporary file's name carries is provably gone.
/// Why:
///   Only an answer of the kernel that no process of that number exists proves it;
///  a number another process took over is alive and its file stays,
///  and an answer the platform refuses proves nothing and keeps the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isGone = (pid) => readBirthIdentity(pid) === undefined
/// ```
pub fn writer_is_gone(pid: u32) -> bool {
    return matches!(crate::process_identity::process_birth_identity(i64::from(pid)), Ok(None));
}

/// What:
///  Whether a file of this name is the remains of a killed write of this package:
///  its writer is provably gone,
///  or it names no writer and is far older than any write takes.
/// Why:
///   Both answers are the safe side:
///  a live writer's temporary is never removed,
///  and neither is one whose writer the platform cannot judge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAbandoned = (name, modified, grace) => { const pid = writerOf(name); return pid !== undefined ? isGone(pid) : olderThan(modified, grace) }
/// ```
pub fn is_abandoned(name: &str, modified: std::io::Result<std::time::SystemTime>, grace: std::time::Duration) -> bool {
    if let Some(pid) = writer_of(name) {
        return writer_is_gone(pid);
    }
    return match modified.map(|written| { return written.elapsed(); }) {
        Ok(Ok(age)) => age > grace,
        // A time in the future is a clock that moved, and an unreadable time proves no age either.
        Ok(Err(_in_the_future)) => false,
        Err(_unreadable) => false,
    };
}

/// What:
///  How much a sweep of one folder removed.
/// Why:
///   A caller that flushes the folder or says what it did needs the count,
///  and a sweep that removed nothing costs no write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sweeps(folder) // 2
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Swept {
    /// The temporaries removed.
    pub removed: usize,
}

/// What:
///  Whether a name is one this package stages a record under,
///  in this release or in the one before it.
/// Why:
///   A folder may hold a person's own files,
/// so the sweep looks only at names this package makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isOurs = (name) => TEMPORARY_SUFFIXES.some((suffix) => name.endsWith(suffix)) || name.includes('.tmp-') || name.includes('.linktmp-')
/// ```
fn candidates_a_temporary(name: &str) -> bool {
    return TEMPORARY_SUFFIXES.iter().any(|suffix| { return name.ends_with(suffix); }) || name.contains(TEMPORARY_SEPARATOR) || name.contains(LINK_TEMPORARY_SEPARATOR);
}

/// What:
///  Removes the temporaries a killed write of this package left in `folder`,
///  and nothing else:
///  no record,
///  no folder a live writer works in,
///  and no name whose maker cannot be proven gone.
/// Why:
///   Every record this package writes under a temporary name puts it in the folder of the record,
///  so the folder is where the remains of a killed write are,
///  and a sweep beside the write that follows is the one place that finds them all.
/// A name of the shape a release now writes carries its writer,
///  and only the kernel's answer that no such process exists removes it;
/// a name without a writer is one an earlier release wrote,
///  and it is removed only once it is far older than any write takes.
/// A folder that cannot be read is no failure:
///  the write that follows reports the real one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { removed } = sweepTemporaries(folder, grace) // 1
/// ```
pub fn sweep_temporaries(folder: &Path, grace: std::time::Duration) -> Swept {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Swept::default();
    };
    let mut swept: Swept = Swept::default();
    for entry in entries.flatten() {
        let listed = entry.file_name();
        let Some(name) = listed.to_str() else {
            continue;
        };
        if !candidates_a_temporary(name) {
            continue;
        }
        if !is_abandoned(name, entry.metadata().and_then(|metadata| { return metadata.modified(); }), grace) {
            continue;
        }
        if entry.file_type().is_ok_and(|kind| { return kind.is_dir(); }) {
            // A record's temporary may be a folder: the copy of a batch's conclusion state is one.
            if crate::private_storage::remove_tree(entry.path().as_path()).is_ok() {
                swept.removed += 1;
            }
            continue;
        }
        if std::fs::remove_file(entry.path()).is_ok() {
            swept.removed += 1;
        }
    }
    return swept;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "temporary_file_of_a_writer_tests.rs"]
mod tests;
