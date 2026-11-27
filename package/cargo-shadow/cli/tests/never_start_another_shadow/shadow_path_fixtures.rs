//! What:
//! The fixtures the end-to-end tests of "a shadow never starts a shadow" build:
//! disposable `PATH` folders of shadow copies,
//! launcher records and real programs,
//! and a run of the built `cargo-shadow` under the name `cargo` with a deadline.
//! Why:
//! The tests in `tests/never_start_another_shadow.rs` each need the same fixtures,
//! and the file held more code lines than a reader can hold at once;
//! the fixtures were split out of it to keep the test file under the line limit.
//! A test of such a loop must itself never wait without end,
//! so every run here is polled against a deadline and killed when it passes it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // tests/never-start-another-shadow/shadow-path-fixtures.ts
//! export const runBounded = (command: ChildProcess): Outcome => ...
//! export const commandFor = (link: string, pathFolders: string[]): ChildProcess => ...
//! ```

use std::fs;
use std::os::unix::fs::PermissionsExt;
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
pub(crate) const SHADOW_EXECUTABLE: &str = env!("CARGO_BIN_EXE_cargo-shadow");

/// The file name of the shadow program inside each folder of copies.
pub(crate) const SHADOW_FILE: &str = "cargo-shadow";

/// The tool the shadow shadows.
pub(crate) const TOOL: &str = "cargo";

/// The arguments of a run that the shadow passes straight to the real tool.
pub(crate) const PASS_THROUGH_ARGUMENTS: &[&str] = &["fmt", "--check"];

/// The name of the environment variable the arrival marker travels in (`src/shadow_identity.rs`).
pub(crate) const ARRIVAL_VARIABLE: &str = "MONOCHROMATIC_SHADOW_STARTED_BY";

/// The version the built shadow carries,
/// which the copies below change.
pub(crate) const OWN_VERSION: &str = env!("CARGO_PKG_VERSION");

/// How long a run may take before it counts as a loop.
/// A run that ends takes a few milliseconds;
/// the loop it replaces never ends.
/// The bound is far above the first and below the patience of a test run,
/// and it only matters in the failing case.
pub(crate) const LOOP_BOUND: Duration = Duration::from_secs(5);

/// How often a run is polled while it has not ended.
const POLL_INTERVAL: Duration = Duration::from_millis(20);

/// What a bounded run did.
pub(crate) enum Outcome {
    /// It ended by itself.
    Finished(Output),
    /// It was still running at the bound and was killed.
    Looped,
}

/// Runs a prepared command with the deadline `LOOP_BOUND`.
pub(crate) fn run_bounded(mut command: Command) -> Outcome {
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
pub(crate) fn finished(outcome: Outcome) -> Output {
    match outcome {
        Outcome::Finished(output) => return output,
        Outcome::Looped => panic!(
            "the run was still going after {LOOP_BOUND:?}: the shadows are starting each other"
        ),
    }
}

/// Copies a program and keeps it executable.
pub(crate) fn copy_program(
    source: &Path,
    target: &Path,
) {
    fs::create_dir_all(target.parent().expect("a parent folder")).expect("make the folder");
    fs::copy(source, target).expect("copy the program");
    fs::set_permissions(target, fs::Permissions::from_mode(0o755)).expect("make it executable");
}

/// A copy of the shadow that names another version:
/// the version text inside it changes in place,
/// so the copy is a different file with a different identity text,
/// as an older release would be.
pub(crate) fn copy_of_another_version(target: &Path) {
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
    fs::create_dir_all(target.parent().expect("a parent folder")).expect("make the folder");
    fs::write(target, bytes).expect("write the copy");
    fs::set_permissions(target, fs::Permissions::from_mode(0o755)).expect("make it executable");
}

/// A shadow link:
/// a symlink named after the tool,
/// to the shadow program of its folder.
pub(crate) fn link_as_tool(folder: &Path) -> PathBuf {
    let link: PathBuf = folder.join(TOOL);
    std::os::unix::fs::symlink(folder.join(SHADOW_FILE), &link).expect("make the shadow link");
    return link;
}

/// A script standing in for the real tool:
/// prints which tool answered and the arrival marker it was started with.
pub(crate) fn write_real_script(folder: &Path) {
    fs::create_dir_all(folder).expect("make the folder");
    fs::write(folder.join(TOOL), format!("#!/bin/sh\nprintf 'REAL {TOOL} %s\\n' \"$*\"\nprintf 'MARKER %s\\n' \"${ARRIVAL_VARIABLE}\"\n")).expect("write the script");
    fs::set_permissions(folder.join(TOOL), fs::Permissions::from_mode(0o755))
        .expect("make it executable");
}

/// A scratch folder beside the build output.
pub(crate) fn scratch() -> tempfile::TempDir {
    std::fs::create_dir_all(env!("CARGO_TARGET_TMPDIR")).expect("make the scratch root");
    return tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a scratch directory");
}

/// The command that starts a shadow link with a bare environment and the given `PATH`.
pub(crate) fn command_for(
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
pub(crate) fn expected_first_line() -> String {
    return format!("REAL {TOOL} {}", PASS_THROUGH_ARGUMENTS.join(" "));
}

/// The start time of a process,
/// as `/proc/<pid>/stat` field 22 says it.
pub(crate) fn birth_of(pid: u32) -> String {
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
