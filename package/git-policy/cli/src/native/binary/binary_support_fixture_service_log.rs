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
//!  They are never copied to this process's own standard error,
//! which is a stream the fixture does not own:
//! a process that runs the tests may have the journal on it,
//! and then a throwaway fixture's service words would be in the user journal of the machine
//! (`doc/troubleshooting/git-policy-cli-test-services-write-into-the-user-journal.md`).
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
        let entry: &mut KeptHere = all.entry(root.to_path_buf()).or_insert_with(|| { return KeptHere { bytes: Vec::new(), file_failure: None }; });
        let first: bool = entry.file_failure.is_none();
        entry.file_failure = Some(String::from(reason));
        first
    };
    if first {
        say_unless_the_journal(log_file_failure_note(path, reason).as_str());
    }
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
pub fn kept_stderr_pipe(root: &Path) -> std::io::Result<OwnedFd> {
    let (mut reader, writer): (std::io::PipeReader, std::io::PipeWriter) = std::io::pipe()?;
    let path: PathBuf = root.join(SERVICE_LOG_FILE);
    {
        // A fixture root that is fresh holds no log file, so nothing of an earlier fixture under the same name is kept;
        // a root that already holds one keeps the words of the services it served before, as the file does.
        let mut all = kept().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if !path.exists() {
            let _gone = all.remove(root);
        }
        let _entry = all.entry(root.to_path_buf()).or_insert_with(|| { return KeptHere { bytes: Vec::new(), file_failure: None }; });
    }
    let key: PathBuf = root.to_path_buf();
    std::thread::spawn(move || {
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
                Ok(0) => return,
                Ok(read) => {
                    let said: &[u8] = &chunk[..read];
                    {
                        let mut all = kept().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                        let entry = all.entry(key.clone()).or_insert_with(|| { return KeptHere { bytes: Vec::new(), file_failure: None }; });
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
                    return;
                }
            }
        }
    });
    return Ok(OwnedFd::from(writer));
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
    let (in_memory, failure): (Vec<u8>, Option<String>) = {
        let mut all = kept().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let entry = all.entry(root.to_path_buf()).or_insert_with(|| { return KeptHere { bytes: Vec::new(), file_failure: None }; });
        (entry.bytes.clone(), entry.file_failure.clone())
    };
    let kept_bytes: Vec<u8> = if in_memory.is_empty() { bytes_of_the_file(root) } else { in_memory };
    let said: String = String::from_utf8_lossy(kept_bytes.as_slice()).into_owned();
    return match failure {
        Some(reason) => format!("the fixture's service log file holds nothing ({reason}); what its services said is kept here ({} bytes kept):\n{said}", kept_bytes.len()),
        None => format!("the service logged ({} bytes kept):\n{said}", kept_bytes.len()),
    };
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
