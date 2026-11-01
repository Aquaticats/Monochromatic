//! What:
//!  Load the policy configuration of the worktree one invocation runs in,
//!  and render
//!       the events configuration loading can produce.
//! Why:
//!  Both wrapped Git commands and `git cli-git check`/`fix` need the same answer:
//!      what does the `cli-git.config.jsonc` of the worktree Git reported say.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const loaded = await loadIdentityConfig(location.identity);
//! ```

/// Import the sibling modules this lookup combines.
use super::config_error::ConfigError;
use super::config_file::{LoadedConfig, ignored_legacy_notice, load_repository_config};
use super::config_schema::CliGitConfig;
use super::diagnostics::{
    EngineFailureCode, LEGACY_CONFIG_IGNORED_CODE, render_configuration_warning,
    render_engine_failure,
};
use super::worktree_identity::{WorktreeIdentity, worktree_root};
/// `PathBuf` is an owned filesystem path of raw OS bytes.
use std::path::PathBuf;

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
        });
    };
    return load_repository_config(root);
}

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

/// Disposable-repository controls stay out of the release executable.
#[cfg(test)]
#[path = "invocation_config_tests.rs"]
mod tests;
