//! What:
//! The disposable `PATH` a `tests/never_start_another_shadow.rs` test builds:
//! shadow copies of another version,
//! a launcher with or without a record,
//! a real tool script,
//! and the command that starts one link through that `PATH`.
//! Why:
//! Every test differs only in what it puts on the `PATH`,
//! so one set of builders keeps the arrangements identical and the copies cheap.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function copyOfAnotherVersion(target: string): void { /* change the version text inside a copy */ }
//! function commandFor(link: string, folders: string[]): Command { /* bare PATH of shadow, launchers, real */ }
//! ```

/// Imports the file helpers that build the copies and records.
use std::fs;
/// Imports the executable-mode helper.
use std::os::unix::fs::PermissionsExt;
/// Imports the borrowed and owned path types.
use std::path::{
    Path,
    PathBuf,
};
/// Imports the child-process builder the command helpers return.
use std::process::Command;

/// The built shadow.
pub(crate) const SHADOW_EXECUTABLE: &str = env!("CARGO_BIN_EXE_pnpm-shadow");

/// The file name of the shadow program inside each folder of copies.
pub(crate) const SHADOW_FILE: &str = "pnpm-shadow";

/// The tool the shadow shadows.
pub(crate) const TOOL: &str = "pnpm";

/// The arguments of a run that the shadow passes straight to the real tool.
const PASS_THROUGH_ARGUMENTS: &[&str] = &["run", "task"];

/// The name of the environment variable the arrival marker travels in (`src/shadow_identity.rs`).
pub(crate) const ARRIVAL_VARIABLE: &str = "MONOCHROMATIC_SHADOW_STARTED_BY";

/// The version the built shadow carries,
/// which the copies below change.
pub(crate) const OWN_VERSION: &str = env!("CARGO_PKG_VERSION");

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
