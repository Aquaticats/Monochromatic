//! What:
//!  Load the policy configuration of the worktree one invocation runs in,
//!  render
//!       the events configuration loading can produce,
//!  and word the hint each of those events carries.
//! Why:
//!  Both wrapped Git commands and `git monochromatic-git-policy-cli check`/`fix` need the same answer:
//!      what does the `monochromatic-git-policy-cli.config.jsonc` of the worktree Git reported say.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const loaded = await loadIdentityConfig(location.identity);
//! ```

/// Import the sibling modules this lookup combines.
use crate::config_error::ConfigError;
use crate::config_file::{
    LoadedConfig,
    ignored_legacy_notice,
    load_repository_config,
};
use crate::config_schema::CliGitConfig;
use crate::diagnostics::{
    EngineFailureCode,
    LEGACY_CONFIG_IGNORED_CODE,
    render_configuration_warning,
    render_engine_failure,
};
/// The JSON string escaping of a hint's text.
use crate::json_record::quote;
use crate::worktree_identity::{
    WorktreeIdentity,
    worktree_root,
};
/// `PathBuf` is an owned filesystem path of raw OS bytes.
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{
    AtomicBool,
    Ordering,
};

/// What:
///  Load the configuration for the worktree Git reported for this invocation.
///       `&WorktreeIdentity` borrows that answer;
///       `Result<LoadedConfig, ConfigError>` is the settings or the rejection.
/// Why:
///   The caller already asked Git where the command runs,
///  once,
///  so this function
///       starts no process.
///  Outside a worktree (no repository,
///  a bare repository,
///  the
///       inside of `.git`) there is no configuration file and the defaults apply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function loadIdentityConfig(identity: WorktreeIdentity): Promise<LoadedConfig>;
/// ```
pub fn load_identity_config(identity: &WorktreeIdentity) -> Result<LoadedConfig, ConfigError> {
    // `let Some(root) = ... else { ... };` unwraps the worktree top level or exits.
    let Some(root) = worktree_root(identity) else {
        // `Ok(...)` is the success variant: no worktree means the defaults.
        return Ok(LoadedConfig {
            config: CliGitConfig::defaults(),
            // `None` records that no file was read.
            source: None,
            // `Vec::new()` is the empty owned list.
            ignored_legacy: Vec::<PathBuf>::new(),
            earlier_name_notice: None,
        });
    };
    let loaded: LoadedConfig = load_repository_config(root)?;
    // The one line about the settings file's name is said once per process, however many times the settings are loaded.
    if let Some(notice) = &loaded.earlier_name_notice
        && !EARLIER_NAME_SAID.swap(true, Ordering::Relaxed)
    {
        let _written = std::io::stderr().write_all(notice.as_bytes());
    }
    return Ok(loaded);
}

/// What:
///  Whether this process has already told the person about the settings file's name.
///  `AtomicBool` is a flag that threads share without a lock.
/// Why:
///   One invocation loads the settings more than once;
///  the owner decided on "one notice".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let earlierNameSaid = false;
/// ```
static EARLIER_NAME_SAID: AtomicBool = AtomicBool::new(false);

/// What:
///  Render the `config-invalid` event for a rejected configuration.
///       `String` is the owned,
///  line-terminated JSON line.
/// Why:
///   Configuration failures are machine-readable events with exit status 2,
///  on
///       standard error for wrapped commands and standard output for direct commands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function configInvalidEvent(error: ConfigError): string;
/// ```
pub fn config_invalid_event(error: &ConfigError) -> String {
    return render_engine_failure(0, EngineFailureCode::ConfigInvalid, error.message.as_str());
}

