//! What:
//! The small file and process helpers every installer shares:
//! a download checked against the lock's SHA-256,
//!           archive extraction,
//! restoring execute bits,
//! an atomic symlink,
//! and the failure message for a program that
//!           exited non-zero.
//! Why:
//! Each helper has one behaviour the installers rely on,
//! and one place to test it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // install-support.ts
//! export function downloadVerified(ctx: Context, url: string, target: string, sha256: string): string
//! export function extractArchive(ctx: Context, archive: string, into: string): void
//! export function restoreExecBits(root: string, names: string[]): string[]
//! export function atomicSymlink(linkPath: string, target: string): void
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `DEFAULT_MAX_DOWNLOAD_BYTES` from `super::http`.
use super::http::DEFAULT_MAX_DOWNLOAD_BYTES;
/// Imports what this module uses from `super::model`.
use super::model::{Artifact, LockedTool};
/// Imports what this module uses from `super::runner`.
use super::runner::{RunOutcome, RunRequest};
/// Imports the shared named helpers.
use super::util::{file_name_os, file_name_text};

/// How many levels below the install directory `restore_exec_bits` looks for the launchers.
const EXEC_SCAN_MAX_DEPTH: usize = 3;

/// The archive file-name endings `tar` or `unzip` can unpack.
pub const ARCHIVE_SUFFIXES: [&str; 6] = [".tar.gz", ".tgz", ".tar.xz", ".tar.bz2", ".tar", ".zip"];

/// What:
/// The artifact row of the host's platform for a tool,
/// or an error naming the tool and the platform.
/// Why:
/// A lock without a row for this platform cannot install the tool here;
/// the message says so rather than
///           failing later on an empty URL.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function artifactForHost(tool: LockedTool, platform: string): Artifact  // throws
/// ```
pub fn artifact_for_host<'a>(tool: &'a LockedTool, platform: &str) -> Result<&'a Artifact, ToolchainError> {
    for (row_platform, artifact) in &tool.artifacts {
        if row_platform == platform {
            return Ok(artifact);
        }
    }
    return Err(ToolchainError::failure(format!(
        "the toolchain lock has no artifact for {} on {platform}; run `mpm-shadow update` on a machine that resolves it, \
         or add the platform to the plan",
        tool.declaration.id
    )));
}

/// What:
/// The error text for a program that failed:
/// what was being done,
/// for which tool,
/// and what the program said.
/// Why:
/// Every installer reports failures the same way,
/// so a person reading the sync report always sees the tool and
///           the tool's own message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function failedStep(what: string, tool: string, outcome: RunOutcome): ToolchainError
/// ```
pub fn failed_step(what: &str, tool_id: &str, outcome: &RunOutcome) -> ToolchainError {
    return ToolchainError::failure(format!("{what} failed for {tool_id}: {}", outcome.failure_text()));
}

/// Runs a request and turns a non-zero exit into a `failed_step` error naming `what` and the tool.
pub fn run_checked(ctx: &Context, request: &RunRequest, what: &str, tool_id: &str) -> Result<RunOutcome, ToolchainError> {
    let outcome: RunOutcome = ctx.run(request);
    if !outcome.succeeded() {
        return Err(failed_step(what, tool_id, &outcome));
    }
    return Ok(outcome);
}

/// What:
/// Downloads `url` to `target`,
/// refusing the file unless its SHA-256 equals `sha256` (the value in the lock).
/// Why:
/// This is the verification step of the bootstrap and of every archive install:
/// the bytes are written beside
///           the target,
/// hashed while they stream in,
/// compared with the lock,
/// and renamed into place only on a match,
///           so an unverified file never sits at the target path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function downloadVerified(ctx: Context, url: string, target: string, sha256: string): string
/// ```
pub fn download_verified(ctx: &Context, url: &str, target: &Path, sha256: &str) -> Result<PathBuf, ToolchainError> {
    let mut temporary_name = file_name_os(target);
    temporary_name.push(".download");
    let temporary: PathBuf = target.with_file_name(temporary_name);
    let observed: String = match ctx.http.download(url, &temporary, DEFAULT_MAX_DOWNLOAD_BYTES) {
        Ok(hash) => hash,
        Err(error) => {
            // Best effort: a partial download is useless, and the error below is what matters.
            let _ = fs::remove_file(&temporary);
            return Err(ToolchainError::failure(format!("download failed: {error}")));
        }
    };
    if observed != sha256 {
        let _ = fs::remove_file(&temporary);
        return Err(ToolchainError::failure(format!(
            "SHA-256 mismatch for {url}: the lock expects {sha256}, the download is {observed}; \
             the file was discarded and nothing was installed from it"
        )));
    }
    if let Err(error) = fs::rename(&temporary, target) {
        return Err(ToolchainError::failure(format!("cannot move the verified download to {}: {error}", target.display())));
    }
    return Ok(target.to_path_buf());
}

