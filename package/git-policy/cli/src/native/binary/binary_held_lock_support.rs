//! What:
//!  The harness of the controls that start a command,
//! wait until its standard error shows a line,
//! and act on the repository
//!  at that moment (remove a held `index.lock`,
//! move a branch),
//! shared by the merge and pull controls.
//! Why:
//!  A control about a held lock needs the command to be provably waiting before the lock goes,
//! and a broken wrapper must fail
//!  the control instead of holding it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ended = await runActingOnLine(command, 'waits for the index lock', () => rm(lock));
//! ```

use super::support::{TIMED_OUT_SIGNAL, without_hint_fields};
use std::io::{BufRead, Read};
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Stdio};

/// How long a control waits for something that must happen,
/// so a broken wrapper fails it and never holds it.
pub(super) const PATIENCE: std::time::Duration = std::time::Duration::from_secs(60);

/// What a merge printed and how it ended.
pub(super) struct Ended {
    /// Exit status.
    pub(super) code: Option<i32>,
    /// Standard error,
    /// without hint fields.
    pub(super) stderr: String,
}

/// Run `command` to its end,
/// and run `act` once,
/// when a line of standard error contains `needle`.
pub(super) fn run_acting_on_line(command: &mut Command, needle: &str, act: &mut dyn FnMut()) -> Ended {
    let mut child = command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("process starts");
    let mut standard_output = child.stdout.take().expect("piped stdout");
    // Read concurrently, so a full pipe never blocks the child while this thread waits on its standard error.
    let output_reader = std::thread::spawn(move || {
        let mut bytes: Vec<u8> = Vec::new();
        let _read = standard_output.read_to_end(&mut bytes);
        return bytes;
    });
    let mut errors = std::io::BufReader::new(child.stderr.take().expect("piped stderr"));
    let mut stderr: Vec<u8> = Vec::new();
    let mut acted: bool = false;
    loop {
        let line_start: usize = stderr.len();
        if errors.read_until(b'\n', &mut stderr).expect("read standard error") == 0 {
            break;
        }
        if !acted && String::from_utf8_lossy(&stderr[line_start..]).contains(needle) {
            acted = true;
            act();
        }
    }
    let status = child.wait().expect("process finishes");
    assert_ne!(status.signal(), Some(TIMED_OUT_SIGNAL), "the command hit its bound before it ended");
    let _output = output_reader.join().expect("standard output reader");
    return Ended { code: status.code(), stderr: String::from_utf8_lossy(without_hint_fields(stderr.as_slice()).as_slice()).into_owned() };
}
