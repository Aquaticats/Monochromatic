//! What:
//!  The check,
//! before anything starts,
//! that the Git LFS upload a push is about to run can start.
//! Why:
//!  A push of a repository with LFS keys is preceded by `git-lfs push`,
//! which the parent starts.
//!  When no directory of `PATH` holds `git-lfs`,
//! one stat per `PATH` directory,
//! with no process started,
//! finds that
//!  first,
//! and the parent says so with the full explanation.
//! A push that bypasses the upload (`--no-verify`,
//!  `GIT_LFS_SKIP_PUSH`,
//! or the switch `-c hook.cli-git-lfs-pre-push.enabled=false`) needs no program,
//! so it is never
//!  refused here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusal = hookIsActive(prefix, region) && !programOnPath(env.PATH) ? missingProgramMessage() : undefined;
//! ```

/// The invocation's environment values.
use crate::child_environment::environment_value;
/// The table-driven read of a push's options.
use crate::command_push::parse_push_region;
/// The hook switch,
/// the lookup and the refusal text.
use crate::lfs_pre_push::{hook_switched_off, missing_program_message, program_on_path, skip_requested};
/// Owned OS strings for Git arguments.
use std::ffi::OsString;

/// What:
///  The refusal text when the upload hook is active and `git-lfs` is not on the invocation's `PATH`.
/// Why:
///   A region Git itself would refuse is left to Git,
/// which says why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lfsRefusal(globalPrefix, region, environment): string | undefined;
/// ```
pub fn lfs_refusal(
    global_prefix: &[OsString],
    region: &[OsString],
    environment: &[(OsString, OsString)],
) -> Option<String> {
    if hook_switched_off(global_prefix, environment) || skip_requested(environment) {
        return None;
    }
    let parsed = parse_push_region(region, &[]).ok()?;
    if parsed.no_verify || parsed.dry_run {
        return None;
    }
    let path: OsString = environment_value(environment, "PATH").unwrap_or_default();
    if program_on_path(path.as_os_str()) {
        return None;
    }
    return Some(missing_program_message());
}

/// The refusal is controlled in the test module.
#[cfg(test)]
#[path = "lfs_push_precheck_tests.rs"]
mod tests;
