//! The guard as the watcher starts it:
//! a child of the watched user,
//! because a user bus refuses a connection from root,
//! whose every line the watcher writes to its log,
//! the journal and the person's desktop.

/// The guard's decisions and their reports.
use super::{Guard, GuardMemory, GuardSteps, MANAGER_BUS_NAME, POLL_INTERVAL, REEXEC_GAP};
/// The word a loud line carries.
use super::LOUD_MARKER;
/// Names a user chose,
/// escaped so the line cannot repaint a terminal.
use crate::report_cause::quote_name;
/// The search path the child's command gets.
use crate::bounded_command::SECURE_PATH;
/// Reading its lines and writing them on.
use std::io::{BufRead, BufReader, Write};
/// Starting the child and taking its streams.
use std::process::{Command, Stdio};
/// How often and how long.
use std::time::{Duration, Instant};

/// The command the watcher starts the guard with.
pub const GUARD_COMMAND: &str = "__impostor-guard";

/// What the guard child needs.
pub struct GuardSetup {
    /// The user whose bus is guarded.
    pub uid: u32,
    /// How often the owner is checked when no event arrives.
    pub poll_interval: Duration,
    /// The least time between two re-executions.
    pub reexec_gap: Duration,
}

/// Runs the guard until the process is killed,
/// printing one line per report on standard output,
/// which the watcher writes to the log,
/// the journal and the desktop.
pub fn run_child(setup: &GuardSetup, steps: &mut dyn GuardSteps) -> std::io::Result<()> {
    let mut guard = Guard { steps, memory: GuardMemory::new(setup.reexec_gap) };
    println!("wg-quicker-exempt: guarding {MANAGER_BUS_NAME} on the session bus of user {}", setup.uid);
    loop {
        for report in guard.check(setup.uid, Instant::now()) {
            let line = if report.loud {
                format!("wg-quicker-exempt: {LOUD_MARKER}: {}", report.text)
            } else {
                format!("wg-quicker-exempt: {}", report.text)
            };
            let mut output = std::io::stdout();
            writeln!(output, "{}", quote_name(&line))?;
            output.flush()?;
        }
        // The bus's event wakes the guard at once; the poll is the backstop for a bus that says nothing.
        let _changed = guard.steps.wait_for_change(setup.uid, setup.poll_interval);
    }
}

/// The lines the child prints,
/// which the reporting thread reads.
fn relay(uid: u32, gid: u32, stdout: std::process::ChildStdout, mut child: std::process::Child) {
    let mut reporter = crate::runtime_report::RuntimeReporter::new(uid, gid);
    for line in BufReader::new(stdout).lines().map_while(Result::ok) {
        if line.contains(LOUD_MARKER) {
            let _reported = reporter.report_failure("impostor manager", &line);
        } else {
            crate::log_line::log_line(&line);
        }
    }
    reporter.finish();
    let _status = child.wait();
    crate::log_line::log_line(&format!("wg-quicker-exempt: {LOUD_MARKER}: the guard of {MANAGER_BUS_NAME} ended, so an impostor that takes that name is not removed"));
}

/// Starts the guard as the watched user
/// and sends every line it prints to the watcher's log,
/// the journal and the person's desktop.
pub fn spawn(uid: u32, gid: u32) -> Result<(), String> {
    let executable = match std::env::current_exe() {
        Ok(executable) => executable,
        Err(error) => return Err(format!("the path of this program is unknown: {error}")),
    };
    let mut command = Command::new("setpriv");
    command
        .args([
            "--reuid".to_string(),
            uid.to_string(),
            "--regid".to_string(),
            gid.to_string(),
            "--clear-groups".to_string(),
            "--no-new-privs".to_string(),
            executable.to_string_lossy().into_owned(),
            GUARD_COMMAND.to_string(),
            uid.to_string(),
            POLL_INTERVAL.as_millis().to_string(),
            REEXEC_GAP.as_millis().to_string(),
        ])
        .env_clear()
        .env("PATH", SECURE_PATH)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => return Err(format!("setpriv could not start the guard: {error}")),
    };
    let Some(stdout) = child.stdout.take() else {
        return Err("the guard has no output stream".to_string());
    };
    let started = std::thread::Builder::new().name("impostor-guard".to_string()).spawn(move || return relay(uid, gid, stdout, child));
    return match started {
        Ok(_handle) => Ok(()),
        Err(error) => Err(format!("the guard's reporting thread could not start: {error}")),
    };
}
