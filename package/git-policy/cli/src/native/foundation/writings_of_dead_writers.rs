//! What:
//!  Removes the temporary files a write left behind, whose writer is gone.
//! Why:
//!  A temporary file is its write's own, so no later write takes it up: a kill between the write and the rename,
//! or a write that failed after creating the file, would otherwise leave one more `*.writing` name on every start
//! in one runtime directory, and nothing would ever collect them (the sixth review of issue 6293, finding 4).
//!  A name whose process number is still a process with that birth identity is left alone:
//! that writer may still be writing its file, and the safe side is never to take a live writer's work away.
//!  A number a stranger took after the writer ended is the one case that leaves a file: its birth identity differs from
//! anything these names could prove, and the file is a few bytes more than a write that failed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! removeWritingsOfDeadWriters(folder, 'real-git') // unlink `real-git.<pid>.<n>.writing` for every pid that is not that process
//! ```

use crate::process_identity::process_birth_identity;
use std::path::Path;

/// What:
///  Removes every temporary file in `folder` named `<stem>.<pid>.<n>.writing` whose process is gone.
/// Why:
///  The writer's number is the first part of the name after the stem,
/// and the birth identity is what tells a live writer from a number a stranger took.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of readdir(folder)) if (name.startsWith(`${stem}.`) && name.endsWith('.writing') && !isThatProcess(pidOf(name))) unlink(name);
/// ```
pub(crate) fn remove_writings_of_dead_writers(folder: &Path, stem: &str) {
    let listing: std::fs::ReadDir = match std::fs::read_dir(folder) {
        Ok(listing) => listing,
        // A folder that cannot be listed is the caller's own to report; this sweep only collects what it finds.
        Err(_cannot_list) => return,
    };
    for entry in listing.flatten() {
        let name = entry.file_name();
        let Some(text) = name.to_str() else { continue };
        let Some(rest) = text.strip_prefix(stem) else { continue };
        let Some(parts) = rest.strip_prefix('.').filter(|_rest| { return _rest.ends_with(".writing"); }) else { continue };
        let Some(pid_text) = parts.split('.').next() else { continue };
        let Ok(pid) = pid_text.parse::<i64>() else { continue };
        // A writer that is still that process may still be writing its file.
        if matches!(process_birth_identity(pid), Ok(Some(_that_process))) {
            continue;
        }
        let _removed = std::fs::remove_file(entry.path());
    }
}
