//! What:
//!  Disposable-directory controls for configuration discovery and reading.
//! Why:
//!  The loader must read one regular bounded file,
//!  report legacy executable
//!      configuration,
//!  and never follow links or execute anything.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const root = await mkdtemp(...); await writeFile(join(root, 'cli-git.config.jsonc'), '{}');
//! ```

/// Import the loader under test and the settings it produces.
use super::{
    CONFIG_FILE_NAME, LoadedConfig, MAX_CONFIG_BYTES, ignored_legacy_notice, load_repository_config,
};
use crate::config_schema::CliGitConfig;
use crate::policy_registry::{PolicyId, Severity};
use std::path::{Path, PathBuf};

/// Create one empty fixture directory owned by this test process and test name.
fn fixture(name: &str) -> PathBuf {
    return crate::test_support::fixture_with_prefix("native-config-file", name);
}

/// Remove only the fixture directory this test created.
fn remove(root: &Path) {
    std::fs::remove_dir_all(root).expect("remove only the fixture");
}

/// Return the rejection message for one fixture root.
fn rejection(root: &Path) -> String {
    return load_repository_config(root)
        .expect_err("fixture must be rejected")
        .message;
}

/// A repository without any configuration file and one with an empty file load the same
/// settings:
///  built-ins on,
///  the four optional policies off.
#[test]
fn absent_and_empty_configuration_load_the_same_defaults() {
    let root: PathBuf = fixture("absent");
    assert_eq!(
        load_repository_config(root.as_path()),
        Ok(LoadedConfig {
            config: CliGitConfig::defaults(),
            source: None,
            ignored_legacy: Vec::<PathBuf>::new(),
        })
    );
    assert_eq!(
        CliGitConfig::defaults()
            .policies
            .setting(PolicyId::ForbiddenStrings)
            .severity,
        Severity::Off
    );
    std::fs::write(root.join(CONFIG_FILE_NAME), "{}").expect("write empty object");
    let configured: LoadedConfig = load_repository_config(root.as_path()).expect("empty file");
    assert_eq!(configured.config, CliGitConfig::defaults());
    assert_eq!(
        configured
            .config
            .policies
            .setting(PolicyId::ForbiddenStrings)
            .severity,
        Severity::Off
    );
    remove(root.as_path());
}

