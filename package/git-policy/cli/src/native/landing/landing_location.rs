//! What:
//!  Where a forwarded command's real index lives:
//!  two `git rev-parse` runs that report
//!       the git directory
//!  and the index Git would use.
//! Why:
//!  An index writer waits for the commit service's hold on the real index.
//!  A command outside a repository,
//!  or one writing another index,
//!  waits for
//!      nothing.
//!  Each pathname is asked for on its own,
//! and each answer loses exactly its one terminating line feed,
//! because a pathname may hold the line feed that would have separated two answers of one report.
//!  `health/doctor.rs` reads its path answers the same way (`locate_hooks`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const gitDir = stripOneLineFeed((await nanoSpawn(gitPath, [...globalArgs, 'rev-parse', '--path-format=absolute', '--absolute-git-dir'])).stdout);
//! const indexPath = stripOneLineFeed((await nanoSpawn(gitPath, [...globalArgs, 'rev-parse', '--path-format=absolute', '--git-path', 'index'])).stdout);
//! ```

/// Running real Git for the location queries.
use crate::path_bytes::os_from_path_bytes;
use crate::repo_read_adapter::landing_of;
use crate::git_system_config::with_real_git;
use crate::repo_read_layout::read_layout;
use crate::transaction_git::{GitContext, GitRequest, run_git};
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  The index paths of one repository selection.
/// Why:
///   `LandingLocation`;
///  the real index is the worktree's `<git-dir>/index`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandingLocation = { cwd: string; gitDir: string; indexPath: string; realIndexPath: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LandingLocation {
    /// Per-worktree Git directory.
    pub git_dir: PathBuf,
    /// Index Git would write,
    ///  honouring `GIT_INDEX_FILE`.
    pub index_path: PathBuf,
    /// The worktree's real index,
    ///  `<git-dir>/index`.
    pub real_index_path: PathBuf,
}

/// What:
///  The arguments the Git directory query adds to the selection's global options.
/// Why:
///   The query asks for one pathname alone,
///  so a line feed inside it cannot end another answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['rev-parse', '--path-format=absolute', '--absolute-git-dir']
/// ```
fn git_directory_arguments() -> [&'static str; 3] {
    return ["rev-parse", "--path-format=absolute", "--absolute-git-dir"];
}

/// What:
///  The arguments the index query adds to the selection's global options.
/// Why:
///   The index is the second pathname,
///  and it is asked for on its own for the same reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['rev-parse', '--path-format=absolute', '--git-path', 'index']
/// ```
fn index_arguments() -> [&'static str; 4] {
    return ["rev-parse", "--path-format=absolute", "--git-path", "index"];
}

/// What:
///  One answer without the single line feed Git ends it with,
///  or the answer itself when it has none.
/// Why:
///   Exactly one line feed is the terminator;
///  a pathname that ends with a line feed of its own keeps it,
///  and a pathname that holds one in the middle keeps that too,
///  reading answers the way `health/doctor.rs` reads them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// answer.endsWith('\n') ? answer.slice(0, -1) : answer
/// ```
fn without_the_terminator(answer: &[u8]) -> &[u8] {
    if answer.ends_with(b"\n") {
        return &answer[..answer.len() - 1];
    }
    return answer;
}

/// What:
///  One pathname Git printed,
///  as the path it names,
///  refusing an empty answer.
/// Why:
///   An empty line would point the lock at the wrong directory;
///  the paths stay bytes (`path_bytes.rs`),
///  so a name that is not UTF-8 is the name Git printed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytes = stripOneLineFeed(answer); if (bytes.length === 0) throw new TypeError(...); Buffer.from(bytes)
/// ```
fn path_of_answer(answer: &[u8], what: &str) -> Result<PathBuf, String> {
    let bytes: &[u8] = without_the_terminator(answer);
    if bytes.is_empty() {
        // A refusal names the text a person reads, with replacement characters for bytes that are not UTF-8.
        return Err(format!("Unexpected rev-parse {what}: {:?}", String::from_utf8_lossy(answer)));
    }
    return Ok(PathBuf::from(os_from_path_bytes(bytes)));
}

