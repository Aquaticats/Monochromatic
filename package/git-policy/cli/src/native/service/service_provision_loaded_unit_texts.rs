//! What:
//!  The record of the two unit texts the service manager was last told about,
//! kept beside the notes of the worktrees,
//! inside the
//!  directory the manager reads the wrapper's units from.
//! Why:
//!  A unit file that holds the text this wrapper would write is not by itself an installation the manager runs:
//! the manager
//!  starts,
//! reloads and caches units on its own schedule,
//! so a process killed between writing a template and
//!  asking for the reload leaves files that match while the manager still holds the text it loaded
//!  (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
//! the sequence of the interrupted upgrade).
//! Every later commit then reaches the service of the release that was replaced,
// ends with
//!  `ClientFailure::OtherRelease`,
//! and follows the same fast path again,
//! so no commit can repair it.
//! The record makes the question "was the manager told about exactly these texts" one small read,
//! and it is
//!  written only after the manager accepted the reload,
//! so a kill before the reload leaves the older digest and the next commit asks again.
//! It lives under the runtime directory with the units,
//! which is removed at logout,
//! and it is not a unit
//!  name,
//! so the manager never reads it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // recordLoadedUnitTexts(runtimeDir, texts); const told = unitTextsTheManagerWasTold(runtimeDir, texts);
//! ```

use crate::private_storage::create_private_directory_all;
use crate::service_instance::digest_of_bytes;
use crate::service_provision_host_and_manager::runtime_unit_dir;
use crate::service_provision_unit_file_ownership_and_writes::write_atomically;
use crate::service_units_of_removed_worktrees::NOTES_FOLDER;
use std::path::{Path, PathBuf};

/// What:
///  The name of the record inside the notes folder,
/// which the sweep of removed worktrees ignores because it does not end
///  like a note.
/// Why:
///   The folder is the one place in the unit directory where the wrapper keeps files the manager never reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LOADED_TEXTS_RECORD = 'units-the-manager-was-told';
/// ```
pub const LOADED_TEXTS_RECORD: &str = "units-the-manager-was-told";

/// The byte between the units of the digested text,
/// which neither a unit name nor a unit text contains.
const FIELD_SEPARATOR: u8 = 0;

/// Where the record of a runtime directory is.
fn record_path(runtime_dir: &Path) -> PathBuf {
    return runtime_unit_dir(runtime_dir).join(NOTES_FOLDER).join(LOADED_TEXTS_RECORD);
}

/// What:
///  The digest of the unit names and texts the manager is told about,
/// as the record holds it.
/// Why:
///   Both templates make one record,
/// because the manager is reloaded for both,
/// and a change of either is a text the
///  manager was not told about.
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
///  Whether the record says the manager was told about exactly these texts.
/// Why:
///   The provisioning asks this on every commit,
/// before it touches the bus,
/// and both the fast path and the
///  rewriting path decide the reload with it.
/// A record that is absent or altered is no record:
/// the wrapper asks for the reload
///  instead of taking the manager's state on trust.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readFileSync(recordPath) === digestOf(texts)
/// ```
pub fn remembers(runtime_dir: &Path, texts: &[(&'static str, String)]) -> bool {
    return match std::fs::read_to_string(record_path(runtime_dir)) {
        Ok(record) => record.trim_end() == digest(texts),
        Err(_absent) => false,
    };
}

/// What:
///  Records that the manager was told about these texts.
/// Why:
///   This is written only after the manager accepted the reload,
/// so a record is the manager's own answer and
///  never a guess of this program's;
/// a write that fails is reported by the caller and costs the next commit one
///  reload,
/// never a commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkdir(recordDir, { recursive: true, mode: 0o700 }); writeFileSync(recordPath, digestOf(texts) + '\n')
/// ```
pub fn record(runtime_dir: &Path, texts: &[(&'static str, String)]) -> std::io::Result<()> {
    let path: PathBuf = record_path(runtime_dir);
    if let Some(folder) = path.parent() {
        create_private_directory_all(folder)?;
    }
    return write_atomically(path.as_path(), format!("{}\n", digest(texts)).as_str());
}
