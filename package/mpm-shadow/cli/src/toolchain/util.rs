//! What:
//! Small named helpers the toolchain modules share:
//! text predicates,
//! a tool lookup by id,
//! file-name conversions,
//! and the one atomic file write.
//! Why:
//! The unified linter asks for a named function wherever a closure captures nothing or only borrows;
//! these are those functions,
//! written once and tested once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // util.ts
//! export const allDigits = (text: string) => /^\d*$/.test(text);
//! export const findTool = (tools: LockedTool[], id: string) => tools.find((tool) => tool.declaration.id === id);
//! ```

/// Imports `OsString` from `std::ffi`.
use std::ffi::OsString;
/// Imports the file-system module the atomic write uses.
use std::fs;
/// Imports `Path` and `PathBuf` from `std::path`.
use std::path::{Path, PathBuf};

/// Imports the toolchain error type.
use super::error::ToolchainError;
/// Imports the locked tool type.
use super::model::LockedTool;

/// Whether every character of `text` is an ASCII digit (true for the empty text).
pub fn all_ascii_digits(text: &str) -> bool {
    for character in text.chars() {
        if !character.is_ascii_digit() {
            return false;
        }
    }
    return true;
}

/// Whether the first character of `text` is an ASCII digit.
pub fn starts_with_ascii_digit(text: &str) -> bool {
    return match text.chars().next() {
        Some(character) => character.is_ascii_digit(),
        None => false,
    };
}

/// Whether `text` contains any of `needles`.
pub fn contains_any(text: &str, needles: &[&str]) -> bool {
    for needle in needles {
        if text.contains(needle) {
            return true;
        }
    }
    return false;
}

/// Whether `text` ends with any of `suffixes`.
pub fn ends_with_any(text: &str, suffixes: &[&str]) -> bool {
    for suffix in suffixes {
        if text.ends_with(suffix) {
            return true;
        }
    }
    return false;
}

/// The tool with this id,
/// or `None`.
pub fn find_tool<'a>(tools: &'a [LockedTool], id: &str) -> Option<&'a LockedTool> {
    for tool in tools {
        if tool.declaration.id == id {
            return Some(tool);
        }
    }
    return None;
}

/// The final path component as text (invalid UTF-8 replaced),
/// or the empty text.
pub fn file_name_text(path: &Path) -> String {
    return match path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => String::new(),
    };
}

/// The final path component as an owned `OsString`,
/// or an empty one.
pub fn file_name_os(path: &Path) -> OsString {
    return match path.file_name() {
        Some(name) => name.to_os_string(),
        None => OsString::new(),
    };
}

/// A version segment as a number (0 when it is not one).
pub fn number_or_zero(part: &str) -> u64 {
    return part.parse::<u64>().unwrap_or(0);
}

/// What:
/// The mode a file this crate creates gets when no kept mode is named:
/// the kernel takes the process's umask off it.
/// Why:
/// A lock,
/// a launch record or the record of postinstall links has no predecessor to inherit from,
/// so they keep the mode any new file gets.
#[cfg(unix)]
const NEW_FILE_MODE: u32 = 0o666;

/// What:
/// The step the crate's atomic write offers a test:
/// it runs the instant the temporary beside `target` holds every byte of the new file,
/// before the rename and before anything else this write does.
/// Why:
/// That instant is the only one a reader could see the new content before it takes the target's name,
/// so it is where a test reads the file a reader could open and the mode it would find;
/// a write that put the mode on after its bytes would be caught here,
/// and the wording is deliberately about the bytes alone, not about the mode,
/// so a control measures what a reader sees rather than what the write promised.
/// It is also where a test can end the run as a kill would,
/// which is the step whose leftovers the next write must clear.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BeforeRename = (temporary: string) => void;
/// ```
pub(crate) type BeforeRename<'a> = &'a dyn Fn(&Path) -> Result<(), ToolchainError>;

/// The step the real calls pass:
/// nothing happens between the write and the rename.
fn no_step(_temporary: &Path) -> Result<(), ToolchainError> {
    return Ok(());
}

/// The dot-named temporary beside `target` that one write of it fills.
fn temporary_of(target: &Path) -> PathBuf {
    let mut temporary_name: OsString = OsString::from(".");
    temporary_name.push(file_name_os(target));
    temporary_name.push(format!(".{}.tmp", std::process::id()));
    return target.with_file_name(temporary_name);
}

