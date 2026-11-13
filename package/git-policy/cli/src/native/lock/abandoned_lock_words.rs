//! What:
//!  The two lines the person reads about a lock:
//!  the one that says what was cleared and why,
//!  and the one that says what stands and why nothing was removed.
//! Why:
//!  A command that clears a lock must say which file went and what proved its holder gone,
//!  so no other command is needed to understand it;
//!  a command that leaves one must name the lock,
//!  what is known about the holder,
//!  and the one action that clears it.
//!  The proof itself is `abandoned_lock_proving.rs`;
//!  these are only its words.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! clearanceLine(clearance); standingLockLine(lock, commonDir);
//! ```

/// The proof and the evidence a clearance acted on.
use crate::abandoned_lock_proving::{Clearance, Evidence};
/// The record's words for a holder that is gone.
use crate::lock_holder_record_reads::{gone_holder_words, identities_recorded};
/// How a lock's PID file is read and judged.
use crate::stale_ref_lock_judge::Judge;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  The line the person reads when a lock of a dead holder was cleared.
/// Why:
///  One line says which lock went,
///  what proved its holder gone,
///  and what became of the half-written file,
///  so the person needs no other command to understand what happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: cleared ${lock}, whose holder is gone ...`
/// ```
pub fn clearance_line(clearance: &Clearance) -> String {
    let who: String = match clearance.evidence {
        Evidence::PidFile => format!(
            "its PID file names {}, and this repository's own record of the processes its commands started names that same process",
            gone_holder_words(clearance.pid)
        ),
        Evidence::ServiceMarker => format!(
            "the commit service's own marker in it names {}, and that process no longer runs",
            gone_holder_words(clearance.pid)
        ),
    };
    return format!(
        "monochromatic-git-policy-cli: cleared {}, which a Git that no longer runs left: {who}. A Git fills a lock and renames it over its target, so the file it never renamed held only that Git's unfinished copy of the index, the reference list, HEAD or the shallow file, and the repository has the file it had before.\n",
        clearance.lock.display()
    );
}

/// What:
///  The line for a lock that stands and cannot be proved abandoned,
///  when its PID file names a number a live process has now,
///  or a number no process has and which no record of this boot explains.
/// Why:
///  Those are the two states a person meets as a lock that cannot release itself:
///  a number in use means the lock may be a live program's,
///  and nothing here may prove it is not;
///  a number no record explains is a lock the wrapper did not take.
///  The line names the lock,
///  what is known about the holder,
///  and the one action that clears it.
///  A lock whose record proves its writer gone is cleared and never reaches this line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// standingLockLine(lock, commonDir) // '...names PID 42, which a live process has now...' | undefined
/// ```
pub fn standing_lock_line(lock: &Path, common_dir: &Path) -> Option<String> {
    let born: Option<std::time::SystemTime> = std::fs::symlink_metadata(lock).ok().and_then(created_at);
    let crate::stale_ref_lock_found::PidFile::Names(pid) = Judge::pid_file_state(lock, born) else {
        return None;
    };
    let recorded: bool = !identities_recorded(common_dir, pid).is_empty();
    let live: bool = matches!(crate::process_identity::process_birth_identity(pid), Ok(Some(_running)));
    let held: bool = crate::abandoned_lock_proving::a_writer_holds_it_open(lock);
    let why: &str = match (live, recorded, held) {
        // Nothing is claimed about a live number: the lock may be that program's, and its own command ends by itself.
        (true, _, _) => "which a live process has now, so nothing here proves the lock is not its own write",
        (false, false, _) => "which no process in this process table has, and no record of the processes this repository's own commands started names that number",
        // The records may be dead and the writer may still be at work: a program holding the lock open for writing keeps its
        //  name, because it can still write into the file.
        (false, true, true) => "which no process in this process table has, and a program holds it open for writing now, so its name may not go",
        (false, true, false) => "which no process in this process table has, and whose records this repository holds name a process of another boot, another PID namespace or another machine, so nothing here proves it gone",
    };
    return Some(format!(
        "monochromatic-git-policy-cli: {}, whose PID file names {pid}, {why}, so nothing was removed. A Git in another PID namespace, or one on another machine that shares this repository, writes a number this table has never had. If no program is using this repository, the lock was left by a program that was killed or has ended, and removing the file {} lets the command run.\n",
        lock.display(),
        lock.display()
    ));
}

/// What:
///  When a file was created,
///  where its file system records that.
/// Why:
///  Git writes a lock's PID file after the lock,
///  so the two times are what tells this lock's holder from an earlier one's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// statSync(path).birthtimeMs
/// ```
fn created_at(metadata: std::fs::Metadata) -> Option<std::time::SystemTime> {
    return metadata.created().ok();
}
