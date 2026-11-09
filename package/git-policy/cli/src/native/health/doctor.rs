//! What:
//!  `git monochromatic-git-policy-cli doctor`:
//! list the traditional hooks a repository holds and the external hooks its Git
//!  configuration defines,
//! and note the retired `hooks.concurrentCommits` key.
//! Why:
//!  Traditional hooks (executable scripts in `<git-common-dir>/hooks` or in a `core.hooksPath` directory) are
//!  unsupported,
//! and their behavior with the wrapper is undefined.
//!  The commit path warns when `core.hooksPath` is configured but never scans a directory,
//! because a scan costs
//!  file system calls on every commit;
//! this command is where the scan happens,
//! on request.
//!  Exit status 0 means none were found,
//! 1 means at least one was (a configured hook counts),
//! and 2 means the
//!  repository could not be located.
//!  A hook file that is byte for byte the shim `git lfs` writes for its own name is expected and harmless,
//! because the
//!  wrapper does the Git LFS upload itself and starts Git with traditional hooks off;
//! it is explained apart from the
//!  other hooks and does not make the exit status 1.
//!  It also says on standard error whether Git's own fsmonitor is off (one line) or which file and scope turn it on
//!  (a finding with the command);
//! that never changes the exit status (`doctor_fsmonitor.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const hooks = (await Promise.all(directories.map(scanHooksDirectory))).flat();
//! process.exitCode = hooks.length === 0 ? 0 : 1;
//! ```

/// The decision this command ends in.
use crate::action::{Action, ENGINE_FAILURE_EXIT_CODE};
/// Configured external hooks and the retired `hooks` key.
use crate::discarded_prune::doctor_section;
use crate::doctor_config::{ConfigFindings, config_findings, render_hook_lines, render_notes};
/// Whether Git's own fsmonitor is off,
/// or which setting turns it on.
use crate::doctor_fsmonitor::fsmonitor_report;
/// Lexical path normalization,
/// so one directory named two ways is scanned once.
use crate::lexical_path::normalized_path;
/// One required Git start with the caller's context.
use crate::transaction_git::{GitContext, GitRequest};
/// The same Git start awaited by a task of the runtime.
use crate::transaction_git_awaiting::run_git_awaiting;
/// The process's one runtime,
/// which the doctor's report is built on.
use crate::lazy_runtime::run_on_runtime;
/// `ErrorKind` tells a missing hooks directory from one that cannot be read.
use std::io::ErrorKind;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// `Duration` is how long a file system read of the report may take.
use std::time::Duration;
/// The ceiling of each file system read of the report.
use crate::wait_ceilings::REPORT_SCAN_CEILING;
/// The job of such a read, run on a thread of its own.
use crate::work_awaited_within_a_ceiling::work_within;

/// Exit status when at least one traditional hook was found.
pub const HOOKS_FOUND_EXIT_CODE: i32 = 1;

/// File name suffix of the samples Git installs,
/// which are never hooks.
const SAMPLE_SUFFIX: &[u8] = b".sample";

/// What:
///  Where a repository keeps hooks.
/// Why:
///   Git reads hooks from `<git-common-dir>/hooks` unless `core.hooksPath` names another directory,
/// and a
///  repository can hold hooks in both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HooksLocation = { commonDirectory: string; configured?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HooksLocation {
    /// Absolute common Git directory,
    /// whose `hooks` directory is the default.
    pub common_directory: PathBuf,
    /// Absolute directory a configured `core.hooksPath` names,
    /// resolved the way Git resolves it.
    pub configured: Option<PathBuf>,
}

/// What:
///  The directories to scan,
/// each once.
/// Why:
///   The default directory comes first,
/// then the configured one unless it is the same directory.
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
///   Git runs a hook only when it is executable and prints a hint for a file it skips,
/// so a plain file is not a hook.
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
///   Without Unix permission bits every regular file counts,
/// because the platform has no mode to test.
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
///  The names in one directory,
/// as one read of it.
///  `fn(&Path) -> std::io::Result<Vec<PathBuf>>` is a function pointer,
/// so a control can stand a read that never answers where the
///  listing stands.
/// Why:
///   The listing of a directory is where a mount that stopped answering waits,
/// so it is the read the bound is placed
///  on,
/// and it is a value the report is given rather than a call it makes itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EntryListing = (directory: string) => string[];
/// ```
pub type EntryListing = fn(&Path) -> std::io::Result<Vec<PathBuf>>;

