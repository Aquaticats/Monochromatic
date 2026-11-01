//! What:
//!  What `git cli-git doctor` reports from the configuration: the external hooks Git configuration defines, and the
//!  retired `hooks.concurrentCommits` key.
//! Why:
//!  The wrapper runs no external hook, so a configured hook is a finding the author must act on, and a `hooks` key in
//!  `cli-git.config.jsonc` is accepted for older files but means nothing; doctor says both plainly.
//!  The commit path prints the first as a warning from the snapshot it already holds; doctor lists it on request.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const findings = await configFindings({ context, cwd }); // { hooks: [...], retiredHooksKey: boolean }
//! ```

use super::config_file::load_repository_config;
use super::config_snapshot::read_config_snapshot;
use super::external_hooks::{ConfiguredHook, configured_hooks, describe_hook, explain_external_hooks};
use super::transaction_git::{GitContext, GitRequest, run_git};
use std::path::{Path, PathBuf};

/// What:
///  What the configuration contributes to the doctor report.
/// Why:
///   Standard output lists the hooks one per line, and standard error carries the explanation and the notes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfigFindings = { hooks: ConfiguredHook[]; retiredHooksKey: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigFindings {
    /// The configured external hooks, none of which runs.
    pub hooks: Vec<ConfiguredHook>,
    /// Whether `cli-git.config.jsonc` has the retired `hooks` key.
    pub retired_hooks_key: bool,
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
    let hooks: Vec<ConfiguredHook> = read_config_snapshot(context, cwd)
        .map(|snapshot| { return configured_hooks(&snapshot); })
        .unwrap_or_default();
    // A configuration that cannot be loaded is not doctor's to report: the command works beside a broken file, and
    // every other command names the problem.
    let retired_hooks_key: bool = top_level(context, cwd)
        .and_then(|root| { return load_repository_config(root.as_path()).ok(); })
        .is_some_and(|loaded| { return loaded.config.concurrency.hooks.present; });
    return ConfigFindings { hooks, retired_hooks_key };
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
///   The ways forward for hooks, and the note that `hooks.concurrentCommits` is accepted and ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// renderNotes(findings);
/// ```
pub fn render_notes(findings: &ConfigFindings) -> String {
    let mut text: String = String::new();
    if !findings.hooks.is_empty() {
        text.push_str("git cli-git doctor: ");
        text.push_str(explain_external_hooks(findings.hooks.as_slice()).as_str());
        text.push('\n');
    }
    if findings.retired_hooks_key {
        text.push_str(
            "git cli-git doctor: cli-git.config.jsonc has a `hooks` key (`hooks.concurrentCommits`). It is accepted so \
             older files still load, and ignored: the wrapper runs no external hook, so there is no hook lock to \
             tune. Delete the key when convenient.\n",
        );
    }
    return text;
}

/// Reading the findings is controlled in the test module.
#[cfg(test)]
#[path = "doctor_config_tests.rs"]
mod tests;
