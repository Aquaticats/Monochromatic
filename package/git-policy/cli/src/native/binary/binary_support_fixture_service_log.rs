//! What:
//!  What a fixture's commit services write on their standard error:
//! the pipe that keeps it while they run,
//! the bytes of it kept in this process's memory beside the file,
//! and the end of them that a failure carries.
//! Why:
//!  A service that refuses a step or ends says so only on its own error stream,
//! an inherited stream mixes that line with the output of every other control running at the same moment,
//! and a panic removes the fixture the line belongs to;
//! so the words go to a file beside the fixture,
//! which is the one place a control reads them from.
//!  A file that cannot be made or written must not cost a failure its words:
//! the same bytes are kept in this process's memory,
//! `service_log` hands them on,
//! and the words it writes say why the file holds none.
//!  The copy runs on a thread of its own,
//! so the file and the memory hold what a service wrote only once that thread has read it:
//! the pipe closes when the last process holding it ends,
//! and `kept_stderr_pipe` answers a channel that says the copy has ended.
//! A read that decides anything waits for that event first (`wait_for_the_copies`),
//! and a copy that has not ended by the bound is said in the answer rather than shown as an empty log.
//!  The service's own lines never leave the fixture:
//! nothing copies them to a stream the fixture does not own,
//! because a process that runs the tests may have the journal on that stream,
//! and then a throwaway fixture's service words would be in the user journal of the machine
//! (`doc/troubleshooting/git-policy-cli-test-services-write-into-the-user-journal.md`).
//!  The one edge is a failure that quotes them:
//! `service_log` hands a failure a quotation,
//! and the test runner prints that failure on its own stream,
//! which may be the journal.
//! So every line of the quotation begins with `QUOTED_MARK`,
//! which no line a service writes can begin with
//! (a service's line on standard error begins with its own name),
//! and a reader of the journal can tell the quotation from a line a service wrote.
//!  The one line that says the log file failed is written on this process's standard error,
//! and only after asking whether that stream is the journal's
//! (`names_the_stream`, the reading `systemd.exec(5)` gives for `$JOURNAL_STREAM`),
//! so even that line reaches no journal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! command.stderr = keptStderrPipe(root); // copies into `${root}/service.log` and into this process's memory
//! const said = serviceLog(root).slice(-8 * 1024);
//! ```

use std::collections::HashMap;
use std::io::{Read, Seek, Write};
use std::os::fd::OwnedFd;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// The file under the fixture root that keeps what its services wrote on standard error.
/// A failure names it,
/// so the service's own words outlive the fixture that a panic removes.
pub const SERVICE_LOG_FILE: &str = "service.log";

/// The environment variable the service manager sets to the device and inode of the stream it connected to the journal
/// (`systemd.exec(5)`).
pub const STREAM_VARIABLE: &str = "JOURNAL_STREAM";

/// How many bytes of a service log a failure carries:
/// a service writes a line per landing,
/// so the end of the log says how it ended,
/// and the rest would bury the failure it belongs to.
const SERVICE_LOG_KEPT_BYTES: usize = 8 * 1024;

/// What:
///  The words each fixture's services wrote,
/// kept in this process's memory,
/// and why the file under the fixture holds none of them when it could not be written.
/// Why:
///   One write that fails must cost a diagnostic its file but never its words:
/// a failure quotes `service_log`,
/// which reads this when the file is missing, short or unwritable.
///  A fixture root is one key: several services of one fixture write into one log,
/// as they append to one file.
struct KeptHere {
    /// The last bytes the fixture's services said.
    bytes: Vec<u8>,
    /// Why `service.log` could not be opened or written,
    /// once it could not.
    file_failure: Option<String>,
    /// Why a copy of a stream had not ended within its bound,
    /// once one had not.
    copy_unfinished: Option<String>,
}

/// The memory above, made on the first fixture's first service.
static KEPT: OnceLock<Mutex<HashMap<PathBuf, KeptHere>>> = OnceLock::new();

/// The memory above.
fn kept() -> &'static Mutex<HashMap<PathBuf, KeptHere>> {
    return KEPT.get_or_init(|| { return Mutex::new(HashMap::new()); });
}

