//! What:
//!  Output of a child process captured into two anonymous files instead of two pipes, and read back from the start whenever the
//!  caller asks.
//! Why:
//!  A pipe lives only as long as the process that holds its read end. A step the wrapper leaves running (a write in place that
//!  cannot be run twice and has outlasted its bound) would meet a closed pipe when it next wrote, if the command that started it
//!  had exited: the write fails with `EPIPE` or the process dies of `SIGPIPE`, which is the cut the owner ruled out ("Never cut a
//!  branch write (Recommended)"). A file the child holds open cannot be closed by the command, so the child outlives the command
//!  and no thread has to read for it. The same holds for a repair tool that starts a helper which keeps its inherited output open
//!  after the tool has exited: with files, no reader waits for the helper. The files are anonymous (no name to remove, nothing left
//!  behind), the parent reads them by position so the offset the child moves is not disturbed, and a child that is still running
//!  after the command has gone is reaped by the system's init or subreaper.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const capture = new Capture(); spawn(command, { stdio: ['null', capture.out, capture.err] }); await exit(child); const { stdout, stderr } = capture.read();
//! ```

use std::fs::File;
use std::os::unix::fs::FileExt;
use std::process::Stdio;

/// The most bytes of one stream a snapshot of a step's output holds, the last ones.
///  Measured on this host's real repositories (`/var/home/user/temp/agent/ceiling-measure/measure-output.ts`, `measure-output.out`,
///  2026-11-04): the largest output of a step the wrapper captures in full was the diffstat `git merge` prints for the merge with the most
///  changed files in the Monochromatic history (4096 files), 638,784 bytes; the others were far smaller (`cargo metadata --no-deps` of
///  35 manifests, at most 21,012 bytes; `pnpm install --offline --frozen-lockfile`, 634 bytes; `mpm install --frozen-lockfile`, 639
///  bytes; a merge of the other repository on the host, 2,381 bytes). 8 MiB is 13.1 times the largest, about the output of a merge of
///  50,000 files. A step that writes more has the start of its output left out, said in words (`Captured::notes`), and a reader that
///  parses an output (the `cargo metadata` of a repair) refuses one that was cut. The coordinator's request for a cited value
///  (2026-11-04); the margin is open to the owner's veto.
pub const STEP_OUTPUT_LIMIT: usize = 8 * 1024 * 1024;

/// Two anonymous files that collect a child's standard output and standard error.
pub struct Capture {
    /// Standard output.
    out: File,
    /// Standard error.
    err: File,
}

/// One anonymous file: a `memfd`, where the system has them.
fn anonymous_file() -> std::io::Result<File> {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::io::FromRawFd;
        // SAFETY: the name is a NUL-terminated literal; the call returns a new descriptor or -1.
        let descriptor: libc::c_int = unsafe { libc::memfd_create(c"cli-git-output".as_ptr(), libc::MFD_CLOEXEC) };
        if descriptor < 0 {
            return Err(std::io::Error::last_os_error());
        }
        // SAFETY: the descriptor is new, open, and owned by nothing else.
        return Ok(unsafe { File::from_raw_fd(descriptor) });
    }
    // UNTESTED ON OTHER SYSTEMS: only Linux has `memfd_create`, and the owner said to ignore macOS until told. An unnamed file made by
    // C's `tmpfile` would be made in `/tmp`, which this repository never uses (the device behind it is unknowable), so there is none.
    #[cfg(not(target_os = "linux"))]
    {
        return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "anonymous output files need memfd_create, which this system lacks"));
    }
}

impl Capture {
    /// Two new empty anonymous files.
    pub fn new() -> std::io::Result<Self> {
        return Ok(Self { out: anonymous_file()?, err: anonymous_file()? });
    }

    /// The two files as the standard output and standard error of a child.
    pub fn stdio(&self) -> std::io::Result<(Stdio, Stdio)> {
        return Ok((Stdio::from(self.out.try_clone()?), Stdio::from(self.err.try_clone()?)));
    }

