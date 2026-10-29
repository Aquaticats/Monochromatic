//! What:
//!  Pattern-selection and merged-setting regressions.
//! Why:
//!  Matching must distinguish ignored,
//!  unmatched and configured files without changing rule precedence.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('configuration matching', () => { /* glob and precedence controls */ });
//! ```

/// Import production parsing,
///  preparation and result types.
use super::{FileConfiguration, PreparedConfiguration, prepare_configuration};
use crate::config_lookup::{CONFIG_NAME, ConfigurationSource};
use crate::configuration::parse_configuration;
use monochromatic_jsonc_edit::{JsoncValue, parse_jsonc};
use std::path::Path;

/// Build a configuration with a platform-native absolute base but no filesystem access.
fn prepare(source: &str) -> PreparedConfiguration {
    let base = crate::test_scratch::scratch_root().join("monochromatic-lint-pattern-fixture");
    return prepare_configuration(ConfigurationSource {
        path: base.join(CONFIG_NAME),
        base,
        blocks: parse_configuration(source).expect("configuration fixture parses"),
    })
    .expect("configuration patterns compile");
}

/// Extract configured rules while making the other outcome variants test failures.
fn rules(configuration: &PreparedConfiguration, path: &str) -> JsoncValue {
    let result = configuration
        .resolve(Path::new(path))
        .expect("file settings resolve");
    if let FileConfiguration::Configured { rules } = result {
        return rules;
    }
    panic!("fixture path must be configured");
}