/// What:
///  Whether `variable` names the file with this device and inode,
/// which is how `systemd.exec(5)` says to tell that a stream is still the journal.
/// Why:
///   The variable alone is not enough: a redirection leaves it set.
/// The reading is the service's own (`journald_native::stream_matches`),
/// kept here as the same three lines so this support needs nothing but `std`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesTheStream = (variable, device, inode) => variable?.split(':').map(Number) is [device, inode]
/// ```
pub fn names_the_stream(variable: Option<&str>, device: u64, inode: u64) -> bool {
    let Some(text) = variable else {
        return false;
    };
    let Some((named_device, named_inode)) = text.split_once(':') else {
        return false;
    };
    return named_device.trim().parse::<u64>().ok() == Some(device) && named_inode.trim().parse::<u64>().ok() == Some(inode);
}

/// What:
///  Whether this process's standard error is the journal's stream.
/// Why:
///   The one line about a failed log file is the only thing this support writes on a stream the fixture does not own,
/// and a run given the journal would put it there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const journalOnStderr = namesTheStream(process.env.JOURNAL_STREAM, statSync('/proc/self/fd/2').dev, statSync('/proc/self/fd/2').ino)
/// ```
fn stderr_is_the_journal() -> bool {
    let Ok(named) = std::env::var(STREAM_VARIABLE) else {
        return false;
    };
    let Ok(status) = std::fs::metadata("/proc/self/fd/2") else {
        return false;
    };
    return names_the_stream(Some(named.as_str()), status.dev(), status.ino());
}

/// What:
///  The one line this support writes when `service.log` could not be opened or written,
/// naming the file and the reason,
/// and saying where the words are instead.
/// Why:
///   A test's output must say that logging failed and why,
/// and a person reads the reason there rather than deducing it from an empty failure message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const note = (path, reason) => `the fixture's service log ${path} took nothing (${reason}); the words are kept in this process and a failure quotes them`
/// ```
pub fn log_file_failure_note(path: &Path, reason: &str) -> String {
    return format!(
        "the fixture's service log {} took nothing ({reason}); what its services said is kept in this process, and a failure quotes it",
        path.display()
    );
}

/// What:
///  Write `text` as one line on this process's standard error,
/// unless that stream is the journal's.
/// Why:
///   This process's standard error is the stream a person running the tests reads,
/// and a run given the journal must get nothing.
/// A stream that cannot be written is no failure of the fixture.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!journalOnStderr()) console.error(text);
/// ```
fn say_unless_the_journal(text: &str) {
    if stderr_is_the_journal() {
        return;
    }
    let mut stderr = std::io::stderr();
    let _said = stderr.write_all(format!("{text}\n").as_bytes());
    let _flushed = stderr.flush();
}

/// What:
///  The bytes in `kept` cut to the last `SERVICE_LOG_KEPT_BYTES`,
/// keeping the end,
/// which is where a service says how it ended.
/// Why:
///   The memory is one buffer per fixture for the whole run of a test process,
/// and a fixture's services may write megabytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// kept.splice(0, Math.max(0, kept.length - LIMIT))
/// ```
fn keep_the_end(kept: &mut Vec<u8>) {
    let too_many: usize = kept.len().saturating_sub(SERVICE_LOG_KEPT_BYTES);
    if too_many > 0 {
        kept.drain(..too_many);
    }
}

/// What:
///  Record that the log file could not be opened or written,
///  and say it on this process's standard error once.
/// Why:
///   The reason belongs to `service_log`,
/// which a failure quotes,
/// and to the test's output,
///  which a person reading a passing or a failing run sees once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entry.fileFailure = reason; if (first) sayUnlessTheJournal(logFileFailureNote(path, reason))
/// ```
fn record_failure(root: &Path, path: &Path, reason: &str) {
    let first: bool = {
        let mut all = kept().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let entry: &mut KeptHere = all.entry(root.to_path_buf()).or_insert_with(|| { return KeptHere { bytes: Vec::new(), file_failure: None, copy_unfinished: None }; });
        let first: bool = entry.file_failure.is_none();
        entry.file_failure = Some(String::from(reason));
        first
    };
    if first {
        say_unless_the_journal(log_file_failure_note(path, reason).as_str());
    }
}

