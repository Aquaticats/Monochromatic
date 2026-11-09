//! What:
//!  Run a read-only real-Git query and capture its output as raw bytes.
//! Why:
//!  Repository facts (top level,
//!  Git directory) come from Git itself,
//!  and the
//!      paths it prints need not be UTF-8.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const { stdout } = await spawn(gitPath, ['rev-parse', '--git-dir'], { stdout: 'pipe' });
//! ```

/// Import the shared child-command builder.
use crate::forwarding::git_command;
use crate::repo_read_hook::try_metadata;
/// What:
///  `OsString` is owned operating-system text of raw OS bytes (sibling `String`
///       must be UTF-8).
/// Why:
///   Query arguments include the caller's unchanged global options.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string[] argv, but byte-preserving.
/// ```
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw OS bytes.
use std::path::{Path, PathBuf};
/// `Stdio` selects where a child's stream is connected.
use std::process::Stdio;
/// How long a query may run.
use std::time::Duration;

/// What:
///  What one metadata query produced.
///       `Vec<u8>` is an owned byte list (`u8` is one byte);
///  sibling `String` would
///       require UTF-8.
/// Why:
///   Git prints paths as raw bytes,
///  and a failed query is an ordinary answer
///       ("not a repository"),
///  not an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MetadataOutput = { success: boolean; code: number | null; stdout: Buffer; stderr: Buffer };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataOutput {
    /// Whether Git exited with status zero.
    pub success: bool,
    /// Git's exit code,
    ///  or nothing when a signal ended it.
    ///  Some queries answer through
    /// their exit code alone (`git diff-index --quiet` exits 1 for "differs").
    pub code: Option<i32>,
    /// Exact bytes Git wrote to standard output,
    ///  including what it printed before failing.
    pub stdout: Vec<u8>,
    /// Exact bytes Git wrote to standard error.
    pub stderr: Vec<u8>,
}

/// What:
///  Run real Git with captured output and no standard input.
///       `std::io::Result<T>` is `Result<T, std::io::Error>`.
/// Why:
///   A query must never consume the caller's stdin (which belongs to the
///       forwarded command) or write to the caller's streams.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runMetadataGit(realGit, args, overlay): Promise<MetadataOutput>;
/// ```
pub fn run_metadata_git(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
) -> std::io::Result<MetadataOutput> {
    return run_metadata_git_within(real_git, arguments, overlay, crate::wait_ceilings::PLUMBING_STEP_CEILING);
}

/// What:
///  `run_metadata_git` where the query's own ceiling may be named.
/// Why:
///   A query that never answers has nothing else to end it
/// (`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
/// finding 5),
///  and a control needs a ceiling short enough to reach.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runMetadataGitWithin(realGit, args, overlay, ceiling): Promise<MetadataOutput>;
/// ```
pub fn run_metadata_git_within(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
    ceiling: Duration,
) -> std::io::Result<MetadataOutput> {
    // An exact read-only command line the in-process readers know costs no Git process.
    if let Some(answered) = try_metadata(arguments, crate::git_system_config::with_real_git(overlay, real_git).as_slice()) {
        return Ok(answered);
    }
    // What: `.stdin(Stdio::null())` connects the child's input to nothing, and the captured run takes its output into files
    //       and waits for it under the ceiling. A trailing `?` returns a start failure, or a query that did not answer.
    // Why:  A query that never answers has no watch and no meter, so its ceiling is the only end it has.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const result = await execWithDeadline(realGit, args, { stdin: 'ignore' }, ceiling);
    // ```
    let mut command: std::process::Command = git_command(real_git, arguments, overlay);
    command.stdin(Stdio::null());
    let output: std::process::Output = crate::captured_child_run_to_its_end::output_of_within_bound(&mut command, ceiling)?;
    // `Ok(...)` is the success variant carrying the captured result.
    return Ok(MetadataOutput {
        success: output.status.success(),
        code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    });
}

/// What:
///  Remove exactly one final line break (`\n` or `\r\n`) from Git output.
///       `&[u8]` borrows bytes;
///  the result borrows a shorter view of the same bytes.
/// Why:
///   Git terminates each printed value with one line break;
///  anything before it,
///       including other line breaks,
///  is part of the value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stripGitLine(output: Buffer): Buffer;
/// ```
pub fn strip_git_line(output: &[u8]) -> &[u8] {
    // What: `.strip_suffix(..)` is `Some(shorter)` when the bytes end with the suffix.
    // Why:  The two-byte ending is tried first so its `\r` is not left behind.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (output.endsWith('\r\n')) return output.slice(0, -2);
    // ```
    if let Some(stripped) = output.strip_suffix(b"\r\n") {
        return stripped;
    }
    if let Some(stripped) = output.strip_suffix(b"\n") {
        return stripped;
    }
    return output;
}

/// What:
///  Turn path bytes printed by Git into a path,
///  without decoding on Unix.
///       `Option<PathBuf>` is "an owned path or nothing".
///  `#[cfg(unix)]` and
///       `#[cfg(not(unix))]` each compile one of the two inner blocks.
/// Why:
///   Unix paths are arbitrary bytes and must round-trip exactly;
///  other systems can
///       only build a path from valid UTF-8.
///  An empty value is never a path anywhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathFromGitBytes(bytes: Buffer): string | undefined;
/// ```
pub fn path_from_git_bytes(bytes: &[u8]) -> Option<PathBuf> {
    if bytes.is_empty() {
        // `None` is the "absent" variant.
        return None;
    }
    #[cfg(unix)]
    {
        // The trait adds `from_vec`, which wraps raw bytes as OS text unchanged.
        use std::os::unix::ffi::OsStringExt;
        // `.to_vec()` copies the borrowed bytes into an owned list.
        return Some(PathBuf::from(OsString::from_vec(bytes.to_vec())));
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        // `match` unpacks the decoding: text becomes a path, other bytes are not one.
        match std::str::from_utf8(bytes) {
            Ok(text) => return Some(PathBuf::from(text)),
            Err(_) => return None,
        }
    }
}

/// Byte-level controls stay out of the release executable.
#[cfg(test)]
#[path = "git_metadata_tests.rs"]
mod tests;
