//! What:
//!  Test-only fault points of the resident service:
//! `CLI_GIT_TEST_ONLY_SERVICE_FAULT=<point>` ends the service with
//!  `SIGKILL` the moment it reaches the named point of a batch.
//! Why:
//!  The kill tests need a deterministic instant,
//! as the transaction phase markers give the wrapper
//!  (`transaction_phase.rs`).
//! The marker is compiled into the release executable,
//! and only the explicit variable arms it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (process.env.CLI_GIT_TEST_ONLY_SERVICE_FAULT === point) process.kill(process.pid, 'SIGKILL');
//! ```

/// Environment variable arming one fault point.
pub const FAULT_VARIABLE: &str = "CLI_GIT_TEST_ONLY_SERVICE_FAULT";

/// The instants of a batch at which a kill is tested,
/// in the order a batch reaches them.
pub const FAULT_POINTS: [&str; 5] = ["after-objects", "after-intent", "after-ref", "after-index", "before-ack"];

/// The instant in the wrapper after the post-commit gate promoted a landed commit's held push into `pending`,
/// and before the wake that would run it:
/// a kill here leaves a push record nothing was told about,
/// so the service has to find it by itself
/// (`service_claim_watch.rs`,
/// `held_push_notices` in `service_handoff.rs`).
/// Named here because the controls of both halves of that promise reach it,
/// and it is inert unless the variable names it.
pub const AFTER_PROMOTE: &str = "after-promote";

/// The instant after a command wrote its verdict into a held push record and before it promoted or forgot that record:
/// a kill here leaves the record saying what the command decided,
/// so a later command settles it from those words instead of
///  waiting for a command that is gone
/// (`held_push_holder_and_verdict.rs`).
pub const AFTER_THE_VERDICT: &str = "after-the-verdict";

/// The instant after `index.lock` was created by exclusive creation
/// (where the filesystem places no file whole)
/// and before its marker is written:
/// a kill here leaves a lock without its marker,
/// as plain Git leaves one.
pub const LOCK_CREATED_UNMARKED: &str = "lock-created-unmarked";

/// The instant after the marker was written into a lock created by exclusive creation,
/// before it is flushed.
pub const LOCK_MARKED_UNSYNCED: &str = "lock-marked-unsynced";

/// What:
///  Ends the process if `point` is the armed one.
/// Why:
///   A kill at a point is the strongest crash a test can name without injecting a system call failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reach('after-intent')
/// ```
pub fn reach(point: &str) {
    #[cfg(test)]
    hooks::run(point);
    if matches!(std::env::var(FAULT_VARIABLE), Ok(armed) if armed == point) {
        crate::service_log!("test fault at {point}");
        #[cfg(unix)]
        // SAFETY: `kill` takes no pointers; the process ends itself.
        unsafe {
            libc::kill(libc::getpid(), libc::SIGKILL);
        }
        std::process::exit(137);
    }
}

/// What:
///  Says that a step of a landing happened,
/// for a control that checks the order of durable steps.
/// Why:
///   A directory sync cannot be seen from outside the process,
/// so a control reads the order the landing reports;
/// the release
///  executable keeps nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// note('sync-index-dir')
/// ```
pub fn note(event: &str) {
    #[cfg(test)]
    hooks::record(event);
    #[cfg(not(test))]
    let _unused: &str = event;
}

/// What:
///  Whether a control made the step at `point` fail.
/// Why:
///   A landing whose index update fails after the branch moved is the case recovery exists for,
/// and Git cannot be made to fail
///  there on demand.
/// A control names the point;
/// the release executable never fails a step this way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (fails('build-index')) return false
/// ```
pub fn fails(point: &str) -> bool {
    #[cfg(test)]
    return hooks::failing(point);
    #[cfg(not(test))]
    return {
        let _unused: &str = point;
        false
    };
}

/// Environment variable naming a directory in which the service leaves one file for each commit request it has queued.
pub const QUEUED_DIRECTORY_VARIABLE: &str = "CLI_GIT_TEST_ONLY_SERVICE_QUEUED_DIRECTORY";

/// What:
///  A character as one that may sit in a file name:
/// ASCII letters and digits stay,
/// anything else becomes an underscore.
/// Why:
///  `queued` names a marker file after a request identity,
/// which must not name a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fileNameCharacter = (c: string) => /[A-Za-z0-9]/.test(c) ? c : '_';
/// ```
fn file_name_character(character: char) -> char {
    return if character.is_ascii_alphanumeric() { character } else { '_' };
}

/// What:
///  Leaves a file named for request `id` in the directory the variable names,
/// when it is armed.
/// Why:
///   A control that sends requests in a set order needs to know that the service holds one before it sends the next;
/// a pause
///  of its own can only guess.
/// The file appears after the request is in the queue,
/// so what the control sees is the service's own
///  state.
/// Only the explicit variable arms it,
/// as with the fault points.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (process.env.CLI_GIT_TEST_ONLY_SERVICE_QUEUED_DIRECTORY) writeFile(join(dir, safe(id)), '')
/// ```
pub fn queued(id: &str) {
    let Some(directory) = std::env::var_os(QUEUED_DIRECTORY_VARIABLE) else { return };
    let name: String = id.chars().map(file_name_character).collect();
    let path: std::path::PathBuf = std::path::Path::new(&directory).join(format!("queued-{name}"));
    if let Err(error) = std::fs::write(&path, b"") {
        crate::service_warn!("cannot leave the queued-request file {}: {error}", path.display());
    }
}

/// Environment variable pinning the number the service counts its batches from,
/// in place of the clock reading it starts from.
pub const FIRST_BATCH_VARIABLE: &str = "CLI_GIT_TEST_ONLY_SERVICE_FIRST_BATCH";

/// What:
///  The number the variable pins the service's batch count to,
/// when it is armed with a number.
/// Why:
///   Two services that share a journal count from clock readings,
/// which are equal only by chance;
/// a control that needs them equal
///  pins both.
/// Only the explicit variable arms it,
/// as with the fault points.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pinned = Number(process.env.CLI_GIT_TEST_ONLY_SERVICE_FIRST_BATCH) // undefined when unset or not a number
/// ```
pub fn pinned_first_batch() -> Option<u64> {
    return match std::env::var(FIRST_BATCH_VARIABLE) {
        Ok(text) => text.parse::<u64>().ok(),
        Err(_unset) => None,
    };
}

/// Hooks a control sets to run its own code at a point of a landing,
/// on the thread that reaches the point;
/// never part of the
///  release executable.
#[cfg(test)]
#[path = "service_fault_hooks_test_support.rs"]
pub(crate) mod hooks;