/// A single star stays within one directory and globstar reaches descendants.
#[test]
fn file_globs_preserve_directory_boundaries() {
    let config = prepare(r#"[{"files":["*.rs"],"rules":{}}]"#);
    assert!(matches!(
        config.resolve(Path::new("main.rs")).expect("match"),
        FileConfiguration::Configured { .. }
    ));
    assert_eq!(
        config.resolve(Path::new("src/main.rs")).expect("nested"),
        FileConfiguration::Unconfigured
    );
    let recursive = prepare(r#"[{"files":["**/*.rs"],"rules":{}}]"#);
    assert!(matches!(
        recursive
            .resolve(Path::new("src/main.rs"))
            .expect("nested match"),
        FileConfiguration::Configured { .. }
    ));
}

/// Trailing-slash expansion is limited to directory exclusions,
///  not every pattern.
#[test]
fn file_and_ignore_patterns_do_not_gain_unrequested_wildcards() {
    let files = prepare(r#"[{"files":["src/"],"rules":{}}]"#);
    assert_eq!(
        files
            .resolve(Path::new("src/file.rs"))
            .expect("literal directory selector"),
        FileConfiguration::Unconfigured
    );
    let ignores = prepare(r#"[{"files":["**/*"],"ignores":["exact.rs"],"rules":{}}]"#);
    assert_eq!(
        ignores
            .resolve(Path::new("exact.rs"))
            .expect("exact exclusion"),
        FileConfiguration::Unconfigured
    );
    assert!(matches!(
        ignores
            .resolve(Path::new("exact.rs.md"))
            .expect("not an exclusion prefix"),
        FileConfiguration::Configured { .. }
    ));
}

/// Global exclusions remove files even when an ordinary block would otherwise select them.
#[test]
fn global_directory_exclusions_take_precedence() {
    let config = prepare(
        r#"[
      {"files":["**/*.rs"],"rules":{}},
      {"ignores":["generated/"]}
    ]"#,
    );
    assert_eq!(
        config
            .resolve(Path::new("generated/deep/file.rs"))
            .expect("ignore"),
        FileConfiguration::Ignored
    );
    assert!(matches!(
        config.resolve(Path::new("src/file.rs")).expect("source"),
        FileConfiguration::Configured { .. }
    ));
}

/// Block-local exclusions do not globally exclude a file selected by another block.
#[test]
fn local_exclusions_leave_other_blocks_available() {
    let config = prepare(
        r#"[
      {"files":["**/*.rs"],"ignores":["tests/"],"rules":{"rust/max-lines":{"severity":"error","max":100}}},
      {"files":["tests/**/*.rs"],"rules":{"rust/require-rustdoc":{"severity":"warn"}}}
    ]"#,
    );
    let actual = rules(&config, "tests/deep/file.rs");
    let expected =
        parse_jsonc(r#"{"rust/require-rustdoc":{"severity":"warn"}}"#).expect("expected rules");
    assert_eq!(actual, expected);
}

/// Later severities retain earlier options,
///  and arrays concatenate in matching-block order.
#[test]
fn ordered_blocks_merge_before_defaults_are_filled() {
    let config = prepare(
        r#"[
      {"files":["**/*"],"rules":{"rust/max-lines":{"severity":"error","max":120},"markdown/lfs-image-url":{"severity":"error","exclude":["a/"]}}},
      {"files":["**/*"],"rules":{"rust/max-lines":{"severity":"warn"},"markdown/lfs-image-url":{"exclude":["b/"]}}}
    ]"#,
    );
    let actual = rules(&config, "file.rs");
    let expected = parse_jsonc(r#"{"rust/max-lines":{"severity":"warn","max":120},"markdown/lfs-image-url":{"severity":"error","exclude":["a/","b/"]}}"#).expect("expected merged rules");
    assert_eq!(actual, expected);
}

/// Defaults fill selected rules only;
///  an empty matching rules record enables nothing.
#[test]
fn rule_defaults_do_not_enable_absent_rules() {
    let empty = prepare(r#"[{"files":["**/*"],"rules":{}}]"#);
    assert!(
        rules(&empty, "file.rs")
            .entries()
            .expect("record")
            .is_empty()
    );
    let selected = prepare(
        r#"[{"files":["**/*"],"rules":{"rust/max-lines":{"severity":"off"},"markdown/lfs-image-url":{"severity":"warn"}}}]"#,
    );
    let actual = rules(&selected, "file.rs");
    let expected = parse_jsonc(r#"{"rust/max-lines":{"severity":"off","max":300},"markdown/lfs-image-url":{"severity":"warn","exclude":[]}}"#).expect("defaults");
    assert_eq!(actual, expected);
}

/// Partial settings need an explicit severity after all matching blocks have been merged.
#[test]
fn unresolved_severity_is_a_configuration_error() {
    let config = prepare(r#"[{"files":["**/*.rs"],"rules":{"rust/max-lines":{"max":10}}}]"#);
    let error = config
        .resolve(Path::new("file.rs"))
        .expect_err("missing severity");
    assert!(error.message.contains("needs severity"));
}

/// Virtual paths participate in the same matching,
///  including exclusion of snippet descendants.
#[test]
fn virtual_paths_keep_their_host_suffixes() {
    let config = prepare(r#"[{"files":["**/*.rs"],"ignores":["**/*.md/**"],"rules":{}}]"#);
    assert_eq!(
        config
            .resolve(Path::new("doc/example.md/0.rs"))
            .expect("snippet excluded"),
        FileConfiguration::Unconfigured
    );
    assert!(matches!(
        config.resolve(Path::new("src/main.rs")).expect("real Rust"),
        FileConfiguration::Configured { .. }
    ));
}

/// Explicit parent-relative names can match,
///  while passing an absolute name is a caller error.
#[test]
fn relative_parent_selection_is_not_silently_rebased() {
    let config = prepare(r#"[{"files":["../external/*.rs"],"rules":{}}]"#);
    assert!(matches!(
        config
            .resolve(Path::new("../external/main.rs"))
            .expect("outside selection"),
        FileConfiguration::Configured { .. }
    ));
    let error = config
        .resolve(crate::test_scratch::scratch_root().as_path())
        .expect_err("absolute candidate");
    assert!(error.message.contains("relative"));
}

/// Invalid glob syntax fails when preparing configuration rather than silently matching nothing.
#[test]
fn invalid_patterns_are_rejected() {
    let base = crate::test_scratch::scratch_root();
    let source = ConfigurationSource {
        path: base.join(CONFIG_NAME),
        base,
        blocks: parse_configuration(r#"[{"files":["["]}]"#).expect("valid schema"),
    };
    let error = match prepare_configuration(source) {
        Ok(_) => panic!("invalid pattern must fail"),
        Err(error) => error,
    };
    assert!(error.message.contains("Invalid configuration pattern"));
}

/// Byte-valued Unix filenames stay matchable without lossy Unicode conversion.
#[cfg(unix)]
#[test]
fn non_utf8_file_names_match_by_native_bytes() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let config = prepare(r#"[{"files":["**/*.rs"],"rules":{}}]"#);
    let path = std::path::PathBuf::from(OsString::from_vec(b"src/\xff.rs".to_vec()));
    assert!(matches!(
        config.resolve(path.as_path()).expect("byte path"),
        FileConfiguration::Configured { .. }
    ));
}
