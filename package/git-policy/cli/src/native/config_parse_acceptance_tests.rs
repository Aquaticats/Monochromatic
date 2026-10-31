//! What:
//!  Acceptance controls for the configuration schema.
//! Why:
//!  A valid document must yield exactly the settings it states and the defaults for
//!      everything it leaves out,
//!  so a silently dropped or widened setting fails here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parseConfig('{}')).toEqual(defaults());
//! ```

/// Import the parser under test,
///  the typed settings and the registry identities.
use super::parse_config;
use crate::config_schema::{
    CliGitConfig, ForbiddenStringsOptions, MarkdownAutofixOptions, MarkdownRule, PolicySetting,
    RulesFileEntry, unlisted_severity,
};
use crate::policy_registry::{POLICY_REGISTRY, PolicyId, Severity};

/// An empty object,
///  with or without comments and trailing commas,
///  is exactly the defaults.
#[test]
fn empty_documents_equal_the_defaults() {
    let defaults: CliGitConfig = CliGitConfig::defaults();
    for source in [
        "{}",
        " { } ",
        "// only a comment\n{}\n",
        "/* block */ { /* inner */ }",
        "{ \"policies\": {}, }",
        "{ \"hooks\": {}, \"indexLock\": {}, \"landing\": {}, }",
    ] {
        assert_eq!(parse_config(source), Ok(defaults.clone()), "{source}");
    }
    assert!(!defaults.concurrency.hooks.concurrent_commits);
    assert_eq!(
        defaults.concurrency.index_lock.unproven_owner_timeout_ms,
        1_000
    );
    assert_eq!(defaults.concurrency.landing.reserve_after_lost_races, 1);
    assert!(defaults.policies.forbidden_strings.builtin_rules);
    assert_eq!(
        defaults.policies.markdown_autofix,
        MarkdownAutofixOptions {
            rules: vec![MarkdownRule::LfsImageUrl],
            exclude: Vec::<String>::new(),
        }
    );
    for descriptor in POLICY_REGISTRY {
        assert_eq!(
            defaults.policies.setting(descriptor.id),
            PolicySetting {
                id: descriptor.id,
                severity: unlisted_severity(descriptor),
                explicit: false,
            }
        );
    }
}

/// The settings every policy must have after loading this repository's configuration.
fn assert_repository_settings(config: &CliGitConfig, dependent_bump: (Severity, bool)) {
    for (id, severity, explicit) in [
        (PolicyId::RequireRoot, Severity::Error, false),
        (PolicyId::LinkedWorktreeOnly, Severity::Error, false),
        (PolicyId::BranchWorktreeOnly, Severity::Error, false),
        (PolicyId::AddExplicit, Severity::Error, false),
        (PolicyId::FinalNewline, Severity::Warn, false),
        (PolicyId::MarkdownAutofix, Severity::Warn, true),
        (PolicyId::ForbiddenRootContext, Severity::Error, true),
        (
            PolicyId::DependentVersionBump,
            dependent_bump.0,
            dependent_bump.1,
        ),
        (PolicyId::ForbiddenStrings, Severity::Error, true),
    ] {
        assert_eq!(
            config.policies.setting(id),
            PolicySetting {
                id,
                severity,
                explicit
            }
        );
    }
    assert_eq!(config.policies.settings.len(), POLICY_REGISTRY.len());
    assert_eq!(
        config.policies.markdown_autofix,
        MarkdownAutofixOptions {
            rules: vec![MarkdownRule::LfsImageUrl],
            exclude: vec![String::from("package/ssg/")],
        }
    );
    assert!(config.policies.forbidden_strings.builtin_rules);
    assert_eq!(config.concurrency, CliGitConfig::defaults().concurrency);
}

/// This repository's translated configuration must name all four optional policies to keep
/// the severities the incumbent applies.
#[test]
fn repository_translation_names_all_four_optional_policies() {
    let config: CliGitConfig = parse_config(
        r#"{
          "policies": {
            // Rewrites Markdown image links that point at LFS-tracked files.
            "markdown/autofix": ["warn", { "rules": ["lfs-image-url"], "exclude": ["package/ssg/"] }],
            "mono/forbidden-root-context": "error",
            // The incumbent ran this one without the root configuration naming it.
            "mono/dependent-version-bump": "error",
            // The rules file the root `mise.toml` names in `FORBIDDEN_STRINGS_RULES` today.
            "security/forbidden-strings": ["error", { "builtinRules": true, "rulesFile": ".cache/forbidden-strings.rules.txt" }],
          },
        }"#,
    )
    .expect("translated repository configuration");
    assert_repository_settings(&config, (Severity::Error, true));
    assert_eq!(
        config.policies.forbidden_strings,
        ForbiddenStringsOptions {
            builtin_rules: true,
            rules_files: Some(vec![RulesFileEntry {
                path: String::from(".cache/forbidden-strings.rules.txt"),
                optional: false,
            }]),
        }
    );
}