/// What:
/// Opens the temporary beside `target`, asking for `mode` when the platform has modes.
/// Why:
/// The mode has to be on the file from its creation,
/// so the kernel's own default is never wider than the mode the finished file keeps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openTemporary(temporary: string, mode?: number): FileHandle
/// ```
#[cfg(unix)]
fn open_temporary(temporary: &Path, mode: Option<u32>) -> std::io::Result<fs::File> {
    /// Imports `OpenOptionsExt` from `std::os::unix::fs`,
    /// which carries the mode a created file is asked for.
    use std::os::unix::fs::OpenOptionsExt;
    return fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(mode.unwrap_or(NEW_FILE_MODE))
        .open(temporary);
}

/// Opens the temporary beside `target`;
/// a platform without modes creates it with whatever a new file gets.
#[cfg(not(unix))]
fn open_temporary(temporary: &Path, _mode: Option<u32>) -> std::io::Result<fs::File> {
    return fs::OpenOptions::new().write(true).create(true).truncate(true).open(temporary);
}

/// What:
/// Puts `mode` on the open temporary,
/// before one byte of the new file is written into it.
/// Why:
/// The kernel takes the umask off the mode `open` was asked for,
/// which can only be narrower than the kept mode;
/// this makes the file carry exactly the kept mode,
/// never wider,
/// whatever the umask,
/// from the instant the content starts arriving.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keepMode(temporary: FileHandle, mode?: number): void
/// ```
#[cfg(unix)]
fn keep_mode(temporary: &fs::File, mode: Option<u32>) -> std::io::Result<()> {
    /// Imports `PermissionsExt` from `std::os::unix::fs`.
    use std::os::unix::fs::PermissionsExt;
    return match mode {
        Some(bits) => temporary.set_permissions(fs::Permissions::from_mode(bits)),
        None => Ok(()),
    };
}

/// Puts `mode` on the open temporary;
/// a platform without modes has none to put.
#[cfg(not(unix))]
fn keep_mode(_temporary: &fs::File, _mode: Option<u32>) -> std::io::Result<()> {
    return Ok(());
}

/// What:
/// Whether the process `pid` is gone,
/// among the platforms that can answer.
/// Why:
/// A temporary left by a kill belongs to nobody once its process is gone,
/// and removing a live run's temporary would break that run's rename.
/// `/proc` answers this on Linux;
/// where it is absent or unreadable the answer is "not gone", so nothing is removed on a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function processIsGone(pid: number): boolean
/// ```
#[cfg(target_os = "linux")]
fn process_is_gone(pid: u32) -> bool {
    if pid == std::process::id() || !Path::new("/proc/self").exists() {
        return false;
    }
    return !Path::new("/proc").join(pid.to_string()).exists();
}

/// Whether the process `pid` is gone;
/// no platform but Linux can answer, so the answer is no.
#[cfg(not(target_os = "linux"))]
fn process_is_gone(_pid: u32) -> bool {
    return false;
}

