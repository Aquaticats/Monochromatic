//! What:
//!  `git cli-git doctor`: list the traditional hooks a repository holds.
//! Why:
//!  Traditional hooks (executable scripts in `<git-common-dir>/hooks` or in a `core.hooksPath` directory) are
//!  unsupported, and their behavior with the wrapper is undefined.
//!  The commit path warns when `core.hooksPath` is configured but never scans a directory, because a scan costs
//!  file system calls on every commit; this command is where the scan happens, on request.
//!  Exit status 0 means none were found, 1 means at least one was, and 2 means the repository could not be located.
//!  A hook file that is byte for byte the shim `git lfs` writes for its own name is expected and harmless, because the
//!  wrapper does the Git LFS upload itself and starts Git with traditional hooks off; it is explained apart from the
//!  other hooks and does not make the exit status 1.
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

/// The four hooks `git lfs install` writes.
const LFS_SHIM_NAMES: [&str; 4] = ["pre-push", "post-checkout", "post-commit", "post-merge"];

/// The longest shim file the comparison reads, in bytes; a longer file is not a shim.
const LFS_SHIM_LIMIT: u64 = 1024;

/// The text git-lfs 3.8.0 writes (`lfs/hook.go`, `hookBaseContent` and its `write`, which appends a line feed),
///  with the hook's own name where `{{Command}}` stands.
const LFS_SHIM_TEMPLATE: &str = r##"#!/bin/sh
command -v git-lfs >/dev/null 2>&1 || { printf >&2 "\n%s\n\n" "This repository is configured for Git LFS but 'git-lfs' was not found on your path. If you no longer wish to use Git LFS, remove this hook by deleting the '{{Command}}' file in the hooks directory (set by 'core.hookspath'; usually '.git/hooks')."; exit 2; }
git lfs {{Command}} "$@"
"##;

/// What:
///  The text git-lfs writes for the hook called `name`.
/// Why:
///   Recognition is by these exact bytes, so an edited or foreign file is never taken for a shim.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// template.replaceAll('{{Command}}', name)
/// ```
pub fn lfs_shim_text(name: &str) -> String {
    return LFS_SHIM_TEMPLATE.replace("{{Command}}", name);
}

/// What:
///  Whether a hook file is exactly the git-lfs shim for its own name.
/// Why:
///   Only a regular file whose name is one of the four hooks and whose bytes equal that hook's shim counts; a link,
///  a file that differs by one byte, and a shim under another hook's name are ordinary unsupported hooks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isFile(path) && readFileSync(path).equals(lfsShimText(basename(path)))
/// ```
pub fn is_lfs_shim(path: &Path) -> bool {
    use std::io::Read;
    let Some(name) = path.file_name().and_then(|found| { return found.to_str(); }) else {
        return false;
    };
    if !LFS_SHIM_NAMES.contains(&name) {
        return false;
    }
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return false;
    };
    if !metadata.file_type().is_file() {
        return false;
    }
    let Ok(file) = std::fs::File::open(path) else {
        return false;
    };
    let mut content: Vec<u8> = Vec::new();
    if file.take(LFS_SHIM_LIMIT).read_to_end(&mut content).is_err() {
        return false;
    }
    return content == lfs_shim_text(name).as_bytes();
}

/// What:
///  The explanation of the Git LFS shims a repository holds.
/// Why:
///   They are listed apart from the unsupported hooks and say what is going on, because the person who finds them
///  after deleting them needs to know they come back, that this is normal, and that nothing needs doing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const explanation = (shims: string[]) => '...';
/// ```
pub fn render_lfs_shims(shims: &[PathBuf]) -> String {
    let mut text: String = String::from(
        "git cli-git doctor: the hook files below are the Git LFS shims, byte for byte, and nothing needs doing:\n",
    );
    for shim in shims {
        text.push_str("  ");
        text.push_str(shim.display().to_string().as_str());
        text.push('\n');
    }
    text.push_str(
        "git-lfs writes these files again whenever it cleans a file in a Git that cli-git did not start (for example \
`git add` of an LFS file run as `/usr/bin/git` directly), so they can come back after you delete them; this is \
normal. cli-git uploads the LFS objects itself before every push and starts Git with traditional hooks off, so these \
files never run through cli-git. You may delete them or leave them. Only a script that starts Git without cli-git, \
such as `/usr/bin/git` directly, would run them.\n",
    );
    return text;
}

/// What:
///  Decide what `git cli-git doctor` prints and exits with.
/// Why:
///   Exit 0 with no output means no traditional hook was found, exit 1 lists each one on standard output, and exit 2
///  reports a repository or a directory that could not be examined.
///  Git LFS shims are explained on standard error apart from the other hooks and leave an exit status of 0 as it is;
///  any other hook beside them still ends in 1 with the unsupported report.
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
    let found: Vec<PathBuf> = match find_traditional_hooks(hook_directories(&location).as_slice()) {
        Ok(listed) => listed,
        Err(error) => return doctor_failure(format!("cannot read a hooks directory: {error}").as_str()),
    };
    let (shims, hooks): (Vec<PathBuf>, Vec<PathBuf>) =
        found.into_iter().partition(|path| { return is_lfs_shim(path.as_path()); });
    let shim_report: String = if shims.is_empty() { String::new() } else { render_lfs_shims(shims.as_slice()) };
    if hooks.is_empty() {
        return Action::Exit {
            code: 0,
            stdout: String::new(),
            stderr: shim_report,
        };
    }
    return Action::Exit {
        code: HOOKS_FOUND_EXIT_CODE,
        stdout: render_hooks(hooks.as_slice()),
        stderr: format!("{FOUND_EXPLANATION}{shim_report}"),
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