/// `rulesFile` takes one path (a required file, as before lists existed)
///  or an array of paths and `{"path", "optional"}` objects, in the order written.
#[test]
fn rules_file_takes_a_path_or_an_array_of_entries() {
    let single: CliGitConfig = parse_config(
        r#"{ "policies": { "security/forbidden-strings": ["error", { "rulesFile": "a,b?.txt" }] } }"#,
    )
    .expect("single path");
    // A configured string is never split: JSON already says which file is meant.
    assert_eq!(
        single.policies.forbidden_strings.rules_files,
        Some(vec![RulesFileEntry { path: String::from("a,b?.txt"), optional: false }])
    );
    let listed: CliGitConfig = parse_config(
        r#"{ "policies": { "security/forbidden-strings": { "rulesFile": [
          "forbidden-strings.append.txt",
          { "path": "forbidden-strings.append.local.txt", "optional": true },
          { "path": "extra.txt" },
          { "path": "never.txt", "optional": false },
        ] } } }"#,
    )
    .expect("array of entries");
    assert_eq!(
        listed.policies.forbidden_strings.rules_files,
        Some(vec![
            RulesFileEntry { path: String::from("forbidden-strings.append.txt"), optional: false },
            RulesFileEntry { path: String::from("forbidden-strings.append.local.txt"), optional: true },
            RulesFileEntry { path: String::from("extra.txt"), optional: false },
            RulesFileEntry { path: String::from("never.txt"), optional: false },
        ])
    );
    assert!(listed.policies.forbidden_strings.builtin_rules);
}

/// A word-for-word translation of the TypeScript `policies` map names only three of them:
/// `mono/dependent-version-bump`,
///  which the incumbent enforced unlisted,
///  silently stops.
#[test]
fn literal_translation_silently_stops_the_unlisted_policy() {
    let config: CliGitConfig = parse_config(
        r#"{
          "policies": {
            "markdown/autofix": ["warn", { "rules": ["lfs-image-url"], "exclude": ["package/ssg/"] }],
            "mono/forbidden-root-context": "error",
            "security/forbidden-strings": ["error", { "builtinRules": true }],
          },
        }"#,
    )
    .expect("literal translation");
    assert_repository_settings(&config, (Severity::Off, false));
    assert_eq!(
        config.policies.forbidden_strings,
        ForbiddenStringsOptions {
            builtin_rules: true,
            rules_files: None,
        }
    );
}

