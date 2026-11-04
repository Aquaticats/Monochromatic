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

    /// Everything the child wrote so far to its standard output and its standard error, read from the start.
    pub fn read(&self) -> (Vec<u8>, Vec<u8>) {
        return (read_all(&self.out), read_all(&self.err));
    }
}

/// Everything in `file` from position zero; what could not be read is left out, which is what a partly written file holds.
fn read_all(file: &File) -> Vec<u8> {
    let mut collected: Vec<u8> = Vec::new();
    let mut buffer: [u8; 8192] = [0; 8192];
    let mut position: u64 = 0;
    loop {
        match file.read_at(&mut buffer, position) {
            Ok(0) | Err(_) => return collected,
            Ok(read) => {
                collected.extend_from_slice(&buffer[..read]);
                position += read as u64;
            }
        }
    }
}

/// Capture controls stay out of the release executable.
#[cfg(test)]
#[path = "output_capture_tests.rs"]
mod tests;