/// What:
/// Unpacks an archive into `into` with the system's `unzip` (for `.zip`) or `tar` (everything else;
/// `tar`
///           detects gzip,
/// xz and bzip2 itself).
/// Why:
/// No zip or xz decoder crate is approved,
/// and these tools exist on every machine the toolchain targets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function extractArchive(ctx: Context, archive: string, into: string): void
/// ```
pub fn extract_archive(ctx: &Context, archive: &Path, into: &Path) -> Result<(), ToolchainError> {
    if let Err(error) = fs::create_dir_all(into) {
        return Err(ToolchainError::failure(format!("cannot create {}: {error}", into.display())));
    }
    let archive_text: String = archive.to_string_lossy().to_string();
    let into_text: String = into.to_string_lossy().to_string();
    let request: RunRequest = if archive_text.ends_with(".zip") {
        RunRequest::new("unzip", &["-q", "-o", &archive_text, "-d", &into_text])
    } else {
        RunRequest::new("tar", &["--extract", "--file", &archive_text, "--directory", &into_text])
    };
    let outcome: RunOutcome = ctx.run(&request.with_env(ctx.path_env()));
    if !outcome.succeeded() {
        let program: &str = if archive_text.ends_with(".zip") { "unzip" } else { "tar" };
        return Err(ToolchainError::failure(format!("{program} failed for {archive_text}: {}", outcome.failure_text())));
    }
    return Ok(());
}

/// What:
/// Unpacks a `.conda` package into `into`.
/// A `.conda` file is a zip holding `info-*.tar.zst` and `pkg-*.tar.zst`;
/// the payload is the `pkg-` archive,
/// so `unzip` opens the outer file into a staging directory and `tar --zstd` unpacks the payload.
/// Why:
/// The TypeScript sync copied the `.conda` file as if it were the program,
/// which installed nothing usable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function extractConda(ctx: Context, archive: string, into: string): void
/// ```
pub fn extract_conda(ctx: &Context, archive: &Path, into: &Path) -> Result<(), ToolchainError> {
    let staging: PathBuf = into.join(".conda-staging");
    // Best effort: a staging directory left by a killed run is cleared; absence is the normal case.
    let _ = fs::remove_dir_all(&staging);
    if let Err(error) = fs::create_dir_all(&staging) {
        return Err(ToolchainError::failure(format!("cannot create {}: {error}", staging.display())));
    }
    let archive_text: String = archive.to_string_lossy().to_string();
    let staging_text: String = staging.to_string_lossy().to_string();
    let opened: RunOutcome = ctx.run(&RunRequest::new("unzip", &["-q", "-o", &archive_text, "-d", &staging_text]).with_env(ctx.path_env()));
    if !opened.succeeded() {
        return Err(ToolchainError::failure(format!("unzip failed for {archive_text}: {}", opened.failure_text())));
    }
    let mut names: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(&staging) {
        for entry in entries.flatten() {
            names.push(entry.path());
        }
    }
    names.sort();
    let mut payload: Option<PathBuf> = None;
    for path in names {
        let name: String = file_name_text(&path);
        if name.starts_with("pkg-") && name.ends_with(".tar.zst") {
            payload = Some(path);
            break;
        }
    }
    let Some(payload_path) = payload else {
        return Err(ToolchainError::failure(format!("{archive_text} holds no pkg-*.tar.zst payload")));
    };
    let payload_text: String = payload_path.to_string_lossy().to_string();
    let into_text: String = into.to_string_lossy().to_string();
    let unpacked: RunOutcome = ctx.run(&RunRequest::new("tar", &["--extract", "--zstd", "--file", &payload_text, "--directory", &into_text]).with_env(ctx.path_env()));
    // Best effort: the staging directory only held the outer archive's members.
    let _ = fs::remove_dir_all(&staging);
    if !unpacked.succeeded() {
        return Err(ToolchainError::failure(format!("tar --zstd failed for {payload_text}: {}", unpacked.failure_text())));
    }
    return Ok(());
}

