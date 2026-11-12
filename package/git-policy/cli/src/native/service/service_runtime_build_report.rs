//! What:
//!  The one place that says how the service's own runtime was built,
//! the words a task that cannot get a worker ends the service with,
//! and the way a task the service cannot serve without waits for that build and says so.
//! Why:
//!  Three tasks of a service cannot be left out:
//! the acceptor of connections,
//! the after-commit work,
//! and the shutdown after a stop.
//! Each of them used to end the service at `BUILD_DEADLINE` itself when the runtime came out as a stand-in,
//! which failed a person's commit for a build that arrived a moment later
//! (`RUNTIME_BUILD_PATIENCE` in `service/wait_ceilings.rs`).
//! A service that ends this way must say which way its build went,
//! because a build the system refused and one that was merely slow ask different things of the person,
//! and a service that waits for that build must say so while it waits,
//! because the socket it holds takes connections and answers none of them until its runtime arrives.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const task = startWhatTheServiceCannotServeWithout('the acceptor of connections', accept(listener, shared));
//! if (task === undefined) { console.error(noWorkerWords('the acceptor of connections')); process.exit(4); }
//! ```

use crate::lazy_runtime::{BUILD_DEADLINE, BuiltFlavor, RuntimeBuild, a_build_is_still_on_its_way, flavor_words, runtime_build, spawn_on_runtime_within, tasks_can_run};
use crate::wait_ceilings::RUNTIME_BUILD_PATIENCE;

/// What:
///  The line that says the runtime this process serves from is a stand-in rather than the multi-thread one,
/// or nothing when it is the multi-thread one.
/// Why:
///   A healthy service builds its runtime in milliseconds and says nothing;
/// a stand-in is the one state whose cause a person must be able to read,
/// and its two causes ask different things of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (build.flavor.kind === 'multi') return undefined; return `... ${build.flavor} in ${build.spent}ms`;
/// ```
fn stand_in_words(build: RuntimeBuild) -> Option<String> {
    if matches!(build.flavor, BuiltFlavor::MultiThread { .. }) {
        return None;
    }
    let flavor: String = flavor_words(build.flavor);
    return Some(format!(
        "this service serves from a runtime that is not the multi-thread one: it was built as {flavor} in {} ms, and the build of a service's own runtime is held to \
         {BUILD_DEADLINE:?} so that one the system refuses cannot hold a commit. A build that missed that deadline is kept for the work it was built for, and a service \
         waits up to {} ms more for it before it gives up on the tasks it cannot serve without",
        build.spent.as_millis(),
        RUNTIME_BUILD_PATIENCE.as_millis()
    ));
}

/// What:
///  Writes `stand_in_words` to the service's log when there is a line.
/// Why:
///  The reading and the wording are one function of their own,
/// so a control states a runtime build and reads the line without capturing a standard error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = standInWords(build); if (line) console.error(line);
/// ```
pub(crate) fn say_how_the_runtime_was_built() {
    let Some(build) = runtime_build() else { return };
    let Some(line) = stand_in_words(build) else { return };
    crate::service_warn!("{line}");
}

/// What:
///  The words of the wait for the build of the runtime a service cannot serve without,
/// naming the task that waits,
/// the deadline that build missed,
/// and the wait it is given.
/// Why:
///  This wait happens behind a socket that already takes connections and answers none of them,
/// so a person whose commit waits for the service must be able to read what the wait is for in the service's log.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// waitingWords('the after-commit work') // '... waits up to 60000 ms for it'
/// ```
fn waiting_words(what: &str) -> String {
    return format!(
        "the multi-thread runtime this service cannot serve without is still being built: {what} runs only on a worker of its own, the build of that runtime \
         outlasted its {BUILD_DEADLINE:?} deadline, and this service waits up to {} ms for it to arrive before it ends and lets the service manager start it \
         again. Nothing is answered until it arrives",
        RUNTIME_BUILD_PATIENCE.as_millis()
    );
}

/// What:
///  Starts `future` as a task the service cannot do its work without,
/// saying on the service's log what is waited for while it waits for a build that outlasted its deadline,
/// or answering nothing when no worker arrives in `RUNTIME_BUILD_PATIENCE`.
/// Why:
///   The wait and its line belong together,
/// so the three tasks a service cannot be without reach both the same way;
/// a build the system refused has nothing coming and is never waited for,
/// and the caller is answered at once instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const task = startWhatTheServiceCannotServeWithout('the acceptor of connections', acceptConnections(listener, shared));
/// ```
pub(crate) fn start_what_the_service_cannot_serve_without<F>(what: &str, future: F) -> Option<tokio::task::JoinHandle<F::Output>>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    if !tasks_can_run() && a_build_is_still_on_its_way() {
        crate::service_warn!("{}", waiting_words(what));
    }
    return spawn_on_runtime_within(RUNTIME_BUILD_PATIENCE, future);
}

/// What:
///  The words a task that must have a worker ends the service with,
/// naming the task,
/// how the build went,
/// the wait a build that missed its deadline is given,
/// and what a person can check.
/// Why:
///  A service that answers no one ends and is started again by the manager,
/// so the person reads this line to know whether their machine refuses threads or is slow to start them;
/// it names the wait as the bound it is and never claims one this process did not keep,
/// because a build the system refused has nothing coming and is not waited for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new Error(noWorkerWords('the acceptor of connections'));
/// ```
pub(crate) fn no_worker_words(what: &str) -> String {
    let built: String = match runtime_build() {
        Some(build) => format!("the runtime was built as {} in {} ms", flavor_words(build.flavor), build.spent.as_millis()),
        None => String::from("the runtime was never built"),
    };
    return format!(
        "cannot start {what}: this process has no worker thread to run it on, so it ends here and the manager starts it again. {built}. A build that missed its \
         {BUILD_DEADLINE:?} deadline is waited for up to {} ms, and no wait brings a build the system refused or one that failed. The system may refuse \
         threads or be slow to start them: check `ulimit -u`, the control group's `pids.max` and its `memory.max`",
        RUNTIME_BUILD_PATIENCE.as_millis()
    );
}

/// The words of a runtime build are controlled in this module's test file.
#[cfg(test)]
#[path = "service_runtime_build_report_tests.rs"]
mod tests;