/// What:
///  The names in one directory,
/// read by the system.
/// Why:
///   This is the production listing the bounded scan takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listEntries = (directory) => readdirSync(directory, { withFileTypes: true }).map((entry) => join(directory, entry.name));
/// ```
fn list_entries(directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut entries: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(directory)? {
        entries.push(entry?.path());
    }
    return Ok(entries);
}

/// What:
///  The traditional hooks in one directory:
/// executable regular files and symbolic links,
/// except `*.sample`.
/// Why:
///   A missing directory holds none;
/// any other failure to read it is reported,
/// because silence would pass a
///  directory the command could not look at.
///  A link counts whatever it points at (a dangling link too),
/// because Git follows it when it starts the hook.
///  Every call here reads the file system,
/// and a mount that stops answering holds such a call in the kernel with no end,
/// so
///  the whole scan runs on a thread of its own and is awaited only to `REPORT_SCAN_CEILING`.
///  A scan that gives no answer is an error,
/// never an empty directory:
/// silence would read as "no hooks here".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readdir(directory)).filter((name) => !name.endsWith('.sample') && (isSymlink(name) || isExecutableFile(name)));
/// ```
pub fn scan_hooks_directory(directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    return scan_hooks_directory_with(list_entries, REPORT_SCAN_CEILING, directory);
}

/// What:
///  `scan_hooks_directory` with the listing and the ceiling given.
/// Why:
///   The listing and the ceiling are the two things a control varies;
/// `REPORT_SCAN_CEILING` says why the production value is
///  what it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// scanHooksDirectoryWith(listEntries, REPORT_SCAN_CEILING, directory);
/// ```
pub fn scan_hooks_directory_with(listing: EntryListing, ceiling: Duration, directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    let named: PathBuf = directory.to_path_buf();
    let scanned: Option<std::io::Result<Vec<PathBuf>>> = work_within(ceiling, move || {
        let entries: Vec<PathBuf> = match listing(named.as_path()) {
            Ok(found) => found,
            Err(error) if error.kind() == ErrorKind::NotFound || error.kind() == ErrorKind::NotADirectory => {
                return Ok(Vec::new());
            }
            Err(error) => return Err(error),
        };
        let mut hooks: Vec<PathBuf> = Vec::new();
        for path in entries {
            if path.file_name().is_some_and(|name| { return name.as_encoded_bytes().ends_with(SAMPLE_SUFFIX); }) {
                continue;
            }
            let metadata: std::fs::Metadata = std::fs::symlink_metadata(path.as_path())?;
            let kind: std::fs::FileType = metadata.file_type();
            if kind.is_symlink() || (kind.is_file() && is_executable(&metadata)) {
                hooks.push(path);
            }
        }
        hooks.sort();
        return Ok(hooks);
    });
    let Some(answer) = scanned else {
        return Err(std::io::Error::new(
            ErrorKind::TimedOut,
            format!("the listing of {} gave no answer within {ceiling:?}", directory.display()),
        ));
    };
    return answer;
}

/// What:
///  The traditional hooks in every directory,
/// in directory order and then name order.
/// Why:
///   The report lists them as a person reads them:
/// the default directory first.
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
///  The report:
/// one hook path per line.
/// Why:
///   The output is for people and scripts alike,
/// so it carries nothing but the paths.
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
const FOUND_EXPLANATION: &str = "git monochromatic-git-policy-cli doctor: the traditional hooks listed on standard output never run through \
monochromatic-git-policy-cli, which supports no external hook program and runs no program named by any configuration. Delete each script, \
or express what it checked as a policy under policies in monochromatic-git-policy-cli.config.jsonc.\n";

/// What:
///  Run one Git query in `cwd` and return its standard output as bytes.
/// Why:
///   Locating the repository takes a few small questions,
/// and each failure is reported with Git's own words.
/// A path answer is
///  bytes, not text:
/// a pathname is a byte string,
/// and one that is not UTF-8 is a directory Git runs hooks in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { stdout } = await git(args, { cwd });
/// ```
async fn query(context: &GitContext, cwd: &Path, arguments: &[&str]) -> Result<Option<Vec<u8>>, String> {
    let request: GitRequest = GitRequest::new(cwd, arguments);
    let output = match run_git_awaiting(context, &request).await {
        Ok(done) => done,
        Err(error) => return Err(format!("git {} could not start: {error}", arguments.join(" "))),
    };
    if output.code == Some(1) && arguments.first() == Some(&"config") {
        return Ok(None);
    }
    if !output.succeeded() {
        return Err(format!("git {} failed: {}", arguments.join(" "), output.error_text()));
    }
    return Ok(Some(output.stdout));
}

