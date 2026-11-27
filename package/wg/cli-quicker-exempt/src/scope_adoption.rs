//! Moves processes into the updates slice by asking the user's systemd manager to adopt them.
//! Processes that leave the same origin cgroup join one long-lived scope for that origin,
//! created once with the origin's limits,
//! so together they stay within the origin's cap instead of each carrying a copy of it.
//! The manager creates the scope for the first process,
//! later processes are attached to it,
//! and it ends by itself when its last process exits.
//! Writing to `cgroup.procs` directly would put processes into a cgroup the manager never created.
//! The manager would not garbage-collect it,
//! and its pid-to-unit map would go stale.
//! That breaks the single-writer rule of systemd's cgroup delegation document.
//! The manager can only move processes inside its own subtree,
//! `user@<uid>.service`,
//! so other processes are reported and left where they are.

/// The checks that prove a move,
/// and the stop of a scope it left empty.
#[path = "scope_adoption_confirmation.rs"]
mod adoption_confirmation;
/// The records and the words of one adoption.
#[path = "scope_adoption_words.rs"]
mod adoption_words;
/// The adopter that reaches the user's manager over its own private socket.
#[path = "scope_adoption_production_adopter.rs"]
mod production_adopter;

/// Why a move that met another watcher's origin-scope sweep is left for the next scan,
/// in the words the log line carries.
pub use adoption_words::ADOPTION_DEFERRED;
/// Why an adoption did not happen.
pub use adoption_words::AdoptError;
/// One adoption of a process group into the scope of its origin.
pub use adoption_words::AdoptRequest;
/// How the processes got into the scope.
pub use adoption_words::Joined;
/// New limits for a scope that is running.
pub use adoption_words::LimitRequest;
/// Adopts over the user manager's own private socket.
pub use production_adopter::PrivateSocketAdopter;

/// Syscall and process failures.
use std::io;
/// Path view of the slice and cgroup arguments.
use std::path::Path;

/// Seam between the policy and the user's service manager.
pub trait Adopter {
    /// Creates the slice when it does not exist.
    fn ensure_slice(
        &self,
        uid: u32,
        slice: &str,
        slice_directory: &Path,
    ) -> io::Result<()>;
    /// The name of the scope of `origin` that is running right now,
    /// current spelling or an earlier one,
    /// so its later processes join it instead of one more scope with the same cap.
    fn existing_scope_name(
        &self,
        uid: u32,
        slice: &str,
        origin: &str,
    ) -> Option<String>;
    /// Moves the requested processes into the scope of their origin,
    /// creating the scope with the request's limits when it does not exist.
    fn adopt(
        &self,
        request: &AdoptRequest<'_>,
    ) -> Result<Joined, AdoptError>;
    /// Gives a running scope the limits of its origin as they are now.
    fn apply_limits(
        &self,
        request: &LimitRequest<'_>,
    ) -> Result<(), AdoptError>;
}
