//! What:
//!  Publish a plan's drift to the real filesystem, file by file, through hidden staging names.
//! Why:
//!  Each file is replaced atomically (written beside its target under a hidden name with its final mode, then
//!  renamed), so a reader never sees half a file; atomicity is per file, and a partial run reconverges on the next
//!  run. Containment is rechecked at each mutation: no ancestor may be a link, whatever the plan saw earlier.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function publish(root: string, report: PlanReport, drifts: Drift[]): PublishOutcome
//! ```

use crate::diagnostic::Diagnostic;
use crate::drift::{Drift, DriftReason};
use crate::plan::PlanReport;
use crate::real_fs::{EntryKind, absolute, ancestors_are_real_directories, kind_of};
use crate::view_types::Planned;
use std::fs;
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;

/// The marker every staging file's name contains; names also start with `.` so Mise and Git tooling skip them.
pub const STAGING_MARKER: &str = ".fe-staging-";

/// What:
///  What a publish changed, by path.
/// Why:
///  The `done` event reports the counts and the lists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PublishOutcome = { written: string[]; modes: string[]; removed: string[] };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PublishOutcome {
    /// Files created or replaced.
    pub written: Vec<String>,
    /// Files whose permission bits were repaired.
    pub modes: Vec<String>,
    /// Files and directories removed.
    pub removed: Vec<String>,
}

/// What:
///  Wrap an I/O failure as a diagnostic that names the path.
/// Why:
///  Every filesystem error in this module reads the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ioFailure(action: string, path: string, error: Error): Diagnostic
/// ```
fn io_failure(action: &str, path: &str, error: &std::io::Error) -> Diagnostic {
    let message = format!("cannot {action} {path}: {error}");
    return Diagnostic::new("io-failure", &message).with_path(path);
}

/// What:
///  Create the missing directories above `rel`, checking each step is a real directory.
/// Why:
///  Creating a directory next to a planted link must not follow the link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ensureParents(root: string, rel: string): void
/// ```
fn ensure_parents(root: &Path, rel: &str) -> Result<(), Diagnostic> {
    let segments: Vec<&str> = rel.split('/').collect();
    let mut current = root.to_path_buf();
    for segment in segments.iter().take(segments.len().saturating_sub(1)) {
        current.push(segment);
        match fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if kind_of(&metadata) != EntryKind::Directory {
                    let message = format!("{} is not a directory; refusing to write {rel} through it", current.display());
                    return Err(Diagnostic::new("symlink-rejected", &message).with_path(rel));
                }
            }
            Err(_) => {
                if let Err(error) = fs::create_dir(&current) {
                    return Err(io_failure("create the directory", &current.display().to_string(), &error));
                }
            }
        }
    }
    return Ok(());
}

/// What:
///  Replace one file atomically: write a hidden sibling with the final mode, then rename it over the target.
/// Why:
///  The rename is the only moment the target changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function replaceFile(root: string, rel: string, content: Uint8Array, mode?: number): void
/// ```
fn replace_file(root: &Path, rel: &str, content: &[u8], planned_mode: Option<u32>) -> Result<(), Diagnostic> {
    ancestors_are_real_directories(root, rel)?;
    ensure_parents(root, rel)?;
    let target = absolute(root, rel);
    let existing_mode = match fs::symlink_metadata(&target) {
        Ok(metadata) if kind_of(&metadata) == EntryKind::File => Some(metadata.permissions().mode() & 0o7777),
        _ => None,
    };
    let final_mode = planned_mode.or(existing_mode).unwrap_or(0o644);
    let name = target.file_name().map(|name| return name.to_string_lossy().to_string()).unwrap_or_default();
    let staging = target.with_file_name(format!(".{name}{STAGING_MARKER}{}", std::process::id()));
    let _ = fs::remove_file(&staging);
    let mut file = match fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&staging) {
        Ok(file) => file,
        Err(error) => return Err(io_failure("create the staging file for", rel, &error)),
    };
    let written = file.write_all(content).and_then(|()| return file.set_permissions(fs::Permissions::from_mode(final_mode)));
    let synced = written.and_then(|()| return file.sync_all());
    if let Err(error) = synced {
        let _ = fs::remove_file(&staging);
        return Err(io_failure("write the staging file for", rel, &error));
    }
    if let Err(error) = fs::rename(&staging, &target) {
        let _ = fs::remove_file(&staging);
        return Err(io_failure("replace", rel, &error));
    }
    return Ok(());
}

/// What:
///  Remove abandoned staging files left beside the paths this run will touch.
/// Why:
///  A run killed between write and rename leaves a hidden file; the next run cleans it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cleanStaging(root: string, drifts: Drift[]): void
/// ```
fn clean_staging(root: &Path, drifts: &[Drift]) {
    let mut directories: Vec<String> = Vec::new();
    for drift in drifts {
        let parent = drift.path.rsplit_once('/').map(|pair| return pair.0.to_string()).unwrap_or_default();
        if !directories.contains(&parent) {
            directories.push(parent);
        }
    }
    for directory in directories {
        let full = if directory.is_empty() { root.to_path_buf() } else { absolute(root, &directory) };
        let Ok(reader) = fs::read_dir(&full) else { continue };
        for entry in reader.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') && name.contains(STAGING_MARKER) {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

/// What:
///  Apply the drift: writes and mode repairs first, then file removals, then directory removals deepest-first.
/// Why:
///  Ordering keeps a directory from being removed before its files, and a replaced file never disappears first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function publish(root: string, report: PlanReport, drifts: Drift[]): PublishOutcome
/// ```
pub fn publish(root: &Path, report: &PlanReport, drifts: &[Drift]) -> Result<PublishOutcome, Diagnostic> {
    clean_staging(root, drifts);
    let mut outcome = PublishOutcome::default();
    for drift in drifts {
        let planned_mode = match report.view.overlay().get(&drift.path) {
            Some(Planned::File { mode, .. }) => *mode,
            _ => None,
        };
        match drift.reason {
            DriftReason::Missing | DriftReason::Content | DriftReason::ReplaceLink => {
                let content = drift.expected.clone().unwrap_or_default();
                replace_file(root, &drift.path, &content, planned_mode)?;
                outcome.written.push(drift.path.clone());
            }
            DriftReason::Mode => {
                ancestors_are_real_directories(root, &drift.path)?;
                let mode = planned_mode.unwrap_or(0o644);
                if let Err(error) = fs::set_permissions(absolute(root, &drift.path), fs::Permissions::from_mode(mode)) {
                    return Err(io_failure("change the mode of", &drift.path, &error));
                }
                outcome.modes.push(drift.path.clone());
            }
            DriftReason::Extra | DriftReason::ExtraDirectory => {}
        }
    }
    for drift in drifts {
        if drift.reason == DriftReason::Extra {
            ancestors_are_real_directories(root, &drift.path)?;
            if let Err(error) = fs::remove_file(absolute(root, &drift.path)) {
                return Err(io_failure("remove", &drift.path, &error));
            }
            outcome.removed.push(drift.path.clone());
        }
    }
    let mut directories: Vec<&Drift> = drifts.iter().filter(|drift| return drift.reason == DriftReason::ExtraDirectory).collect();
    directories.sort_by(|left, right| return right.path.len().cmp(&left.path.len()));
    for drift in directories {
        ancestors_are_real_directories(root, &drift.path)?;
        if let Err(error) = fs::remove_dir(absolute(root, &drift.path)) {
            return Err(io_failure("remove the directory", &drift.path, &error));
        }
        outcome.removed.push(drift.path.clone());
    }
    return Ok(outcome);
}