/// What:
///  One answer without the single line feed Git ends it with.
/// Why:
///   Exactly one line feed is the terminator;
/// a pathname that ends with a line feed of its own keeps it,
///  which trimming every trailing line feed threw away.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// bytes.endsWith('\n') ? bytes.slice(0, -1) : bytes
/// ```
fn without_the_terminator(answer: &[u8]) -> &[u8] {
    return match answer.strip_suffix(b"\n".as_slice()) {
        Some(body) => body,
        None => answer,
    };
}

/// What:
///  A pathname from the bytes Git printed for it.
/// Why:
///   A pathname is a byte string here,
/// so a directory whose name is not UTF-8 is scanned like any other,
///  as Git runs the hooks in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.from(bytes); // Node pathnames are byte strings
/// ```
#[cfg(unix)]
fn path_answer(bytes: &[u8]) -> PathBuf {
    use std::os::unix::ffi::OsStrExt;
    return PathBuf::from(std::ffi::OsStr::from_bytes(bytes));
}

/// What:
///  A pathname from the bytes Git printed for it,
/// for a platform that spells pathnames in text.
/// Why:
///   Refusing a pathname Git named would hide a directory it runs hooks in,
/// and naming it approximately is
///  better than not looking at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new TextDecoder().decode(bytes);
/// ```
#[cfg(not(unix))]
fn path_answer(bytes: &[u8]) -> PathBuf {
    return PathBuf::from(String::from_utf8_lossy(bytes).into_owned());
}

/// What:
///  Ask Git for one pathname and read its own answer.
/// Why:
///   Each pathname is asked for alone:
/// a pathname may hold the line feed that separated two answers in one report,
///  and then a shared report cannot be split into them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await gitQuestion(context, cwd, args);
/// ```
async fn path_answer_question(context: &GitContext, cwd: &Path, arguments: &[&str]) -> Result<PathBuf, String> {
    let answer: Vec<u8> = query(context, cwd, arguments).await?.unwrap_or_default();
    let value: &[u8] = without_the_terminator(answer.as_slice());
    if value.is_empty() {
        return Err(format!("git {} printed no path", arguments.join(" ")));
    }
    return Ok(path_answer(value));
}

/// What:
///  Locate the hooks of the repository that Git selects from `cwd` and the caller's global options.
/// Why:
///   Git alone knows the common directory (worktrees,
/// `--git-dir`,
/// `GIT_DIR`) and how `core.hooksPath` expands.
///  A relative `core.hooksPath` is relative to where Git runs hooks,
/// which is the worktree root,
/// or the Git
///  directory of a bare repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const location = await locateHooks({ context, cwd });
/// ```
pub async fn locate_hooks(context: &GitContext, cwd: &Path) -> Result<HooksLocation, String> {
    // One question per pathname: a pathname may hold the line feed that a shared report is split at.
    let common: PathBuf = path_answer_question(context, cwd, &["rev-parse", "--path-format=absolute", "--git-common-dir"]).await?;
    let git_directory: PathBuf = path_answer_question(context, cwd, &["rev-parse", "--path-format=absolute", "--absolute-git-dir"]).await?;
    // An answer with nothing left after its terminator counts as no answer, as `.filter(...)` made it.
    let configured: Option<PathBuf> = match query(context, cwd, &["config", "--type=path", "--get", "core.hooksPath"]).await? {
        Some(answer) => {
            let value: &[u8] = without_the_terminator(answer.as_slice());
            if value.is_empty() { None } else { Some(path_answer(value)) }
        }
        None => None,
    };
    let Some(chosen) = configured else {
        return Ok(HooksLocation {
            common_directory: common,
            configured: None,
        });
    };
    if chosen.is_absolute() {
        return Ok(HooksLocation {
            common_directory: common,
            configured: Some(chosen),
        });
    }
    // A relative value resolves where Git runs hooks; a repository without a worktree has no top level.
    let base: PathBuf = match path_answer_question(context, cwd, &["rev-parse", "--path-format=absolute", "--show-toplevel"]).await {
        Ok(top) => top,
        Err(_) => git_directory,
    };
    return Ok(HooksLocation {
        common_directory: common,
        configured: Some(base.join(chosen)),
    });
}

/// The four hooks `git lfs install` writes.
const LFS_SHIM_NAMES: [&str; 4] = ["pre-push", "post-checkout", "post-commit", "post-merge"];

/// The longest shim file the comparison reads,
/// in bytes;
/// a longer file is not a shim.
const LFS_SHIM_LIMIT: u64 = 1024;

