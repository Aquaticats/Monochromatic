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

/// The ceiling every read carries,
///  and the watch that ends a read which reaches it.
use crate::child_class::ChildClass;
use crate::child_watch::{StepBound, WatchTerms, output_watched_within};
/// Import the shared child-command builder.
use crate::forwarding::git_command;
use crate::repo_read_hook::try_metadata;
/// The longest a read of a check runs.
use crate::wait_ceilings::PLUMBING_STEP_CEILING;
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
/// The ceiling value and the sentence a failed read is reported with.
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

/// The longest one read-only Git query of the check path runs,
///  whatever it shows,
///  before the
///       wrapper ends it.
///  It is `PLUMBING_STEP_CEILING` (two minutes):
///  the longest read step of the service's
///       landings over 1,651 landings was 9,663 ms,
/// so two minutes is twelve times it,
///  and the longest
///       client preparation of the commit timing log was 250,280 ms,
///  which is the whole of every read
///       inside it;
///  the bound is below `ANSWER_CEILING`,
///  so the person is told what blocked the check
///       before the commit service's own bound would speak.
///  Measured figures:
///  `src/native/service/wait_ceilings.rs`.
///  Owner,
///  2026-11-03:
///  "nothing should ever wait forever or indefinitely".
pub const READ_CEILING: Duration = PLUMBING_STEP_CEILING;

/// What:
///  The name a read is announced and reported under:
///  the Git program and the subcommand it runs,
///  when
///       the arguments name one the wrapper can say without a pathname.
/// Why:
///   The person must be able to tell which read blocked,
///  and a pathname can hold the text a scan
///       exists to keep out of output,
///  so only an alphabetic word before `--` is named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const subject = `${git} ${word}`; // word: /^[a-z][a-z-]*$/
/// ```
fn read_subject(real_git: &Path, arguments: &[OsString]) -> String {
    // `for argument in arguments` visits each in order, so the first word names the read.
    for argument in arguments {
        let bytes: &[u8] = argument.as_encoded_bytes();
        // `--` ends the command line the subcommand can be in; everything after it is a path.
        if bytes == b"--" {
            break;
        }
        if bytes.is_empty() || bytes[0] == b'-' {
            continue;
        }
        let mut named: bool = true;
        for byte in bytes {
            if byte.is_ascii_lowercase() || *byte == b'-' {
                continue;
            }
            named = false;
            break;
        }
        if named {
            return format!("{} {}", real_git.display(), argument.to_string_lossy());
        }
        break;
    }
    return format!("{} (a read-only Git query)", real_git.display());
}

/// What:
///  The sentence a read that reached `READ_CEILING` is reported with,
///  and nothing for any other
///       failure.
/// Why:
///   A read the wrapper ended is not a broken installation:
///  a `core.fsmonitor` program,
///  a stalled
///       mount or a Git waiting on an index showed no end,
///  the wrapper ended it,
///  and the index,
///  the
///       worktree and the staged changes are as they were,
///  so the person runs the command again.
///  Every other
///       failure keeps the wording its caller already has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stopped = error.code === 'ETIMEDOUT' ? `stopped waiting for the Git read of ${fact}: ${error}` : undefined;
/// ```
pub fn read_stopped_text(fact: &str, error: &std::io::Error) -> Option<String> {
    if error.kind() != std::io::ErrorKind::TimedOut {
        return None;
    }
    return Some(format!(
        "monochromatic-git-policy-cli stopped waiting for the Git read of {fact}: {error}"
    ));
}

/// What:
///  Run real Git with captured output and no standard input,
///  under `READ_CEILING`.
///       `std::io::Result<T>` is `Result<T, std::io::Error>`.
/// Why:
///   A query must never consume the caller's stdin (which belongs to the
///       forwarded command) or write to the caller's streams,
///  and it must not outlast its
///       ceiling.
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
    return run_metadata_git_within(real_git, arguments, overlay, READ_CEILING);
}

/// What:
///  `run_metadata_git` with the ceiling the caller chose,
///  which the controls shorten.
/// Why:
///   A control cannot wait two minutes for a real ceiling,
///  and the seam is the ceiling itself,
///  never a race
///       between a write and a read.
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
    let subject: String = read_subject(real_git, arguments);
    // What: The watch captures both output streams through pipes and ends the child when the wait reaches the
    //       ceiling; `.stdin(Stdio::null())` connects the child's input to nothing.
    // Why:  `Command::output()` waits for the child with nothing watching it, so a Git that never returns holds the
    //       check for good and the person's only way out is a kill.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const output = await spawnWatched(command, { waitingFor: subject, ceiling });
    // ```
    let mut command: std::process::Command = git_command(real_git, arguments, overlay);
    // The wrapper reads the repository itself here, with the output captured and the input closed, so nothing of the
    //  person's runs under this start and a wrapper that arrives as its Git came from the file that Git is
    //  (`started_as_git.rs`).
    crate::started_as_git::mark_real_git_start(&mut command, crate::started_as_git::StartMode::Query);
    command.stdin(Stdio::null());
    let terms: WatchTerms = WatchTerms::for_class(ChildClass::Plumbing);
    let quiet: Duration = terms.limit;
    // What: The clock of the child this read starts, kept as the step the commit's timing record lists.
    // Why:  A read the in-process readers decline is a Git process, and a commit that waits seconds on one showed that
    //       wait in its phase and in no step, so the record could not name it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const began = Date.now(); const output = await watched(...); noteStep(label, began, Date.now() - began);
    // ```
    let began: std::time::Instant = std::time::Instant::now();
    let watched: std::io::Result<std::process::Output> = output_watched_within(
        &mut command,
        subject.as_str(),
        subject.as_str(),
        terms,
        true,
        StepBound::EndAt(ceiling),
    );
    // What: A read the watch ended is reported as a read of a check, not as a step of a commit: nothing this command
    //       did can be finished or undone by the next Git command, and the words of a watched step say that it can.
    // Why:  The person must be told what stayed as it was and what to run again.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (error.code === 'ETIMEDOUT') throw new Error(`${subject} showed no sign of work for ${quiet}s, or ran ${ceiling}s, and was ended; nothing changed; run the command again`);
    // ```
    crate::transaction_step_durations::note_step(
        crate::add_timing::git_step_label(arguments).as_str(),
        began,
        began.elapsed(),
    );
    let output: std::process::Output = match watched {
        Ok(captured) => captured,
        Err(error) if error.kind() == std::io::ErrorKind::TimedOut => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                format!(
                    "{subject} showed no sign of work for {} seconds, or ran for the {} seconds a check allows one read, and was ended. The read changed nothing, so the index and the worktree are as they were; run the command again",
                    quiet.as_secs(),
                    ceiling.as_secs()
                ),
            ));
        }
        Err(error) => return Err(error),
    };
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
