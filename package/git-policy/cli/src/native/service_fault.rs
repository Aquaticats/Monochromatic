//! What:
//!  Test-only fault points of the resident service: `CLI_GIT_TEST_ONLY_SERVICE_FAULT=<point>` ends the service with
//!  `SIGKILL` the moment it reaches the named point of a batch.
//! Why:
//!  The kill tests need a deterministic instant, as the transaction phase markers give the wrapper
//!  (`transaction_phase.rs`). The marker is compiled into the release executable, and only the explicit variable arms it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (process.env.CLI_GIT_TEST_ONLY_SERVICE_FAULT === point) process.kill(process.pid, 'SIGKILL');
//! ```

/// Environment variable arming one fault point.
pub const FAULT_VARIABLE: &str = "CLI_GIT_TEST_ONLY_SERVICE_FAULT";

/// The instants of a batch at which a kill is tested, in the order a batch reaches them.
pub const FAULT_POINTS: [&str; 5] = ["after-objects", "after-intent", "after-ref", "after-index", "before-ack"];

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
    if std::env::var(FAULT_VARIABLE).is_ok_and(|armed| { return armed == point; }) {
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
///  Says that a step of a landing happened, for a control that checks the order of durable steps.
/// Why:
///   A directory sync cannot be seen from outside the process, so a control reads the order the landing reports; the release
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
///   A landing whose index update fails after the branch moved is the case recovery exists for, and Git cannot be made to fail
///  there on demand. A control names the point; the release executable never fails a step this way.
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
///  Leaves a file named for request `id` in the directory the variable names, when it is armed.
/// Why:
///   A control that sends requests in a set order needs to know that the service holds one before it sends the next; a pause
///  of its own can only guess. The file appears after the request is in the queue, so what the control sees is the service's own
///  state. Only the explicit variable arms it, as with the fault points.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (process.env.CLI_GIT_TEST_ONLY_SERVICE_QUEUED_DIRECTORY) writeFile(join(dir, safe(id)), '')
/// ```
pub fn queued(id: &str) {
    let Some(directory) = std::env::var_os(QUEUED_DIRECTORY_VARIABLE) else { return };
    let name: String = id.chars().map(|character| { return if character.is_ascii_alphanumeric() { character } else { '_' }; }).collect();
    let path: std::path::PathBuf = std::path::Path::new(&directory).join(format!("queued-{name}"));
    if let Err(error) = std::fs::write(&path, b"") {
        crate::service_warn!("cannot leave the queued-request file {}: {error}", path.display());
    }
}

/// Environment variable pinning the number the service counts its batches from, in place of the clock reading it starts from.
pub const FIRST_BATCH_VARIABLE: &str = "CLI_GIT_TEST_ONLY_SERVICE_FIRST_BATCH";

/// What:
///  The number the variable pins the service's batch count to, when it is armed with a number.
/// Why:
///   Two services that share a journal count from clock readings, which are equal only by chance; a control that needs them equal
///  pins both. Only the explicit variable arms it, as with the fault points.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pinned = Number(process.env.CLI_GIT_TEST_ONLY_SERVICE_FIRST_BATCH) // undefined when unset or not a number
/// ```
pub fn pinned_first_batch() -> Option<u64> {
    return std::env::var(FIRST_BATCH_VARIABLE).ok().and_then(|text| { return text.parse::<u64>().ok(); });
}

/// Hooks a control sets to run its own code at a point of a landing, on the thread that reaches the point; never part of the
///  release executable.
#[cfg(test)]
pub(crate) mod hooks {
    use std::cell::RefCell;

    thread_local! {
        /// The steps of a landing on this thread, in the order they happened.
        static EVENTS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
        /// The point and the code to run at it.
        static HOOK: RefCell<Option<(String, std::rc::Rc<dyn Fn()>)>> = const { RefCell::new(None) };
        /// The points whose steps fail on this thread.
        static FAILING: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    }

    /// Make the step at `point` fail on this thread's landings until `clear_failures`.
    pub(crate) fn fail_at(point: &str) {
        FAILING.with(|failing| { failing.borrow_mut().push(point.to_string()); });
    }

    /// Let every step succeed again.
    pub(crate) fn clear_failures() {
        FAILING.with(|failing| { failing.borrow_mut().clear(); });
    }

    /// Whether the step at `point` fails.
    pub(super) fn failing(point: &str) -> bool {
        return FAILING.with(|failing| { return failing.borrow().iter().any(|named| { return named == point; }); });
    }

    /// Run `code` whenever the landing of this thread reaches `point`.
    pub(crate) fn set(point: &str, code: std::rc::Rc<dyn Fn()>) {
        HOOK.with(|hook| { *hook.borrow_mut() = Some((point.to_string(), code)); });
    }

    /// Remember that `event` happened.
    pub(super) fn record(event: &str) {
        EVENTS.with(|events| { events.borrow_mut().push(event.to_string()); });
    }

    /// The steps reported on this thread since the last call, in order.
    pub(crate) fn take_events() -> Vec<String> {
        return EVENTS.with(|events| { return std::mem::take(&mut *events.borrow_mut()); });
    }

    /// Run the hook if it is for `point`.
    pub(super) fn run(point: &str) {
        let found: Option<std::rc::Rc<dyn Fn()>> = HOOK.with(|hook| { return hook.borrow().as_ref().filter(|(named, _)| { return named == point; }).map(|(_, code)| { return std::rc::Rc::clone(code); }); });
        if let Some(hook) = found {
            hook();
        }
    }
}
