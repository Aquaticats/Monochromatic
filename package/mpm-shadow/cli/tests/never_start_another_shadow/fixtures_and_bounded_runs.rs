//! What:
//! The fixtures and the bounded run every test of this crate shares:
//! a scratch folder beside the build output,
//! copies of the shadow at another version,
//! the shadow links and real-tool scripts a test puts on its own `PATH`,
//! the command that starts one of them,
//! and the run that is killed once it passes `LOOP_BOUND`.
//! Why:
//! Every test here meets the same failure mode,
//! a shadow that starts another shadow,
//! so every run is polled against one deadline instead of waiting without end,
//! and one copy of that wait means a test cannot accidentally wait forever.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // never_start_another_shadow/fixtures-and-bounded-runs.ts
//! export const LOOP_BOUND_MS = 5000;
//! export function runBounded(command: Command): Outcome
//! export function copyOfAnotherVersion(target: string): void
//! export function commandFor(link: string, pathFolders: string[]): Command
//! ```

use std::fs;
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Child,
    Command,
    Output,
    Stdio,
};
use std::thread::sleep;
use std::time::{
    Duration,
    Instant,
};
/// The built shadow.
pub const SHADOW_EXECUTABLE: &str = env!("CARGO_BIN_EXE_mpm-shadow");

/// The file name of the shadow program inside each folder of copies.
pub const SHADOW_FILE: &str = "mpm-shadow";

/// The tool the shadow shadows.
pub const TOOL: &str = "mpm";

/// The arguments of a run that the shadow passes straight to the real tool.
pub const PASS_THROUGH_ARGUMENTS: &[&str] = &["some-tool-verb", "x"];

/// The name of the environment variable the arrival marker travels in (`src/shadow_identity.rs`).
pub const ARRIVAL_VARIABLE: &str = "MONOCHROMATIC_SHADOW_STARTED_BY";

/// The version the built shadow carries,
/// which the copies below change.
pub const OWN_VERSION: &str = env!("CARGO_PKG_VERSION");

/// How long a run may take before it counts as a loop.
/// A run that ends takes a few milliseconds;
/// the loop it replaces never ends.
/// The bound is far above the first and below the patience of a test run,
/// and it only matters in the failing case.
pub const LOOP_BOUND: Duration = Duration::from_secs(5);

/// How often a run is polled while it has not ended.
pub const POLL_INTERVAL: Duration = Duration::from_millis(20);

/// What a bounded run did.
pub enum Outcome {
    /// It ended by itself.
    Finished(Output),
    /// It was still running at the bound and was killed.
    Looped,
}

/// Runs a prepared command with the deadline `LOOP_BOUND`.
pub fn run_bounded(mut command: Command) -> Outcome {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child: Child = command.spawn().expect("start the shadow");
    let started: Instant = Instant::now();
    loop {
        if child.try_wait().expect("poll the shadow").is_some() {
            return Outcome::Finished(child.wait_with_output().expect("collect the output"));
        }
        if started.elapsed() >= LOOP_BOUND {
            child.kill().expect("kill the looping shadow");
            child.wait().expect("reap the looping shadow");
            return Outcome::Looped;
        }
        sleep(POLL_INTERVAL);
    }
}

/// The output of a run that must have ended,
/// or a failure that says it looped.
pub fn finished(outcome: Outcome) -> Output {
    match outcome {
        Outcome::Finished(output) => return output,
        Outcome::Looped => panic!(
            "the run was still going after {LOOP_BOUND:?}: the shadows are starting each other"
        ),
    }
}

/// Copies a program and keeps it executable,
/// through a child `cp` process.
pub fn copy_program(
    source: &Path,
    target: &Path,
) {
    // What:     A test starts the copy as the shadow.
    // Why:      A copy this process made would be open for writing here, and a child of
    //           another test thread could inherit that descriptor, so its start would be
    //           refused with `ETXTBSY` (issue 6527).
    crate::through_a_child_process::copy_program(source, target);
}

/// A copy of the shadow that names another version:
/// the version text inside it changes in place,
/// so the copy is a different file with a different identity text,
/// as an older release would be.
pub fn copy_of_another_version(target: &Path) {
    let mut bytes: Vec<u8> = fs::read(SHADOW_EXECUTABLE).expect("read the shadow");
    let needle: &[u8] = OWN_VERSION.as_bytes();
    let mut other: Vec<u8> = needle.to_vec();
    let last: usize = other.len() - 1;
    other[last] = if other[last] == b'9' { b'8' } else { b'9' };
    let mut index: usize = 0;
    while index + needle.len() <= bytes.len() {
        if &bytes[index..index + needle.len()] == needle {
            bytes[index..index + needle.len()].copy_from_slice(&other);
            index += needle.len();
        } else {
            index += 1;
        }
    }
    // What:     The version text of the copy is rewritten, and a test starts the copy.
    // Why:      A child process writes it for the reason `copy_program` gives.
    crate::through_a_child_process::write_program(target, bytes.as_slice());
}

/// A shadow link:
/// a symlink named after the tool,
/// to the shadow program of its folder.
pub fn link_as_tool(folder: &Path) -> PathBuf {
    let link: PathBuf = folder.join(TOOL);
    std::os::unix::fs::symlink(folder.join(SHADOW_FILE), &link).expect("make the shadow link");
    return link;
}

/// A script standing in for the real tool:
/// prints which tool answered and the arrival marker it was started with.
pub fn write_real_script(folder: &Path) {
    // A test starts this script as the real tool, so a child process writes it.
    crate::through_a_child_process::write_program(
        folder.join(TOOL).as_path(),
        format!("#!/bin/sh\nprintf 'REAL {TOOL} %s\\n' \"$*\"\nprintf 'MARKER %s\\n' \"${ARRIVAL_VARIABLE}\"\n").as_bytes(),
    );
}

/// A scratch folder beside the build output.
pub fn scratch() -> tempfile::TempDir {
    std::fs::create_dir_all(env!("CARGO_TARGET_TMPDIR")).expect("make the scratch root");
    return tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a scratch directory");
}

/// The command that starts a shadow link with a bare environment and the given `PATH`.
pub fn command_for(
    link: &Path,
    path_folders: &[&Path],
) -> Command {
    let path_variable: String = path_folders
        .iter()
        .map(|folder| {
            return folder.display().to_string();
        })
        .collect::<Vec<String>>()
        .join(":");
    let mut command: Command = Command::new(link);
    command.args(PASS_THROUGH_ARGUMENTS);
    command.current_dir(env!("CARGO_TARGET_TMPDIR"));
    command.env_clear();
    command.env("PATH", path_variable);
    return command;
}

/// The two lines the real script prints:
/// what ran and the marker it saw.
pub fn expected_first_line() -> String {
    return format!("REAL {TOOL} {}", PASS_THROUGH_ARGUMENTS.join(" "));
}

/// The start time of a process,
/// as `/proc/<pid>/stat` field 22 says it.
pub fn birth_of(pid: u32) -> String {
    let text: String = fs::read_to_string(format!("/proc/{pid}/stat")).expect("read the stat file");
    let after_name: &str = text
        .rsplit_once(')')
        .expect("the name ends with a bracket")
        .1;
    return after_name
        .split_whitespace()
        .nth(19)
        .expect("field 22")
        .to_string();
}
