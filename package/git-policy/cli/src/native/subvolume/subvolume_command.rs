//! What:
//!  `git monochromatic-git-policy-cli migrate-to-subvolume`:
//! find the repository's main checkout from the working directory,
//! run the
//!  requested mode (migrate,
//! rollback,
//! remove the old tree) and turn the result into an exit status and text.
//! Why:
//!  The command runs for minutes (the real checkout took about sixteen minutes of passes),
//! so progress lines go to
//!  standard error as they happen;
//! refusals and failures end with the cause and every way forward.
//! Exit status 0 means
//!  done,
//! already done or previewed,
//! 1 means a condition refused it before anything changed,
//! and 2 means a step
//!  failed after the journal began,
//! with the instruction to run the command again or to roll back.
//! The main checkout is
//!  found by reading `.git` in process (a directory,
//! or a gitfile naming the common directory),
//! never by asking Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const code = await execute({ platform, args, root, interrupt, say });
//! ```

use crate::action::{Action, ENGINE_FAILURE_EXIT_CODE};
use crate::subvolume_arguments::{MIGRATE_HELP, MigrateArguments, MigrateMode};
use crate::single_owner_claim::{ClaimOutcome, OwnerClaim, take_claim};
use crate::subvolume_error::{Interrupt, MigrateError};
use crate::subvolume_migrate::{MigrateOptions, Verdict, migrate};
use crate::subvolume_platform::{LinuxPlatform, Platform};
use crate::subvolume_preflight::{Refusal, layout_of};
use crate::subvolume_rollback::{refusals_of_the_mode, remove_old_tree, rollback};
use crate::subvolume_run::Run;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The exit status of a refusal:
/// a condition stood in the way and nothing was changed.
pub const REFUSED_EXIT_CODE: i32 = 1;

/// What:
///  The main checkout of the repository that contains `start`,
/// found without starting Git.
/// Why:
///   Walk up to the first `.git`.
/// A directory there marks the main checkout.
/// A file there is a linked worktree's
///  gitfile (`gitdir: <admin directory>`);
/// the administrative directory's `commondir` names the common directory,
///  whose parent is the main checkout when the common directory is called `.git`.
/// Anything else (a bare repository,
/// a
///  submodule,
/// an unreadable gitfile) is reported,
/// because converting the wrong directory would be harmful.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mainCheckoutRoot(start: string): string; // throws with the reason
/// ```
pub fn main_checkout_root(start: &Path) -> Result<PathBuf, String> {
    let real: PathBuf = match std::fs::canonicalize(start) {
        Ok(resolved) => resolved,
        Err(error) => return Err(format!("cannot resolve {}: {error}", start.display())),
    };
    let mut directory: PathBuf = real.clone();
    loop {
        let dot_git: PathBuf = directory.join(".git");
        if let Ok(metadata) = std::fs::symlink_metadata(&dot_git) {
            if metadata.is_dir() {
                return Ok(directory);
            }
            return common_root_of_gitfile(dot_git.as_path());
        }
        let Some(parent) = directory.parent() else {
            return Err(format!("{} is not inside a Git working tree (no .git above it)", real.display()));
        };
        directory = parent.to_path_buf();
    }
}

