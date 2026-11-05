//! What:
//!  The real Git version check,
//!  run once per invocation before work that depends on Git's
//!       behavior:
//!  a commit,
//!  hooks and locks.
//! Why:
//!  The native tables and protocols follow the Git release the test image runs the tests with,
//!  and a Git older than that release is not refused:
//!  the owner chose to warn once and go on
//!  ("Raise minimum, drop the test, merely print a warning on older git and not refuse.",
//!  2026-11-04),
//!  so the constant below is the release the wrapper was tested with.
//!  Only a Git whose release cannot be read stops the command,
//!  because nothing is known about it.
//!  Read-only commands are never checked
//!      (`doc/handover/cli-git-rust-implementation.md`,
//!  "Adopted without a question";
//!  that section moved on 2026-11-03 to
//!  `doc/planning/cli-git-rust-open-decisions.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await requireReadableGit({ gitPath }); // once; warns when the release is older than the tested one
//! ```

/// The per-executable cache of the version text.
use crate::git_version_cache::{binary_identity, cache_directory, read_version_text, store_version_text};
/// Running real Git.
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
/// OS strings and paths for the cache location.
use std::ffi::OsString;
use std::io::Write;
use std::path::Path;
/// `OnceLock` holds the one answer of this process.
use std::sync::OnceLock;

/// The Git release the wrapper was tested with, and the minimum below which every run prints a warning.
///
/// Keep it in step with the test image: the gate's base image installs the distribution's current `git` package
///  (`native-test-base.Containerfile`), and `mise run //package/git-policy/cli:test:git-image` prints the release it
///  got. When that release moves past this one, change this constant to it in the same commit as the first gate run
///  on the new release, together with the release named in `README.md` and `SPEC.md` ("tested with").
pub const MINIMUM_GIT_VERSION: (u64, u64, u64) = (2, 56, 0);

/// A Git release as its major,
///  minor and patch numbers.
pub type GitRelease = (u64, u64, u64);

/// The answer of this process's one check.
static CHECKED: OnceLock<Result<GitRelease, String>> = OnceLock::new();

/// What:
///  The release numbers of `git version` output,
///  such as `git version 2.56.0.windows.1`.
/// Why:
///   Only the first three dotted numbers decide support;
///  platform suffixes vary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseGitVersion(output: string): [number, number, number] | undefined;
/// ```
pub fn parse_git_version(output: &[u8]) -> Option<(u64, u64, u64)> {
    let text: &str = std::str::from_utf8(output).ok()?;
    let release: &str = text.strip_prefix("git version ")?.split_whitespace().next()?;
    let mut parts: std::str::Split<'_, char> = release.split('.');
    let major: u64 = parts.next()?.parse::<u64>().ok()?;
    let minor: u64 = parts.next()?.parse::<u64>().ok()?;
    let patch_text: &str = parts.next()?;
    // A release candidate such as `2.56.0-rc1` keeps its leading digits.
    let digits: &str = &patch_text[..patch_text.find(not_digit).unwrap_or(patch_text.len())];
    let patch: u64 = digits.parse::<u64>().ok()?;
    return Some((major, minor, patch));
}

/// What:
///  Whether a character is not an ASCII digit.
/// Why:
///   A named predicate keeps the search free of closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (character: string) => !/[0-9]/.test(character)
/// ```
fn not_digit(character: char) -> bool {
    return !character.is_ascii_digit();
}

/// What:
///  The message for a Git whose release cannot be read.
/// Why:
///   The person must know which Git was asked and what to do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: real Git at ${path} reports ${found}, so its release cannot be read ...`
/// ```
pub fn unsupported_message(context: &GitContext, found: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: real Git at {} reports {found}, so its release cannot be read; commits, hooks and the \
         locks that coordinate with them depend on it. Nothing was run. Point the wrapper at a working \
         Git, then run the command again.",
        context.real_git.display()
    );
}

/// What:
///  The warning for a Git older than the release the wrapper was tested with,
///  or `None` for that release and every later one.
/// Why:
///   An older Git is not refused;
///  the person learns which Git was found,
///  which release the wrapper was tested with,
///  and that the command continues.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function olderGitWarning(context, release): string | undefined;
/// ```
pub fn older_git_warning(context: &GitContext, release: GitRelease) -> Option<String> {
    if release >= MINIMUM_GIT_VERSION {
        return None;
    }
    return Some(format!(
        "monochromatic-git-policy-cli: warning: real Git at {} is version {}.{}.{}, older than {}.{}.{}, the release this wrapper was tested \
         with; the command continues with it.\n",
        context.real_git.display(),
        release.0,
        release.1,
        release.2,
        MINIMUM_GIT_VERSION.0,
        MINIMUM_GIT_VERSION.1,
        MINIMUM_GIT_VERSION.2
    ));
}

/// What:
///  Read the release from the text `git version` printed.
/// Why:
///   One judgment serves the text a Git process printed and the text the cache kept,
///  so both paths answer the same release or refuse the same unreadable text with the same message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function judgeGitVersion(context, stdout): [number, number, number]; // throws the unsupported message
/// ```
fn judge_version_text(context: &GitContext, stdout: &[u8]) -> Result<GitRelease, String> {
    let Some(version) = parse_git_version(stdout) else {
        let shown: String = String::from_utf8_lossy(stdout).trim().to_string();
        return Err(unsupported_message(context, format!("an unreadable version {shown:?}").as_str()));
    };
    return Ok(version);
}

