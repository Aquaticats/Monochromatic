//! What:
//!  What `git monochromatic-git-policy-cli doctor` reports from the configuration: the external hooks Git configuration defines, and the
//!  retired `hooks.concurrentCommits`, `indexLock.unprovenOwnerTimeoutMs` and `landing.reserveAfterLostRaces` keys.
//! Why:
//!  The wrapper runs no external hook, so a configured hook is a finding the author must act on, and each retired key in
//!  `monochromatic-git-policy-cli.config.jsonc` is accepted for older files but means nothing; doctor says both plainly.
//!  The commit path prints the first as a warning from the snapshot it already holds; doctor lists it on request.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const findings = await configFindings({ context, cwd }); // { hooks: [...], retiredHooksKey, retiredIndexLockKey, retiredLandingKey }
//! ```

use crate::config_file::load_repository_config;
use crate::config_schema::{CliGitConfig, ConcurrencyConfig};
use crate::discarded_prune::Limits;
use crate::config_snapshot::read_config_snapshot;
use crate::external_hooks::{ConfiguredHook, configured_hooks, describe_hook, explain_external_hooks};
use crate::transaction_git::{GitContext, GitRequest, run_git};
use std::path::{Path, PathBuf};

/// What:
///  What the configuration contributes to the doctor report.
/// Why:
///   Standard output lists the hooks one per line, and standard error carries the explanation and the notes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfigFindings = { hooks: ConfiguredHook[]; retiredHooksKey: boolean;
///   retiredIndexLockKey: boolean; retiredLandingKey: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigFindings {
    /// The configured external hooks, none of which runs.
    pub hooks: Vec<ConfiguredHook>,
    /// Whether `monochromatic-git-policy-cli.config.jsonc` has the retired `hooks` key.
    pub retired_hooks_key: bool,
    /// Whether `monochromatic-git-policy-cli.config.jsonc` sets the retired `indexLock.unprovenOwnerTimeoutMs`.
    pub retired_index_lock_key: bool,
    /// Whether `monochromatic-git-policy-cli.config.jsonc` sets the retired `landing.reserveAfterLostRaces`.
    pub retired_landing_key: bool,
    /// The limits of the folder of saved copies in force: the file's `discardedCopies`, else the defaults.
    pub discarded: Limits,
}

/// What:
///  The worktree top level, when Git reports one.
/// Why:
///   The JSONC file sits at the top level; a bare repository has none and so no file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['rev-parse', '--show-toplevel']).catch(() => undefined)
/// ```
fn top_level(context: &GitContext, cwd: &Path) -> Option<PathBuf> {
    let request: GitRequest = GitRequest::new(cwd, &["rev-parse", "--show-toplevel"]);
    let output = run_git(context, &request).ok()?;
    if !output.succeeded() {
        return None;
    }
    let text: String = String::from_utf8(output.stdout).ok()?;
    let trimmed: &str = text.trim_end_matches('\n');
    if trimmed.is_empty() {
        return None;
    }
    return Some(PathBuf::from(trimmed));
}

/// What:
///  Read the configuration findings for `cwd`.
/// Why:
///   One snapshot read gives the hooks; one file read gives the retired key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// configFindings({ context, cwd });
/// ```
pub fn config_findings(context: &GitContext, cwd: &Path) -> ConfigFindings {
    // `match` on the `Option`: a snapshot that could not be read gives no hooks, as `.unwrap_or_default()` did.
    let hooks: Vec<ConfiguredHook> = match read_config_snapshot(context, cwd) {
        Some(snapshot) => configured_hooks(&snapshot),
        None => Vec::new(),
    };
    // A configuration that cannot be loaded is not doctor's to report: the command works beside a broken file, and
    // every other command names the problem.
    let loaded: Option<CliGitConfig> = match top_level(context, cwd) {
        Some(root) => match load_repository_config(root.as_path()) {
            Ok(found) => Some(found.config),
            Err(_) => None,
        },
        None => None,
    };
    let (retired_hooks_key, retired_index_lock_key, retired_landing_key): (bool, bool, bool) = match loaded.as_ref() {
        Some(found) => {
            let retired: ConcurrencyConfig = found.concurrency;
            (retired.hooks.present, retired.index_lock.present, retired.landing.present)
        }
        None => (false, false, false),
    };
    let discarded: Limits = match loaded {
        Some(found) => found.discarded,
        None => Limits::default(),
    };
    return ConfigFindings {
        hooks,
        retired_hooks_key,
        retired_index_lock_key,
        retired_landing_key,
        discarded,
    };
}

/// What:
///  The lines for standard output: one per configured hook.
/// Why:
///   The hooks sit beside the traditional hooks in the report a script can read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// findings.hooks.map(describeHook).join('\n')
/// ```
pub fn render_hook_lines(findings: &ConfigFindings) -> String {
    let mut text: String = String::new();
    for hook in &findings.hooks {
        text.push_str(describe_hook(hook).as_str());
        text.push('\n');
    }
    return text;
}

/// What:
///  The explanation and notes for standard error.
/// Why:
///   The ways forward for hooks, and a note for each retired key that is accepted and ignored, with the remedy of
///  removing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// renderNotes(findings);
/// ```
pub fn render_notes(findings: &ConfigFindings) -> String {
    let mut text: String = String::new();
    if !findings.hooks.is_empty() {
        text.push_str("git monochromatic-git-policy-cli doctor: ");
        text.push_str(explain_external_hooks(findings.hooks.as_slice()).as_str());
        text.push('\n');
    }
    if findings.retired_hooks_key {
        text.push_str(
            "git monochromatic-git-policy-cli doctor: monochromatic-git-policy-cli.config.jsonc has a `hooks` key (`hooks.concurrentCommits`). It is accepted so \
             older files still load, and ignored: the wrapper runs no external hook, so there is no hook lock to \
             tune. Delete the key when convenient.\n",
        );
    }
    if findings.retired_index_lock_key {
        text.push_str(
            "git monochromatic-git-policy-cli doctor: monochromatic-git-policy-cli.config.jsonc sets `indexLock.unprovenOwnerTimeoutMs`. The key does nothing: the \
             commit service waits for another process's `index.lock` for as long as the lock stays there, whether the process \
             is proven to hold it or not, says what it knows about the owner while it waits, and reads no timeout from this \
             file. Remove the key from monochromatic-git-policy-cli.config.jsonc.\n",
        );
    }
    if findings.retired_landing_key {
        text.push_str(
            "git monochromatic-git-policy-cli doctor: monochromatic-git-policy-cli.config.jsonc sets `landing.reserveAfterLostRaces`. The key does nothing: the \
             commit service lands commits in the order they arrive and no longer reserves a landing slot for a commit \
             that lost races. Remove the key from monochromatic-git-policy-cli.config.jsonc.\n",
        );
    }
    return text;
}

/// Reading the findings is controlled in the test module.
#[cfg(test)]
#[path = "doctor_config_tests.rs"]
mod tests;