/// The text git-lfs 3.8.0 writes (`lfs/hook.go`,
/// `hookBaseContent` and its `write`,
/// which appends a line feed),
///  with the hook's own name where `{{Command}}` stands.
const LFS_SHIM_TEMPLATE: &str = r##"#!/bin/sh
command -v git-lfs >/dev/null 2>&1 || { printf >&2 "\n%s\n\n" "This repository is configured for Git LFS but 'git-lfs' was not found on your path. If you no longer wish to use Git LFS, remove this hook by deleting the '{{Command}}' file in the hooks directory (set by 'core.hookspath'; usually '.git/hooks')."; exit 2; }
git lfs {{Command}} "$@"
"##;

/// What:
///  The text git-lfs writes for the hook called `name`.
/// Why:
///   Recognition is by these exact bytes,
/// so an edited or foreign file is never taken for a shim.
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
///   Only a regular file whose name is one of the four hooks and whose bytes equal that hook's shim counts;
/// a link,
///  a file that differs by one byte,
/// and a shim under another hook's name are ordinary unsupported hooks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isFile(path) && readFileSync(path).equals(lfsShimText(basename(path)))
/// ```
pub fn is_lfs_shim(path: &Path) -> std::io::Result<bool> {
    use std::io::Read;
    let Some(name) = path.file_name().and_then(std::ffi::OsStr::to_str) else {
        return Ok(false);
    };
    if !LFS_SHIM_NAMES.contains(&name) {
        return Ok(false);
    }
    let named: PathBuf = path.to_path_buf();
    let wanted: String = name.to_string();
    // The type check, the open and the read of the file all wait on the file system, so they run under the ceiling.
    let compared: Option<std::io::Result<bool>> = work_within(REPORT_SCAN_CEILING, move || {
        let Ok(metadata) = std::fs::symlink_metadata(named.as_path()) else {
            return Ok(false);
        };
        if !metadata.file_type().is_file() {
            return Ok(false);
        }
        let Ok(file) = std::fs::File::open(named.as_path()) else {
            return Ok(false);
        };
        let mut content: Vec<u8> = Vec::new();
        if file.take(LFS_SHIM_LIMIT).read_to_end(&mut content).is_err() {
            return Ok(false);
        }
        return Ok(content == lfs_shim_text(wanted.as_str()).as_bytes());
    });
    return match compared {
        Some(answer) => answer,
        None => Err(std::io::Error::new(
            ErrorKind::TimedOut,
            format!("the read of {} gave no answer within {REPORT_SCAN_CEILING:?}", path.display()),
        )),
    };
}

/// What:
///  The explanation of the Git LFS shims a repository holds.
/// Why:
///   They are listed apart from the unsupported hooks and say what is going on,
/// because the person who finds them
///  after deleting them needs to know they come back,
/// that this is normal,
/// and that nothing needs doing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const explanation = (shims: string[]) => '...';
/// ```
pub fn render_lfs_shims(shims: &[PathBuf]) -> String {
    let mut text: String = String::from(
        "git monochromatic-git-policy-cli doctor: the hook files below are the Git LFS shims, byte for byte, and nothing needs doing:\n",
    );
    for shim in shims {
        text.push_str("  ");
        text.push_str(shim.display().to_string().as_str());
        text.push('\n');
    }
    text.push_str(
        "git-lfs writes these files again whenever it cleans a file in a Git that monochromatic-git-policy-cli did not start (for example \
`git add` of an LFS file run as `/usr/bin/git` directly), so they can come back after you delete them; this is \
normal. monochromatic-git-policy-cli uploads the LFS objects itself before every push and starts Git with traditional hooks off, so these \
files never run through monochromatic-git-policy-cli. You may delete them or leave them. Only a script that starts Git without monochromatic-git-policy-cli, \
such as `/usr/bin/git` directly, would run them.\n",
    );
    return text;
}

/// What:
///  Decide what `git monochromatic-git-policy-cli doctor` prints and exits with,
/// entering the process's runtime once for the whole report.
/// Why:
///   Exit 0 with no output means no traditional hook was found,
/// exit 1 lists each one on standard output,
/// and exit 2
///  reports a repository or a directory that could not be examined.
///  Git LFS shims are explained on standard error apart from the other hooks and leave an exit status of 0 as it is;
///  any other hook beside them still ends in 1 with the unsupported report.
///  The report is built on the runtime (`doctor_action_with_section`),
/// so this face is the one a caller that is still synchronous uses,
/// and it goes with the last of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function doctor({ context, cwd }): Promise<Action>;
/// ```
pub fn doctor_action(context: &GitContext, cwd: &Path) -> Action {
    return match run_on_runtime(doctor_action_with_section(context, cwd, None, &no_filesystem_section)) {
        Ok(action) => action,
        Err(refusal) => doctor_failure(format!("the runtime could not be entered: {refusal}").as_str()),
    };
}

