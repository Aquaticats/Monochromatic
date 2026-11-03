//! What:
//!  Starting one package-manager program for a worktree repair: finding it on a search path, running it in a directory with
//!  its output captured and a time limit, and saying how it ended.
//! Why:
//!  The repairs of a new worktree (`package_repair.rs`) start tools the repository uses (pnpm, Cargo, mpm-shadow). Each one
//!  must end in a plain verdict the add can word for the person: the program is missing, it failed with a reason, it ran
//!  past its limit, or it worked. Nothing here decides which tool runs or what its output means.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const program = findProgram('pnpm', process.env.PATH);
//! const run = await runTool(program, { arguments: ['install'], directory, set: [] }, limit);
//! ```

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// What:
///  One program start: its arguments, the directory it runs in, and the variables added to its environment.
/// Why:
///   The program path is found separately, so a test can name a fixture and a person's search path is read once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolCommand = { arguments: string[]; directory: string; set: [string, string][] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolCommand {
    /// The arguments after the program name.
    pub arguments: Vec<OsString>,
    /// The working directory.
    pub directory: PathBuf,
    /// Variables added to the inherited environment.
    pub set: Vec<(OsString, OsString)>,
}

/// What:
///  How a started program ended.
/// Why:
///   The repair words each case differently, so they stay apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolEnd = { kind: 'succeeded' } | { kind: 'failed'; reason: string } | { kind: 'timed-out' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ToolEnd {
    /// Exit status zero.
    Succeeded,
    /// The program did not start, or ended with a failure status; the text names the status and the tail of its output.
    Failed(String),
    /// The program was still running when its limit passed, and was stopped.
    TimedOut,
}

/// What:
///  The end of one run and how long it took.
/// Why:
///   The time is reported to the person and measured by the docs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolRun = { end: ToolEnd; millis: number; stdout: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolRun {
    /// How the program ended.
    pub end: ToolEnd,
    /// Wall time from start to end, in milliseconds.
    pub millis: u128,
    /// What the program wrote to its standard output; empty when it did not run to its end.
    pub stdout: Vec<u8>,
}

/// The most bytes of a program's output kept for its failure text.
const OUTPUT_TAIL_BYTES: usize = 600;

/// How often a running program is checked against its limit.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// What:
///  The first executable file named `name` in a search path.
/// Why:
///   The program is the one the person's environment resolves; a missing one is a verdict, not a failure to start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findProgram(name: string, searchPath?: string): string | undefined;
/// ```
pub fn find_program(name: &str, search_path: Option<&OsStr>) -> Option<PathBuf> {
    use std::os::unix::fs::PermissionsExt;
    let search: &OsStr = search_path?;
    for directory in std::env::split_paths(search) {
        // An empty entry means the current directory in a shell; a repair never runs a program found there.
        if directory.as_os_str().is_empty() {
            continue;
        }
        let candidate: PathBuf = directory.join(name);
        let runnable: bool = std::fs::metadata(&candidate).is_ok_and(|found: std::fs::Metadata| { return found.is_file() && found.permissions().mode() & 0o111 != 0; });
        if runnable {
            return Some(candidate);
        }
    }
    return None;
}

/// What:
///  The last bytes of what a program printed, as text, with the cut ragged edge dropped.
/// Why:
///   A failure is worded with the end of the output, where tools print their reason; the start is progress noise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tailOf(bytes: Uint8Array): string;
/// ```
fn tail_of(bytes: &[u8]) -> String {
    let cut: usize = bytes.len().saturating_sub(OUTPUT_TAIL_BYTES);
    let text: String = String::from_utf8_lossy(&bytes[cut..]).into_owned();
    // Lines that only say a tool waited for another process say nothing about why it failed.
    let informative: String = text.lines().filter(|line: &&str| { return !line.trim_start().starts_with("Blocking waiting for file lock"); }).collect::<Vec<&str>>().join("\n");
    let trimmed: &str = informative.trim();
    if cut == 0 {
        return trimmed.to_owned();
    }
    return trimmed.split_once('\n').map_or(trimmed, |(_ragged, rest): (&str, &str)| { return rest; }).trim().to_owned();
}