/// What:
///  Start `git version` and return what it printed and whether it exited with success.
/// Why:
///   Only a successful run is worth caching;
///  a failed start is the unsupported message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { stdout, exitCode } = await runGit({ args: ['version'] });
/// ```
fn run_git_version(context: &GitContext) -> Result<GitOutput, String> {
    let mut request: GitRequest = GitRequest::new(std::path::Path::new("."), &["version"]);
    request.without_prefix = true;
    return match run_git(context, &request) {
        Ok(finished) => Ok(finished),
        Err(error) => Err(unsupported_message(context, format!("nothing ({error})").as_str())),
    };
}

/// What:
///  Ask Git its version without the cache.
/// Why:
///   `check_git_version` remembers this answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function askGitVersion(gitPath): Promise<[number, number, number]>;
/// ```
pub fn ask_git_version(context: &GitContext) -> Result<(u64, u64, u64), String> {
    let output: GitOutput = run_git_version(context)?;
    return judge_version_text(context, output.stdout.as_slice());
}

/// What:
///  Ask for the Git version through the per-executable cache in `cache_directory`,
///  starting `git version` only on a miss.
/// Why:
///   The text is read from the cache only when the executable's identity matches the recorded one exactly
///  and the text still parses;
///  a stale,
///  unreadable or absent entry,
///  an executable that is not identifiable,
///  or no cache directory all run Git.
///  The identity is taken before the start and again after it,
///  and the text is recorded only when both agree and Git succeeded,
///  so an executable replaced during the start is never recorded under the old identity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function askGitVersionCached(context, cacheDirectory?: string): Promise<[number, number, number]>;
/// ```
pub fn ask_git_version_cached(context: &GitContext, cache_directory: Option<&Path>) -> Result<GitRelease, String> {
    let Some(directory) = cache_directory else {
        return ask_git_version(context);
    };
    let Some(identity) = binary_identity(context.real_git.as_path()) else {
        return ask_git_version(context);
    };
    if let Some(text) = read_version_text(directory, &identity)
        && parse_git_version(text.as_slice()).is_some()
    {
        return judge_version_text(context, text.as_slice());
    }
    let output: GitOutput = run_git_version(context)?;
    let judged: Result<GitRelease, String> = judge_version_text(context, output.stdout.as_slice());
    let unchanged: bool = binary_identity(context.real_git.as_path()).as_ref() == Some(&identity);
    if judged.is_ok() && output.succeeded() && unchanged {
        store_version_text(directory, &identity, output.stdout.as_slice());
    }
    return judged;
}

/// What:
///  The process-wide version ask:
///  the cache in the user's cache directory when the environment names one,
///  and the warning for an older Git printed once to standard error.
/// Why:
///   `check_git_version` takes only the Git context;
///  the environment is the process's own,
///  as the Git children inherit it.
///  Unit tests of the library never touch the real home, so they ask without a cache.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// askGitVersionCached(context, cacheDirectory(process.env))
/// ```
fn ask_git_version_with_process_cache(context: &GitContext) -> Result<GitRelease, String> {
    let answer: Result<GitRelease, String> = if cfg!(test) {
        ask_git_version(context)
    } else {
        let environment: Vec<(OsString, OsString)> = std::env::vars_os().collect();
        ask_git_version_cached(context, cache_directory(environment.as_slice()).as_deref())
    };
    // This function runs once per process (`check_git_version` keeps its answer), so the warning prints once.
    if let Ok(release) = &answer
        && let Some(warning) = older_git_warning(context, *release)
    {
        let mut stream = std::io::stderr();
        let _ = stream.write_all(warning.as_bytes());
        let _ = stream.flush();
    }
    return answer;
}

/// What:
///  Require a Git whose release can be read,
///  asking at most once per process.
/// Why:
///   The check costs a Git process,
///  so later dependent steps reuse the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await requireReadableGit({ gitPath });
/// ```
pub fn check_git_version(context: &GitContext) -> Result<(u64, u64, u64), String> {
    return check_once(&CHECKED, context, ask_git_version_with_process_cache);
}

/// What:
///  Return the answer already stored in `cell`,
///  or ask once and store the answer.
/// Why:
///   `check_git_version` keeps one answer per process;
///  a separate cell lets the controls
///       prove the ask runs once without sharing the process-wide answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const checked ??= await askGitVersion(context);
/// ```
pub fn check_once(
    cell: &OnceLock<Result<GitRelease, String>>,
    context: &GitContext,
    ask: fn(&GitContext) -> Result<GitRelease, String>,
) -> Result<GitRelease, String> {
    if let Some(known) = cell.get() {
        return known.clone();
    }
    let answer: Result<(u64, u64, u64), String> = ask(context);
    // Another thread may have set it meanwhile; the first answer stands.
    let _ = cell.set(answer.clone());
    return answer;
}

/// Version controls stay out of the release executable.
#[cfg(test)]
#[path = "git_version_tests.rs"]
mod tests;