/// What:
///  The seam in front of a copy:
///  while a control holds it,
///  the copy has read nothing,
///  so a read that races the copy is held at the code's own seam instead of a sleep that only makes the race likely.
/// Why:
///   A control that had to sleep to meet the race would prove what the clock did,
/// not what the code does;
/// this seam is the same place the copy runs, held open.
///  A copy waits for a permit,
///  and a control that never releases it leaves the copy held until the fixture ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const seam = holdTheCopy(root); // that root's copies wait here; seam.release() lets them go
/// ```
pub struct CopySeam {
    /// The root whose copies this seam holds.
    root: PathBuf,
    /// The queue the held copies wait on;
    /// closing it, by dropping this end, lets every one of them go.
    release: Mutex<Option<std::sync::mpsc::Sender<()>>>,
}

/// The seam each root's copies wait at, while a control holds them:
/// one root's seam never holds another fixture's copies, because tests run at once in one process.
static SEAM: OnceLock<Mutex<HashMap<PathBuf, std::sync::Arc<Mutex<std::sync::mpsc::Receiver<()>>>>>> = OnceLock::new();

/// How many of each root's copies sit at its seam right now:
/// a control reads it to prove every copy it started is held before it lets the seam go,
/// so the control tests the release and not the machine's speed.
static WAITING_AT_THE_SEAM: OnceLock<Mutex<HashMap<PathBuf, usize>>> = OnceLock::new();

/// The counts above.
fn waiting_at_the_seam() -> &'static Mutex<HashMap<PathBuf, usize>> {
    return WAITING_AT_THE_SEAM.get_or_init(|| { return Mutex::new(HashMap::new()); });
}

/// What:
///  Counts one copy of `root` entering (or, with `leaving`, leaving) that root's seam.
/// Why:
///  A control that proves its copies are held needs the count to come from the copies themselves, never from a clock.
fn count_at_the_seam(root: &Path, leaving: bool) {
    let mut waiting = waiting_at_the_seam().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let count: usize = waiting.get(root).copied().unwrap_or(0);
    let next: usize = if leaving { count.saturating_sub(1) } else { count + 1 };
    let _previous = waiting.insert(root.to_path_buf(), next);
}

/// What:
///  How many of `root`'s copies wait at its seam now.
/// Why:
///  A control proves its copies are held before it releases the seam, so its answer does not depend on the clock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = copiesWaitingAtTheSeam(root)
/// ```
pub fn copies_waiting_at_the_seam(root: &Path) -> usize {
    let waiting = waiting_at_the_seam().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    return waiting.get(root).copied().unwrap_or(0);
}

