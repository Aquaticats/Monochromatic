//! What:
//!  The commit service compiles the repository's forbidden-strings rules into the shared cache on its own time,
//! so the client that scans a commit finds them compiled.
//! Why:
//!  The client compiles the rules in the prepare phase of a commit when the cache holds no compiled form of their text:
//! between 1.2 s and 1.3 s as measured on 2026-11-07 under the host's load at the time,
//! and about 20 s on the first commit of a fresh cache under heavier load
//! (`forbidden_regex::parallel::build_rules`,
//! sampled on 2026-11-07).
//! The cache is shared by every worktree and keyed by the joined text of the rules files,
//! so one compile serves them all.
//! The service warms it on the first request it sees for a repository,
//! after a landing that changed a rules file,
//! and,
//! after a restart,
//! for the repository it remembers in its own state directory.
//! It never runs on the landing thread or the work loop:
//! the compile is a job of the blocking pool,
//! and a second warm while one runs is skipped (the next landing tries again).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! afterLanding(journal, repository, changedPaths); // fire and forget
//! ```

/// The client's own load of the rules,
/// so what is published is exactly what the client would look for.
use crate::scanner_adapter::CandidateScanner;
/// The choice of the rules files,
/// as the client makes it.
use crate::scanner_selection::{rules_candidate_paths, rules_source};
/// The repository's wrapper configuration.
use crate::config_file::load_repository_config;
/// The blocking pool,
/// for the compile.
use crate::blocking_job::run_blocking_job;
/// The process's runtime,
/// for the task that waits for the pool.
use crate::lazy_runtime::spawn_on_runtime;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// The file in the service's state directory that names the repository it serves.
pub const REMEMBERED_FILE: &str = "rules-repository";

/// The address space a warm needs beyond what the service already maps,
/// in bytes.
/// Measured on 2026-11-07 with this repository's real rules:
/// the service's peak address space went from 252 MiB without a warm to 503 MiB with one (resident 7 MiB to 227 MiB),
/// and its size before the warm was 154 MiB,
/// so the warm reached 349 MiB above what was mapped;
/// the value is that measurement with a margin for larger rules (rule `LQM`:
/// a shipped limit cites its measurement).
pub const WARM_ADDRESS_SPACE: u64 = 512 * 1024 * 1024;

/// What:
///  How many warm jobs have ended,
/// and how many of them warmed the cache,
/// so a control waits for the service's own answer instead of watching the clock.
/// Why:
///   A control of "a landing that changed no rules path warms nothing" otherwise sleeps and hopes the job ran,
/// and one of "the first landing warms" otherwise polls for the artifact and reads a slow host as a warm
///  that never came.
/// Neither counter exists in the release executable.
#[cfg(test)]
pub static WARM_JOBS_ENDED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// How many of the warm jobs that ended did compile the rules,
/// for the controls;
/// nothing in the release executable.
#[cfg(test)]
pub static WARMS_MADE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Records for the controls that a warm job ended,
/// and whether it warmed the cache.
#[cfg(test)]
fn note_a_warm_job_ended(warmed: bool) {
    if warmed {
        WARMS_MADE.fetch_add(1, Ordering::SeqCst);
    }
    WARM_JOBS_ENDED.fetch_add(1, Ordering::SeqCst);
}

/// Does nothing in the release executable.
#[cfg(not(test))]
fn note_a_warm_job_ended(_warmed: bool) {}

/// Whether the address space the service has left holds a warm.
/// Why:
///  A failed allocation ends a Rust program with an abort,
/// whatever it was doing,
/// so a warm that does not fit would end the service itself,
/// in the middle of a landing or before it listens
/// (`fault_limits::a_memory_limit_close_to_exhaustion_on_the_service_leaves_a_recoverable_repository`);
/// the warm is optional,
/// so it is skipped when a limit leaves less room than a compile needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// warmFits(addressSpaceLeft) // addressSpaceLeft === undefined || addressSpaceLeft >= WARM_ADDRESS_SPACE
/// ```
pub fn warm_fits(address_space_left: Option<u64>) -> bool {
    return match address_space_left {
        Some(left) => left >= WARM_ADDRESS_SPACE,
        None => true,
    };
}