/// What:
///  Read a pipe to its end on a thread of its own and hand back everything it carried.
/// Why:
///   A program that fills a pipe nobody reads stops; reading both of its pipes at once keeps it running.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const collected = collect(child.stdout);
/// ```
fn collect<R: std::io::Read + Send + 'static>(mut pipe: R) -> std::thread::JoinHandle<Vec<u8>> {
    return std::thread::spawn(move || {
        let mut bytes: Vec<u8> = Vec::new();
        // A read that fails ends the collection; what was read before it is still the program's output.
        let _ended = pipe.read_to_end(&mut bytes);
        return bytes;
    });
}

/// What:
///  How a finished program's status is worded.
/// Why:
///   A signal has no code, and the person should see which of the two ended the program.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function describeStatus(status: ExitStatus): string;
/// ```
fn describe_status(status: std::process::ExitStatus) -> String {
    use std::os::unix::process::ExitStatusExt;
    return match (status.code(), status.signal()) {
        (Some(code), _) => format!("exit status {code}"),
        (None, Some(signal)) => format!("stopped by signal {signal}"),
        (None, None) => String::from("ended without a status"),
    };
}

/// What:
///  Run a program to its end or its limit.
/// Why:
///   The add must never wait without bound on a tool, and must keep the tail of its output for the log. The program leads a
///  process group of its own, so the stop at the limit reaches the children it started too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runTool(program: string, command: ToolCommand, limit: number): Promise<ToolRun>;
/// ```
pub fn run_tool(program: &Path, command: &ToolCommand, limit: Duration) -> ToolRun {
    use std::os::unix::process::CommandExt;
    let started: Instant = Instant::now();
    let mut starting: Command = Command::new(program);
    starting.args(&command.arguments).current_dir(&command.directory).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).process_group(0);
    for (name, value) in &command.set {
        starting.env(name, value);
    }
    let mut child: Child = match starting.spawn() {
        Ok(running) => running,
        Err(error) => return ToolRun { end: ToolEnd::Failed(format!("{} could not start: {error}", program.display())), millis: started.elapsed().as_millis(), stdout: Vec::new() },
    };
    let out_reader: Option<std::thread::JoinHandle<Vec<u8>>> = child.stdout.take().map(collect);
    let error_reader: Option<std::thread::JoinHandle<Vec<u8>>> = child.stderr.take().map(collect);
    let status: Option<std::process::ExitStatus> = loop {
        match child.try_wait() {
            Ok(Some(done)) => break Some(done),
            Ok(None) if started.elapsed() >= limit => break None,
            Ok(None) => std::thread::sleep(POLL_INTERVAL),
            Err(error) => return ToolRun { end: ToolEnd::Failed(format!("{} could not be waited for: {error}", program.display())), millis: started.elapsed().as_millis(), stdout: Vec::new() },
        }
    };
    let Some(finished) = status else {
        stop_group(&child);
        // The group is gone, so the pipes close and the readers end; the reaped child leaves no zombie.
        let _reaped = child.wait();
        for reader in [out_reader, error_reader].into_iter().flatten() {
            let _output = reader.join();
        }
        return ToolRun { end: ToolEnd::TimedOut, millis: started.elapsed().as_millis(), stdout: Vec::new() };
    };
    let stdout: Vec<u8> = out_reader.map(|reader| { return reader.join().unwrap_or_default(); }).unwrap_or_default();
    let errors: Vec<u8> = error_reader.map(|reader| { return reader.join().unwrap_or_default(); }).unwrap_or_default();
    let millis: u128 = started.elapsed().as_millis();
    if finished.success() {
        return ToolRun { end: ToolEnd::Succeeded, millis, stdout };
    }
    let combined: Vec<u8> = [stdout.as_slice(), errors.as_slice()].concat();
    let text: String = tail_of(combined.as_slice());
    return ToolRun { end: ToolEnd::Failed(format!("{}: {text}", describe_status(finished))), millis, stdout: Vec::new() };
}

/// What:
///  Stop the child's whole process group.
/// Why:
///   pnpm and Cargo start helpers; stopping only the leader would leave them running against the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.kill(-child.pid, 'SIGKILL');
/// ```
fn stop_group(child: &Child) {
    let Ok(leader) = libc::pid_t::try_from(child.id()) else { return };
    // SAFETY: `kill` takes a process-group id and a signal number and touches no memory; the group was made by this process.
    let _stopped = unsafe { libc::kill(-leader, libc::SIGKILL) };
}

/// Command controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_command_tests.rs"]
mod tests;
