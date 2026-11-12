//! What:
//!  The environment every Git child of a service push receives,
//!  and the overlay the service adds to it.
//! Why:
//!  Both faces of the push (the synchronous one and the awaited one) build their Git here,
//! so the two cannot hand
//!  Git different variables;
//! and the list a push's programs are found through is the shell's,
//! which the client that made the commit
//!  recorded (`service_client_program_search`),
//! never the user manager's,
//! which the person committing can neither see nor change.
//! It lives beside the handlers so that file passes the package's length limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [inherited, overlay] = pushEnvironment(extra, gitDir);
//! ```

use crate::child_environment::service_child_overlay;
use crate::service_client_program_search::{MISSING_VARIABLE, RECORD_VARIABLE, record_path, read};
use std::ffi::OsString;
use std::path::Path;

/// What:
///  The list every Git child of a push receives,
///  and the overlay the service adds to it.
/// Why:
///   The overlay turns on the lock PID files (`core.lockfilePid`),
/// which name the owner of a lock a stopped push
///  leaves,
/// and adds the never-prompt pair,
/// because the host's askpass program is in this environment and a dialog nobody
///  answers is a push that waits.
/// The program search list of the shell that committed is laid over the service's own,
/// so
///  `git-lfs`,
/// a credential helper and `ssh` are found where that shell finds them,
/// and a marker names the record the list
///  came from,
/// so a refusal says which list it searched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [inherited, overlay] = pushEnvironment(extra, gitDir);
/// ```
pub fn push_environment_of(
    extra: &[(OsString, OsString)],
    git_dir: &Path,
) -> (Vec<(OsString, OsString)>, Vec<(OsString, OsString)>) {
    let mut inherited: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    inherited.extend(extra.iter().cloned());
    let record: std::path::PathBuf = record_path(git_dir);
    let mut from_the_client: Vec<(OsString, OsString)> = match read(git_dir) {
        Some(variables) => {
            let mut pairs: Vec<(OsString, OsString)> = variables;
            pairs.push((OsString::from(RECORD_VARIABLE), record.clone().into_os_string()));
            pairs
        }
        None => vec![(OsString::from(MISSING_VARIABLE), record.clone().into_os_string())],
    };
    let mut overlay: Vec<(OsString, OsString)> = service_child_overlay(inherited.as_slice());
    overlay.append(&mut from_the_client);
    return (inherited, overlay);
}