/// What:
///  Waits, bounded, for `count` of `root`'s copies to sit at its seam, and answers whether they all did.
/// Why:
///  The seam holds a copy that has reached it, so the count is the proof a control needs before it releases them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(waitUntilCopiesWaitAtTheSeam(root, 65, 10_000)).toBe(true)
/// ```
pub fn wait_until_copies_wait_at_the_seam(root: &Path, count: usize, bound: std::time::Duration) -> bool {
    let deadline: std::time::Instant = std::time::Instant::now() + bound;
    while std::time::Instant::now() < deadline {
        if copies_waiting_at_the_seam(root) >= count {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    return copies_waiting_at_the_seam(root) >= count;
}

/// The seams above.
fn seam() -> &'static Mutex<HashMap<PathBuf, std::sync::Arc<Mutex<std::sync::mpsc::Receiver<()>>>>> {
    return SEAM.get_or_init(|| { return Mutex::new(HashMap::new()); });
}

/// What:
///  Holds every copy of `root`'s service streams at the seam until the answer is released.
/// Why:
///   The race between a child's end and the copy of its last bytes is a schedule,
/// and a control meets it here without a clock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const seam = holdTheCopy(root)
/// ```
pub fn hold_the_copy(root: &Path) -> CopySeam {
    let (release, held): (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>) = std::sync::mpsc::channel();
    let _installed = seam().lock().unwrap_or_else(std::sync::PoisonError::into_inner).insert(root.to_path_buf(), std::sync::Arc::new(Mutex::new(held)));
    return CopySeam { root: root.to_path_buf(), release: Mutex::new(Some(release)) };
}

impl CopySeam {
    /// What:
    ///  Lets the held copies go, however many wait.
    /// Why:
    ///  The seam is one place; releasing it lets every copy that waits there read what its service wrote.
    ///  A copy waits in `recv`, so closing the queue is what wakes all of them:
    /// a count of permits would be a limit this interface has no reason to have.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// seam.release() // closes the queue: every waiting copy answers at once
    /// ```
    pub fn release(&self) {
        let _cleared = seam().lock().unwrap_or_else(std::sync::PoisonError::into_inner).remove(self.root.as_path());
        let release: Option<std::sync::mpsc::Sender<()>> = self.release.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take();
        drop(release);
    }
}

/// Waits at the seam when a control holds this root's copies,
/// and returns at once when none does.
fn wait_at_the_seam(root: &Path) {
    let held: Option<std::sync::Arc<Mutex<std::sync::mpsc::Receiver<()>>>> = seam().lock().unwrap_or_else(std::sync::PoisonError::into_inner).get(root).cloned();
    let Some(gate) = held else { return };
    count_at_the_seam(root, false);
    let receiver = gate.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let _released = receiver.recv();
    count_at_the_seam(root, true);
}

/// What:
///  Waits, bounded, for the copies of `root`'s service streams that a caller holds to end,
///  and answers whether they all did;
///  a copy that misses the bound is said in what `service_log` answers.
/// Why:
///   A copy runs on a thread of its own,
/// so the file and the memory hold what a service wrote only once that thread has read it.
/// The pipe closes when the last process holding it ends,
/// so a copy whose service ended always ends,
/// and this wait is that event rather than a sleep of a guessed length.
///  A descendant that inherited the pipe and holds it open must not make the wait last past the bound:
/// the bound ends it,
/// and the answer says so rather than showing an empty log.
///  The copies are the caller's,
/// because the caller is what started them and what decides from the log;
///  a wait that gives up leaves the copies it did not see end in the list,
///  so the next read waits for them again instead of answering at once for a copy that still runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settled = waitForTheCopies(copies, root, 10_000); // every copyEnded arrived, or the answer says it did not
/// ```
pub fn wait_for_the_copies(held: &mut Vec<std::sync::mpsc::Receiver<()>>, root: &Path, bound: std::time::Duration) -> bool {
    let began: std::time::Instant = std::time::Instant::now();
    let mut all_ended: bool = true;
    let mut at: usize = 0;
    while at < held.len() {
        let left: std::time::Duration = bound.saturating_sub(began.elapsed());
        match held[at].recv_timeout(left) {
            // It ended: nothing waits for it again.
            Ok(()) => {
                let _gone = held.remove(at);
            }
            // Its copy is gone without a word, which is no end it can ever say: it is waited for no longer.
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                let _gone = held.remove(at);
            }
            // The bound ended first: this copy is still to be waited for by the next read.
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                all_ended = false;
                at += 1;
            }
        }
    }
    if all_ended {
        // Every copy ended, so the words are complete: a sentence an earlier give-up left is no longer true of them.
        forget_the_copy_did_not_end(root);
    } else {
        say_the_copy_did_not_end(root, bound);
    }
    return all_ended;
}

/// What:
///  Records that a copy of `root`'s streams had not ended within `bound`,
///  so the next answer of `service_log` says the log it shows may be incomplete.
/// Why:
///   A read that waits for the copy and does not get it must not look like a read that got an empty log:
/// the sentence names the bound and the reason,
/// and it stays until the root is made fresh.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entry.copyUnfinished = `the copy of the service's stream had not finished within ${bound}`
/// ```
pub fn say_the_copy_did_not_end(root: &Path, bound: std::time::Duration) {
    let mut all = kept().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let entry = all.entry(root.to_path_buf()).or_insert_with(|| { return KeptHere { bytes: Vec::new(), file_failure: None, copy_unfinished: None }; });
    entry.copy_unfinished = Some(format!("the copy of the service's stream had not finished within {bound:?}"));
}

