//! What:
//!  A scan of the processes run on the blocking pool and awaited for no longer than its ceiling.
//! Why:
//!  One read of a command line can wait for a process stuck in the kernel (`earlier_service_process_look.rs`),
//! so the wait for the scan is bounded:
//! a scan that does not answer in time counts as one that could not look,
//! its job is left behind in the pool (it ends when the kernel lets the read go),
//! and whoever asked is told to try again.
//! The wait is an `.await`,
//! so a task beside the look keeps running.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const found = await Promise.race([runInPool(() => scan(look)), sleep(ceiling).then(() => whenUnanswered())]);
//! ```

use crate::blocking_job::run_blocking_job;
use crate::lazy_runtime::run_on_runtime;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// What:
///  Runs `scanner` on the blocking pool and awaits it for `ceiling`;
/// `when_unanswered` gives the answer when the scan could not run or did not answer in time.
/// Why:
///  The caller decides what "could not look" means in its own terms.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await scanUnderCeiling(look, ceiling, procRoot, scan, cannotLook)
/// ```
pub(crate) async fn scan_under_ceiling<Look, Found>(look: Look, ceiling: Duration, proc_root: PathBuf, scanner: fn(&Look, &Path) -> Found, when_unanswered: fn() -> Found) -> Found
where
    Look: Send + 'static,
    Found: Send + 'static,
{
    let scan = run_blocking_job(move || { return scanner(&look, proc_root.as_path()); });
    return match tokio::time::timeout(ceiling, scan).await {
        Ok(Ok(found)) => found,
        Ok(Err(error)) => {
            crate::service_error!("the scan could not run ({error}), so it is taken as not made");
            when_unanswered()
        }
        Err(error) => {
            crate::service_error!("the scan gave no answer ({error}), so it is taken as not made");
            when_unanswered()
        }
    };
}

/// `scan_under_ceiling` for a caller that is not a task:
/// run to its end on the process's runtime;
/// a runtime that cannot start,
/// or a call from inside it,
/// is a scan that could not be made.
pub(crate) fn scan_under_ceiling_to_its_end<Look, Found>(look: Look, ceiling: Duration, proc_root: &Path, scanner: fn(&Look, &Path) -> Found, when_unanswered: fn() -> Found) -> Found
where
    Look: Send + 'static,
    Found: Send + 'static,
{
    return match run_on_runtime(scan_under_ceiling(look, ceiling, proc_root.to_path_buf(), scanner, when_unanswered)) {
        Ok(found) => found,
        Err(error) => {
            crate::service_error!("the scan could not be started ({error}), so it is taken as not made");
            when_unanswered()
        }
    };
}
