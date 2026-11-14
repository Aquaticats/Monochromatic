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
/// in the order a batch reaches them;
/// the three `prestage-*` points are reached by the job that makes the next batch's marker,
/// which runs beside the landing that started it.
pub const FAULT_POINTS: [&str; 8] = [
    "after-objects",
    "after-intent",
    PRESTAGE_START,
    PRESTAGE_STAGED,
    PRESTAGE_FLUSHED,
    "after-ref",
    "after-index",
    "before-ack",
];

/// The instant the service starts making the marker of the next batch's index lock,
/// before the marker's file exists.
/// Why:
///  The marker is made while a landing holds the lock,
/// so a kill here leaves the batch's intent and nothing else:
///  the next service settles it and the next commit needs no marker.
pub const PRESTAGE_START: &str = "prestage-start";

/// The instant the marker made ahead exists under its staging name and is recorded for the shutdown that removes it,
/// before its flush.
/// Why:
///  The file is not the lock,
/// so a kill here leaves a file the next service's `remove_dead_staging` collects and no lock
///  that could block a commit.
pub const PRESTAGE_STAGED: &str = "prestage-staged";

/// The instant the marker made ahead is complete and its flush has been asked for,
/// before the landing whose commit may take it can go on.
/// Why:
///  A kill here leaves a complete marker under its staging name and still no lock,
/// which the next service collects by the
///  name it carries.
pub const PRESTAGE_FLUSHED: &str = "prestage-flushed";

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

/// Environment variable naming a pipe the service waits on in the window between its address being bound and its first greeting,
/// which is what holds it where a slow recovery of the batches an earlier service left open holds it.
/// Why:
///   To a command,
/// a service in that window is a service that is alive and greets nothing yet,
/// and that is the state the fourth review of issue 6293 (finding 1) read its chain in.
/// A control that waited for a recovery of more than the start wait would be a wait made of load:
/// the step it needs held is a kernel read,
/// which the control holds and releases.
/// The hold ends when the control writes one byte,
/// and at `HOLD_PATIENCE` in any case,
/// so a stale variable cannot wedge a service.
pub const HOLD_BEFORE_SERVING_VARIABLE: &str = "CLI_GIT_TEST_ONLY_SERVICE_HOLD_BEFORE_SERVING";

/// The longest a service waits on the pipe a control holds,
/// and what it waits in all.
pub const HOLD_PATIENCE: std::time::Duration = std::time::Duration::from_secs(60);

/// How often the hold looks whether the control wrote its byte.
pub const HOLD_POLL: std::time::Duration = std::time::Duration::from_millis(10);

/// What:
///  Holds the service before its first greeting,
/// while the control named by the variable holds the other end of a pipe.
/// Why:
///   The pre-greeting window is where a service is alive with an address and nothing answering,
/// and no control can make a recovery last by itself without manufacturing load.
/// The pipe is the control's own:
/// the byte it writes ends the hold,
/// and the patience ends it if the control never does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdBeforeServing(); // reads one byte from the named pipe, or returns at the patience
/// ```
pub fn hold_before_serving() {
    let Some(path) = std::env::var_os(HOLD_BEFORE_SERVING_VARIABLE) else { return };
    let began: std::time::Instant = std::time::Instant::now();
    let deadline: std::time::Instant = began + HOLD_PATIENCE;
    // The pipe is opened without waiting for its writer, so a control that never opens it costs one look and nothing more.
    use std::os::unix::fs::OpenOptionsExt as _;
    let pipe: std::fs::File = loop {
        match std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK).open(std::path::Path::new(&path)) {
            Ok(pipe) => break pipe,
            // A pipe nobody made yet is a control that has not started its service yet: the hold waits for it as it waits for the byte.
            Err(_not_there_yet) if std::time::Instant::now() < deadline => std::thread::sleep(HOLD_POLL),
            Err(_not_there) => return,
        }
    };
    crate::service_log!("test hold before serving at {}: a control holds this service where a slow recovery holds it", std::path::Path::new(&path).display());
    use std::io::Read;
    let mut one: [u8; 1] = [0u8; 1];
    while std::time::Instant::now() < deadline {
        let mut source: &std::fs::File = &pipe;
        // Only the byte ends the hold: a read of nothing is a control that has not released it yet, and a pipe whose writer is not
        //  there yet reads the same.
        if matches!(source.read(&mut one), Ok(released) if released > 0) {
            break;
        }
        std::thread::sleep(HOLD_POLL);
    }
    crate::service_log!("the hold before serving ended after {} ms", began.elapsed().as_millis());
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
