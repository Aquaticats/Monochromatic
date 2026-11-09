//! What:
//!  Which repository the service commands (`monochromatic-git-policy-cli service ...`) mean:
//! the global options name it,
//! or Git is asked for
//!  the Git directory of the current directory.
//! Why:
//!  `management.rs` plans the service commands;
//! the question of which repository is asked of Git and answered as a
//!  `Selection`,
//! in a file it uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // statusGitDir(globalArguments, environment, realGit): Selection
//! ```

use crate::git_system_config::with_real_git;
use crate::repo_read_layout::read_layout_after_success_for;
/// What:
///  `OsString` is owned operating-system text of raw OS bytes (sibling `String`
///       must be UTF-8).
/// Why:
///   Arguments and environment values are never decoded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string[] argv, but byte-preserving.
/// ```
use std::ffi::OsString;
/// `PathBuf` is an owned filesystem path of raw OS bytes.
use std::path::{Path, PathBuf};
/// What:
///  The global options of an invocation that choose the directory it runs in (`-C <path>`,
/// in order),
/// or nothing when
///  another option could choose the repository too.
/// Why:
///   `git -C /path/B monochromatic-git-policy-cli service status` asks about B,
/// wherever it is run from;
/// `-c <name>=<value>` changes no directory,
///  while `--git-dir` and its kin name a repository the in-process reader does not resolve.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function directoryOptions(globalArguments): string[] | undefined;
/// ```
fn directory_options(global_arguments: &[OsString]) -> Option<Vec<OsString>> {
    let mut kept: Vec<OsString> = Vec::new();
    let mut options = global_arguments.iter();
    while let Some(option) = options.next() {
        let value: &OsString = options.next()?;
        if option == "-C" {
            kept.push(option.clone());
            kept.push(value.clone());
        } else if option != "-c" {
            return None;
        }
    }
    return Some(kept);
}

/// Which repository a `git monochromatic-git-policy-cli service status` invocation selects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Selection {
    /// The Git directory of the worktree it selects.
    Found(PathBuf),
    /// Git says the invocation is outside a repository.
    Outside,
    /// Neither the reader nor Git could say,
    /// with the reason.
    Unresolved(String),
}

/// What:
///  The repository a `git <global options> monochromatic-git-policy-cli service status` selects:
/// the Git directory of its worktree,
/// outside a
///  repository,
/// or unresolved.
/// Why:
///   The command needs no valid configuration,
/// so the structure of the files decides and the configuration values do not.
///  The reader learns the system configuration file from the real Git `real_git` names,
/// and resolves `-C` options only.
///  Whatever it declines (`--git-dir`,
/// `--work-tree`,
/// `GIT_DIR`,
/// an unusual layout) is put to the real Git once,
/// which resolves the
///  selection as every other command does;
/// "outside a repository" is Git's own answer,
/// and anything Git cannot answer is
///  unresolved,
/// never an invitation to count unrelated services.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function statusGitDir(globalArguments, environment, realGit): string | undefined;
/// ```
pub(crate) fn status_git_dir(global_arguments: &[OsString], environment: &[(OsString, OsString)], real_git: Option<&Path>) -> Selection {
    // The real Git is added to the environment only when one was found.
    let named: Vec<(OsString, OsString)> = match real_git {
        Some(found) => with_real_git(environment, found),
        None => environment.to_vec(),
    };
    if let Some(prefix) = directory_options(global_arguments)
        && let Some(layout) = read_layout_after_success_for(prefix.as_slice(), named.as_slice())
    {
        return Selection::Found(layout.git_dir);
    }
    let Some(real) = real_git else {
        return Selection::Unresolved(String::from("no real Git was found to ask"));
    };
    let mut asking = std::process::Command::new(real);
    asking.args(global_arguments).args(["rev-parse", "--path-format=absolute", "--absolute-git-dir"]);
    // One `env` call per entry, in order, as `envs` made; `LC_ALL` is set after them so it wins.
    for (name, value) in environment {
        asking.env(name, value);
    }
    asking.env("LC_ALL", "C").stdin(std::process::Stdio::null());
    // This start of the real Git says so too, like every other (`started_as_git.rs`), and it is a start of the wrapper's own
    //  reading: nothing of the person's runs under it.
    crate::started_as_git::mark_real_git_start(&mut asking, crate::started_as_git::StartMode::Query);
    let asked = crate::captured_child_run_to_its_end::output_of_within_bound(&mut asking, crate::wait_ceilings::PLUMBING_STEP_CEILING);
    let output = match asked {
        Ok(found) => found,
        Err(error) => return Selection::Unresolved(format!("Git could not be started: {error}")),
    };
    let said: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
    if output.status.success() && !said.is_empty() {
        return Selection::Found(PathBuf::from(said));
    }
    let complaint: String = String::from_utf8_lossy(output.stderr.as_slice()).trim().to_owned();
    if output.status.code() == Some(NOT_A_REPOSITORY_EXIT_CODE) && complaint.contains("not a git repository") {
        return Selection::Outside;
    }
    return Selection::Unresolved(format!("Git said: {complaint}"));
}

/// The exit status Git gives for a fatal error such as running outside a repository.
const NOT_A_REPOSITORY_EXIT_CODE: i32 = 128;