/// What:
///  A filesystem section that is always empty.
/// Why:
///   `doctor_action` prints no filesystem section,
/// and `doctor_action_with_section` takes its section as a function,
///  so this named function stands where a closure that ignored its arguments stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noFilesystemSection = (_commonDirectory: string, _home: string | undefined) => '';
/// ```
fn no_filesystem_section(_common_directory: &Path, _home: Option<&Path>) -> String {
    return String::new();
}

/// What:
///  The same decision with a filesystem section appended to standard error.
///  `&dyn Fn(&Path, Option<&Path>) -> String` is a borrowed function from the repository's common Git directory and
/// the home the invocation names to the section text.
/// Why:
///   The section is informational and comes last,
/// after every hook line and note;
/// a run that could not examine the
///  repository (status 2) prints none,
/// and the exit status never depends on it.
/// Git selected the common directory
///  (so `-C`,
/// `--git-dir` and linked worktrees are honored),
/// and the caller supplies the text function and the home the invocation names,
/// so the
///  exact-output controls pass one that returns nothing and stay independent of the host.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function doctorWithSection({ context, cwd, home, section }): Promise<Action>;
/// ```
pub async fn doctor_action_with_section(
    context: &GitContext,
    cwd: &Path,
    home: Option<&Path>,
    filesystem_section: &dyn Fn(&Path, Option<&Path>) -> String,
) -> Action {
    let location: HooksLocation = match locate_hooks(context, cwd).await {
        Ok(found) => found,
        Err(message) => return doctor_failure(message.as_str()),
    };
    // The scan reads directories and shim files under its ceiling (`scan_hooks_directory`), so a mount that stops
    // answering ends the read instead of the report.
    let found: Vec<PathBuf> = match find_traditional_hooks(hook_directories(&location).as_slice()) {
        Ok(listed) => listed,
        Err(error) => return doctor_failure(format!("cannot read a hooks directory: {error}").as_str()),
    };
    let mut shims: Vec<PathBuf> = Vec::new();
    let mut hooks: Vec<PathBuf> = Vec::new();
    for path in found {
        match is_lfs_shim(path.as_path()) {
            Ok(true) => shims.push(path),
            Ok(false) => hooks.push(path),
            Err(error) => return doctor_failure(format!("cannot read a hook file: {error}").as_str()),
        }
    }
    let shim_report: String = if shims.is_empty() { String::new() } else { render_lfs_shims(shims.as_slice()) };
    let findings: ConfigFindings = config_findings(context, cwd).await;
    let notes: String = format!("{}{}", render_notes(&findings), fsmonitor_report(context, cwd).await);
    // A report changes nothing, so the earlier release's names in this directory are read where they are and never moved
    // (`SPEC.md`, section Doctor): moving them is left to the commands that write there.
    let discarded: String = doctor_section(location.common_directory.as_path(), findings.discarded);
    // The host facts read `statfs` and the mount table under their own ceiling (`host_section`).
    let section: String = filesystem_section(location.common_directory.as_path(), home);
    if hooks.is_empty() && findings.hooks.is_empty() {
        return Action::Exit {
            code: 0,
            stdout: String::new(),
            stderr: format!("{shim_report}{notes}{discarded}{section}"),
        };
    }
    let explanation: &str = if hooks.is_empty() { "" } else { FOUND_EXPLANATION };
    return Action::Exit {
        code: HOOKS_FOUND_EXIT_CODE,
        stdout: format!("{}{}", render_hooks(hooks.as_slice()), render_hook_lines(&findings)),
        stderr: format!("{explanation}{shim_report}{notes}{discarded}{section}"),
    };
}

/// What:
///  The action for a doctor run that could not examine the repository.
/// Why:
///   Status 2 is the wrapper's status for a failure that is not a finding.
/// The command's own prefix is used here and not the wrapper's,
/// so a report that could not be started reads like every other doctor failure
/// (`invocation/management.rs`,
/// `plan_doctor`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { code: 2, stdout: '', stderr: `git monochromatic-git-policy-cli doctor: ${message}\n` }
/// ```
pub(crate) fn doctor_failure(message: &str) -> Action {
    return Action::Exit {
        code: ENGINE_FAILURE_EXIT_CODE,
        stdout: String::new(),
        stderr: format!("git monochromatic-git-policy-cli doctor: {message}\n"),
    };
}

/// Doctor controls stay out of the release executable.
#[cfg(test)]
#[path = "doctor_tests.rs"]
mod tests;