/// What:
///  Takes back the sentence a give-up left, because every copy has ended since.
/// Why:
///  The sentence says a read ran out its bound and that the log it shows may be incomplete;
/// once a later wait sees every copy end, the words are complete and the sentence would mislead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entry.copyUnfinished = undefined
/// ```
fn forget_the_copy_did_not_end(root: &Path) {
    let mut all = kept().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(entry) = all.get_mut(root) {
        entry.copy_unfinished = None;
    }
}

/// What:
///  The marker every line of a quoted service log begins with.
/// Why:
///   A failure quotes what a service said,
/// and that text then travels on the stream the test runner writes its own report to.
/// A line a service writes on its standard error begins with its own stamp
/// (`monochromatic-git-policy-cli-service: `),
/// so a line that begins with this marker is a quotation and never a service's own line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const QUOTED = '[fixture service log] ';
/// ```
pub const QUOTED_MARK: &str = "[fixture service log] ";

/// What:
///  `said` with every line marked as a quotation.
/// Why:
///   The journal of this machine shows the runner's own report under the runner's identifier;
/// this marker lets a reader of the journal tell a quoted service line from a line a service wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const quoted = said.split(NEWLINE).map((line) => QUOTED + line).join(NEWLINE)
/// ```
pub fn quoted(said: &str) -> String {
    let mut out: String = String::with_capacity(said.len() + said.lines().count() * QUOTED_MARK.len());
    for line in said.lines() {
        out.push_str(QUOTED_MARK);
        out.push_str(line);
        out.push('\n');
    }
    return out;
}

/// What:
///  Forget what the services of `root` said, when that root is made fresh.
/// Why:
///   The words of every service of a fixture root are kept until that root's fixture ends,
/// whichever service started when:
/// a root may serve several worktrees,
/// and a start must never take away what an earlier service of the same root said.
///  So the memory is cleared here,
/// where a fixture root is created anew,
/// and never when another pipe is made for a root that already has one
/// (a review of 2026-11-14, finding 2: the second pipe of a root whose log file could not be made discarded the first
/// service's words).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forgetTheLogOf(root) // called by the fixture that makes the root
/// ```
pub fn forget_the_log_of(root: &Path) {
    let mut all = kept().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let _gone = all.remove(root);
}

