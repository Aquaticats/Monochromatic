//! What:
//!  `git cli-git doctor`: list the traditional hooks a repository holds.
//! Why:
//!  Traditional hooks (executable scripts in `<git-common-dir>/hooks` or in a `core.hooksPath` directory) are
//!  unsupported, and their behavior with the wrapper is undefined.
//!  The commit path warns when `core.hooksPath` is configured but never scans a directory, because a scan costs
//!  file system calls on every commit; this command is where the scan happens, on request.
//!  Exit status 0 means none were found, 1 means at least one was, and 2 means the repository could not be located.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const hooks = (await Promise.all(directories.map(scanHooksDirectory))).flat();
//! process.exitCode = hooks.length === 0 ? 0 : 1;
//! ```

/// The decision this command ends in.
use super::action::{Action, ENGINE_FAILURE_EXIT_CODE};
/// Lexical path normalization, so one directory named two ways is scanned once.
use super::hook_dispatch_plan::normalized_path;
/// One required Git start with the caller's context.
use super::transaction_git::{GitContext, GitRequest, run_git};
/// `ErrorKind` tells a missing hooks directory from one that cannot be read.
use std::io::ErrorKind;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Exit status when at least one traditional hook was found.
pub const HOOKS_FOUND_EXIT_CODE: i32 = 1;

/// File name suffix of the samples Git installs, which are never hooks.
const SAMPLE_SUFFIX: &[u8] = b".sample";

/// What:
///  Where a repository keeps hooks.
/// Why:
///   Git reads hooks from `<git-common-dir>/hooks` unless `core.hooksPath` names another directory, and a
///  repository can hold hooks in both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HooksLocation = { commonDirectory: string; configured?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HooksLocation {
    /// Absolute common Git directory, whose `hooks` directory is the default.
    pub common_directory: PathBuf,
    /// Absolute directory a configured `core.hooksPath` names, resolved the way Git resolves it.
    pub configured: Option<PathBuf>,
}

/// What:
///  The directories to scan, each once.
/// Why:
///   The default directory comes first, then the configured one unless it is the same directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unique([join(commonDirectory, 'hooks'), configured].filter(Boolean));
/// ```
pub fn hook_directories(location: &HooksLocation) -> Vec<PathBuf> {
    let mut directories: Vec<PathBuf> = vec![normalized_path(location.common_directory.join("hooks").as_path())];
    if let Some(configured) = &location.configured {
        let normalized: PathBuf = normalized_path(configured.as_path());
        if !directories.contains(&normalized) {
            directories.push(normalized);
        }
    }
    return directories;
}

/// What:
///  Whether a file mode makes the file runnable by someone.
/// Why:
///   Git runs a hook only when it is executable and prints a hint for a file it skips, so a plain file is not a hook.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (stats.mode & 0o111) !== 0
/// ```
#[cfg(unix)]
fn is_executable(metadata: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    return metadata.permissions().mode() & 0o111 != 0;
}

/// What:
///  Whether a file is runnable.
/// Why:
///   Without Unix permission bits every regular file counts, because the platform has no mode to test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// true
/// ```
#[cfg(not(unix))]
fn is_executable(_metadata: &std::fs::Metadata) -> bool {
    return true;
}

/// What:
///  The traditional hooks in one directory: executable regular files and symbolic links, except `*.sample`.
/// Why:
///   A missing directory holds none; any other failure to read it is reported, because silence would pass a
///  directory the command could not look at.
///  A link counts whatever it points at (a dangling link too), because Git follows it when it starts the hook.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readdir(directory)).filter((name) => !name.endsWith('.sample') && (isSymlink(name) || isExecutableFile(name)));
/// ```
pub fn scan_hooks_directory(directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    let entries = match std::fs::read_dir(directory) {
        Ok(found) => found,
        Err(error) if error.kind() == ErrorKind::NotFound || error.kind() == ErrorKind::NotADirectory => {
            return Ok(Vec::new());
        }
        Err(error) => return Err(error),
    };
    let mut hooks: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let found = entry?;
        if found.file_name().as_encoded_bytes().ends_with(SAMPLE_SUFFIX) {
            continue;
        }
        let metadata: std::fs::Metadata = std::fs::symlink_metadata(found.path())?;
        let kind: std::fs::FileType = metadata.file_type();
        if kind.is_symlink() || (kind.is_file() && is_executable(&metadata)) {
            hooks.push(found.path());
        }
    }
    hooks.sort();
    return Ok(hooks);
}

/// What:
///  The traditional hooks in every directory, in directory order and then name order.
/// Why:
///   The report lists them as a person reads them: the default directory first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await Promise.all(directories.map(scanHooksDirectory))).flat();
/// ```
pub fn find_traditional_hooks(directories: &[PathBuf]) -> std::io::Result<Vec<PathBuf>> {
    let mut hooks: Vec<PathBuf> = Vec::new();
    for directory in directories {
        hooks.extend(scan_hooks_directory(directory.as_path())?);
    }
    return Ok(hooks);
}

/// What:
///  The report: one hook path per line.
/// Why:
///   The output is for people and scripts alike, so it carries nothing but the paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hooks.map((hook) => `${hook}\n`).join('')
/// ```
pub fn render_hooks(hooks: &[PathBuf]) -> String {
    let mut text: String = String::new();
    for hook in hooks {
        text.push_str(hook.display().to_string().as_str());
        text.push('\n');
    }
    return text;
}