/// What:
///  Render one warning event per legacy file left beside the JSONC file.
/// Why:
///   The stale executable configuration is reported on every configuration load,
///       never ignored silently;
///  events are numbered from zero in file order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function legacyWarningEvents(loaded: LoadedConfig): string;
/// ```
pub fn legacy_warning_events(loaded: &LoadedConfig) -> String {
    // `String::new()` is empty owned text; `mut` allows appending.
    let mut events: String = String::new();
    // `if let Some(source) = &loaded.source` borrows the JSONC path when a file was read.
    if let Some(source) = &loaded.source {
        // `.iter().enumerate()` yields `(index, item)` pairs; `usize` is the index type.
        for (index, legacy) in loaded.ignored_legacy.iter().enumerate() {
            events.push_str(
                render_configuration_warning(
                    // `as u64` widens the index to the event number type.
                    index as u64,
                    LEGACY_CONFIG_IGNORED_CODE,
                    ignored_legacy_notice(legacy.as_path(), source.as_path()).as_str(),
                    // `.to_string_lossy()` renders the path as text, replacing undecodable bytes.
                    legacy.to_string_lossy().as_ref(),
                )
                .as_str(),
            );
        }
    }
    return events;
}

/// What:
///  The hint for an engine failure,
/// by the code's wire spelling.
/// Why:
///   An engine failure stops the command before Git runs;
/// the person needs to know whether to retry,
/// fix a file,
/// or
///  report a defect.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function engineFailureHint(code: string): string | undefined;
/// ```
pub fn engine_failure_hint(code: &str) -> Option<&'static str> {
    match code {
        "config-invalid" => {
            return Some(
                "Hint: if you just edited monochromatic-git-policy-cli.config.jsonc, fix the key the message names or revert the edit; \
                 an unknown key, a wrong type or an unknown policy id fails before any policy runs.",
            );
        }
        "core-incomplete" => {
            return Some(
                "Hint: if another session was changing the repository at the same moment (a commit, a checkout or a \
                 rebase running elsewhere), run the command again; if it repeats, run `git status` to check that the \
                 repository is healthy.",
            );
        }
        "content-unavailable" => {
            return Some(
                "Hint: if you are committing or pushing content that is not in this clone (a missing object, a \
                 shallow history, or a file removed during the command), fetch the history with `git fetch`, make \
                 sure the files exist, and retry.",
            );
        }
        "policy-incomplete" => {
            return Some(
                "Hint: if you are committing or pushing, a policy could not finish reading the content, so it \
                 decided nothing; fix the cause the message names (an unreadable file, a path with a line break) \
                 and retry.",
            );
        }
        "patch-invalid" | "patch-conflict" => {
            return Some(
                "Hint: if you are committing files that changed while the command ran or that another policy also \
                 rewrites, save your edits, run `git monochromatic-git-policy-cli fix -- <path>` to apply the corrections, review the \
                 result with `git diff`, and commit again.",
            );
        }
        "fix-cycle" | "fix-pass-limit" => {
            return Some(
                "Hint: two corrections keep undoing each other on this file; apply one of them by hand, stage the \
                 file, and commit again. Report the pair of policies if the file is ordinary text.",
            );
        }
        "transaction-failed" => {
            return Some(
                "Hint: if you were committing, run `git status` and `git log --oneline -3` to see whether the \
                 commit landed before you retry; staged files stay staged unless the message says otherwise, and the \
                 message names the step that failed.",
            );
        }
        "legacy-config-ignored" => {
            return Some(
                "Hint: if you are still keeping the old configuration files, delete them once \
                 monochromatic-git-policy-cli.config.jsonc holds the settings; they are no longer read.",
            );
        }
        "traditional-hooks-unsupported" => {
            return Some(
                "Hint: scripts in the hooks directory never run through monochromatic-git-policy-cli. If one still checks something for you, \
                 express it as a policy under policies in monochromatic-git-policy-cli.config.jsonc; `git monochromatic-git-policy-cli doctor` lists the \
                 scripts it finds and exits 1 while any remain.",
            );
        }
        _ => return None,
    }
}

/// What:
///  Append the `hint` field to a JSON event line when a hint exists.
///  `&mut String` lends the line for appending.
/// Why:
///   The `message` field stays byte-identical to the retired wrapper's;
/// the hint is an additive field that readers of
///  the line see without any change to the message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (hint !== undefined) event.hint = hint;
/// ```
pub fn push_hint_field(
    line: &mut String,
    hint: Option<&str>,
) {
    if let Some(text) = hint {
        line.push_str(",\"hint\":");
        line.push_str(quote(text).as_str());
    }
}

/// Disposable-repository controls stay out of the release executable.
#[cfg(test)]
#[path = "invocation_config_tests.rs"]
mod tests;
