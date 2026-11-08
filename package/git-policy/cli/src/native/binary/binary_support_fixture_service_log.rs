//! What:
//!  What a fixture's commit services write on their standard error:
//! the pipe that keeps it while they run,
//! and the end of it that a failure carries.
//! Why:
//!  A service that refuses a step or ends says so only on its own error stream,
//! an inherited stream mixes that line with the output of every other control running at the same moment,
//! and a panic removes the fixture the line belongs to;
//! so the words go to a file beside the fixture and to the stream a person reads.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! command.stderr = keptStderrPipe(root); // copies into `${root}/service.log` and into process.stderr
//! const said = serviceLog(root).slice(-8 * 1024);
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

/// What:
///  The writing end of a pipe whose other end copies what a service writes on standard error into
///  `SERVICE_LOG_FILE` under `root` and into this test's own error stream.
/// Why:
///   An inherited stream loses the line that explains a failure with the fixture a panic removes,
/// and a file alone would take the service's words off the output a person reads while it runs;
/// the copy keeps both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [reader, writer] = os.pipe(); spawn(() => copy(reader, [logFile, process.stderr])); return writer;
/// ```
pub fn kept_stderr_pipe(root: &Path) -> std::io::Result<OwnedFd> {
    let (mut reader, writer): (std::io::PipeReader, std::io::PipeWriter) = std::io::pipe()?;
    let path: std::path::PathBuf = root.join(SERVICE_LOG_FILE);
    std::thread::spawn(move || {
        let mut file: Option<std::fs::File> = std::fs::OpenOptions::new().create(true).append(true).open(path.as_path()).ok();
        let mut chunk: [u8; 4096] = [0; 4096];
        loop {
            match std::io::Read::read(&mut reader, &mut chunk) {
                Ok(0) => return,
                Ok(read) => {
                    let said: &[u8] = &chunk[..read];
                    if let Some(held) = file.as_mut() {
                        let _written = std::io::Write::write_all(held, said);
                    }
                    let _shown = std::io::Write::write_all(&mut std::io::stderr(), said);
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(_) => return,
            }
        }
    });
    return Ok(OwnedFd::from(writer));
}

/// What:
///  The end of the log of a fixture's services,
/// as a failure carries it.
/// Why:
///   The line that settles a failure is one line;
/// a whole log would bury the failure it belongs to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = serviceLog(root).slice(-8 * 1024);
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
