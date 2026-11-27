//! What:
//! The launcher records that stand beside a toolchain launcher:
//! a JSON object whose `real` field names the program the launcher starts.
//! Why:
//! The toolchain puts a launcher between a link and a tool;
//! the launcher's own bytes carry the shadow identity,
//! so it is judged by where its record leads,
//! not by what it is.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function shadowBehind(candidate: string): string | undefined { /* follow .launch records up to MAX_LAUNCHER_HOPS */ }
//! ```
/// Imports the owned operating-system text type,
/// which carries a marker value that need not be valid UTF-8.
use std::ffi::OsString;
/// Imports the file-system module that reads the record.
use std::fs;
/// Imports the borrowed and owned path types.
use std::path::{
    Path,
    PathBuf,
};

/// Imports the reader for the identity section.
use super::elf_section::identity_of_open_file;

/// The suffix of the record the toolchain writes beside a launcher (`package/mpm-shadow/cli/README.md`,
/// section "The launcher").
pub const LAUNCH_RECORD_SUFFIX: &str = ".launch";

/// How many launchers in a row are followed before the chain counts as one that never reaches a program.
/// The toolchain makes one launcher per tool and a launcher names a program,
/// so a chain of two is already unusual;
/// a longer one is a loop of records.
/// Delegated pick under the owner's rule that nothing waits without end;
/// not a measurement.
pub const MAX_LAUNCHER_HOPS: usize = 8;

/// What:
/// `fn record_target(program: &Path) -> Option<PathBuf>` is the program that the launcher record beside `program` names,
/// when there is one.
/// Why:
/// The toolchain puts a launcher (a copy of the `mpm-shadow` program,
/// which carries the identity text) between a link and a tool;
/// the record beside it,
/// a JSON object with a `real` field,
/// names what it starts.
/// A launcher is judged by where it leads,
/// not by what it is.
/// An unreadable or unparsable record is not a launcher record.
fn record_target(program: &Path) -> Option<PathBuf> {
    let mut record_name: OsString = program.as_os_str().to_os_string();
    record_name.push(LAUNCH_RECORD_SUFFIX);
    let text: String = fs::read_to_string(PathBuf::from(record_name)).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    return value.get("real")?.as_str().map(PathBuf::from);
}

/// What:
/// `pub fn shadow_behind(candidate: &Path) -> Option<String>` is the identity of the shadow that starting `candidate` would start,
/// when it would:
/// the candidate itself,
/// or the program a launcher record leads to,
/// through further launchers up to `MAX_LAUNCHER_HOPS`.
/// Why:
/// A path or a version says nothing about what a file is.
/// A candidate that leads to a shadow is skipped without being started.
/// A chain of records that never ends counts as leading to a shadow (it would start launchers without end).
/// A file that cannot be opened or a record that names a missing file is "cannot tell":
/// the arrival marker is the backstop for what this cannot see.
pub fn shadow_behind(candidate: &Path) -> Option<String> {
    let mut current: PathBuf = candidate.to_path_buf();
    for _hop in 0..=MAX_LAUNCHER_HOPS {
        let resolved: PathBuf = fs::canonicalize(&current).ok()?;
        if let Some(target) = record_target(&resolved) {
            current = target;
            continue;
        }
        let file: fs::File = fs::File::open(&resolved).ok()?;
        return identity_of_open_file(&file);
    }
    return Some(format!(
        "a chain of more than {MAX_LAUNCHER_HOPS} launchers that never reaches a program"
    ));
}

/// Whether a launcher record stands beside the program once its path is resolved,
/// which makes the program a launcher whose own bytes name no tool.
pub(super) fn has_launcher_record(program: &Path) -> bool {
    return match fs::canonicalize(program) {
        Ok(resolved) => record_target(&resolved).is_some(),
        Err(_) => false,
    };
}

/// What:
/// `pub fn identity_of_open_file_unless_launcher(file: &fs::File, program: &Path) -> Option<String>` is the identity of the shadow that
/// starting `program` would start,
/// judged on the open file when no launcher record stands beside the path and by where the record leads when one does.
/// Why:
/// A launcher is a copy of the `mpm-shadow` program,
/// which carries the identity:
/// its own bytes say nothing about the tool it starts.
/// Without a record the open file is the whole answer,
/// and it is the file that is about to be started.
pub fn identity_of_open_file_unless_launcher(
    file: &fs::File,
    program: &Path,
) -> Option<String> {
    let has_record: bool = has_launcher_record(program);
    if has_record {
        return shadow_behind(program);
    }
    return identity_of_open_file(file);
}
