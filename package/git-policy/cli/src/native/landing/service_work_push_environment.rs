//! What:
//!  The environment every Git child of a service push receives,
//!  and the overlay the service adds to it.
//! Why:
//!  Both faces of the push (the synchronous one and the awaited one) build their Git here,
//! so the two cannot hand
//!  Git different variables;
//! and the list a push's programs are found through is the shell's that asked for the push,
//! which
//!  travelled to this service with that request and lives only in its memory
//! (`service_push_programs_held_in_memory`),
//! never the user manager's,
//! which the person
//!  committing can neither see nor change.
//!  The five names that decide where a program is found and how it is reached are taken from the
//!  service's environment and replaced by the request's,
//! so a value the manager fixed can never stand in for one the person set,
//! and a value the person
//!  cleared can never come back from the manager.
//! A push whose values this service no longer holds keeps the service's own environment,
//! which its refusal names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [inherited, overlay] = pushEnvironment(extra, programs);
//! ```

use crate::child_environment::service_child_overlay;
use crate::service_push_programs_held_in_memory::{CARRIED, FROM_THE_REQUEST, FROM_THE_SERVICE, PushPrograms};
use std::ffi::{OsStr, OsString};

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
/// The program search list of the command that asked for the push is laid over the service's own,
/// so
///  `git-lfs`,
/// a credential helper and `ssh` are found where that command finds them,
/// and a marker names that list,
/// so a refusal says whose list it searched.
///  `programs` is nothing when this service holds no values for the push record it is running:
/// a service that restarted has none,
/// and the push then either searches the service's own list and says so,
/// or stops before it runs, because the transport its command chose is gone
/// (`push_without_its_transport`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [inherited, overlay] = pushEnvironment(extra, programs);
/// ```
pub fn push_environment_of(extra: &[(OsString, OsString)], programs: Option<&PushPrograms>) -> (Vec<(OsString, OsString)>, Vec<(OsString, OsString)>) {
    let mut inherited: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    inherited.extend(extra.iter().cloned());
    let mut overlay: Vec<(OsString, OsString)> = service_child_overlay(inherited.as_slice());
    let Some(programs) = programs else {
        // The refusal must not let the person believe their own shell was searched.
        overlay.push((OsString::from(FROM_THE_SERVICE), OsString::from("1")));
        return (inherited, overlay);
    };
    // What: Every name the command's own list decides is the command's alone.
    // Why:  A name the request left out is a name that shell had unset, and the manager's value for it names a program or a
    //  transport nobody asked for; replacing only the names that travelled would let the manager's `GIT_SSH_COMMAND` speak
    //  for a push whose command had none.
    inherited.retain(|(name, _value)| { return !CARRIED.iter().any(|carried| { return name.as_os_str() == OsStr::new(carried); }); });
    overlay.extend(programs.as_pairs());
    overlay.push((OsString::from(FROM_THE_REQUEST), OsString::from("1")));
    return (inherited, overlay);
}
