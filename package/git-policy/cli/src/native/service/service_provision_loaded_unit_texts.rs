//! What:
//!  The record,
//! per unit instance,
//! of the two unit texts the service manager last read for it,
//! kept beside the notes of the
//!  worktrees,
//! inside the directory the manager reads the wrapper's units from.
//! Why:
//!  A unit file that holds the text this wrapper would write is not by itself an installation the manager runs:
//! the manager
//!  starts,
//! caches and unloads units on its own schedule,
//! so a process killed between writing a template and making the
//!  manager read it leaves files that match while the manager still holds a text it read earlier
//!  (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
//! the sequence of the interrupted upgrade).
//! Every later commit would then reach a service of the release that was replaced,
//! end with
//!  `ClientFailure::OtherRelease`,
//! and follow the same fast path again,
//! so no commit could repair it.
//! The record makes the question "does the manager hold this instance at the texts on disk" one small read,
//! and it
//!  is written only after the manager was watched dropping the instance,
//! so a kill before that leaves the older digest and the
//!  next commit repairs again.
//! It lives under the runtime directory with the units,
//! which is removed at logout,
//! and its name is not a unit
//!  name,
//! so the manager never reads it.
//! The record is per instance,
//! not per runtime directory,
//! because the manager holds one instance at a time:
//! a
//!  digest shared by every worktree would call one worktree's freshly loaded units true for another's stale ones.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // recordLoadedUnitTexts(runtimeDir, instance, texts); const told = unitTextsTheManagerWasTold(runtimeDir, instance, texts);
//! ```

use crate::private_storage::create_private_directory_all;
use crate::service_instance::digest_of_bytes;
use crate::service_provision_host_and_manager::runtime_unit_dir;
use crate::service_provision_unit_file_ownership_and_writes::write_atomically;
use crate::service_units_of_removed_worktrees::NOTES_FOLDER;
use std::path::{Path, PathBuf};

/// What:
///  The ending of the record of one instance inside the notes folder,
/// which the sweep of removed worktrees removes with the note it belongs
///  to.
/// Why:
///   The folder is the one place in the unit directory where the wrapper keeps files the manager never reads,
/// and a name
///  that does not end like a unit keeps it out of the manager's unit search.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LOADED_TEXTS_ENDING = '.units-the-manager-read';
/// ```
pub const LOADED_TEXTS_ENDING: &str = ".units-the-manager-read";

/// The byte between the units of the digested text,
/// which neither a unit name nor a unit text contains.
const FIELD_SEPARATOR: u8 = 0;

/// Where the record of one instance of a runtime directory is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unitTextsRecordPath(runtimeDir, instance) // join(runtimeDir, 'systemd/user/<notes>', instance + LOADED_TEXTS_ENDING)
/// ```
pub fn record_path(runtime_dir: &Path, instance: &str) -> PathBuf {
    return runtime_unit_dir(runtime_dir).join(NOTES_FOLDER).join(format!("{instance}{LOADED_TEXTS_ENDING}"));
}

/// What:
///  The digest of the unit names and texts one instance's units are loaded from,
/// as the record holds it.
/// Why:
///   Both templates make one record,
/// because both belong to the instance,
/// and a change of either is a text the manager
///  may not have read for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// digestOfBytes(Buffer.concat(texts.flatMap(([name, text]) => [Buffer.from(name), SEPARATOR, Buffer.from(text), SEPARATOR])))
/// ```
fn digest(texts: &[(&'static str, String)]) -> String {
    let mut bytes: Vec<u8> = Vec::new();
    for (name, text) in texts {
        bytes.extend_from_slice(name.as_bytes());
        bytes.push(FIELD_SEPARATOR);
        bytes.extend_from_slice(text.as_bytes());
        bytes.push(FIELD_SEPARATOR);
    }
    return digest_of_bytes(bytes.as_slice());
}

/// What:
///  Whether the record of this instance says the manager last read exactly these texts for it.
/// Why:
///   The provisioning asks this on every commit,
/// before it touches the bus,
/// and both the fast path and the
///  repairing path decide the unload with it.
/// A record that is absent or altered is no record:
/// the wrapper unloads the
///  instance and starts it again instead of taking the manager's state on trust.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readFileSync(recordPath(runtimeDir, instance)) === digestOf(texts)
/// ```
pub fn remembers(runtime_dir: &Path, instance: &str, texts: &[(&'static str, String)]) -> bool {
    return match std::fs::read_to_string(record_path(runtime_dir, instance)) {
        Ok(record) => record.trim_end() == digest(texts),
        Err(_absent) => false,
    };
}

/// What:
///  Drops the record of an instance,
/// so the next provisioning unloads it and starts it again.
/// Why:
///   A service of another release that will not retire is proof that the instance the manager holds and the files on disk
///  disagree,
/// and a record that says otherwise is the one thing that keeps every later commit on the fast path
///  (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 2 of the round-2
///  review).
/// A record that is already gone is the repair someone else finished.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rmSync(recordPath(runtimeDir, instance), { force: true })
/// ```
pub fn forget(runtime_dir: &Path, instance: &str) {
    let _removed = std::fs::remove_file(record_path(runtime_dir, instance));
}

/// What:
///  Records that the manager last read these texts for this instance.
/// Why:
///   This is written only after the manager was watched dropping the instance,
/// or after a start of it and never
///  before,
/// so a record is the manager's own state and never a guess of this program's;
/// a write that fails is reported by
///  the caller and costs the next commit one unload,
/// never a commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkdir(recordDir, { recursive: true, mode: 0o700 }); writeFileSync(recordPath(runtimeDir, instance), digestOf(texts) + '\n')
/// ```
pub fn record(runtime_dir: &Path, instance: &str, texts: &[(&'static str, String)]) -> std::io::Result<()> {
    let path: PathBuf = record_path(runtime_dir, instance);
    if let Some(folder) = path.parent() {
        create_private_directory_all(folder)?;
    }
    return write_atomically(path.as_path(), format!("{}\n", digest(texts)).as_str());
}
