//! What:     The small file and process helpers every installer shares: a download checked against the lock's SHA-256,
//!           archive extraction, restoring execute bits, an atomic symlink, and the failure message for a program that
//!           exited non-zero.
//! Why:      Each helper has one behaviour the installers rely on, and one place to test it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // install-support.ts
//! export function downloadVerified(ctx: Context, url: string, target: string, sha256: string): string
//! export function extractArchive(ctx: Context, archive: string, into: string): void
//! export function restoreExecBits(root: string, names: string[]): string[]
//! export function atomicSymlink(linkPath: string, target: string): void
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use super::context::Context;
use super::error::ToolchainError;
use super::http::DEFAULT_MAX_DOWNLOAD_BYTES;
use super::model::{Artifact, LockedTool};
use super::runner::{RunOutcome, RunRequest};

/// How many levels below the install directory `restore_exec_bits` looks for the launchers.
const EXEC_SCAN_MAX_DEPTH: usize = 3;

/// The archive file-name endings `tar` or `unzip` can unpack.
pub const ARCHIVE_SUFFIXES: [&str; 6] = [".tar.gz", ".tgz", ".tar.xz", ".tar.bz2", ".tar", ".zip"];

/// What:     The artifact row of the host's platform for a tool, or an error naming the tool and the platform.
/// Why:      A lock without a row for this platform cannot install the tool here; the message says so rather than
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

/// What:     The error text for a program that failed: what was being done, for which tool, and what the program said.
/// Why:      Every installer reports failures the same way, so a person reading the sync report always sees the tool and
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

/// What:     Downloads `url` to `target`, refusing the file unless its SHA-256 equals `sha256` (the value in the lock).
/// Why:      This is the verification step of the bootstrap and of every archive install: the bytes are written beside
///           the target, hashed while they stream in, compared with the lock, and renamed into place only on a match,
///           so an unverified file never sits at the target path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function downloadVerified(ctx: Context, url: string, target: string, sha256: string): string
/// ```
pub fn download_verified(ctx: &Context, url: &str, target: &Path, sha256: &str) -> Result<PathBuf, ToolchainError> {
    let mut temporary_name = target.file_name().map(|name| return name.to_os_string()).unwrap_or_default();
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

/// What:     Unpacks an archive into `into` with the system's `unzip` (for `.zip`) or `tar` (everything else; `tar`
///           detects gzip, xz and bzip2 itself).
/// Why:      No zip or xz decoder crate is approved, and these tools exist on every machine the toolchain targets.
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

/// Marks a file executable (mode 0755 on Unix; nothing to do elsewhere).
pub fn make_executable(path: &Path) -> Result<(), ToolchainError> {
    #[cfg(unix)]
    {
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

/// What:     Marks the files named in `names` executable anywhere within `EXEC_SCAN_MAX_DEPTH` levels below `root`;
///           returns the names it fixed.
/// Why:      Some archives (zip files above all) unpack launchers without their execute bit.
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
            let mut paths: Vec<PathBuf> = entries.flatten().map(|entry| return entry.path()).collect();
            paths.sort();
            for path in paths {
                let name: String = path.file_name().map(|file| return file.to_string_lossy().to_string()).unwrap_or_default();
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

/// What:     Creates (or replaces) the symlink `link_path` pointing at `target`: the link is made under a temporary
///           name in the same directory and renamed over the final name, so readers see the old link or the new one.
/// Why:      The exposure directory is on `PATH` while a sync rewrites it.
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
