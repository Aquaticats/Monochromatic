//! What:
//!  Controls on this repository's own `cli-git.config.jsonc`, read through the real loader.
//! Why:
//!  The native wrapper runs the four optional policies only when the file lists them, and reads the private rules
//!      file only when `rulesFile` names it.
//!  A line dropped from the file would silently stop a policy the incumbent enforced,
//!  so the file the repository
//!      ships is checked here instead of an inline copy of it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const { config } = await loadRepositoryConfig(repositoryRoot);
//! // expect(config.policies['mono/dependent-version-bump'].explicit).toBe(true);
//! ```

/// Import the loader and the settings it produces.
use super::{CONFIG_FILE_NAME, LoadedConfig, load_repository_config};
use crate::config_schema::{
    ForbiddenStringsOptions, MarkdownAutofixOptions, MarkdownRule, PolicySetting, RulesFileEntry,
};
use crate::policy_registry::{PolicyId, Severity};
use std::path::PathBuf;

/// What:
///  The repository top level:
///  this crate sits at `package/git-policy/cli`,
///  three directories below it.
///       `env!` reads a Cargo-provided variable at build time.
/// Why:
///   The gate image copies the file to the same relative place,
///  so one expression serves the host and the image.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repositoryRoot = resolve(packageDirectory, '../../..');
/// ```
fn repository_root() -> PathBuf {
    return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
}

/// Load the shipped file;
///  a missing or invalid file fails the test with the path and the loader's words.
fn load_shipped() -> LoadedConfig {
    let root: PathBuf = repository_root();
    return load_repository_config(root.as_path()).unwrap_or_else(|error| {
        panic!(
            "the repository's {CONFIG_FILE_NAME} under {} must load: {}",
            root.display(),
            error.message
        )
    });
}

/// The file the loader read is the repository's JSONC file,
///  not a default.
#[test]
fn the_repository_ships_a_configuration_file() {
    let loaded: LoadedConfig = load_shipped();
    assert_eq!(
        loaded.source,
        Some(repository_root().join(CONFIG_FILE_NAME))
    );
}

/// All four optional policies are listed,
///  at the severities the incumbent enforced.
///  An unlisted optional policy is off,
///  so an unlisted one fails this test.
#[test]
fn the_repository_configuration_lists_all_four_optional_policies() {
    let loaded: LoadedConfig = load_shipped();
    for (id, severity) in [
        (PolicyId::MarkdownAutofix, Severity::Warn),
        (PolicyId::ForbiddenRootContext, Severity::Error),
        (PolicyId::DependentVersionBump, Severity::Error),
        (PolicyId::ForbiddenStrings, Severity::Error),
    ] {
        assert_eq!(
            loaded.config.policies.setting(id),
            PolicySetting {
                id,
                severity,
                explicit: true
            },
            "{id:?} must be listed in the repository configuration at the incumbent's severity"
        );
    }
}

/// The scanner reads the rules file the repository generates,
///  without depending on an environment variable,
///  and loads the embedded baseline.
#[test]
fn the_repository_configuration_sets_rules_file() {
    let loaded: LoadedConfig = load_shipped();
    assert_eq!(
        loaded.config.policies.forbidden_strings,
        ForbiddenStringsOptions {
            builtin_rules: true,
            rules_files: Some(vec![RulesFileEntry {
                path: String::from(".cache/forbidden-strings.rules.txt"),
                optional: false,
            }]),
        }
    );
}

/// The Markdown policy keeps the rule and the exclusion the incumbent configuration states.
#[test]
fn the_repository_configuration_keeps_the_markdown_options() {
    let loaded: LoadedConfig = load_shipped();
    assert_eq!(
        loaded.config.policies.markdown_autofix,
        MarkdownAutofixOptions {
            rules: vec![MarkdownRule::LfsImageUrl],
            exclude: vec![String::from("package/ssg/")],
        }
    );
}
