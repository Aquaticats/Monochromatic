//! What:
//!  Runs a forwarded index writer again when real Git lost the race for the index to the commit service, instead of showing the
//!  person Git's refusal.
//! Why:
//!  The wait for the service's hold ends when the hold is released, but the service may take the index again for the next
//!  landing before the writer's own Git has started (a burst of commits lands one batch after another), and Git then ends with
//!  its refusal to create the index lock. For the few commands that take that lock before they change anything and read no
//!  standard input, running them again once the service lets go is the same command the person typed. The refusal is kept in
//!  a file while this can happen and is shown only when it is final, so a transient loss is never printed. Every other
//!  command, and a lock that a foreign program holds, keep the behavior of native Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (let attempt = 0; attempt < ATTEMPTS; attempt++) { const run = await runCapturingStderr(git); if (!lostToService(run)) return show(run); await waitForServiceHold(); }
//! ```

use super::forwarding::{exit_code, git_command, outcome_of, run_real_git};
use super::scratch_location::scratch_file;
use super::service_index_wait::{foreign_holds, wait_for_service_hold_within};
use super::wait_ceilings::SERVICE_HOLD_CEILING;
use super::wait_progress::Cadence;
use super::start_retry::RetryStart;
use super::worktree_copy_forward::ForwardedRun;
use std::ffi::OsString;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::process::Stdio;

/// How many times the command is started at most while the service keeps taking the index first.
pub const ATTEMPTS: usize = 50;

/// What Git's refusal to create the index lock always names.
const LOCK_REFUSAL: &str = "index.lock";

/// The status Git dies with, which is also the status of its refusal to create a lock.
const DIE_STATUS: i32 = 128;

/// Options of `git add` that change nothing about its input and read no standard input.
const ADD_OPTIONS: [&str; 20] = [
    "-v", "-n", "-f", "-A", "-u", "-N", "-q", "--verbose", "--dry-run", "--force", "--all", "--update", "--intent-to-add",
    "--ignore-errors", "--refresh", "--renormalize", "--no-all", "--ignore-removal", "--sparse", "--no-warn-embedded-repo",
];

/// Options of `git rm` that read no standard input.
const REMOVE_OPTIONS: [&str; 10] = ["-f", "-n", "-q", "-r", "--cached", "--force", "--dry-run", "--quiet", "--ignore-unmatch", "--sparse"];

/// Options of `git mv` that read no standard input.
const MOVE_OPTIONS: [&str; 8] = ["-f", "-k", "-n", "-v", "--force", "--dry-run", "--verbose", "--sparse"];

/// What:
///  Whether the command is one that may be run again after it lost the index: it takes the index lock before it changes
///  anything, reads no standard input, asks the person nothing, and every option it has is one this list knows.
/// Why:
///   An unknown option, an interactive form or a pathspec file could make a second run differ from the first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mayRunAgain('add', ['-A', '--', 'a.txt'])
/// ```
pub fn may_run_again(word: &[u8], after_word: &[OsString]) -> bool {
    let known: &[&str] = match word {
        b"add" => &ADD_OPTIONS,
        b"rm" => &REMOVE_OPTIONS,
        b"mv" => &MOVE_OPTIONS,
        _ => return false,
    };
    for argument in after_word {
        let Some(text) = argument.to_str() else { return false };
        if text == "--" {
            break;
        }
        if text.starts_with('-') && text != "-" && !known.contains(&text) {
            return false;
        }
    }
    return true;
}

/// Starts Git once with its standard error in `captured`, and returns its exit status.
fn run_once(real_git: &Path, arguments: &[OsString], overlay: &[(OsString, OsString)], captured: std::fs::File) -> std::io::Result<i32> {
    let mut command = git_command(real_git, arguments, overlay);
    command.stderr(Stdio::from(captured));
    let status = command.status_retrying()?;
    return Ok(exit_code(outcome_of(status)));
}

/// The command with every stream inherited, as a forwarded command runs when nothing needs to be kept.
fn run_plainly(real_git: &Path, arguments: &[OsString], overlay: &[(OsString, OsString)]) -> ForwardedRun {
    return match run_real_git(real_git, arguments, overlay) {
        Ok(outcome) => ForwardedRun::Ran(exit_code(outcome)),
        Err(error) => ForwardedRun::Failed { stderr: format!("cli-git: cannot start {}: {error}\n", real_git.display()), code: super::action::ENGINE_FAILURE_EXIT_CODE },
    };
}

/// What:
///  Runs the command, again while it ends with Git's refusal to create the index lock and the commit service has the index
///  (or has just let it go), and shows the last attempt's standard error.
/// Why:
///   The final attempt's words are Git's own; an attempt that is run again shows nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runAgainWhileServiceHoldsIndex({ realGit, arguments, overlay, environment, realIndex, name })
/// ```
pub fn run_again_while_service_holds(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
    environment: &[(OsString, OsString)],
    real_index: &Path,
    name: &str,
    shown: &mut dyn Write,
) -> ForwardedRun {
    return run_again_within(real_git, arguments, overlay, environment, real_index, name, SERVICE_HOLD_CEILING, shown);
}

/// `run_again_while_service_holds` whose whole series of reruns and waits ends after `ceiling`, whatever the count: the last
///  attempt's words are shown and its status is the command's.
pub fn run_again_within(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
    environment: &[(OsString, OsString)],
    real_index: &Path,
    name: &str,
    ceiling: std::time::Duration,
    shown: &mut dyn Write,
) -> ForwardedRun {
    let began: std::time::Instant = std::time::Instant::now();
    for attempt in 1..=ATTEMPTS {
        // Without a place to keep the output the command runs as it always did, and its refusal, if any, is Git's own.
        let Ok(mut file) = scratch_file(environment) else { return run_plainly(real_git, arguments, overlay) };
        let Ok(for_child) = file.try_clone() else { return run_plainly(real_git, arguments, overlay) };
        let code: i32 = match run_once(real_git, arguments, overlay, for_child) {
            Ok(code) => code,
            Err(error) => {
                return ForwardedRun::Failed { stderr: format!("cli-git: cannot start {}: {error}\n", real_git.display()), code: super::action::ENGINE_FAILURE_EXIT_CODE };
            }
        };
        let mut said: Vec<u8> = Vec::new();
        let read_back = file.seek(SeekFrom::Start(0)).and_then(|_| { return file.read_to_end(&mut said); });
        let lost_to_service: bool = code == DIE_STATUS
            && read_back.is_ok()
            && String::from_utf8_lossy(said.as_slice()).contains(LOCK_REFUSAL)
            && !foreign_holds(real_index);
        if lost_to_service && attempt < ATTEMPTS && began.elapsed() < ceiling {
            wait_for_service_hold_within(real_index, name, ceiling.saturating_sub(began.elapsed()), Cadence::from_environment(), &mut std::io::stderr());
            continue;
        }
        // A closed terminal is not a reason to stop the command the person asked for.
        let _shown = shown.write_all(said.as_slice()).and_then(|()| { return shown.flush(); });
        return ForwardedRun::Ran(code);
    }
    return ForwardedRun::Ran(DIE_STATUS);
}

/// The retry is controlled in the test module.
#[cfg(test)]
#[path = "service_hold_retry_tests.rs"]
mod tests;
