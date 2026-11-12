//! What:
//!  Turns a plain forward of a push into a stacked push or a push that uploads Git LFS objects first,
//! and adds the
//!  warnings and reference-lock notices a forwarded command owes the caller.
//! Why:
//!  A push that may lose a race,
//! or that carries LFS objects,
//! must run as a child so the wrapper can explain a failure;
//!  every other forward stays a process replacement.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // stackIfPush(action, env): Action; needsLfsUpload(command, realGit, overlay): [boolean, string, settings?]
//! ```

/// Import the sibling modules the decision combines.
use crate::action::{Action, PushForward};
use crate::command_push::stackable_option_tokens;
use crate::external_hooks::external_hooks_warning;
use crate::lfs_pre_push::uses_lfs;
use crate::lfs_upload_plan::settings_from_snapshot;
use crate::config_snapshot::{ConfigSnapshot, read_config_snapshot};
use crate::push_stacking::STACKED_PUSH_MARKER;
use crate::command_push::parse_push_region;
use crate::child_environment::environment_value;
use crate::forwarded_lock_notice::{forwarded_lock_notice, is_covered};
use crate::stale_ref_lock_judge::Judge;
use crate::forwarded_command::{ResolvedCommand, resolve_forwarded_command};
use crate::transaction_git::GitContext;
/// What:
///  `OsString` is owned operating-system text of raw OS bytes (sibling `String`
///       must be UTF-8).
/// Why:
///   Arguments and environment values are never decoded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // process.argv and process.env, but byte-preserving.
/// ```
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw OS bytes.
use std::path::Path;
/// What:
///  Whether `command` is a push that must upload LFS objects first,
/// and the configuration warning for configured
///  external hooks,
/// when the same read shows any,
/// as empty text otherwise.
///  A dry-run push reads the configuration too,
/// for the warning,
/// but uploads nothing.
/// Why:
///   A push that sends nothing (`--dry-run`) needs no upload,
/// and a repository without LFS keys needs none either.
///  One configuration read answers it,
/// made with the invocation's own global options;
/// when Git cannot list its
///  configuration the hook is added anyway,
/// because it then decides with Git's own answers.
///  The warning costs no read of its own:
/// a push is the one forwarded command that already holds the snapshot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function needsLfsUpload(command, realGit, overlay): [boolean, string, settings];
/// ```
fn needs_lfs_upload(
    command: &ResolvedCommand,
    real_git: &Path,
    overlay: &[(OsString, OsString)],
) -> (bool, String, Option<Vec<(String, String)>>) {
    if command.word() != b"push" {
        return (false, String::new(), None);
    }
    // A push region that does not parse is not known to be a dry run.
    let dry_run: bool = match parse_push_region(command.region(), &[]) {
        Ok(region) => region.dry_run,
        Err(_refused) => false,
    };
    let context: GitContext = GitContext {
        real_git: real_git.to_path_buf(),
        overlay: overlay.to_vec(),
        global_prefix: command.arguments[..command.subcommand_index].to_vec(),
        common_dir: None,
    };
    let snapshot: Option<ConfigSnapshot> = match std::env::current_dir() {
        Ok(directory) => read_config_snapshot(&context, directory.as_path()),
        Err(_) => None,
    };
    let warning: String = external_hooks_warning(snapshot.as_ref()).unwrap_or_default();
    if dry_run {
        return (false, warning, None);
    }
    return match snapshot {
        Some(found) => {
            let uses: bool = uses_lfs(&found);
            // The settings are read only for a repository that uses LFS.
            let settings: Option<Vec<(String, String)>> = if uses { Some(settings_from_snapshot(&found)) } else { None };
            (uses, warning, settings)
        }
        None => (true, warning, None),
    };
}

