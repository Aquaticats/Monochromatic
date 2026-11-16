//! What:
//!  What a fixture's commit services write on their standard error:
//! the pipe that keeps it while they run,
//! the end of it that a failure carries,
//! and the event that says the copy of it has ended.
//! Why:
//!  A service that refuses a step or ends says so only on its own error stream,
//! an inherited stream mixes that line with the output of every other control running at the same moment,
//! and a panic removes the fixture the line belongs to;
//! so the words go to a file beside the fixture and to the stream a person reads.
//!  The copy runs on a thread of its own, so the file holds what a service wrote only once that thread has written it:
//! the pipe closes when the last service holding it ends,
//! the thread then ends,
//! and `kept_stderr_pipe` answers a channel that says so.
//! A read that decides anything waits for that event first
//! (`settle_the_service_log`), once every service holding the stream has ended (`end_the_pressure`);
//! a line in the log is never what a control's verdict rests on.
//! The read that waits and the stop it waits after are separate calls on purpose:
//! stopping a service is not waiting for a thread, and a control that reads while its service still runs says so.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { writer, copyEnded } = keptStderrPipe(root); // copies into `${root}/service.log` and into process.stderr
//! await copyEnded; const said = serviceLog(root).slice(-8 * 1024);
//! ```

use std::os::fd::OwnedFd;
use std::path::Path;

/// The file under the fixture root that keeps what its services wrote on standard error.
/// A failure names it,
/// so the service's own words outlive the fixture that a panic removes.
pub const SERVICE_LOG_FILE: &str = "service.log";

/// How many bytes of a service log a failure carries:
/// a service writes a line per landing,
/// so the end of the log says how it ended,
/// and the rest would bury the failure it belongs to.
const SERVICE_LOG_KEPT_BYTES: u64 = 8 * 1024;

/// How many bytes of a service's stream the copy of it writes at a time:
/// one read of the pipe, and one write into the log,
/// so a service's words reach the log in steps and a copy can stand between two of them.
/// A control that needs bytes the copy has not written yet writes more than one of these
/// (`fault_limits::a_case_reads_a_service_log_only_after_the_copy_of_its_stream_ended_or_says_it_could_not_wait`).
pub const SERVICE_LOG_COPY_CHUNK_BYTES: usize = 4096;

/// What:
///  The writing end of a pipe whose other end copies what a service writes on standard error into
///  `SERVICE_LOG_FILE` under `root` and into this test's own error stream,
///  and the end of a channel that says the copy has ended.
/// Why:
///   An inherited stream loses the line that explains a failure with the fixture a panic removes,
/// and a file alone would take the service's words off the output a person reads while it runs;
/// the copy keeps both.
///  A read that decides anything cannot wait on the clock for the copy: the pipe closes when the service's own end
/// closes it, and the copy ends after that, so the channel's message is the event the read waits for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [reader, writer] = os.pipe(); const copyEnded = spawn(() => copy(reader, [logFile, process.stderr])); return { writer, copyEnded };
/// ```
pub fn kept_stderr_pipe(root: &Path) -> std::io::Result<(OwnedFd, std::sync::mpsc::Receiver<()>)> {
    let (mut reader, writer): (std::io::PipeReader, std::io::PipeWriter) = std::io::pipe()?;
    let path: std::path::PathBuf = root.join(SERVICE_LOG_FILE);
    let (ended, when_it_ends): (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut file: Option<std::fs::File> = std::fs::OpenOptions::new().create(true).append(true).open(path.as_path()).ok();
        let mut chunk: [u8; SERVICE_LOG_COPY_CHUNK_BYTES] = [0; SERVICE_LOG_COPY_CHUNK_BYTES];
        loop {
            match std::io::Read::read(&mut reader, &mut chunk) {
                Ok(0) => break,
                Ok(read) => {
                    let said: &[u8] = &chunk[..read];
                    if let Some(held) = file.as_mut() {
                        let _written = std::io::Write::write_all(held, said);
                    }
                    let _shown = std::io::Write::write_all(&mut std::io::stderr(), said);
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(_) => break,
            }
        }
        // The end of the copy is the event a deciding read waits for; a receiver already dropped is no one to tell.
        let _said = ended.send(());
    });
    return Ok((OwnedFd::from(writer), when_it_ends));
}

/// What:
///  The end of the log of a fixture's services,
/// as a failure carries it.
/// Why:
///   The line that settles a failure is one line;
/// a whole log would bury the failure it belongs to.
///  What this reads is evidence for a person,
/// never the event a control's verdict rests on:
/// the file holds what a service wrote only once the copy of its stream has ended,
/// so a caller that stands by these words waits for that end (`settle_the_service_log`) after the services holding the
/// stream have ended,
/// which is what a fault case does (`binary_fault_limits_tests.rs`, `the_service_log_of`);
/// a caller that cannot wait names the moment it read rather than the service.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = serviceLog(root).slice(-8 * 1024); // after copyEnded
/// ```
pub fn service_log(root: &Path) -> String {
    use std::io::{Read, Seek};
    let path: std::path::PathBuf = root.join(SERVICE_LOG_FILE);
    let Ok(mut file) = std::fs::File::open(path.as_path()) else {
        return String::from("(the fixture's service wrote nothing)");
    };
    let length: u64 = file.metadata().map(|metadata| { return metadata.len(); }).unwrap_or_default();
    let _skipped = file.seek(std::io::SeekFrom::Start(length.saturating_sub(SERVICE_LOG_KEPT_BYTES)));
    let mut kept: Vec<u8> = Vec::new();
    let _read = file.read_to_end(&mut kept);
    return format!("the service logged ({} bytes kept):\n{}", kept.len(), String::from_utf8_lossy(kept.as_slice()));
}
