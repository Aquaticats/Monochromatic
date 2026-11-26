//! What:
//!  One `#[ignore]` child test run in a process that may write no file past a size limit.
//! Why:
//!  A file-size limit is the privilege-free way to make a write fail half way,
//!  and the limit is a property of the whole process,
//!  so the child runs alone and the parent's other tests never see it.
//!  The child's three streams are the null device or pipes this file reads:
//!  it inherits no stream of this process,
//!  so a parent whose standard output is a regular file already past the limit
//!  cannot make the child's first print fail with `EFBIG`
//!  and turn a passing control red.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! runChildUnderFileSizeLimit(name, 4096, directory); // spawnSync(testBinary, ['--exact', name, '--ignored'], { limits: { fsize: limit } })
//! ```

/// The one request for a child of this test binary, and the wait that ends it.
use super::child_test_process::{ChildTest, finished};
/// `OsStr` is borrowed operating-system text of raw bytes.
use std::ffi::OsStr;
/// `ExitStatus` is how a process ended: an exit code or the signal that ended it.
use std::process::ExitStatus;

/// Variable that tells a child test the directory it works in.
pub(crate) const LIMITED_DIRECTORY_VARIABLE: &str = "CLI_GIT_LIMITED_CHILD_DIRECTORY";

/// What:
///  Run one `#[ignore]` child test in a process that may write no file past `limit` bytes,
/// and return how it ended.
/// Why:
///  A file-size limit is the privilege-free way to make a write fail half way:
/// the bytes below the limit are written
///  and the rest fails with `EFBIG` (`SIGXFSZ` is ignored so that the error reaches the caller instead of ending the
///  process).
/// The limit is a property of the whole process,
/// so the child runs alone and the parent's other tests
///  never see it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runChildUnderFileSizeLimit(testName: string, limit: number, directory: string): ExitStatus {
///   const ended = finished({ exactTest: testName, ignored: true, environment: [[LIMITED_DIRECTORY_VARIABLE, directory]], beforeTheExec: () => { ignore(SIGXFSZ); setrlimit(RLIMIT_FSIZE, limit); } });
///   return ended.status;
/// }
/// ```
pub(crate) fn run_child_under_file_size_limit(test_name: &str, limit: u64, directory: &std::path::Path) -> ExitStatus {
    let environment: [(&str, &OsStr); 1] = [(LIMITED_DIRECTORY_VARIABLE, directory.as_os_str())];
    let ended = finished(ChildTest {
        exact_test: test_name,
        environment: environment.as_slice(),
        ignored: true,
        before_the_exec: Some(Box::new(move || {
            // SAFETY: between fork and exec only `signal` and `setrlimit`, both async-signal-safe, run.
            unsafe {
                libc::signal(libc::SIGXFSZ, libc::SIG_IGN);
                let cap: libc::rlimit = libc::rlimit { rlim_cur: limit, rlim_max: limit };
                if libc::setrlimit(libc::RLIMIT_FSIZE, &cap) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
            }
            return Ok(());
        })),
        ..ChildTest::default()
    })
    .expect("the child under the size limit ends within the deadline");
    return ended.status;
}
