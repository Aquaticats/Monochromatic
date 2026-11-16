//! What:
//!  The names a writer gives its own temporary file,
//!  and the sweep that removes one only on a proof about the process that made it.
//! Why:
//!  A temporary file is its write's own, so no later write takes it up: a kill between the write and the rename, or a write
//! that failed after creating the file, would otherwise leave one `*.writing` name per interrupted write, and nothing would
//! collect them (the sixth review of issue 6293, finding 4).
//!  A file is removed only when its name proves a process that is not that process any more: the name carries the writer's
//! number **and its birth identity**, so a number a stranger took names the stranger and not the writer
//! (`process_identity.rs`, field 22 of `/proc/<pid>/stat`).
//!  A record that cannot be read is **not** that proof: the file stays, because a writer whose record the kernel would not
//! give may be alive and about to rename its file (the eighth review of issue 6293, finding 7).
//!  The name and the sweep live together, so a name shape and the shape the sweep reads cannot drift apart
//! (the eighth review, finding 3: a sweep given the published name never matched the files its writer made).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const temp = temporaryFile(published, 7); // `<published>.<pid>.<birthIdentity>.<n>.writing`
//! removeWritingsOfDeadWriters(folder, publishedName); // unlink every such name whose (pid, identity) is not a live process
//! ```

use crate::process_identity::{current_birth_identity, process_birth_identity};
use std::path::{Path, PathBuf};

/// The ending every temporary file of this module carries.
pub const WRITING_ENDING: &str = ".writing";

/// What:
///  The temporary file this write uses for `published`,
///  whole before it is published by a rename that gives it the published name.
/// Why:
///  The name a reader must never see half of is the published one,
/// and the temporary name is this write's own:
/// the writer's process number, its birth identity and a counter of that process,
/// so two writes never share a file and the sweep can prove whose a leftover is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// temporaryFile(publishedPath, number) // `<publishedPath>.<pid>.<birthIdentity>.<number>.writing`
/// ```
pub fn temporary_file(published: &Path, number: u64) -> std::io::Result<PathBuf> {
    let identity: String = current_birth_identity().map_err(|unsupported| {
        return std::io::Error::other(format!("this platform cannot name a process's birth, so no temporary file can be proved whose: {unsupported:?}"));
    })?;
    let folder: &Path = published.parent().unwrap_or_else(|| { return Path::new("."); });
    let name: &str = published.file_name().and_then(std::ffi::OsStr::to_str).unwrap_or("published");
    return Ok(folder.join(format!("{name}.{}.{identity}.{number}{WRITING_ENDING}", std::process::id())));
}