    /// What:
    ///  A snapshot of what the child wrote so far to each stream: the last `limit` bytes of what was there when the snapshot began,
    ///  and a note for everything that was left out.
    /// Why:
    ///   A descendant that keeps appending, or a child that writes more than anyone wants, must not make the reader chase the end
    ///  of a file that grows ahead of it, nor hold the whole of it in memory (sol-1 and sol-2 round 2): the length is read once and
    ///  that many bytes are read at most, from the end of the snapshot backwards by `limit`. A read that fails is not the end of
    ///  the output: the stream's note says it could not be read whole. The tail is kept because a tool prints its reason last.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const { stdout, stderr, notes } = capture.snapshot(limit);
    /// ```
    pub fn snapshot(&self, limit: usize) -> Captured {
        let mut notes: Vec<String> = Vec::new();
        let stdout: Vec<u8> = snapshot_of(&self.out, "standard output", limit, &mut notes, &mut || {});
        let stderr: Vec<u8> = snapshot_of(&self.err, "standard error", limit, &mut notes, &mut || {});
        return Captured { stdout, stderr, notes };
    }

    /// `snapshot` of standard output, calling `after_chunk` after every chunk it read: a control appends to the file there, which a
    ///  reader that chases the end of the file would follow for ever.
    #[cfg(test)]
    pub fn snapshot_stdout_with(&self, limit: usize, after_chunk: &mut dyn FnMut()) -> (Vec<u8>, Vec<String>) {
        let mut notes: Vec<String> = Vec::new();
        let stdout: Vec<u8> = snapshot_of(&self.out, "standard output", limit, &mut notes, after_chunk);
        return (stdout, notes);
    }

    /// Free what the files hold in memory. The child's offset is its own, so what it writes afterwards leaves a hole before it and
    ///  costs only the pages it writes; nothing reads the files after this.
    pub fn discard(&self) {
        let _out: std::io::Result<()> = self.out.set_len(0);
        let _err: std::io::Result<()> = self.err.set_len(0);
    }
}

/// What a snapshot of a child's output holds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Captured {
    /// The tail of what was written to standard output.
    pub stdout: Vec<u8>,
    /// The tail of what was written to standard error.
    pub stderr: Vec<u8>,
    /// What was left out or could not be read, in words for the person.
    pub notes: Vec<String>,
}

/// The last `limit` bytes of the first `length` bytes of `file`, where `length` is the size when the call began; a note for the bytes
///  left out and for a read that failed.
fn snapshot_of(file: &File, name: &str, limit: usize, notes: &mut Vec<String>, after_chunk: &mut dyn FnMut()) -> Vec<u8> {
    let length: u64 = match file.metadata() {
        Ok(found) => found.len(),
        Err(error) => {
            notes.push(format!("the {name} of the step could not be measured ({error}), so none of it is shown"));
            return Vec::new();
        }
    };
    let wanted: u64 = length.min(limit as u64);
    let start: u64 = length - wanted;
    if start > 0 {
        notes.push(format!("{start} bytes at the start of the {name} of the step are left out; the last {wanted} bytes are shown"));
    }
    let mut collected: Vec<u8> = Vec::with_capacity(usize::try_from(wanted).unwrap_or(0));
    let mut buffer: [u8; 8192] = [0; 8192];
    let mut position: u64 = start;
    while position < length {
        let room: usize = usize::try_from((length - position).min(buffer.len() as u64)).unwrap_or(buffer.len());
        match file.read_at(&mut buffer[..room], position) {
            Ok(0) => break,
            Ok(read) => {
                collected.extend_from_slice(&buffer[..read]);
                position += read as u64;
                after_chunk();
            }
            Err(error) => {
                notes.push(format!("the {name} of the step could not be read whole ({error}); {} of {wanted} bytes are shown", collected.len()));
                return collected;
            }
        }
    }
    if (collected.len() as u64) < wanted {
        notes.push(format!("the {name} of the step ended early: {} of {wanted} bytes are shown", collected.len()));
    }
    return collected;
}

/// Capture controls stay out of the release executable.
#[cfg(test)]
#[path = "output_capture_tests.rs"]
mod tests;
