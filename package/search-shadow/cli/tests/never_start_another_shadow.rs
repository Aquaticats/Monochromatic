//! What:
//! End-to-end tests of the rule "a shadow never starts a shadow,
//! of any version,
//! as its real tool".
//! Each test builds a disposable `PATH` of shadow copies,
//! launcher records and real programs,
//! starts the built `search-shadow` under the name `grep` or `rg`,
//! and gives it a deadline.
//! This file holds what every test of the rule needs:
//! the copy of a shadow,
//! the link named after a tool,
//! the deadline a run is polled against,
//! and the command that starts one.
//! The tests themselves sit in one file per part of the rule:
//! the two copies that find each other,
//! the launcher records,
//! a genuine program behind the shadow,
//! the arrival marker,
//! and the links `link` makes and takes over.
//! Why:
//! Two copies of the shadow at different paths used to pick each other as the real `grep`
//! and replace themselves with each other without end (measured 2026-11-07:
//! about 105 starts per second under `strace`,
//! at a quarter of one CPU).
//! A test of that loop must itself never wait without end,
//! so every run here is polled against a deadline and killed when it passes it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await runWithDeadline(spawn(join(a, 'grep'), ['x', file], { env: { PATH } }), LOOP_BOUND);
//! assert(outcome.kind === 'finished');
//! ```

/// The writer of the program files these tests start,
/// shared with the crate's other test binaries.
#[path = "written_programs/through_a_child_process.rs"]
mod through_a_child_process;

/// What:
/// `#[path = "..."] mod two_copies_at_one_name;` declares a submodule and names the file it is read from.
/// A test crate's root file looks for its submodules beside itself,
/// so a submodule kept under `tests/never_start_another_shadow/` is named here.
/// Why:
/// The tests of one part of the rule stay in one file,
/// and this file keeps the fixtures they all use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './two-copies-at-one-name.integration.test';
/// ```
#[path = "never_start_another_shadow/two_copies_at_one_name.rs"]
mod two_copies_at_one_name;

/// The launcher records:
/// a candidate is judged by the program its record names.
#[path = "never_start_another_shadow/launcher_records.rs"]
mod launcher_records;

/// A genuine compiled program without the identity text still runs behind the shadow.
#[path = "never_start_another_shadow/a_genuine_program_behind_the_shadow.rs"]
mod a_genuine_program_behind_the_shadow;

/// The arrival marker:
/// a shadow that a shadow started stops on one line.
#[path = "never_start_another_shadow/the_arrival_marker.rs"]
mod the_arrival_marker;

/// The links `search-shadow link` makes,
/// leaves alone and takes over.
#[path = "never_start_another_shadow/link_takes_over_a_shadow_link.rs"]
mod link_takes_over_a_shadow_link;

/// What:
/// `use std::fs;` imports the file-system module (copies a program,
/// writes a script,
/// makes a link).
/// Why:
/// Every fixture here is a file in a temporary folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as fs from 'node:fs';
/// ```
use std::fs;

/// Imports the trait that adds `from_mode` to permissions (marks a copy executable).

/// Imports the borrowed and the owned path types of the fixtures.
use std::path::{
    Path,
    PathBuf,
};

/// Imports the types a started run is held by:
/// the running process,
/// its command and its collected output.
use std::process::{
    Child,
    Command,
    Output,
    Stdio,
};

/// Imports the sleep that spaces the polls of a running process.
use std::thread::sleep;

/// Imports the deadline type and the instant an elapsed time is measured from.
use std::time::{
    Duration,
    Instant,
};

/// The built shadow.
const SHADOW_EXECUTABLE: &str = env!("CARGO_BIN_EXE_search-shadow");

/// The name of the environment variable the arrival marker travels in (`src/shadow_identity.rs`).
const ARRIVAL_VARIABLE: &str = "MONOCHROMATIC_SHADOW_STARTED_BY";

/// The version the built shadow carries,
/// which the copies below change.
const OWN_VERSION: &str = env!("CARGO_PKG_VERSION");

/// How long a run may take before it counts as a loop.
/// A run that ends takes a few milliseconds (a start,
/// a path lookup and one more start);
/// the loop it replaces never ends.
/// The bound is far above the first and below the patience of a test run,
/// and it only matters in the failing case.
const LOOP_BOUND: Duration = Duration::from_secs(5);

/// How often a run is polled while it has not ended.
const POLL_INTERVAL: Duration = Duration::from_millis(20);

/// What a bounded run did.
enum Outcome {
    /// It ended by itself.
    Finished(Output),
    /// It was still running at the bound and was killed.
    Looped,
}

/// Runs a prepared command with the deadline `LOOP_BOUND`.
fn run_bounded(mut command: Command) -> Outcome {
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
            // A replaced process keeps its id, so killing this one ends the whole chain.
            child.kill().expect("kill the looping shadow");
            child.wait().expect("reap the looping shadow");
            return Outcome::Looped;
        }
        sleep(POLL_INTERVAL);
    }
}

/// The output of a run that must have ended,
/// or a failure that says it looped.
fn finished(outcome: Outcome) -> Output {
    match outcome {
        Outcome::Finished(output) => return output,
        Outcome::Looped => panic!(
            "the run was still going after {LOOP_BOUND:?}: the shadows are starting each other"
        ),
    }
}

/// Copies a program and keeps it executable,
/// through a child `cp` process.
fn copy_program(
    source: &Path,
    target: &Path,
) {
    // A test starts the copy as the shadow, so a child process makes it (issue 6527).
    crate::through_a_child_process::copy_program(source, target);
}

/// A copy of the shadow that names another version:
/// the version text inside it changes in place,
/// so the copy is a different file with a different identity text,
/// as an older release would be.
fn copy_of_another_version(target: &Path) {
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
    // A test starts the copy as the shadow, so a child process writes it (issue 6527).
    crate::through_a_child_process::write_program(target, bytes.as_slice());
}

/// A shadow link:
/// a symlink named after the tool,
/// to the shadow program of its folder.
fn link_as(
    folder: &Path,
    name: &str,
) -> PathBuf {
    let link: PathBuf = folder.join(name);
    std::os::unix::fs::symlink(folder.join("search-shadow"), &link).expect("make the shadow link");
    return link;
}

/// A script standing in for a real tool:
/// prints which tool answered.
fn write_real_script(
    folder: &Path,
    name: &str,
) {
    // A test starts this script as the real tool, so a child process writes it (issue 6527).
    crate::through_a_child_process::write_program(
        folder.join(name).as_path(),
        format!("#!/bin/sh\nprintf 'REAL {name} %s\\n' \"$*\"\n").as_bytes(),
    );
}

/// A scratch folder beside the build output.
fn scratch() -> tempfile::TempDir {
    return tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("make a scratch directory");
}

/// The command that starts a shadow link with a bare environment and the given `PATH`.
fn command_for(
    link: &Path,
    path_folders: &[&Path],
    arguments: &[&str],
) -> Command {
    let path_variable: String = path_folders
        .iter()
        .map(|folder| {
            return folder.display().to_string();
        })
        .collect::<Vec<String>>()
        .join(":");
    let mut command: Command = Command::new(link);
    command.args(arguments);
    command.env_clear();
    command.env("PATH", path_variable);
    return command;
}