/// What:
///  Turn the forward of a plain push that may be integrated into a run that can integrate it,
///  and put the Git LFS upload in front of every push that publishes.
/// Why:
///   A push that lost a race to another session cannot succeed again,
/// and the wrapper can only look at its status
///  while it is alive;
///  the stacked push itself carries `STACKED_PUSH_MARKER`,
/// so it is forwarded as it is and never stacks again.
///   A real push in a repository with LFS keys also runs as a child that the Git LFS upload (`lfs_upload`) precedes,
///  the stacked push too,
/// so the parent can explain a failed upload;
///  the integration step's own dry-run queries never upload.
///   Every other action is returned unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stackIfPush(action: Action, environment): Action;
/// ```
pub(crate) fn stack_if_push(action: Action, environment: &[(OsString, OsString)]) -> Action {
    let (real_git, arguments, overlay, copy, stderr) = match action {
        Action::Forward {
            real_git,
            arguments,
            overlay,
            stderr,
        } => (real_git, arguments, overlay, None, stderr),
        Action::ForwardSettled {
            real_git,
            arguments,
            overlay,
            copy,
            stderr,
        } => (real_git, arguments, overlay, Some(copy), stderr),
        other => return other,
    };
    let context: GitContext = GitContext {
        real_git: real_git.clone(),
        overlay: overlay.clone(),
        global_prefix: Vec::new(),
        // These queries send no input, so no scratch directory is needed.
        common_dir: None,
    };
    let command: ResolvedCommand = resolve_forwarded_command(&context, environment, arguments.as_slice());
    let (lfs, hook_warning, lfs_settings): (bool, String, Option<Vec<(String, String)>>) =
        needs_lfs_upload(&command, real_git.as_path(), overlay.as_slice());
    let stderr_with_warning: String = format!("{stderr}{hook_warning}");
    let stacked_already: bool = environment_value(environment, STACKED_PUSH_MARKER).is_some();
    let is_push: bool = command.word() == b"push";
    let stack: bool =
        !stacked_already && is_push && stackable_option_tokens(command.region()).is_some();
    // What: Every push runs as a child of the wrapper, whatever it does with LFS objects or stacking.
    // Why:  The wrapper masks the URLs in what that Git prints, and Git names the destination it could not reach on its
    //       standard error and the one it pushed to in the summary on its standard output (audit `alpha-12-lfs`).
    // A push that uploads LFS objects runs as a child like a stackable one: the parent that owns the push is the one
    // that explains a failed upload, and no wrapper is started for it.
    if stack || lfs || is_push {
        // A push that would have become Git itself is judged for the reference locks of a killed Git first, as before.
        let lock_notice: String = if !(stack || lfs) && is_covered(command.word()) {
            forwarded_lock_notice(command.word(), &command.arguments[..command.subcommand_index], environment, &Judge::here())
        } else {
            String::new()
        };
        return Action::ForwardPush {
            forward: Box::new(PushForward {
                real_git,
                subcommand_index: command.subcommand_index,
                arguments: command.arguments,
                overlay,
                copy,
                environment: environment.to_vec(),
                lfs,
                lfs_settings,
                stack,
            }),
            stderr: format!("{stderr_with_warning}{lock_notice}"),
        };
    }
    // A plain forwarded fetch or push becomes Git itself, so the reference locks of a killed Git are judged here, before.
    let lock_notice: String = if is_covered(command.word()) {
        forwarded_lock_notice(command.word(), &command.arguments[..command.subcommand_index], environment, &Judge::here())
    } else {
        String::new()
    };
    let stderr_with_locks: String = format!("{stderr_with_warning}{lock_notice}");
    let forwarded: Vec<OsString> = arguments;
    return match copy {
        Some(facts) => Action::ForwardSettled {
            real_git,
            arguments: forwarded,
            overlay,
            copy: facts,
            stderr: stderr_with_locks,
        },
        None => Action::Forward {
            real_git,
            arguments: forwarded,
            overlay,
            stderr: stderr_with_locks,
        },
    };
}