/// A number no other write of this process has taken, so two writes never share a temporary file.
fn next_temporary_number() -> u64 {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    return COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

/// What:
///  Removes every temporary file of `published` in its own folder whose writer is proven gone.
/// Why:
///  The name carries the writer's number and birth identity,
/// so the question "is that process still that process" is answered by the record the kernel gives,
/// and never by a number alone.
/// A record that cannot be read proves nothing, so a file whose writer may still be alive is left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeWritingsOfDeadWriters(folder, publishedName) // unlink `<publishedName>.<pid>.<identity>.<n>.writing` unless pid is alive with identity
/// ```
pub fn remove_writings_of_dead_writers(published: &Path) {
    let Some(folder) = published.parent() else { return };
    let Some(name) = published.file_name().and_then(std::ffi::OsStr::to_str) else { return };
    let prefix: String = format!("{name}.");
    let listing: std::fs::ReadDir = match std::fs::read_dir(folder) {
        Ok(listing) => listing,
        // A folder that cannot be listed is the caller's own to report; this sweep only collects what it finds.
        Err(_cannot_list) => return,
    };
    for entry in listing.flatten() {
        let file = entry.file_name();
        let Some(text) = file.to_str() else { continue };
        let Some(after_the_name) = text.strip_prefix(prefix.as_str()) else { continue };
        let Some(rest) = after_the_name.strip_suffix(WRITING_ENDING) else { continue };
        let mut parts = rest.split('.');
        // The name carries the writer's number and its birth identity, so the proof is about that process and not about a number.
        let (Some(pid_text), Some(identity)) = (parts.next(), parts.next()) else { continue };
        let Ok(pid) = pid_text.parse::<i64>() else { continue };
        match process_birth_identity(pid) {
            // The writer is still that process: its file stays, whoever else might be writing.
            Ok(Some(that_process)) if that_process.as_str() == identity => {}
            // The process is gone, or a stranger took the number: the file is the dead writer's.
            Ok(None) => {
                let _removed = std::fs::remove_file(entry.path());
            }
            Ok(Some(_another_process)) => {
                let _removed = std::fs::remove_file(entry.path());
            }
            // A record that cannot be read is no proof: the writer may be alive.
            Err(_cannot_read) => {}
        }
    }
}

/// What:
///  Writes `text` to `published` through a temporary file of this module and a rename,
///  and gives back what the rename came to.
/// Why:
///  A reader never sees half of a note,
/// a failed write returns its own file before the refusal is reported,
/// and the sweep of dead writers runs first, so a runtime directory does not collect them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeAtomically(published, text) // sweep, write temp, rename (or unlink the temp and report)
/// ```
pub fn write_atomically(published: &Path, text: &str) -> std::io::Result<()> {
    remove_writings_of_dead_writers(published);
    let temporary: PathBuf = temporary_file(published, next_temporary_number())?;
    if let Err(refused) = std::fs::write(&temporary, text) {
        // A write that failed after creating its file leaves it to no one: this writer takes it back.
        let _left = std::fs::remove_file(&temporary);
        return Err(refused);
    }
    let renamed: std::io::Result<()> = std::fs::rename(&temporary, published);
    if renamed.is_err() {
        let _left = std::fs::remove_file(&temporary);
    }
    return renamed;
}

/// Controls of the names and the sweep stay out of the release executable.
#[cfg(test)]
mod tests {
    use super::*;

    /// A folder of this control's own.
    fn folder(name: &str) -> std::path::PathBuf {
        let root: std::path::PathBuf = crate::test_support::fixture_root().join(format!("writings-{name}"));
        let _stale = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("fixture folder");
        return root;
    }

    /// What:
    ///  A temporary file a writer left when it was killed is collected by the next write of the same published file.
    /// Why:
    ///  The eighth review of issue 6293 (finding 3): the sweep was given the published name of one writer and the
    ///  files its writers made carried another, so nothing was ever collected.
    #[test]
    fn the_file_of_a_writer_that_is_gone_is_collected() {
        let root: std::path::PathBuf = folder("dead-writer");
        let published: std::path::PathBuf = root.join("note");
        // A name the writers' own function makes, for a number no process has: the writer is proved gone, and no child is started.
        let ended_pid: u32 = 999_999;
        let dead: std::path::PathBuf = published.with_file_name(format!("note.{ended_pid}.linux:000000.1{WRITING_ENDING}"));
        std::fs::write(&dead, b"half").expect("the file a killed writer left");
        write_atomically(published.as_path(), "whole\n").expect("the next write");
        assert!(!dead.exists(), "the sweep collected the dead writer's file");
        assert_eq!(std::fs::read_to_string(published.as_path()).expect("the published file"), "whole\n");
    }

    /// What:
    ///  A temporary file of a writer that is still that process is left alone, and so is one whose record cannot be read.
    /// Why:
    ///  A removal on the strength of a process number alone would take a live writer's file away;
    ///  the name carries the birth identity so the proof is about that process (the eighth review, finding 7).
    #[test]
    fn the_file_of_a_writer_that_is_alive_stays() {
        let root: std::path::PathBuf = folder("live-writer");
        let published: std::path::PathBuf = root.join("note");
        let identity: String = current_birth_identity().expect("this process's birth");
        let ours: std::path::PathBuf = published.with_file_name(format!("note.{}.{identity}.1{WRITING_ENDING}", std::process::id()));
        std::fs::write(&ours, b"a live writer's half").expect("a live writer's file");
        // The same number with another birth identity is a stranger: the writer it named is gone, so the file is collected.
        let stranger: std::path::PathBuf = published.with_file_name(format!("note.{}.linux:000000.2{WRITING_ENDING}", std::process::id()));
        std::fs::write(&stranger, b"the same number, another process").expect("the file of a writer whose number was reused");
        remove_writings_of_dead_writers(published.as_path());
        assert!(ours.exists(), "a live writer's own file stays");
        assert!(!stranger.exists(), "a number a stranger holds proves the writer gone, so its file is collected");
    }
}