/// What:
///  The explanation printed beside a non-empty report.
/// Why:
///   The paths alone do not say why they are listed or what to do with them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const explanation: string;
/// ```
const FOUND_EXPLANATION: &str = "git cli-git doctor: the traditional hooks listed on standard output are unsupported, \
and their behavior with cli-git is undefined. Move each into a config hook (`hook.<name>.event` with \
`hook.<name>.command`) and delete the script.\n";

/// What:
///  Run one Git query in `cwd` and return its standard output as text.
/// Why:
///   Locating the repository takes three small questions, and each failure is reported with Git's own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { stdout } = await git(args, { cwd });
/// ```
fn query(context: &GitContext, cwd: &Path, arguments: &[&str]) -> Result<Option<String>, String> {
    let request: GitRequest = GitRequest::new(cwd, arguments);
    let output = run_git(context, &request).map_err(|error| {
        return format!("git {} could not start: {error}", arguments.join(" "));
    })?;
    if output.code == Some(1) && arguments.first() == Some(&"config") {
        return Ok(None);
    }
    if !output.succeeded() {
        return Err(format!("git {} failed: {}", arguments.join(" "), output.error_text()));
    }
    let text: String = String::from_utf8(output.stdout)
        .map_err(|_| { return format!("git {} printed bytes that are not UTF-8", arguments.join(" ")); })?;
    return Ok(Some(text));
}

/// What:
///  Locate the hooks of the repository that Git selects from `cwd` and the caller's global options.
/// Why:
///   Git alone knows the common directory (worktrees, `--git-dir`, `GIT_DIR`) and how `core.hooksPath` expands.
///  A relative `core.hooksPath` is relative to where Git runs hooks, which is the worktree root, or the Git
///  directory of a bare repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const location = await locateHooks({ context, cwd });
/// ```
pub fn locate_hooks(context: &GitContext, cwd: &Path) -> Result<HooksLocation, String> {
    let report: String = query(
        context,
        cwd,
        &["rev-parse", "--path-format=absolute", "--git-common-dir", "--absolute-git-dir"],
    )?
    .unwrap_or_default();
    let mut lines = report.lines();
    let (Some(common), Some(git_directory)) = (lines.next(), lines.next()) else {
        return Err(String::from("git rev-parse did not report the common and Git directories"));
    };
    let configured_text: Option<String> = query(context, cwd, &["config", "--type=path", "--get", "core.hooksPath"])?
        .map(|text| { return text.strip_suffix('\n').unwrap_or(text.as_str()).to_string(); })
        .filter(|text| { return !text.is_empty(); });
    let Some(value) = configured_text else {
        return Ok(HooksLocation {
            common_directory: PathBuf::from(common),
            configured: None,
        });
    };
    if Path::new(value.as_str()).is_absolute() {
        return Ok(HooksLocation {
            common_directory: PathBuf::from(common),
            configured: Some(PathBuf::from(value)),
        });
    }
    // A relative value resolves where Git runs hooks; a repository without a worktree has no top level.
    let base: PathBuf = match query(context, cwd, &["rev-parse", "--path-format=absolute", "--show-toplevel"]) {
        Ok(Some(top)) if !top.trim_end().is_empty() => PathBuf::from(top.trim_end()),
        Ok(_) | Err(_) => PathBuf::from(git_directory),
    };
    return Ok(HooksLocation {
        common_directory: PathBuf::from(common),
        configured: Some(base.join(value)),
    });
}

/// What:
///  Decide what `git cli-git doctor` prints and exits with.
/// Why:
///   Exit 0 with no output means no traditional hook was found, exit 1 lists each one, and exit 2 reports a
///  repository or a directory that could not be examined.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function doctor({ context, cwd }): Promise<Action>;
/// ```
pub fn doctor_action(context: &GitContext, cwd: &Path) -> Action {
    let location: HooksLocation = match locate_hooks(context, cwd) {
        Ok(found) => found,
        Err(message) => return doctor_failure(message.as_str()),
    };
    let hooks: Vec<PathBuf> = match find_traditional_hooks(hook_directories(&location).as_slice()) {
        Ok(found) => found,
        Err(error) => return doctor_failure(format!("cannot read a hooks directory: {error}").as_str()),
    };
    if hooks.is_empty() {
        return Action::Exit {
            code: 0,
            stdout: String::new(),
            stderr: String::new(),
        };
    }
    return Action::Exit {
        code: HOOKS_FOUND_EXIT_CODE,
        stdout: render_hooks(hooks.as_slice()),
        stderr: String::from(FOUND_EXPLANATION),
    };
}

/// What:
///  The action for a doctor run that could not examine the repository.
/// Why:
///   Status 2 is the wrapper's status for a failure that is not a finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { code: 2, stdout: '', stderr: `git cli-git doctor: ${message}\n` }
/// ```
fn doctor_failure(message: &str) -> Action {
    return Action::Exit {
        code: ENGINE_FAILURE_EXIT_CODE,
        stdout: String::new(),
        stderr: format!("git cli-git doctor: {message}\n"),
    };
}

/// Doctor controls stay out of the release executable.
#[cfg(test)]
#[path = "doctor_tests.rs"]
mod tests;
