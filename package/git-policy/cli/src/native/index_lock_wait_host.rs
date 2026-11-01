//! What:
//!  The host side effects of the index-lock wait:
//!  `/proc` and Git-free lock reads,
//!  real sleeps,
//!       operating-system randomness and standard error.
//! Why:
//!  `WaitEffects` is the seam the controls replace;
//!  this is what the executable runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const HOST_WAIT_EFFECTS = { gather, sleep: wait, random: Math.random, report: stderr.write, now: performance.now };
//! ```

/// Foreign-lock evidence readers used by the host.
use super::index_lock_evidence::{Evidence, gather_index_lock_evidence, read_lock_metadata};
/// The wait's side-effect seam.
use super::index_lock_wait::WaitEffects;
/// When the wait repeats its notice.
use super::wait_progress::Cadence;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};
/// `Instant` is a monotonic clock reading.
use std::time::{Duration, Instant};

/// What:
///  The host effects for one wait on one real index.
/// Why:
///   The clock starts when the wait does,
///  so `now_ms` measures the wait's own time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const effects = { ...HOST_WAIT_EFFECTS, now: () => performance.now() - start };
/// ```
pub struct HostWaitEffects {
    /// The worktree's real index;
    ///  the lock is `<real index>.lock`.
    pub real_index: PathBuf,
    /// When the wait started.
    pub started: Instant,
}

/// What:
///  Turn an I/O error into the text the wait reports.
/// Why:
///   A named function keeps the conversion free of closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (error: Error) => error.message
/// ```
fn io_text(error: std::io::Error) -> String {
    return error.to_string();
}

/// What:
///  The lock path of a real index.
/// Why:
///   Git names the lock `<index>.lock` beside the index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${realIndexPath}.lock`
/// ```
pub fn lock_path_of(real_index: &Path) -> PathBuf {
    let mut name: std::ffi::OsString = real_index.as_os_str().to_owned();
    name.push(".lock");
    return PathBuf::from(name);
}

impl WaitEffects for HostWaitEffects {
    fn lock_present(&mut self) -> Result<bool, String> {
        let lock: PathBuf = lock_path_of(self.real_index.as_path());
        match read_lock_metadata(lock.as_path()) {
            Ok(Some(_)) => return Ok(true),
            Ok(None) => return Ok(false),
            Err(error) => return Err(io_text(error)),
        }
    }

    fn gather(&mut self) -> Result<Option<Evidence>, String> {
        match gather_index_lock_evidence(self.real_index.as_path()) {
            Ok(found) => return Ok(found),
            Err(error) => return Err(io_text(error)),
        }
    }

    fn sleep(&mut self, ms: u64) {
        std::thread::sleep(Duration::from_millis(ms));
    }

    fn random(&mut self) -> f64 {
        let mut bytes: [u8; 4] = [0; 4];
        // A failed operating-system random source falls back to the middle of the range,
        // which keeps the schedule's expected value.
        if getrandom::fill(&mut bytes).is_err() {
            return 0.5;
        }
        return f64::from(u32::from_le_bytes(bytes)) / 4_294_967_296.0;
    }

    fn now_ms(&mut self) -> f64 {
        return self.started.elapsed().as_secs_f64() * 1000.0;
    }

    fn report(&mut self, line: &str) {
        use std::io::Write;
        // A standard-error write failure cannot be reported anywhere else, so it is dropped.
        let _ = std::io::stderr().write_all(line.as_bytes());
    }

    fn cadence(&mut self) -> Cadence {
        return Cadence::from_environment();
    }
}

#[cfg(test)]
#[path = "index_lock_wait_host_tests.rs"]
mod tests;