/// What:
///  The landing location the two answers report,
///  or why they are not paths.
/// Why:
///   The index names the worktree's own index,
///  and `GIT_INDEX_FILE` may send the write elsewhere;
///  the real index is always the Git directory's `index`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gitDir = pathOfAnswer(gitDirAnswer); return { gitDir, indexPath: pathOfAnswer(indexAnswer), realIndexPath: join(gitDir, 'index') }
/// ```
pub fn landing_location_of(git_dir_answer: &[u8], index_answer: &[u8]) -> Result<LandingLocation, String> {
    let git_dir: PathBuf = path_of_answer(git_dir_answer, "Git directory")?;
    let index_path: PathBuf = path_of_answer(index_answer, "index path")?;
    return Ok(LandingLocation {
        real_index_path: git_dir.join("index"),
        index_path,
        git_dir,
    });
}

/// What:
///  Resolve the landing location for the global options of one command.
/// Why:
///   `resolveLandingLocation`:
///  outside a repository the answer is nothing,
///  not an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveLandingLocation({ gitPath, globalArgs }): Promise<LandingLocation | typeof LANDING_LOCATION_ABSENT>;
/// ```
pub fn resolve_landing_location(
    context: &GitContext,
    global_arguments: &[OsString],
) -> Result<Option<LandingLocation>, String> {
    // A plain layout is read from the file system; Git is asked only when the reader declines it.
    if let Some(found) = read_layout(global_arguments, with_real_git(context.overlay.as_slice(), context.real_git.as_path()).as_slice()).as_ref().and_then(landing_of) {
        return Ok(Some(found));
    }
    return answer_of(context, global_arguments);
}

/// What:
///  Ask Git for one pathname with the selection's global options,
///  or nothing when Git refused the question.
/// Why:
///   Both pathnames are asked for the same way,
///  and Git refuses both outside a repository;
///  a refusal is what `resolve_landing_location` answers with nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await git(globalArgs, tail); answer.status === 0 ? answer.stdout : undefined
/// ```
fn ask(context: &GitContext, global_arguments: &[OsString], tail: &[&str]) -> Result<Option<Vec<u8>>, String> {
    let mut arguments: Vec<OsString> = global_arguments.to_vec();
    for part in tail {
        arguments.push(OsString::from(*part));
    }
    let mut request: GitRequest = GitRequest::new(Path::new("."), arguments.as_slice());
    request.without_prefix = true;
    return match run_git(context, &request) {
        Ok(output) => {
            if output.succeeded() {
                Ok(Some(output.stdout))
            } else {
                Ok(None)
            }
        }
        Err(error) => Err(error.to_string()),
    };
}

/// What:
///  The landing location the two pathname answers report:
///  none when Git refused either question.
/// Why:
///   A refusal from Git means the command is not inside a repository,
///  and a repository is asked about both pathnames before either answer is used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gitDir = await ask(globalArgs, ['rev-parse', '--path-format=absolute', '--absolute-git-dir']); if (!gitDir) return LANDING_LOCATION_ABSENT;
/// const index = await ask(globalArgs, ['rev-parse', '--path-format=absolute', '--git-path', 'index']); if (!index) return LANDING_LOCATION_ABSENT;
/// return landingLocationOf(gitDir, index);
/// ```
fn answer_of(context: &GitContext, global_arguments: &[OsString]) -> Result<Option<LandingLocation>, String> {
    let Some(git_dir_answer) = ask(context, global_arguments, git_directory_arguments().as_slice())? else {
        return Ok(None);
    };
    let Some(index_answer) = ask(context, global_arguments, index_arguments().as_slice())? else {
        return Ok(None);
    };
    return landing_location_of(git_dir_answer.as_slice(), index_answer.as_slice()).map(Some);
}

/// Location controls stay out of the release executable.
#[cfg(test)]
#[path = "landing_location_tests.rs"]
mod tests;