/// A valid file is parsed and its path is reported as the source.
#[test]
fn valid_configuration_is_loaded_from_the_root() {
    let root: PathBuf = fixture("valid");
    let source: PathBuf = root.join(CONFIG_FILE_NAME);
    std::fs::write(&source, r#"{ "policies": { "final-newline": "error" } }"#).expect("write");
    let loaded: LoadedConfig = load_repository_config(root.as_path()).expect("valid file");
    assert_eq!(loaded.source, Some(source));
    assert_eq!(
        loaded
            .config
            .policies
            .setting(PolicyId::FinalNewline)
            .severity,
        Severity::Error
    );
    assert_eq!(loaded.ignored_legacy, Vec::<PathBuf>::new());
    remove(root.as_path());
}

/// A leading UTF-8 byte order mark is not configuration content.
#[test]
fn byte_order_mark_is_accepted() {
    let root: PathBuf = fixture("bom");
    std::fs::write(
        root.join(CONFIG_FILE_NAME),
        b"\xef\xbb\xbf{ \"hooks\": { \"concurrentCommits\": true } }",
    )
    .expect("write");
    let loaded: LoadedConfig = load_repository_config(root.as_path()).expect("BOM file");
    assert!(loaded.config.concurrency.hooks.concurrent_commits);
    remove(root.as_path());
}

/// Schema failures are prefixed with the file they came from.
#[test]
fn invalid_content_names_the_file_and_the_key() {
    let root: PathBuf = fixture("invalid");
    let source: PathBuf = root.join(CONFIG_FILE_NAME);
    std::fs::write(&source, r#"{ "plugins": {} }"#).expect("write");
    assert_eq!(
        rejection(root.as_path()),
        format!(
            "{}: Configuration key plugins is retired: policies are compiled into cli-git and \
             JSONC configuration cannot load plugin code. Remove the key and configure the \
             shipped policies under policies.",
            source.display()
        )
    );
    remove(root.as_path());
}

/// Bytes that are not UTF-8 are rejected by file name instead of being decoded lossily.
#[test]
fn non_utf8_content_is_rejected() {
    let root: PathBuf = fixture("non-utf8");
    std::fs::write(root.join(CONFIG_FILE_NAME), b"{ \"policies\": {} } \xff").expect("write");
    let message: String = rejection(root.as_path());
    assert!(message.contains("is not UTF-8 text"), "{message}");
    assert!(message.contains(CONFIG_FILE_NAME), "{message}");
    remove(root.as_path());
}

/// The byte cap accepts a file of exactly the limit and rejects one byte more.
#[test]
fn size_limit_is_exact() {
    let root: PathBuf = fixture("size");
    let source: PathBuf = root.join(CONFIG_FILE_NAME);
    let mut content: Vec<u8> = b"{}".to_vec();
    content.resize(MAX_CONFIG_BYTES as usize, b' ');
    std::fs::write(&source, &content).expect("write limit");
    assert_eq!(
        load_repository_config(root.as_path())
            .expect("file of exactly the limit")
            .config,
        CliGitConfig::defaults()
    );
    content.push(b' ');
    std::fs::write(&source, &content).expect("write over limit");
    assert_eq!(
        rejection(root.as_path()),
        format!(
            "Configuration file {} is larger than 1048576 bytes.",
            source.display()
        )
    );
    remove(root.as_path());
}

/// A directory at the configuration path is not configuration.
#[test]
fn non_regular_configuration_path_is_rejected() {
    let root: PathBuf = fixture("directory");
    let source: PathBuf = root.join(CONFIG_FILE_NAME);
    std::fs::create_dir(&source).expect("directory at config path");
    assert_eq!(
        rejection(root.as_path()),
        format!(
            "Configuration path must be a regular file: {}.",
            source.display()
        )
    );
    remove(root.as_path());
}

/// A symbolic link is rejected even when it points at a valid file,
///  and a dangling one is not "absent".
#[cfg(unix)]
#[test]
fn symbolic_link_configuration_is_rejected() {
    let root: PathBuf = fixture("symlink");
    let source: PathBuf = root.join(CONFIG_FILE_NAME);
    let target: PathBuf = root.join("elsewhere.jsonc");
    std::fs::write(&target, "{}").expect("write target");
    std::os::unix::fs::symlink(&target, &source).expect("link");
    let expected: String = format!(
        "Configuration path must not be a symbolic link: {}. Replace it with a regular file.",
        source.display()
    );
    assert_eq!(rejection(root.as_path()), expected);
    std::fs::remove_file(&target).expect("dangle the link");
    assert_eq!(rejection(root.as_path()), expected);
    remove(root.as_path());
}

/// Legacy executable configuration without a JSONC file is a migration error naming both files.
#[test]
fn legacy_configuration_alone_requires_migration() {
    for (legacy_names, reported) in [
        (vec!["cli-git.config.ts"], "cli-git.config.ts"),
        (vec!["cli-git.config.mjs"], "cli-git.config.mjs"),
        (
            vec!["cli-git.config.ts", "cli-git.config.mjs"],
            "cli-git.config.mjs",
        ),
    ] {
        let root: PathBuf = fixture(format!("legacy-{}", legacy_names.len()).as_str());
        for name in &legacy_names {
            // The content would exit nonzero if executed; the loader must never run it.
            std::fs::write(root.join(name), "process.exit(97);\n").expect("write legacy");
        }
        let message: String = rejection(root.as_path());
        assert_eq!(
            message,
            format!(
                "Legacy configuration {} is not executed or read by the native cli-git. \
                 Translate its settings into {} (data only: policies, hooks, indexLock, landing), \
                 then remove the legacy file. Plugins, executable paths, command arrays and trust \
                 settings have no JSONC equivalent because every policy is built in.",
                root.join(reported).display(),
                root.join(CONFIG_FILE_NAME).display()
            ),
            "{legacy_names:?}"
        );
        remove(root.as_path());
    }
}

/// Legacy files beside a JSONC file are reported,
///  never read and never silently dropped.
#[test]
fn legacy_configuration_beside_jsonc_is_reported() {
    let root: PathBuf = fixture("legacy-beside");
    let source: PathBuf = root.join(CONFIG_FILE_NAME);
    std::fs::write(&source, r#"{ "landing": { "reserveAfterLostRaces": 3 } }"#).expect("write");
    // Unreadable as configuration and invalid as a script: only its name may matter.
    std::fs::write(root.join("cli-git.config.ts"), b"\xff\xfe not text").expect("write legacy");
    std::fs::create_dir(root.join("cli-git.config.mjs")).expect("legacy name as directory");
    let loaded: LoadedConfig =
        load_repository_config(root.as_path()).expect("JSONC is authoritative");
    assert_eq!(
        loaded.config.concurrency.landing.reserve_after_lost_races,
        3
    );
    assert_eq!(
        loaded.ignored_legacy,
        vec![
            root.join("cli-git.config.mjs"),
            root.join("cli-git.config.ts")
        ]
    );
    assert_eq!(
        ignored_legacy_notice(loaded.ignored_legacy[1].as_path(), source.as_path()),
        format!(
            "Legacy configuration {} is ignored: {} is authoritative for the native cli-git. \
             Remove the legacy file once no TypeScript cli-git reads it.",
            root.join("cli-git.config.ts").display(),
            source.display()
        )
    );
    remove(root.as_path());
}

/// A root whose entries cannot be inspected is an error,
///  never "no configuration".
#[cfg(unix)]
#[test]
fn uninspectable_root_is_an_error() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("unreadable");
    std::fs::write(root.join(CONFIG_FILE_NAME), "{}").expect("write");
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o000)).expect("lock fixture");
    let outcome = load_repository_config(root.as_path());
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700))
        .expect("unlock fixture");
    let message: String = outcome.expect_err("locked directory").message;
    assert!(message.starts_with("Cannot inspect "), "{message}");
    remove(root.as_path());
}

