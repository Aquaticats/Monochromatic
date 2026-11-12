//! What:
//!  Run the person's `git push` so that the URLs Git prints reach the caller with their secrets masked.
//! Why:
//!  Git names the destination it could not reach on its standard error and names it again in the summary it prints when
//! the refs move ("To <url>"),
//! and a destination can carry a token in its userinfo,
//! in a path segment or in its query.
//! Git removes the userinfo of what it prints itself,
//! and leaves the path and the query as they were typed,
//! so a push whose streams are the caller's printed those secrets
//! (audit `alpha-12-lfs`, finding "A typed push relays URL path and query secrets without the wrapper's mask").
//!  Both streams are therefore read by this module and relayed through the mask of `lfs_output_mask`,
//! a word at a time,
//! so a URL the kernel split between two reads is still one URL when it is masked
//! (`UrlMaskingStream`).
//! What a person still sees:
//! every line as it is written,
//! the server and the port of every URL,
//! the shape of its path and query,
//! and the summary of which refs moved.
//! What is lost:
//! a progress meter,
//! because Git draws one only when its standard error is a terminal and this face gives it a pipe;
//! the relay asks Git for the meter with `--progress` when the caller's standard error is a terminal,
//! so the person sees what native Git would have shown,
//! and a command that asked for `--quiet` keeps its quiet.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const child = spawn(git, ['push', '--progress', ...args], { stdio: ['inherit', 'pipe', 'pipe'] }); pipe(child.stdout, process.stdout, mask); pipe(child.stderr, process.stderr, mask); process.exitCode = code(child)
//! ```

/// How a forwarded run ended,
/// and the real Git it runs.
use crate::worktree_copy_forward::ForwardedRun;
/// The rule that hides the secrets of a URL,
/// and the state a stream read in chunks needs to apply it.
use crate::lfs_output_mask::UrlMaskingStream;
/// The real Git of the person's command,
/// and the variables the wrapper adds.
use crate::forwarding::git_command;
use crate::hooks_off::StartOwner;
/// Starts that ride out a short shortage of processes.
use crate::start_retry::RetryStart;
/// Owned OS strings for the arguments.
use std::ffi::OsString;
/// The real Git.
use std::path::Path;
/// Reading and writing one stream,
/// and waiting for the child.
use std::io::{IsTerminal, Read, Write};
use std::process::Stdio;

/// What the argument list gains when Git is asked for its progress meter.
const PROGRESS_OPTION: &str = "--progress";

/// The spellings that ask Git to print nothing it does not have to,
/// which a metred run would undo.
const QUIET_OPTIONS: [&str; 2] = ["--quiet", "-q"];

/// The most bytes one read of a stream takes.
const READ_BUFFER: usize = 8192;

/// What:
///  The argument list with `--progress` after the subcommand,
/// when the caller's standard error is a terminal and the person asked for neither the meter nor silence.
/// Why:
///   This face reads Git's standard error through a pipe,
/// and Git draws its progress meter only when that stream is a terminal,
/// so the meter native Git would have shown is lost unless Git is asked for it.
/// A command that asked for `--quiet` keeps what it asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withProgress(args: string[], subcommand: number, stderrIsTerminal: boolean): string[]
/// ```
fn with_progress(arguments: &[OsString], subcommand: Option<usize>, stderr_is_terminal: bool) -> Vec<OsString> {
    let mut words: Vec<OsString> = arguments.to_vec();
    let Some(at) = subcommand else {
        return words;
    };
    if !stderr_is_terminal {
        return words;
    }
    let mut asked: bool = false;
    for word in &words {
        let text: String = word.to_string_lossy().into_owned();
        if text == PROGRESS_OPTION || QUIET_OPTIONS.contains(&text.as_str()) {
            asked = true;
        }
    }
    if asked {
        return words;
    }
    let position: usize = (at + 1).min(words.len());
    words.insert(position, OsString::from(PROGRESS_OPTION));
    return words;
}