/// The size of the file that holds the pages this process maps,
/// read without a heap allocation:
/// a service close to its address-space limit must not allocate to find out that it is.
const STATM_BYTES: usize = 128;

/// The pages the process maps (the first number of `/proc/self/statm`),
/// read into a buffer on the stack;
/// nothing when the file cannot be read.
fn mapped_pages() -> Option<u64> {
    let mut buffer: [u8; STATM_BYTES] = [0; STATM_BYTES];
    let mut file: std::fs::File = std::fs::File::open("/proc/self/statm").ok()?;
    let read: usize = std::io::Read::read(&mut file, &mut buffer).ok()?;
    let mut pages: u64 = 0;
    let mut digits: usize = 0;
    for byte in &buffer[..read] {
        if !byte.is_ascii_digit() {
            break;
        }
        pages = pages.saturating_mul(10).saturating_add(u64::from(*byte - b'0'));
        digits += 1;
    }
    return if digits == 0 { None } else { Some(pages) };
}

/// The address space the service may still map:
/// nothing when no limit is set,
/// and none left when a limit is set and the size cannot be read.
/// No step of it allocates on the heap.
fn address_space_left_now() -> Option<u64> {
    let limit: u64 = rustix::process::getrlimit(rustix::process::Resource::As).current?;
    let Some(pages) = mapped_pages() else { return Some(0) };
    // SAFETY: `sysconf` takes a constant and writes nothing.
    let page_bytes: i64 = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    // The largest page size a Linux machine uses stands in when the size cannot be read.
    let page: u64 = u64::try_from(page_bytes).unwrap_or(1 << 16);
    return Some(limit.saturating_sub(pages.saturating_mul(page)));
}

/// Whether a warm may run now:
/// the first question of every trigger,
/// asked before the trigger allocates,
/// writes a file or starts a thread.
fn room_for_a_warm() -> bool {
    return warm_fits(address_space_left_now());
}

/// Whether a warm runs now;
/// a second one is skipped,
/// and the next landing tries again.
static RUNNING: AtomicBool = AtomicBool::new(false);

/// What:
///  Compiles the rules of the repository at `repository_root` into the cache,
/// or finds them there.
/// Why:
///   `CandidateScanner::load` is the client's own load,
/// so what is published is exactly what the client would look for;
/// the
///  environment variable the client reads is not the service's,
/// so the rules named by configuration (or the default file) are warmed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await precompileRules(root) // elapsed
/// ```
///
/// # Errors
///
/// The text of the failure,
/// when the configuration or the rules cannot be loaded.
pub fn precompile_rules(repository_root: &Path) -> Result<Duration, String> {
    let began: Instant = Instant::now();
    let loaded = match load_repository_config(repository_root) {
        Ok(found) => found,
        Err(error) => return Err(error.message),
    };
    let options = &loaded.config.policies.forbidden_strings;
    let rules = match rules_source(options.rules_files.as_deref(), None, repository_root) {
        Ok(found) => found,
        Err(error) => return Err(error.to_string()),
    };
    return match CandidateScanner::load(&rules, options.builtin_rules) {
        Ok(_scanner) => Ok(began.elapsed()),
        Err(error) => Err(error.message),
    };
}

/// Whether any of `changed` (repository-relative,
/// `/` separated) is a rules file of the repository's configuration.
fn changes_a_rules_file(repository_root: &Path, changed: &[String]) -> bool {
    let Ok(loaded) = load_repository_config(repository_root) else { return true };
    let options = &loaded.config.policies.forbidden_strings;
    let Ok(rules) = rules_source(options.rules_files.as_deref(), None, repository_root) else { return true };
    let paths: Vec<Vec<u8>> = rules_candidate_paths(&rules, repository_root);
    for path in changed {
        if paths.contains(&path.as_bytes().to_vec()) {
            return true;
        }
    }
    return false;
}

