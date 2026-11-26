//! What:
//!  Disposable-repository controls for per-invocation configuration loading.
//! Why:
//!  The worktree Git reports decides which file is read,
//!  and a rejected file must
//!      reach the caller as an error and an event naming it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect((await loadIdentityConfig(identityOf(['-C', nested]))).source).toBe(join(repo, 'monochromatic-git-policy-cli.config.jsonc'));
//! ```
#![cfg(unix)]

/// Import the lookup under test and shared fixtures.
use super::{
    config_invalid_event,
    engine_failure_hint,
    legacy_warning_events,
    load_identity_config,
    push_hint_field,
};
use crate::config_error::ConfigError;
use crate::config_file::{
    CONFIG_FILE_NAME,
    LoadedConfig,
};
use crate::config_schema::CliGitConfig;
/// The engine-failure codes and the names a hint is looked up by.
use crate::diagnostics::{
    EngineFailureCode,
    LEGACY_CONFIG_IGNORED_CODE,
    engine_failure_code_name,
};
/// The warn-unsafe hint the escaping control names.
use crate::event_hints::WARN_UNSAFE_HINT;
use crate::policy_registry::{
    PolicyId,
    Severity,
};
use crate::repository_facts::{
    RepositoryFacts,
    git_facts,
};
use crate::repository_location::RepositoryLocation;
use crate::test_support::{
    REAL_GIT,
    fixture,
    git,
    remove,
    repository,
};
/// The traditional-hooks warning's code,
/// whose hint names the doctor.
use crate::traditional_hook_warning::TRADITIONAL_HOOKS_UNSUPPORTED_CODE;
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// Load configuration for the location real Git reports after `-C <directory>`.
fn load(directory: &Path) -> Result<LoadedConfig, ConfigError> {
    let prefix: Vec<OsString> = vec![OsString::from("-C"), directory.as_os_str().to_os_string()];
    let location: RepositoryLocation = git_facts(Path::new(REAL_GIT), prefix.as_slice(), &[])
        .location()
        .expect("location query");
    return load_identity_config(&location.identity);
}

/// The selected worktree's own top-level file is read,
///  from any directory inside it.
#[test]
fn configuration_follows_global_repository_selection() {
    let root: PathBuf = fixture("invocation-selection");
    let first: PathBuf = repository(root.as_path(), "first");
    let second: PathBuf = repository(root.as_path(), "second");
    std::fs::create_dir(first.join("nested")).expect("nested directory");
    std::fs::write(
        first.join(CONFIG_FILE_NAME),
        r#"{ "policies": { "add-explicit": "warn" } }"#,
    )
    .expect("first config");
    std::fs::write(
        second.join(CONFIG_FILE_NAME),
        r#"{ "policies": { "add-explicit": "off" } }"#,
    )
    .expect("second config");
    // A file in a subdirectory is not configuration.
    std::fs::write(first.join("nested").join(CONFIG_FILE_NAME), "invalid").expect("stray file");
    for (directory, source, expected) in [
        (
            first.join("nested"),
            first.join(CONFIG_FILE_NAME),
            Severity::Warn,
        ),
        (second.clone(), second.join(CONFIG_FILE_NAME), Severity::Off),
    ] {
        let loaded: LoadedConfig = load(directory.as_path()).expect("selected configuration");
        assert_eq!(loaded.source, Some(source));
        assert_eq!(
            loaded
                .config
                .policies
                .setting(PolicyId::AddExplicit)
                .severity,
            expected
        );
    }
    remove(root.as_path());
}

/// A linked worktree reads the file at its own top level,
///  not the main worktree's.
#[test]
fn linked_worktree_uses_its_own_top_level() {
    let root: PathBuf = fixture("invocation-linked");
    let main: PathBuf = repository(root.as_path(), "main");
    let linked: PathBuf = root.join("linked");
    git(
        main.as_path(),
        &[
            OsString::from("worktree"),
            OsString::from("add"),
            OsString::from("--quiet"),
            OsString::from("-b"),
            OsString::from("topic"),
            linked.clone().into_os_string(),
        ],
    );
    std::fs::write(
        main.join(CONFIG_FILE_NAME),
        r#"{ "policies": { "final-newline": "off" } }"#,
    )
    .expect("main config");
    std::fs::write(
        linked.join(CONFIG_FILE_NAME),
        r#"{ "policies": { "final-newline": "error" } }"#,
    )
    .expect("linked config");
    let loaded: LoadedConfig = load(linked.as_path()).expect("linked configuration");
    assert_eq!(loaded.source, Some(linked.join(CONFIG_FILE_NAME)));
    assert_eq!(
        loaded
            .config
            .policies
            .setting(PolicyId::FinalNewline)
            .severity,
        Severity::Error
    );
    remove(root.as_path());
}

