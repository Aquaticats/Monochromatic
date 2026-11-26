//! What:
//!  The two things a service's own removal of a worktree reads before it does anything:
//!  the real Git it starts,
//!  and the pool the worktree's commits fill.
//! Why:
//!  The service's handler of a record is handed the record alone,
//!  so the Git it starts is the one this process resolves,
//!  by the same search,
//!  the same refusal of a wrapper,
//!  and the same record of a Git found from the unit,
//! as the wrapper's own start makes.
//!  The pool is named by the worktree's own `.git` pointer,
//!  which the removal deletes,
//!  so both are read while the directory still stands.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const git = theServiceGit(item); // { realGit, overlay: [], globalPrefix: [], commonDir }
//! const pool = thePoolOf(target); // [common, admin] | undefined
//! ```

use crate::real_git::{
    ResolutionInputs,
    ResolvedGit,
    process_resolution_inputs,
    resolve_real_git,
};
use crate::transaction_git::GitContext;
use crate::work_item::WorkItem;
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The common Git directory of this repository and the registration directory of `target`,
///  as a removal reads them before Git deletes the worktree.
/// Why:
///   The pool a worktree's commits fill is named from its registration directory and lives in the common Git directory,
///  and both are named by the worktree's own `.git` pointer,
///  which the removal deletes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const poolOf = (target) => { const admin = gitDirOf(target); return [commonDirectoryOf(admin), admin]; };
/// ```
pub(crate) fn the_pool_of(target: &Path) -> Option<(PathBuf, PathBuf)> {
    // A `.git` pointer that is a named pipe is read at once or refused, and never waited on (`SPEC.md`: a `.git` that is not a
    // plain file ends the run with a reason).
    let pointer: Vec<u8> = crate::plain_file::read_plain_file(
        target.join(".git").as_path(),
        crate::plain_file::SMALL_FILE_LIMIT,
    )
    .ok()?;
    let text: String = String::from_utf8_lossy(pointer.as_slice()).into_owned();
    let named: PathBuf = PathBuf::from(text.trim_end().strip_prefix("gitdir: ")?.trim());
    let admin: PathBuf = if named.is_absolute() {
        named
    } else {
        target.join(named)
    };
    let common: PathBuf =
        crate::common_directory_of_a_git_directory::common_directory_of(admin.as_path());
    return Some((common, admin));
}

/// What:
///  How the commit service starts real Git for a removal it was handed a record for.
/// Why:
///  The service's handler of a record is handed the record alone,
///  so the Git it starts is the one this process resolves,
///  as the wrapper's own start does:
///  the same search,
///  the same refusal of a wrapper,
///  and the same record of a Git found from the unit.
///  The overlay of a command does not apply here:
///  the hooks are off for this whole process (`hooks_off::activate`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const serviceGit = (item) => ({ realGit: resolveRealGit(processResolutionInputs(process.env)), overlay: [], commonDir: item.gitDir });
/// ```
/// What:
///  The words for process facts a Git search needs and this machine would not give.
/// Why:
///  The reason names what could not be read,
///  so a person reads why no Git was started rather than a sentence about a record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noProcessFacts = (error) => `the process facts a Git search needs cannot be read (${error})`;
/// ```
fn no_process_facts(error: std::io::Error) -> String {
    return format!("the process facts a Git search needs cannot be read ({error})");
}

/// What:
///  The words for a removal that found no Git to run.
/// Why:
///  Nothing about the worktree was changed,
///  so the record is kept for another try,
///  and the reason says which record and why nothing ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noGitForARemoval = (failure) => `no Git can be started to remove the worktree a record names: ${failure}`;
/// ```
fn no_git_for_a_removal(failure: crate::real_git::RealGitNotFound) -> String {
    return format!("no Git can be started to remove the worktree a record names: {failure}");
}

pub(crate) fn the_service_git(item: &WorkItem) -> Result<GitContext, String> {
    let environment: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    // The service is a process of its own: the directory it was started in is read here, once, as this work's entry.
    let start: crate::start_directory::StartDirectory =
        crate::start_directory::StartDirectory::read_from_the_process();
    let inputs: ResolutionInputs =
        process_resolution_inputs(environment.as_slice(), &start).map_err(no_process_facts)?;
    let resolved: ResolvedGit = resolve_real_git(&inputs).map_err(no_git_for_a_removal)?;
    return Ok(GitContext {
        real_git: resolved.path,
        overlay: Vec::new(),
        global_prefix: Vec::new(),
        common_dir: Some(item.git_dir.clone()),
    });
}