/// Read a linked worktree's gitfile and return the main checkout it belongs to.
fn common_root_of_gitfile(gitfile: &Path) -> Result<PathBuf, String> {
    // Both reads go through the plain-file reader, which opens without waiting and never follows a link: a named pipe at
    // either name must end discovery with a reason instead of waiting for a writer that never comes.
    let text: String = match crate::plain_file::read_plain_text(gitfile) {
        Ok(read) => read,
        Err(error) => return Err(format!("cannot read {}: {error}", gitfile.display())),
    };
    let Some(named) = text.trim().strip_prefix("gitdir:") else {
        return Err(format!("{} is not a gitfile (no `gitdir:` line)", gitfile.display()));
    };
    let base: &Path = gitfile.parent().unwrap_or(Path::new("/"));
    let admin: PathBuf = base.join(named.trim());
    let common_text: String = match crate::plain_file::read_plain_text(admin.join("commondir").as_path()) {
        Ok(read) => read,
        Err(error) => {
            return Err(format!(
                "{} is not a linked worktree's gitfile ({} has no commondir: {error}), so this is not a main checkout or a linked worktree of one",
                gitfile.display(),
                admin.display()
            ));
        }
    };
    let common: PathBuf = match std::fs::canonicalize(admin.join(common_text.trim())) {
        Ok(resolved) => resolved,
        Err(error) => return Err(format!("cannot resolve the common directory: {error}")),
    };
    if common.file_name() != Some(std::ffi::OsStr::new(".git")) {
        return Err(format!("the common directory {} is not named .git, so the repository has no main checkout directory to convert", common.display()));
    }
    return match common.parent() {
        Some(parent) => Ok(parent.to_path_buf()),
        None => Err(String::from("the common directory has no parent")),
    };
}

/// Write one line to standard error;
/// a failed write is ignored,
/// as the command's report must not fail on a closed terminal.
fn say_on_standard_error(text: &str) {
    let mut stderr = std::io::stderr().lock();
    let _written = writeln!(stderr, "{text}");
}

/// Write the refusals in plain words.
fn say_refusals(say: &mut dyn FnMut(&str), refusals: &[Refusal]) {
    for refusal in refusals {
        say(format!("git monochromatic-git-policy-cli migrate-to-subvolume: refused: {}", refusal.cause).as_str());
        say(format!("  What to do: {}", refusal.remedy).as_str());
    }
    say("Nothing was changed.");
}

/// What:
///  Run the requested mode against a root and return the exit status.
///  `say` receives every line;
/// the platform and interrupt are injected so controls run it on any filesystem.
/// Why:
///   One function holds the status mapping,
/// so the controls check the real command's behavior.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const code = await execute(platform, args, root, interrupt, say);
/// ```
pub fn execute(platform: &dyn Platform, arguments: &MigrateArguments, root: &Path, interrupt: &Interrupt, say: &mut dyn FnMut(&str)) -> i32 {
    // The platform's answer holds for the whole run, and a run on a platform that flushes turns the flushes back on even
    // inside a stand-in's extent, so a real-platform run nested in a control's simulated one keeps its durability.
    let _flushes: crate::subvolume_file_write_steps::WritesKeptInMemory =
        crate::subvolume_file_write_steps::WritesKeptInMemory::for_flush_policy(platform.flushes_to_disk());
    return run_to_a_status(platform, arguments, root, interrupt, say);
}

/// The status mapping of one run,
/// which `execute` wraps in the platform's answer about flushing.
fn run_to_a_status(platform: &dyn Platform, arguments: &MigrateArguments, root: &Path, interrupt: &Interrupt, say: &mut dyn FnMut(&str)) -> i32 {
    let outcome: Result<i32, MigrateError> = run_mode(platform, arguments, root, interrupt, &mut *say);
    match outcome {
        Ok(code) => return code,
        Err(error) => {
            let journal: String = match layout_of(root) {
                Some(layout) => layout.journal_path.display().to_string(),
                None => String::new(),
            };
            say(format!("git monochromatic-git-policy-cli migrate-to-subvolume: failed: {error}").as_str());
            say(format!("The journal {journal} records the steps that finished. Run the command again to resume from there, or add --rollback to restore the original directory.").as_str());
            return ENGINE_FAILURE_EXIT_CODE;
        }
    }
}

