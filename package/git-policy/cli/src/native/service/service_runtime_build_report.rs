//! What:
//!  The one place that says how the service's own runtime was built,
//! and the words a task that cannot get a worker ends the service with.
//! Why:
//!  Three tasks of a service cannot be left out:
//! the acceptor of connections,
//! the after-commit work,
//! and the shutdown after a stop.
//! Each of them used to end the service at `BUILD_DEADLINE` itself when the runtime came out as a stand-in,
//! which failed a person's commit for a build that arrived a moment later
//! (`LATE_BUILD_PATIENCE` in `foundation/lazy_runtime.rs`).
//! A service that ends this way must say which way its build went,
//! because a build the system refused and one that was merely slow ask different things of the person.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! sayHowTheRuntimeWasBuilt(); if (!worker) exit(4); // the line names the flavor and the wait
//! ```

use crate::lazy_runtime::{BUILD_DEADLINE, BuiltFlavor, LATE_BUILD_PATIENCE, RuntimeBuild, flavor_words, runtime_build};

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
        LATE_BUILD_PATIENCE.as_millis()
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
///  The words a task that must have a worker ends the service with,
/// naming the task,
/// how the build went,
/// how long the service waited for it,
/// and what a person can check.
/// Why:
///  A service that answers no one ends and is started again by the manager,
/// so the person reads this line to know whether their machine refuses threads or is slow to start them.
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
        "cannot start {what}: this process has no worker thread to run it on, so it ends here and the manager starts it again. {built}, and it had still \
         given no worker {} ms later, which is how long a service waits for a build that missed its {BUILD_DEADLINE:?} deadline. The system may refuse \
         threads or be slow to start them: check `ulimit -u`, the control group's `pids.max` and its `memory.max`",
        LATE_BUILD_PATIENCE.as_millis()
    );
}

/// The words of a runtime build are controlled in this module's test file.
#[cfg(test)]
#[path = "service_runtime_build_report_tests.rs"]
mod tests;