/// The job of the blocking pool:
/// warms when it is the first look at the repository or a rules file changed,
/// says how long it took,
/// and leaves its answer for a control to read.
fn warm_job(repository_root: &Path, first: bool, changed: &[String]) {
    let warmed: bool = if !first && !changes_a_rules_file(repository_root, changed) { false } else { warm_and_say(repository_root) };
    note_a_warm_job_ended(warmed);
}

/// Compiles the rules into the cache and writes what came of it,
/// giving whether the cache was warmed.
fn warm_and_say(repository_root: &Path) -> bool {
    return match precompile_rules(repository_root) {
        Ok(elapsed) => {
            crate::service_log!("rules warmed repository={:?} warm_ms={}", repository_root, elapsed.as_millis());
            true
        }
        Err(error) => {
            crate::service_log!("rules not warmed repository={:?} reason={:?}", repository_root, error);
            false
        }
    };
}

/// Starts the job on the blocking pool unless one runs;
/// nothing when no worker can run it.
fn spawn_warm(repository_root: PathBuf, first: bool, changed: Vec<String>) {
    if !room_for_a_warm() || RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }
    let started = spawn_on_runtime(async move {
        let _ended = run_blocking_job(move || { warm_job(repository_root.as_path(), first, changed.as_slice()); }).await;
        RUNNING.store(false, Ordering::SeqCst);
    });
    if started.is_none() {
        RUNNING.store(false, Ordering::SeqCst);
    }
}

/// Starts a warm of `repository_root` on the blocking pool;
/// nothing when one is already running.
pub fn warm_in_background(repository_root: PathBuf) {
    spawn_warm(repository_root, true, Vec::new());
}

/// Whether the state directory does not yet name `repository`:
/// the first look at it,
/// which no lock decides,
/// because the file is the
///  one owner of the answer and survives a restart.
fn first_look_at(journal: &Path, repository: &Path) -> bool {
    return match std::fs::read_to_string(journal.join(REMEMBERED_FILE)) {
        Ok(text) => text.trim() != repository.to_string_lossy(),
        Err(_unreadable) => true,
    };
}

/// Writes the repository's root into the state directory,
/// whole or not at all;
/// a failure only costs the warm after a restart.
fn remember(journal: &Path, repository: &Path) {
    let target: PathBuf = journal.join(REMEMBERED_FILE);
    let staging: PathBuf = journal.join(format!("{REMEMBERED_FILE}.new"));
    if let Err(error) = std::fs::write(&staging, format!("{}\n", repository.display())) {
        crate::service_log!("the repository was not remembered for the rules warm in {}: {error}", journal.display());
        return;
    }
    if let Err(error) = std::fs::rename(&staging, &target) {
        crate::service_log!("the repository was not remembered for the rules warm in {}: {error}", journal.display());
    }
}

/// What the service does after a landing:
/// remembers the repository on the first one,
/// and warms on the first one and when a landing
///  changed a path of the rules.
pub fn after_landing(journal: &Path, repository: &Path, changed: &[String]) {
    // Under a limit that leaves no room nothing here runs: not the file read, not the write, not a thread.
    if !room_for_a_warm() {
        return;
    }
    let first: bool = first_look_at(journal, repository);
    if first {
        remember(journal, repository);
    }
    spawn_warm(repository.to_path_buf(), first, changed.to_vec());
}

/// Warms the repository the state directory remembers,
/// at the start of the service.
pub fn warm_remembered(journal: &Path) {
    if !room_for_a_warm() {
        return;
    }
    let Ok(text) = std::fs::read_to_string(journal.join(REMEMBERED_FILE)) else { return };
    let repository: PathBuf = PathBuf::from(text.trim());
    if repository.is_absolute() && repository.is_dir() {
        warm_in_background(repository);
    }
}

/// The warm is controlled in the test module.
#[cfg(test)]
#[path = "service_rules_precompile_tests.rs"]
mod tests;