/// The three modes,
/// as a result so `execute` maps every error the same way.
fn run_mode(platform: &dyn Platform, arguments: &MigrateArguments, root: &Path, interrupt: &Interrupt, say: &mut dyn FnMut(&str)) -> Result<i32, MigrateError> {
    let Some(layout) = layout_of(root) else {
        return Err(MigrateError::Journal(format!("{} has no parent directory", root.display())));
    };
    // One command at a time works on a checkout, whatever its mode: a preview changes nothing and needs no claim. The claim
    // is taken before the journal is read and given back when this function returns, however it returns.
    let _claim: Option<OwnerClaim> = if arguments.dry_run {
        None
    } else {
        match take_claim(layout.parent.as_path(), layout.claim_prefix.as_os_str())? {
            ClaimOutcome::Held(held) => Some(held),
            ClaimOutcome::Busy(owner) => {
                let busy: Refusal = Refusal {
                    cause: format!("another command (process {}) is working on {} right now (it owns {})", owner.pid, root.display(), owner.path.display()),
                    remedy: String::from("Wait for it to finish, or stop it, then run this command again; a command that was killed owns nothing, so a leftover claim file needs no cleanup."),
                };
                say_refusals(say, std::slice::from_ref(&busy));
                return Ok(REFUSED_EXIT_CODE);
            }
        }
    };
    if arguments.mode == MigrateMode::Migrate {
        let options: MigrateOptions = MigrateOptions { dry_run: arguments.dry_run, nested_worktrees: arguments.nested_worktrees, refresh_indexes: arguments.refresh_indexes };
        let verdict: Verdict = migrate(platform, root, options, interrupt, &mut *say)?;
        if let Verdict::Refused(refusals) = verdict {
            say_refusals(say, refusals.as_slice());
            return Ok(REFUSED_EXIT_CODE);
        }
        return Ok(0);
    }
    let mut run: Run<'_> = Run::open(platform, &layout, interrupt, say)?;
    if arguments.dry_run {
        // A preview runs the refusals of the mode it previews, in the words the real run uses, so it never promises a
        // step the real run refuses; it changes nothing either way.
        let refusals: Vec<Refusal> = refusals_of_the_mode(&run, arguments.mode, arguments.discard_unreadable);
        if !refusals.is_empty() {
            say_refusals(run.say, refusals.as_slice());
            return Ok(REFUSED_EXIT_CODE);
        }
        let what: &str = if arguments.mode == MigrateMode::Rollback { "restore the original directory" } else { "delete the original tree" };
        run.line(format!("dry run: the command would {what} of {}; nothing was changed.", layout.root.display()).as_str());
        return Ok(0);
    }
    let refused: Option<Vec<Refusal>> = if arguments.mode == MigrateMode::Rollback {
        rollback(&mut run, arguments.discard_unreadable, arguments.discard_left_out)?
    } else {
        remove_old_tree(&mut run, arguments.discard_unreadable, arguments.discard_left_out)?
    };
    if let Some(refusals) = refused {
        say_refusals(run.say, refusals.as_slice());
        return Ok(REFUSED_EXIT_CODE);
    }
    return Ok(0);
}

/// What:
///  The action of `git monochromatic-git-policy-cli migrate-to-subvolume`:
/// help,
/// or the run from the working directory's repository.
/// Why:
///   Progress goes to standard error live;
/// the action carries only the exit status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planMigrate(args: MigrateArguments, cwd: string): Action;
/// ```
pub fn migrate_action(arguments: &MigrateArguments, cwd: &Path) -> Action {
    if arguments.help {
        return Action::Exit { code: 0, stdout: String::from(MIGRATE_HELP), stderr: String::new() };
    }
    let root: PathBuf = match main_checkout_root(cwd) {
        Ok(found) => found,
        Err(message) => {
            return Action::Exit {
                code: REFUSED_EXIT_CODE,
                stdout: String::new(),
                stderr: format!("git monochromatic-git-policy-cli migrate-to-subvolume: refused: {message}\n  What to do: run the command inside the repository whose main checkout you want to convert.\nNothing was changed.\n"),
            };
        }
    };
    let code: i32 = execute(&LinuxPlatform::default(), arguments, root.as_path(), &Interrupt::never(), &mut say_on_standard_error);
    return Action::Exit { code, stdout: String::new(), stderr: String::new() };
}

/// Command controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_command_tests.rs"]
mod tests;

/// The controls for who owns a migration stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_command_ownership_tests.rs"]
mod ownership_tests;