/// What:
///  The writing end of a pipe whose other end copies what a service writes on standard error into
///  `SERVICE_LOG_FILE` under `root`,
///  and into this process's memory whatever the file does.
/// Why:
///   An inherited stream loses the line that explains a failure with the fixture a panic removes,
/// a fixture's service may not write to anything the fixture does not hold,
/// and the file is what `service_log` hands to a failure.
///  A file that cannot be opened or written is not the end of the words:
/// the copy in memory carries them to `service_log`,
/// the reason is recorded,
/// and one line on this process's standard error says so,
/// unless that stream is the journal's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [reader, writer] = os.pipe(); spawn(() => copy(reader, logFile ?? memory)); return writer;
/// ```
pub fn kept_stderr_pipe(root: &Path) -> std::io::Result<(OwnedFd, std::sync::mpsc::Receiver<()>)> {
    let (mut reader, writer): (std::io::PipeReader, std::io::PipeWriter) = std::io::pipe()?;
    let path: PathBuf = root.join(SERVICE_LOG_FILE);
    let (ended, when_it_ends): (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>) = std::sync::mpsc::channel();
    {
        // The words of the services this root already served stay: a root may serve several worktrees, and a pipe is no
        // reason to take away what an earlier service of the same root said. A fresh root forgets them where it is made.
        let mut all = kept().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let _entry = all.entry(root.to_path_buf()).or_insert_with(|| { return KeptHere { bytes: Vec::new(), file_failure: None, copy_unfinished: None }; });
    }
    let key: PathBuf = root.to_path_buf();
    std::thread::spawn(move || {
        // A control that holds the copy meets the race here, at the place the copy itself runs.
        wait_at_the_seam(key.as_path());
        let mut file: Option<std::fs::File> = match std::fs::OpenOptions::new().create(true).append(true).open(path.as_path()) {
            Ok(opened) => Some(opened),
            Err(error) => {
                record_failure(key.as_path(), path.as_path(), error.to_string().as_str());
                None
            }
        };
        let mut chunk: [u8; 4096] = [0; 4096];
        loop {
            match std::io::Read::read(&mut reader, &mut chunk) {
                Ok(0) => break,
                Ok(read) => {
                    let said: &[u8] = &chunk[..read];
                    {
                        let mut all = kept().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                        let entry = all.entry(key.clone()).or_insert_with(|| { return KeptHere { bytes: Vec::new(), file_failure: None, copy_unfinished: None }; });
                        entry.bytes.extend_from_slice(said);
                        keep_the_end(&mut entry.bytes);
                    }
                    if let Some(held) = file.as_mut()
                        && let Err(error) = held.write_all(said)
                    {
                        record_failure(key.as_path(), path.as_path(), error.to_string().as_str());
                        file = None;
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(error) => {
                    record_failure(key.as_path(), path.as_path(), format!("the pipe could not be read: {error}").as_str());
                    break;
                }
            }
        }
        // The end of the copy is the event a deciding read waits for; a receiver already dropped is no one to tell.
        let _said = ended.send(());
    });
    return Ok((OwnedFd::from(writer), when_it_ends));
}

/// What:
///  The writing end alone, for a start whose words nothing reads:
///  its copy's end is dropped with this answer,
///  so no read waits for it.
/// Why:
///   A start that nothing decides from does not need the copy's end,
///  and a caller that does need it takes `kept_stderr_pipe` and keeps the channel.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// command.stderr = keptStderrPipeOnly(root); // the copy's end is dropped
/// ```
pub fn kept_stderr_pipe_only(root: &Path) -> std::io::Result<OwnedFd> {
    let (writer, when_it_ends) = kept_stderr_pipe(root)?;
    drop(when_it_ends);
    return Ok(writer);
}

/// What:
///  The end of the log of a fixture's services,
/// as a failure carries it,
/// and why the file holds none of it when it could not be written.
/// Why:
///   The line that settles a failure is one line;
/// a whole log would bury the failure it belongs to.
///  The words come from memory first:
/// they are there even when the file could not be made,
/// and a file that was never written is no reason for a failure to say nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = serviceLog(root).slice(-8 * 1024);
/// ```
pub fn service_log(root: &Path) -> String {
    let (in_memory, failure, unfinished): (Vec<u8>, Option<String>, Option<String>) = {
        let mut all = kept().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let entry = all.entry(root.to_path_buf()).or_insert_with(|| { return KeptHere { bytes: Vec::new(), file_failure: None, copy_unfinished: None }; });
        (entry.bytes.clone(), entry.file_failure.clone(), entry.copy_unfinished.clone())
    };
    let kept_bytes: Vec<u8> = if in_memory.is_empty() { bytes_of_the_file(root) } else { in_memory };
    let said: String = String::from_utf8_lossy(kept_bytes.as_slice()).into_owned();
    let opening: String = match (failure, unfinished) {
        (Some(reason), Some(copy)) => format!("the fixture's service log file holds nothing ({reason}), and {copy}; what its services said is kept here ({} bytes kept)", kept_bytes.len()),
        (Some(reason), None) => format!("the fixture's service log file holds nothing ({reason}); what its services said is kept here ({} bytes kept)", kept_bytes.len()),
        (None, Some(copy)) => format!("{copy}; what its services said so far is here ({} bytes kept)", kept_bytes.len()),
        (None, None) => format!("the service logged ({} bytes kept)", kept_bytes.len()),
    };
    return format!("{opening}:\n{}", quoted(said.as_str()));
}

/// The end of `SERVICE_LOG_FILE` under `root`,
/// or nothing when it is not there or cannot be read.
fn bytes_of_the_file(root: &Path) -> Vec<u8> {
    let path: PathBuf = root.join(SERVICE_LOG_FILE);
    let Ok(mut file) = std::fs::File::open(path.as_path()) else {
        return Vec::new();
    };
    let length: u64 = file.metadata().map(|metadata| { return metadata.len(); }).unwrap_or_default();
    let kept: u64 = u64::try_from(SERVICE_LOG_KEPT_BYTES).expect("a size fits");
    let _skipped = file.seek(std::io::SeekFrom::Start(length.saturating_sub(kept)));
    let mut bytes: Vec<u8> = Vec::new();
    let _read = file.read_to_end(&mut bytes);
    return bytes;
}
