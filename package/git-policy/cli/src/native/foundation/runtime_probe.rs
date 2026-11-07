//! What:
//!  The test-only entry that builds the process's runtime and runs one blocking job,
//! then prints what happened.
//! Why:
//!  The runtime has no production caller yet (step T3 of `doc/planning/git-policy-cli-full-tokio.md`),
//! so the controls that
//!  start the built executable under `RLIMIT_NPROC` and `RLIMIT_AS` reach the builder through this entry,
//! as
//!  `CLI_GIT_TEST_ONLY_PHASE_SIGNAL` reaches the transaction phases.
//! Nothing in production sets the variable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (process.env.CLI_GIT_TEST_ONLY_RUNTIME_PROBE) { console.log(await probe()); process.exit(0); }
//! ```

use crate::blocking_job::run_blocking_job;
use crate::lazy_runtime::{BuiltFlavor, RuntimeError, built_flavor, run_on_runtime};
use std::ffi::OsString;

/// The environment variable that makes the executable run this probe instead of a command.
pub const RUNTIME_PROBE_VARIABLE: &str = "CLI_GIT_TEST_ONLY_RUNTIME_PROBE";

/// Whether `environment` asks for the probe.
pub fn is_requested(environment: &[(OsString, OsString)]) -> bool {
    for (name, _value) in environment {
        if name.as_os_str() == RUNTIME_PROBE_VARIABLE {
            return true;
        }
    }
    return false;
}

/// The text of a flavor,
/// as the controls read it.
fn flavor_text(flavor: Option<BuiltFlavor>) -> String {
    return match flavor {
        None => String::from("none"),
        Some(BuiltFlavor::CurrentThread) => String::from("current-thread"),
        Some(BuiltFlavor::MultiThread { workers }) => format!("multi-thread workers={workers}"),
        Some(BuiltFlavor::CurrentThreadAfterDeadline) => String::from("current-thread-after-deadline"),
        Some(BuiltFlavor::CurrentThreadAfterFailure) => String::from("current-thread-after-failure"),
    };
}

/// The blocking job the probe runs:
/// a value that shows the job ran.
fn the_probe_job() -> u8 {
    return 7_u8;
}

/// Builds the runtime,
/// runs one blocking job on it,
/// prints `flavor=...` and `blocking=...` lines,
/// and returns the exit code:
///  zero when a runtime was built,
/// one when none could be.
pub fn run() -> i32 {
    let blocking = run_on_runtime(run_blocking_job(the_probe_job));
    println!("flavor={}", flavor_text(built_flavor()));
    return match blocking {
        Ok(Ok(_value)) => {
            println!("blocking=ok");
            0
        }
        Ok(Err(error)) => {
            println!("blocking=error {error}");
            0
        }
        Err(RuntimeError::Nested) | Err(RuntimeError::CouldNotStart(_)) => {
            println!("blocking=no-runtime");
            1
        }
    };
}
