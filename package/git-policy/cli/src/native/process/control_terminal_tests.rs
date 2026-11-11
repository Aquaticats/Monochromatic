//! Controls for the controlling-terminal probe.
//!
//! A process's controlling terminal is a property of the process,
//! so both answers are checked in helper processes the test binary starts:
//! one put in a session with a pseudo-terminal as its controlling terminal,
//! and one put in a session with none.
//! The same two arrangements are what a person's terminal and the daemon of the 2026-11-10 incident have.

use super::*;

use crate::test_support::{fixture, remove, this_test_binary};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus};
use std::time::{Duration, Instant};

/// The environment variable that tells the helper which answer to expect.
const EXPECTATION_VARIABLE: &str = "CLI_GIT_CONTROL_TERMINAL_EXPECTATION";
/// The environment variable that names the file the helper writes its reading into.
const READING_VARIABLE: &str = "CLI_GIT_CONTROL_TERMINAL_READING";

/// How long a control waits for a helper step before it fails.
const STEP_TIMEOUT: Duration = Duration::from_secs(60);

/// What:
///  Helper entry for both arrangements.
///  It records what the probe answered and then fails when that is not the answer the parent expects,
/// so a wrong reading is a failed helper rather than a failed assertion in the parent.
/// Why:
///   The probe reads the state of the process it runs in,
/// so it can only be checked in a process whose controlling terminal the parent arranged.
/// Outside the helper environment this test does nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const expected = process.env[EXPECTATION_VARIABLE]; if (!expected) return;
/// const answer = controlTerminal(); fs.writeFileSync(reading, answer);
/// expect(answer).toBe(expected);
/// ```
#[test]
fn helper_entry() {
    // `var_os` answers the raw OS text of a variable, and `None` is the ordinary test run.
    let Some(expected) = std::env::var_os(EXPECTATION_VARIABLE) else {
        return;
    };
    let reading: PathBuf = PathBuf::from(std::env::var_os(READING_VARIABLE).expect("the reading file is named with the expectation"));
    // `control_terminal()` is the decision the wrapper's invocation overlay is built from.
    let answered: Terminal = control_terminal();
    // `{:?}` is the debug spelling of the variant, which stands beside the words it is compared with.
    let text: String = format!("{answered:?}");
    std::fs::write(reading.as_path(), text.as_bytes()).expect("the reading is written");
    assert_eq!(text, expected.to_string_lossy(), "the probe answered about the wrong terminal");
}

/// Wait for a path a helper writes,
/// or fail the control at its own bound.
fn wait_for(path: &Path) {
    let began: Instant = Instant::now();
    while !path.exists() {
        assert!(began.elapsed() < STEP_TIMEOUT, "the helper never wrote {}: a probe that waits forever is a defect", path.display());
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// A pseudo-terminal the control keeps open:
/// the master end and the slave end a helper's standard streams are made from.
///  The name is the sibling `Terminal` of this module plus what it is:
/// a device of this control,
/// not an answer about a process.
struct PseudoTerminal {
    /// Kept open so the helper's terminal stays a terminal for its whole run.
    _master: std::fs::File,
    /// What the helper's three standard streams are made from.
    slave: std::fs::File,
}

/// Open a new pseudo-terminal.
fn pseudo_terminal() -> PseudoTerminal {
    use std::os::fd::FromRawFd;
    // SAFETY: `openpty` writes one descriptor per out-pointer and takes three pointers that may be null; the name, the termios
    //  settings and the window size each stay the system default. Both descriptors are owned here at once.
    let (master, slave): (libc::c_int, libc::c_int) = unsafe {
        let mut master: libc::c_int = -1;
        let mut slave: libc::c_int = -1;
        let opened: libc::c_int = libc::openpty(&mut master, &mut slave, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut());
        assert_eq!(opened, 0, "a pseudo-terminal opens");
        (master, slave)
    };
    // SAFETY: both numbers were just returned by `openpty`, and nothing else owns them.
    let master_file: std::fs::File = unsafe { std::fs::File::from_raw_fd(master) };
    // SAFETY: as above; `try_clone` turns the number into as many owned descriptors as the helper's streams need.
    let slave_file: std::fs::File = unsafe { std::fs::File::from_raw_fd(slave) };
    return PseudoTerminal { _master: master_file, slave: slave_file };
}

/// What:
///  Start the helper and wait for its answer.
/// Why:
///   `at` decides the whole difference between the two arrangements:
/// with a terminal the helper is made a session leader and takes the terminal as its controlling one,
/// and without one it is made a session leader of a session that has none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ask(root, expectation, at?): string  // spawn the test binary's helper_entry
/// ```
fn ask(root: &Path, expectation: &str, at: Option<&PseudoTerminal>) -> String {
    use std::os::unix::process::CommandExt;
    let reading: PathBuf = root.join(format!("reading-{expectation}"));
    let mut command: Command = Command::new(this_test_binary().expect("the test binary"));
    command
        .args(["--exact", "process::control_terminal::tests::helper_entry", "--nocapture", "--test-threads=1"])
        .env(EXPECTATION_VARIABLE, expectation)
        .env(READING_VARIABLE, reading.as_os_str());
    let with_terminal: bool = at.is_some();
    match at {
        Some(opened) => {
            command
                .stdin(std::process::Stdio::from(opened.slave.try_clone().expect("duplicate the terminal")))
                .stdout(std::process::Stdio::from(opened.slave.try_clone().expect("duplicate the terminal")))
                .stderr(std::process::Stdio::from(opened.slave.try_clone().expect("duplicate the terminal")));
        }
        None => {
            command
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());
        }
    }
    // SAFETY: between fork and exec only async-signal-safe calls run: `setsid`, and on the terminal route one `ioctl` on
    //  descriptor 0, which is the helper's own terminal.
    unsafe {
        command.pre_exec(move || {
            libc::setsid();
            if with_terminal {
                libc::ioctl(0, libc::TIOCSCTTY, 0);
            }
            return Ok(());
        });
    }
    let helper: Child = command.spawn().expect("the helper starts");
    // The `setsid` above made the helper the leader of a group of its own, which dies with this test thread.
    crate::test_support::own_group(&helper);
    wait_for(reading.as_path());
    let status: ExitStatus = helper.wait_with_output().expect("the helper finishes").status;
    assert!(status.success(), "the helper reported the wrong terminal: {status:?}");
    return std::fs::read_to_string(reading.as_path()).expect("the reading");
}

/// A session with a controlling terminal is read as "a person may answer".
#[test]
fn a_session_with_a_controlling_terminal_is_present() {
    let root: PathBuf = fixture("control-terminal-present");
    let at: PseudoTerminal = pseudo_terminal();
    let answered: String = ask(root.as_path(), "Present", Some(&at));
    assert_eq!(answered, "Present");
    remove(root.as_path());
}

/// A session with none is read as "nobody can answer",
/// which is the state of the daemon in the 2026-11-10 incident.
#[test]
fn a_session_with_no_controlling_terminal_is_absent() {
    let root: PathBuf = fixture("control-terminal-absent");
    let answered: String = ask(root.as_path(), "Absent", None);
    assert_eq!(answered, "Absent");
    remove(root.as_path());
}
