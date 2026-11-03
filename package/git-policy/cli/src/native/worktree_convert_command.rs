//! What:
//!  `git cli-git convert-worktree-to-subvolume`: parse the options, find the worktrees to convert, run the conversion on each,
//!  and turn the results into lines and an exit status.
//! Why:
//!  The command runs for about two seconds per worktree, so progress lines go to standard error as they happen. Exit status 0
//!  means every worktree is converted, already a subvolume, previewed, or skipped under `--all`; 1 means a named worktree was
//!  refused before anything changed; 2 means a step failed after the journal began, with the way forward.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const code = await executeConvert({ platform, args, cwd, interrupt, say });
//! ```

use super::action::Action;
use super::subvolume_command::main_checkout_root;
use super::subvolume_error::Interrupt;
use super::subvolume_index_files::linked_worktrees_inside;
use super::subvolume_platform::{LinuxPlatform, Platform};
use super::worktree_convert_arguments::{CONVERT_HELP, ConvertArguments, RISK_NOTICE};
use super::worktree_convert::{ConvertOptions, Verdict, convert_worktree};
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The exit status of a refusal: a condition stood in the way of a named worktree and nothing was changed.
pub const REFUSED_EXIT_CODE: i32 = 1;

/// The exit status of a failure after the journal began.
pub const FAILED_EXIT_CODE: i32 = 2;

/// The worktrees a run covers: every linked worktree inside the main checkout, or the named paths resolved from `cwd`.
fn targets(arguments: &ConvertArguments, cwd: &Path) -> Result<Vec<PathBuf>, String> {
    if arguments.all {
        let root: PathBuf = main_checkout_root(cwd)?;
        return Ok(linked_worktrees_inside(root.as_path()).into_iter().map(|linked| { return linked.worktree; }).collect());
    }
    return Ok(arguments.paths.iter().map(|word: &std::ffi::OsString| { return cwd.join(word); }).collect());
}

/// What:
///  Run the command over its targets and return the exit status; every line goes through `say`.
/// Why:
///   The platform and the interrupt are injected so the controls run the real command on any filesystem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const code = await execute(platform, arguments, cwd, interrupt, say);
/// ```
pub fn execute(platform: &dyn Platform, arguments: &ConvertArguments, cwd: &Path, interrupt: &Interrupt, say: &mut dyn FnMut(&str)) -> i32 {
    let list: Vec<PathBuf> = match targets(arguments, cwd) {
        Ok(found) => found,
        Err(reason) => {
            say(format!("git cli-git convert-worktree-to-subvolume: refused: {reason}").as_str());
            say("Nothing was changed.");
            return REFUSED_EXIT_CODE;
        }
    };
    let options: ConvertOptions = ConvertOptions { dry_run: arguments.dry_run, abandon: arguments.abandon, command_line_attributes: arguments.command_line_attributes };
    let mut refused: usize = 0;
    let mut converted: usize = 0;
    // Said once, before the first worktree is touched, when the run can delete an old tree.
    if !arguments.dry_run && !arguments.abandon && !list.is_empty() {
        say(RISK_NOTICE);
    }
    for path in &list {
        say(format!("{}", path.display()).as_str());
        match convert_worktree(platform, path.as_path(), options, interrupt, &mut *say) {
            Ok(Verdict::Converted) => {
                converted += 1;
                say("  converted into a subvolume");
            }
            Ok(Verdict::Resumed) => {
                converted += 1;
                say(if arguments.dry_run { "  an unfinished conversion would be completed" } else { "  the unfinished conversion was completed" });
            }
            Ok(Verdict::WouldConvert) => say("  would convert"),
            Ok(Verdict::AlreadySubvolume) => say("  already a subvolume, skipped"),
            Ok(Verdict::Abandoned) => say("  the unfinished conversion was undone"),
            Ok(Verdict::Skipped(reasons)) => {
                refused += 1;
                for reason in reasons {
                    say(format!("  skipped: {reason}").as_str());
                }
            }
            Err(error) => {
                say(format!("  failed: {error}").as_str());
                say("  The conversion is journaled. Run the same command again to finish it, or add --abandon to undo it before its swap.");
                return FAILED_EXIT_CODE;
            }
        }
    }
    say(format!("{} worktree(s) looked at, {converted} converted or completed, {refused} skipped.", list.len()).as_str());
    return if refused > 0 && !arguments.all && !arguments.dry_run { REFUSED_EXIT_CODE } else { 0 };
}

/// What:
///  The action of the command: help, a refusal, or the run on the real platform with progress on standard error.
/// Why:
///   The command reads `.git` itself and calls the filesystem, and starts no Git. It still needs to know which Git the wrapper
///  forwards to (`real_git`, resolved by the caller), because where that Git reads its system files from is a fact only that
///  Git reports, and the wrapper's own environment does not name it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function convertAction(arguments: ConvertArguments, cwd: string, environment: [string, string][], realGit?: string): Action;
/// ```
pub fn convert_action(arguments: &ConvertArguments, cwd: &Path, environment: &[(OsString, OsString)], real_git: Option<&Path>) -> Action {
    if arguments.help {
        return Action::Exit { code: 0, stdout: String::from(CONVERT_HELP), stderr: String::new() };
    }
    let mut say = |text: &str| {
        let mut stderr = std::io::stderr().lock();
        let _written = writeln!(stderr, "{text}");
    };
    let code: i32 = execute(&LinuxPlatform::forwarding_to(environment, real_git), arguments, cwd, &Interrupt::never(), &mut say);
    return Action::Exit { code, stdout: String::new(), stderr: String::new() };
}

/// Command controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_convert_command_tests.rs"]
mod tests;