/// Marks a file executable (mode 0755 on Unix;
/// nothing to do elsewhere).
pub fn make_executable(path: &Path) -> Result<(), ToolchainError> {
    #[cfg(unix)]
    {
        /// Imports `PermissionsExt` from `std::os::unix::fs`.
        use std::os::unix::fs::PermissionsExt;
        if let Err(error) = fs::set_permissions(path, fs::Permissions::from_mode(0o755)) {
            return Err(ToolchainError::failure(format!("cannot mark {} executable: {error}", path.display())));
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    return Ok(());
}

/// What:
/// Marks the files named in `names` executable anywhere within `EXEC_SCAN_MAX_DEPTH` levels below `root`;
///           returns the names it fixed.
/// Why:
/// Some archives (zip files above all) unpack launchers without their execute bit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function restoreExecBits(root: string, names: string[]): string[]
/// ```
pub fn restore_exec_bits(root: &Path, names: &[String]) -> Vec<String> {
    let mut fixed: Vec<String> = Vec::new();
    let mut level: Vec<PathBuf> = vec![root.to_path_buf()];
    for depth in 0..=EXEC_SCAN_MAX_DEPTH {
        let mut next: Vec<PathBuf> = Vec::new();
        for directory in &level {
            let Ok(entries) = fs::read_dir(directory) else {
                continue;
            };
            let mut paths: Vec<PathBuf> = Vec::new();
            for entry in entries.flatten() {
                paths.push(entry.path());
            }
            paths.sort();
            for path in paths {
                let name: String = file_name_text(&path);
                if path.is_dir() {
                    if depth < EXEC_SCAN_MAX_DEPTH {
                        next.push(path);
                    }
                } else if path.is_file() && names.contains(&name) && make_executable(&path).is_ok() {
                    fixed.push(name);
                }
            }
        }
        level = next;
    }
    return fixed;
}

/// What:
/// Creates (or replaces) the symlink `link_path` pointing at `target`:
/// the link is made under a temporary
///           name in the same directory and renamed over the final name,
/// so readers see the old link or the new one.
/// Why:
/// The exposure directory is on `PATH` while a sync rewrites it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function atomicSymlink(linkPath: string, target: string): void
/// ```
pub fn atomic_symlink(link_path: &Path, target: &Path) -> Result<(), ToolchainError> {
    if let Some(parent) = link_path.parent()
        && let Err(error) = fs::create_dir_all(parent)
    {
        return Err(ToolchainError::failure(format!("cannot create {}: {error}", parent.display())));
    }
    let mut temporary_name = std::ffi::OsString::from(".");
    temporary_name.push(link_path.file_name().unwrap_or_default());
    temporary_name.push(format!(".{}.tmp", std::process::id()));
    let temporary: PathBuf = link_path.with_file_name(temporary_name);
    // Best effort: a stale temporary link from a killed run is replaced; absence is the normal case.
    let _ = fs::remove_file(&temporary);
    #[cfg(unix)]
    let linked = std::os::unix::fs::symlink(target, &temporary);
    #[cfg(not(unix))]
    let linked = fs::hard_link(target, &temporary);
    if let Err(error) = linked {
        return Err(ToolchainError::failure(format!(
            "cannot link {} to {}: {error}",
            temporary.display(),
            target.display()
        )));
    }
    if let Err(error) = fs::rename(&temporary, link_path) {
        let _ = fs::remove_file(&temporary);
        return Err(ToolchainError::failure(format!("cannot place the link {}: {error}", link_path.display())));
    }
    return Ok(());
}