/// Each optional policy is off until the file names it,
///  and naming one turns on only that one.
#[test]
fn optional_policies_run_only_when_listed() {
    let optional: [(PolicyId, &str, Severity); 4] = [
        (
            PolicyId::MarkdownAutofix,
            "markdown/autofix",
            Severity::Warn,
        ),
        (
            PolicyId::ForbiddenRootContext,
            "mono/forbidden-root-context",
            Severity::Error,
        ),
        (
            PolicyId::DependentVersionBump,
            "mono/dependent-version-bump",
            Severity::Error,
        ),
        (
            PolicyId::ForbiddenStrings,
            "security/forbidden-strings",
            Severity::Error,
        ),
    ];
    let unlisted: CliGitConfig = parse_config(r#"{ "policies": {} }"#).expect("no policy named");
    for (id, _, _) in optional {
        assert_eq!(
            unlisted.policies.setting(id),
            PolicySetting {
                id,
                severity: Severity::Off,
                explicit: false
            }
        );
    }
    for (listed, name, default_severity) in optional {
        // Listed at the severity the incumbent definition declares as its default.
        let source: String = format!(
            r#"{{ "policies": {{ "{name}": "{}" }} }}"#,
            crate::policy_registry::severity_name(default_severity)
        );
        let config: CliGitConfig = parse_config(source.as_str()).expect(source.as_str());
        assert_eq!(
            crate::policy_registry::policy_descriptor(listed).default_severity,
            default_severity,
            "{name}"
        );
        for (other, _, _) in optional {
            let expected: PolicySetting = if other == listed {
                PolicySetting {
                    id: other,
                    severity: default_severity,
                    explicit: true,
                }
            } else {
                PolicySetting {
                    id: other,
                    severity: Severity::Off,
                    explicit: false,
                }
            };
            assert_eq!(config.policies.setting(other), expected, "{source}");
        }
        // The built-ins are untouched by naming an optional policy.
        assert_eq!(
            config.policies.setting(PolicyId::RequireRoot).severity,
            Severity::Error
        );
        assert_eq!(
            config.policies.setting(PolicyId::FinalNewline).severity,
            Severity::Warn
        );
    }
    // An explicit "off" is recorded as the repository's choice.
    let disabled: CliGitConfig =
        parse_config(r#"{ "policies": { "mono/dependent-version-bump": "off" } }"#)
            .expect("explicit off");
    assert_eq!(
        disabled.policies.setting(PolicyId::DependentVersionBump),
        PolicySetting {
            id: PolicyId::DependentVersionBump,
            severity: Severity::Off,
            explicit: true
        }
    );
}

/// Every policy accepts every severity word,
///  and the setting is marked explicit.
#[test]
fn every_policy_accepts_every_severity() {
    for descriptor in POLICY_REGISTRY {
        for (word, severity) in [
            ("off", Severity::Off),
            ("warn", Severity::Warn),
            ("error", Severity::Error),
        ] {
            let source: String =
                format!(r#"{{ "policies": {{ "{}": "{word}" }} }}"#, descriptor.name);
            let config: CliGitConfig = parse_config(source.as_str()).expect(source.as_str());
            for other in POLICY_REGISTRY {
                let expected: PolicySetting = if other.id == descriptor.id {
                    PolicySetting {
                        id: other.id,
                        severity,
                        explicit: true,
                    }
                } else {
                    PolicySetting {
                        id: other.id,
                        severity: unlisted_severity(other),
                        explicit: false,
                    }
                };
                assert_eq!(config.policies.setting(other.id), expected, "{source}");
            }
        }
    }
}

/// Options override only their own field;
///  a bare severity keeps default options.
#[test]
fn policy_options_apply_field_by_field() {
    let scanner: CliGitConfig = parse_config(
        r#"{ "policies": { "security/forbidden-strings": ["warn", { "builtinRules": false }] } }"#,
    )
    .expect("scanner options");
    assert!(!scanner.policies.forbidden_strings.builtin_rules);
    assert_eq!(
        scanner
            .policies
            .setting(PolicyId::ForbiddenStrings)
            .severity,
        Severity::Warn
    );
    assert_eq!(
        scanner.policies.markdown_autofix,
        CliGitConfig::defaults().policies.markdown_autofix
    );
    let empty_options: CliGitConfig = parse_config(
        r#"{ "policies": { "security/forbidden-strings": ["error", {}], "markdown/autofix": ["off", {}] } }"#,
    )
    .expect("empty option objects");
    assert!(empty_options.policies.forbidden_strings.builtin_rules);
    assert_eq!(
        empty_options.policies.markdown_autofix,
        CliGitConfig::defaults().policies.markdown_autofix
    );
    assert_eq!(
        empty_options
            .policies
            .setting(PolicyId::MarkdownAutofix)
            .severity,
        Severity::Off
    );
    let excluded: CliGitConfig = parse_config(
        r#"{ "policies": { "markdown/autofix": ["error", { "exclude": ["b/", "", "a/**", "b/"] }] } }"#,
    )
    .expect("exclusions");
    assert_eq!(
        excluded.policies.markdown_autofix,
        MarkdownAutofixOptions {
            rules: vec![MarkdownRule::LfsImageUrl],
            exclude: vec![
                String::from("b/"),
                String::new(),
                String::from("a/**"),
                String::from("b/")
            ],
        }
    );
    let bare: CliGitConfig =
        parse_config(r#"{ "policies": { "security/forbidden-strings": "error" } }"#)
            .expect("bare severity");
    assert!(bare.policies.forbidden_strings.builtin_rules);
    assert!(bare.policies.setting(PolicyId::ForbiddenStrings).explicit);
}

/// An options object alone names the policy:
///  it runs at its incumbent default severity
/// with those options,
///  and unknown or retired option keys are still rejected by key.
#[test]
fn options_alone_list_a_policy_at_its_default_severity() {
    let markdown: CliGitConfig =
        parse_config(r#"{ "policies": { "markdown/autofix": { "exclude": ["package/ssg/"] } } }"#)
            .expect("Markdown options alone");
    assert_eq!(
        markdown.policies.setting(PolicyId::MarkdownAutofix),
        PolicySetting {
            id: PolicyId::MarkdownAutofix,
            severity: Severity::Warn,
            explicit: true,
        }
    );
    assert_eq!(
        markdown.policies.markdown_autofix,
        MarkdownAutofixOptions {
            rules: vec![MarkdownRule::LfsImageUrl],
            exclude: vec![String::from("package/ssg/")],
        }
    );
    // The other optional policies stay off: only the named one is listed.
    assert_eq!(
        markdown
            .policies
            .setting(PolicyId::ForbiddenStrings)
            .severity,
        Severity::Off
    );
    let scanner: CliGitConfig = parse_config(
        r#"{ "policies": { "security/forbidden-strings": { "builtinRules": false } } }"#,
    )
    .expect("scanner options alone");
    assert_eq!(
        scanner.policies.setting(PolicyId::ForbiddenStrings),
        PolicySetting {
            id: PolicyId::ForbiddenStrings,
            severity: Severity::Error,
            explicit: true,
        }
    );
    assert!(!scanner.policies.forbidden_strings.builtin_rules);
    // An empty options object is still a listing, with default options.
    let empty: CliGitConfig =
        parse_config(r#"{ "policies": { "security/forbidden-strings": {} } }"#).expect("empty");
    assert_eq!(
        empty.policies.setting(PolicyId::ForbiddenStrings).severity,
        Severity::Error
    );
    assert!(empty.policies.setting(PolicyId::ForbiddenStrings).explicit);
    assert!(empty.policies.forbidden_strings.builtin_rules);
    for (source, message) in [
        (
            r#"{ "policies": { "markdown/autofix": { "command": ["node"] } } }"#,
            "Configuration key policies.markdown/autofix.command is retired: cli-git runs its own coordinated Markdown linter and configuration cannot select a command. Remove the key.",
        ),
        (
            r#"{ "policies": { "security/forbidden-strings": { "rules": [] } } }"#,
            "Unknown configuration key: policies.security/forbidden-strings.rules. Accepted options: builtinRules, rulesFile.",
        ),
        // A policy without options cannot be listed by an object.
        (
            r#"{ "policies": { "mono/dependent-version-bump": {} } }"#,
            "Configuration key policies.mono/dependent-version-bump must be a severity string, found an object.",
        ),
        (
            r#"{ "policies": { "require-root": {} } }"#,
            "Configuration key policies.require-root must be a severity string, found an object.",
        ),
    ] {
        assert_eq!(
            parse_config(source).expect_err(source).message,
            message,
            "{source}"
        );
    }
}

/// Each concurrency key is read independently with its own lower bound.
#[test]
fn concurrency_keys_apply_independently() {
    let all: CliGitConfig = parse_config(
        r#"{
          "hooks": { "concurrentCommits": true },
          "indexLock": { "unprovenOwnerTimeoutMs": 0 },
          "landing": { "reserveAfterLostRaces": 9007199254740991 }
        }"#,
    )
    .expect("all concurrency keys");
    assert!(all.concurrency.hooks.concurrent_commits);
    assert_eq!(all.concurrency.index_lock.unproven_owner_timeout_ms, 0);
    assert_eq!(
        all.concurrency.landing.reserve_after_lost_races,
        9_007_199_254_740_991
    );
    assert_eq!(all.policies, CliGitConfig::defaults().policies);
    let hooks_only: CliGitConfig =
        parse_config(r#"{ "hooks": { "concurrentCommits": false } }"#).expect("hooks");
    assert_eq!(hooks_only, CliGitConfig::defaults());
    let lock_only: CliGitConfig =
        parse_config(r#"{ "indexLock": { "unprovenOwnerTimeoutMs": 2.5e3 } }"#).expect("lock");
    assert_eq!(
        lock_only.concurrency.index_lock.unproven_owner_timeout_ms,
        2_500
    );
    assert_eq!(lock_only.concurrency.landing.reserve_after_lost_races, 1);
    assert!(!lock_only.concurrency.hooks.concurrent_commits);
    let landing_only: CliGitConfig =
        parse_config(r#"{ "landing": { "reserveAfterLostRaces": 1 } }"#).expect("landing");
    assert_eq!(landing_only.concurrency.landing.reserve_after_lost_races, 1);
    assert_eq!(
        landing_only
            .concurrency
            .index_lock
            .unproven_owner_timeout_ms,
        1_000
    );
}