/// A regular file that cannot be opened is an error naming the file.
#[cfg(unix)]
#[test]
fn unreadable_file_is_an_error() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("unopenable");
    let source: PathBuf = root.join(CONFIG_FILE_NAME);
    std::fs::write(&source, "{}").expect("write");
    std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o000)).expect("lock file");
    let message: String = rejection(root.as_path());
    assert!(
        message.starts_with(format!("Cannot open {}", source.display()).as_str()),
        "{message}"
    );
    remove(root.as_path());
}

/// Repository roots need not be UTF-8;
///  the path is used as raw operating-system bytes.
#[cfg(unix)]
#[test]
fn non_utf8_repository_root_is_supported() {
    use std::os::unix::ffi::OsStringExt;
    let parent: PathBuf = fixture("non-utf8-root");
    let root: PathBuf = parent.join(std::ffi::OsString::from_vec(b"repo-\xff".to_vec()));
    std::fs::create_dir(&root).expect("non-UTF-8 directory");
    std::fs::write(
        root.join(CONFIG_FILE_NAME),
        r#"{ "policies": { "add-explicit": "warn" } }"#,
    )
    .expect("write");
    let loaded: LoadedConfig = load_repository_config(root.as_path()).expect("non-UTF-8 root");
    assert_eq!(loaded.source, Some(root.join(CONFIG_FILE_NAME)));
    assert_eq!(
        loaded
            .config
            .policies
            .setting(PolicyId::AddExplicit)
            .severity,
        Severity::Warn
    );
    remove(parent.as_path());
}