/// What:
/// Removes the temporaries a killed write left beside `target`,
/// and leaves the ones whose process still runs.
/// Why:
/// A kill between the create and the rename leaves `.<name>.<pid>.tmp` holding the whole new content,
/// and nothing else would ever clear it;
/// the next write of the same target is the one place that knows both the name shape and which processes are gone.
/// Only the module that names the temporaries may judge which files those are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removeTemporariesOfGoneProcesses(target: string): void
/// ```
fn remove_temporaries_of_gone_processes(target: &Path) {
    let Some(parent) = target.parent() else {
        return;
    };
    let prefix: String = format!(".{}.", file_name_text(target));
    let Ok(entries) = fs::read_dir(parent) else {
        return;
    };
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        let Some(rest) = name.strip_prefix(&prefix) else {
            continue;
        };
        let Some(digits) = rest.strip_suffix(".tmp") else {
            continue;
        };
        if digits.is_empty() || !all_ascii_digits(digits) {
            continue;
        }
        let Ok(pid) = digits.parse::<u32>() else {
            continue;
        };
        if process_is_gone(pid) {
            // Best effort: a leftover that cannot be removed changes nothing for the write below,
            // and the next write of the same target tries again.
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// What:
/// Writes `bytes` to `target` through a dot-named temporary beside it and a rename,
/// so a reader sees the old file or the new one,
/// never half of it.
/// Why:
/// Every file this crate replaces (a launcher,
/// a launch record,
/// the lock) is read by other programs without a lock,
/// and one owner keeps the temporary naming and the failure handling from drifting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function writeAtomically(target: string, bytes: Uint8Array): void
/// ```
pub fn write_atomically(target: &Path, bytes: &[u8]) -> Result<(), ToolchainError> {
    return write_atomically_with_mode(target, bytes, None);
}

/// What:
/// `write_atomically`, giving the file the mode `mode` from its creation until the rename.
/// Why:
/// The plan is a file people own,
/// and the one kept readable by its owner alone (`0600`) must never be readable by anyone else,
/// so its content is never in a file wider than the mode the plan keeps:
/// the temporary is created asking for that mode,
/// the mode is put on it before the first byte arrives,
/// and the rename is the only visible step.
/// `None` leaves the mode a new file gets.
///
/// What a kill leaves, step by step:
/// - before the temporary is created, nothing: the target holds the old content;
/// - after the create and before the mode is put on, the temporary is empty,
///   and narrower than the kept mode, never wider;
/// - while the bytes are written, the temporary holds a prefix of the new content at exactly the kept mode;
/// - after the write and before the rename, it holds the whole new content at exactly the kept mode,
///   and the target still holds the old one;
/// - after the rename, the target holds the new content, complete.
/// Every leftover of the steps above is `.<name>.<pid>.tmp` beside the target,
/// readable no more widely than the target itself,
/// and the next atomic write of that same target removes it
/// (`remove_temporaries_of_gone_processes`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function writeAtomicallyWithMode(target: string, bytes: Uint8Array, mode?: number): void
/// ```
pub fn write_atomically_with_mode(target: &Path, bytes: &[u8], mode: Option<u32>) -> Result<(), ToolchainError> {
    return write_atomically_at(target, bytes, mode, &no_step);
}

/// What:
/// The atomic write itself,
/// with the step a test can read the temporary at and end the run at.
/// Why:
/// One implementation serves the real writes and the controls that prove what a reader and a kill see;
/// the step is the only thing they differ in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeAtomicallyAt(target: string, bytes: Uint8Array, mode: number | undefined, beforeRename: BeforeRename): void
/// ```
pub(crate) fn write_atomically_at(
    target: &Path,
    bytes: &[u8],
    mode: Option<u32>,
    before_rename: BeforeRename,
) -> Result<(), ToolchainError> {
    /// Imports `Write` from `std::io`,
    /// so the bytes go through the descriptor whose mode is already right.
    use std::io::Write;
    remove_temporaries_of_gone_processes(target);
    let temporary: PathBuf = temporary_of(target);
    let mut file: fs::File = match open_temporary(&temporary, mode) {
        Ok(opened) => opened,
        Err(error) => {
            return Err(ToolchainError::failure(format!("cannot write {}: {error}", temporary.display())));
        }
    };
    if let Err(error) = keep_mode(&file, mode) {
        // Best effort: an empty temporary with the wrong mode is useless, and the error below is what matters.
        let _ = fs::remove_file(&temporary);
        return Err(ToolchainError::failure(format!("cannot give {} its mode: {error}", temporary.display())));
    }
    if let Err(error) = file.write_all(bytes) {
        let _ = fs::remove_file(&temporary);
        return Err(ToolchainError::failure(format!("cannot write {}: {error}", temporary.display())));
    }
    // The descriptor is closed before the rename, so a failure to flush the last bytes is this write's failure and
    // not a surprise for whoever reads the target afterwards.
    drop(file);
    if let Err(error) = before_rename(&temporary) {
        // The step stands where a kill lands: nothing after it runs, this write's cleanup included,
        // so the temporary stays exactly as a killed write leaves it.
        return Err(error);
    }
    if let Err(error) = fs::rename(&temporary, target) {
        // Best effort: the temporary file is useless now, and a failed removal changes nothing for the caller.
        let _ = fs::remove_file(&temporary);
        return Err(ToolchainError::failure(format!("cannot replace {}: {error}", target.display())));
    }
    return Ok(());
}