/// No repository,
///  a bare repository and the inside of `.git` read no file and use the defaults.
#[test]
fn locations_without_a_worktree_use_the_defaults() {
    let root: PathBuf = fixture("invocation-none");
    let main: PathBuf = repository(root.as_path(), "main");
    let bare: PathBuf = root.join("bare.git");
    git(
        root.as_path(),
        &[
            OsString::from("init"),
            OsString::from("--quiet"),
            OsString::from("--bare"),
            bare.clone().into_os_string(),
        ],
    );
    let plain: PathBuf = root.join("plain");
    std::fs::create_dir(&plain).expect("plain directory");
    for directory in [plain.clone(), bare.clone(), main.join(".git")] {
        // An invalid file at such a location is never read.
        std::fs::write(directory.join(CONFIG_FILE_NAME), "invalid").expect("stray file");
        assert_eq!(
            load(directory.as_path()),
            Ok(LoadedConfig {
                config: CliGitConfig::defaults(),
                source: None,
                ignored_legacy: Vec::<PathBuf>::new(),
                earlier_name_notice: None,
            }),
            "{directory:?}"
        );
    }
    remove(root.as_path());
}

/// A rejected file is reported with its path and the rejected key.
#[test]
fn a_rejected_file_is_a_configuration_error() {
    let root: PathBuf = fixture("invocation-failures");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let source: PathBuf = repo.join(CONFIG_FILE_NAME);
    std::fs::write(&source, r#"{ "unknown": 1 }"#).expect("invalid config");
    assert_eq!(
        load(repo.as_path()),
        Err(ConfigError::new(
            format!(
                "{}: Unknown configuration key: unknown. Accepted keys: policies, hooks, indexLock, landing, discardedCopies.",
                source.display()
            )
            .as_str()
        ))
    );
    remove(root.as_path());
}

/// Events:
///  one `config-invalid` line per rejection;
///  one numbered warning per legacy file,
///  none without a source.
#[test]
fn events_render_configuration_failures_and_legacy_warnings() {
    assert_eq!(
        config_invalid_event(&ConfigError::new(
            "/r/monochromatic-git-policy-cli.config.jsonc: bad \"key\""
        )),
        "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"config-invalid\",\"message\":\"/r/monochromatic-git-policy-cli.config.jsonc: bad \\\"key\\\"\",\"hint\":\"Hint: if you just edited monochromatic-git-policy-cli.config.jsonc, fix the key the message names or revert the edit; an unknown key, a wrong type or an unknown policy id fails before any policy runs.\"}\n"
    );
    let loaded: LoadedConfig = LoadedConfig {
        config: CliGitConfig::defaults(),
        source: Some(PathBuf::from(
            "/r/monochromatic-git-policy-cli.config.jsonc",
        )),
        ignored_legacy: vec![
            PathBuf::from("/r/cli-git.config.mjs"),
            PathBuf::from("/r/cli-git.config.ts"),
        ],
        earlier_name_notice: None,
    };
    assert_eq!(
        legacy_warning_events(&loaded),
        "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"configuration-warning\",\"code\":\"legacy-config-ignored\",\"message\":\"Legacy configuration /r/cli-git.config.mjs is ignored: /r/monochromatic-git-policy-cli.config.jsonc is authoritative for the native monochromatic-git-policy-cli. Remove the legacy file once no TypeScript monochromatic-git-policy-cli reads it.\",\"path\":\"/r/cli-git.config.mjs\",\"hint\":\"Hint: if you are still keeping the old configuration files, delete them once monochromatic-git-policy-cli.config.jsonc holds the settings; they are no longer read.\"}\n\
         {\"schemaVersion\":1,\"sequence\":1,\"type\":\"configuration-warning\",\"code\":\"legacy-config-ignored\",\"message\":\"Legacy configuration /r/cli-git.config.ts is ignored: /r/monochromatic-git-policy-cli.config.jsonc is authoritative for the native monochromatic-git-policy-cli. Remove the legacy file once no TypeScript monochromatic-git-policy-cli reads it.\",\"path\":\"/r/cli-git.config.ts\",\"hint\":\"Hint: if you are still keeping the old configuration files, delete them once monochromatic-git-policy-cli.config.jsonc holds the settings; they are no longer read.\"}\n"
    );
    let without_legacy: LoadedConfig = LoadedConfig {
        config: CliGitConfig::defaults(),
        source: Some(PathBuf::from(
            "/r/monochromatic-git-policy-cli.config.jsonc",
        )),
        ignored_legacy: Vec::<PathBuf>::new(),
        earlier_name_notice: None,
    };
    assert_eq!(legacy_warning_events(&without_legacy), "");
    // Without a JSONC source there is nothing authoritative to compare a legacy file with.
    let without_source: LoadedConfig = LoadedConfig {
        config: CliGitConfig::defaults(),
        source: None,
        ignored_legacy: vec![PathBuf::from("/r/cli-git.config.ts")],
        earlier_name_notice: None,
    };
    assert_eq!(legacy_warning_events(&without_source), "");
}

/// Every engine failure code,
/// with an exhaustive match so a new variant stops compilation here.
fn every_engine_failure() -> Vec<EngineFailureCode> {
    let all = vec![
        EngineFailureCode::ConfigInvalid,
        EngineFailureCode::CoreIncomplete,
        EngineFailureCode::ContentUnavailable,
        EngineFailureCode::PolicyIncomplete,
        EngineFailureCode::PatchInvalid,
        EngineFailureCode::PatchConflict,
        EngineFailureCode::FixCycle,
        EngineFailureCode::FixPassLimit,
        EngineFailureCode::TransactionFailed,
    ];
    for code in &all {
        match code {
            EngineFailureCode::ConfigInvalid
            | EngineFailureCode::CoreIncomplete
            | EngineFailureCode::ContentUnavailable
            | EngineFailureCode::PolicyIncomplete
            | EngineFailureCode::PatchInvalid
            | EngineFailureCode::PatchConflict
            | EngineFailureCode::FixCycle
            | EngineFailureCode::FixPassLimit
            | EngineFailureCode::TransactionFailed => {}
        }
    }
    return all;
}

/// Every engine failure code has a hint that opens with the common marker and offers a next step.
#[test]
fn every_engine_failure_code_has_a_hint() {
    for code in every_engine_failure() {
        let name: &str = engine_failure_code_name(code);
        let hint: &str = engine_failure_hint(name).unwrap_or_else(|| panic!("{name} has no hint"));
        assert!(hint.starts_with("Hint: "), "{name}: {hint}");
        assert!(hint.len() > 80, "{name} is too thin to help: {hint}");
    }
    assert!(engine_failure_hint(LEGACY_CONFIG_IGNORED_CODE).is_some());
    let traditional: &str = engine_failure_hint(TRADITIONAL_HOOKS_UNSUPPORTED_CODE)
        .expect("the traditional-hooks warning has a hint");
    assert!(
        traditional.starts_with("Hint: ")
            && traditional.contains("git monochromatic-git-policy-cli doctor"),
        "{traditional}"
    );
    assert_eq!(engine_failure_hint("not-a-code"), None);
}

/// The hint field is appended as escaped JSON,
/// and nothing is appended without a hint.
#[test]
fn the_hint_field_is_escaped_json_and_absent_without_a_hint() {
    let mut line: String = String::from("{\"message\":\"m\"");
    push_hint_field(&mut line, None);
    assert_eq!(line, "{\"message\":\"m\"");
    push_hint_field(&mut line, Some("Hint: a \"quoted\" word\nnext"));
    assert_eq!(
        line,
        "{\"message\":\"m\",\"hint\":\"Hint: a \\\"quoted\\\" word\\nnext\""
    );
    assert!(WARN_UNSAFE_HINT.starts_with("Hint: if you"));
}