/// What:
///  Read `reader` to its end and write it to `writer` with every URL's secrets masked.
/// Why:
///   Each read is masked with the bytes the one before held back,
/// so a URL split between two reads is classified as one URL (`UrlMaskingStream`).
/// A stream that ends inside the last word hides it whole,
/// because that word can be a user and a token whose `@` never came.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function relayMasked(reader: Readable, writer: Writable): void
/// ```
pub fn relay_masked(reader: &mut impl Read, writer: &mut impl Write) {
    let mut masking: UrlMaskingStream = UrlMaskingStream::new();
    let mut buffer: [u8; READ_BUFFER] = [0; READ_BUFFER];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => {
                let shown: Vec<u8> = masking.feed(&buffer[..read]);
                if !shown.is_empty() {
                    // Terminal output error: the exit status carries the push's result, and a closed stream has no reader left.
                    let _written = writer.write_all(shown.as_slice());
                    let _flushed = writer.flush();
                }
            }
            // A read that was interrupted by a signal asks for another; any other failure ends the relay.
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }
    let tail: Vec<u8> = masking.finish();
    if !tail.is_empty() {
        // Terminal output error: see above.
        let _written = writer.write_all(tail.as_slice());
        let _flushed = writer.flush();
    }
}

/// What:
///  Run the person's push with its two streams relayed through the mask,
/// and report how Git ended.
/// Why:
///   A push is the forwarded command whose own output names a destination,
/// and the destination can carry a token,
/// so neither stream is handed to the caller as Git wrote it.
///  `out` and `err` are the caller's own streams; a control hands in buffers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runMaskedPush({ realGit, arguments, overlay, subcommand, out, err }): ForwardedRun
/// ```
pub fn run_masked_push(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
    subcommand: Option<usize>,
    out: &mut (impl Write + Send),
    err: &mut (impl Write + Send),
) -> ForwardedRun {
    let words: Vec<OsString> = with_progress(arguments, subcommand, std::io::stderr().is_terminal());
    let mut command: std::process::Command = git_command(real_git, words.as_slice(), overlay, StartOwner::Person);
    command.stdin(Stdio::inherit()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child: std::process::Child = match command.spawn_retrying() {
        Ok(started) => started,
        Err(error) => {
            return ForwardedRun::Failed {
                stderr: format!("monochromatic-git-policy-cli: cannot start {}: {error}\n", real_git.display()),
                code: crate::action::ENGINE_FAILURE_EXIT_CODE,
            };
        }
    };
    let streams: Option<(std::process::ChildStdout, std::process::ChildStderr)> = child.stdout.take().zip(child.stderr.take());
    let Some((mut git_out, mut git_err)) = streams else {
        // A pipe that was not there is this process's own defect, so the child is ended rather than left without a reader.
        let _killed = child.kill();
        let _reaped = child.wait();
        return ForwardedRun::Failed { stderr: String::from("monochromatic-git-policy-cli: the push's streams could not be read\n"), code: crate::action::ENGINE_FAILURE_EXIT_CODE };
    };
    // One relay per stream, so the refs Git reports on standard output keep that stream and the person's shell still
    //  reads them where they expect them.
    std::thread::scope(|scope| {
        scope.spawn(|| { return relay_masked(&mut git_out, out); });
        relay_masked(&mut git_err, err);
    });
    return match child.wait() {
        Ok(status) => match crate::forwarding::outcome_of(status) {
            crate::forwarding::ChildOutcome::Exited(code) => ForwardedRun::Ran(code),
            crate::forwarding::ChildOutcome::Signaled(signal) => ForwardedRun::Killed(signal),
        },
        Err(_) => ForwardedRun::Ran(crate::action::ENGINE_FAILURE_EXIT_CODE),
    };
}

/// The relay is controlled in the test module.
#[cfg(test)]
#[path = "push_output_relay_tests.rs"]
mod tests;
