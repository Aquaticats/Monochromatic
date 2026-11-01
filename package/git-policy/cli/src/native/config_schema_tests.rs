//! What:
//!  Controls for the typed configuration values themselves.
//! Why:
//!  Rule names are printed in diagnostics and passed to the linter,
//!  and the setting
//!      lookup is what every policy asks,
//!  so both are pinned apart from the parser.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(markdownRuleName(markdownRuleFromName('lfs-image-url'))).toBe('lfs-image-url');
//! ```

/// Import the values under test and the registry they are derived from.
use super::{
    CliGitConfig, ConcurrencyConfig, DEFAULT_RESERVE_AFTER_LOST_RACES,
    DEFAULT_UNPROVEN_OWNER_TIMEOUT_MS, MarkdownRule, PolicyConfig, PolicySetting,
    markdown_rule_from_name, markdown_rule_name, unlisted_severity,
};
use crate::policy_registry::{POLICY_REGISTRY, PolicyId, Severity};

/// The shipped rule has one exact spelling in both directions;
///  near misses are not rules.
#[test]
fn markdown_rule_names_are_exact_in_both_directions() {
    assert_eq!(
        markdown_rule_name(MarkdownRule::LfsImageUrl),
        "lfs-image-url"
    );
    assert_eq!(
        markdown_rule_from_name("lfs-image-url"),
        Some(MarkdownRule::LfsImageUrl)
    );
    assert_eq!(
        markdown_rule_from_name(markdown_rule_name(MarkdownRule::LfsImageUrl)),
        Some(MarkdownRule::LfsImageUrl)
    );
    for name in [
        "",
        "LFS-IMAGE-URL",
        "lfs-image-url ",
        " lfs-image-url",
        "lfs-image-urls",
        "lfs-image",
        "markdown/lfs-image-url",
        "lfs_image_url",
    ] {
        assert_eq!(markdown_rule_from_name(name), None, "{name:?}");
    }
}

/// The severity of a policy the configuration does not name:
///  built-ins at their default,
///  the four optional policies off.
#[test]
fn unlisted_policies_run_only_when_built_in() {
    let mut off: Vec<PolicyId> = Vec::<PolicyId>::new();
    for descriptor in POLICY_REGISTRY {
        if descriptor.off_unless_listed {
            assert_eq!(
                unlisted_severity(descriptor),
                Severity::Off,
                "{}",
                descriptor.name
            );
            off.push(descriptor.id);
        } else {
            assert_eq!(
                unlisted_severity(descriptor),
                descriptor.default_severity,
                "{}",
                descriptor.name
            );
            assert_ne!(
                unlisted_severity(descriptor),
                Severity::Off,
                "{}",
                descriptor.name
            );
        }
    }
    assert_eq!(
        off,
        [
            PolicyId::MarkdownAutofix,
            PolicyId::ForbiddenRootContext,
            PolicyId::DependentVersionBump,
            PolicyId::ForbiddenStrings,
        ]
    );
}

/// The lookup returns each policy's own row,
///  in the defaults and after a change.
#[test]
fn setting_lookup_returns_each_policy_row() {
    let mut config: PolicyConfig = PolicyConfig::defaults();
    assert_eq!(config.settings.len(), POLICY_REGISTRY.len());
    for (index, descriptor) in POLICY_REGISTRY.iter().enumerate() {
        assert_eq!(config.settings[index].id, descriptor.id);
        assert_eq!(
            config.setting(descriptor.id),
            PolicySetting {
                id: descriptor.id,
                severity: unlisted_severity(descriptor),
                explicit: false,
            },
            "{}",
            descriptor.name
        );
    }
    // Changing the last row changes only that policy's answer.
    let last: usize = config.settings.len() - 1;
    config.settings[last].severity = Severity::Warn;
    config.settings[last].explicit = true;
    assert_eq!(
        config.setting(PolicyId::ForbiddenStrings),
        PolicySetting {
            id: PolicyId::ForbiddenStrings,
            severity: Severity::Warn,
            explicit: true,
        }
    );
    assert_eq!(
        config.setting(PolicyId::RequireRoot).severity,
        Severity::Error
    );
    assert_eq!(
        config.setting(PolicyId::MarkdownAutofix).severity,
        Severity::Off
    );
}

/// A settings list missing a row answers with the unlisted severity instead of stopping Git.
#[test]
fn setting_lookup_falls_back_to_the_unlisted_severity() {
    let mut config: PolicyConfig = PolicyConfig::defaults();
    config.settings.clear();
    for descriptor in POLICY_REGISTRY {
        assert_eq!(
            config.setting(descriptor.id),
            PolicySetting {
                id: descriptor.id,
                severity: unlisted_severity(descriptor),
                explicit: false,
            },
            "{}",
            descriptor.name
        );
    }
}

/// The defaults run the five built-ins,
///  leave the four optional policies off,
///  and carry the incumbent tuning and options.
#[test]
fn defaults_run_built_ins_only_with_incumbent_tuning() {
    let defaults: CliGitConfig = CliGitConfig::defaults();
    let mut running: Vec<PolicyId> = Vec::<PolicyId>::new();
    for descriptor in POLICY_REGISTRY {
        let setting: PolicySetting = defaults.policies.setting(descriptor.id);
        assert!(!setting.explicit, "{}", descriptor.name);
        if setting.severity != Severity::Off {
            assert_eq!(
                setting.severity, descriptor.default_severity,
                "{}",
                descriptor.name
            );
            running.push(descriptor.id);
        }
    }
    assert_eq!(
        running,
        [
            PolicyId::RequireRoot,
            PolicyId::LinkedWorktreeOnly,
            PolicyId::BranchWorktreeOnly,
            PolicyId::AddExplicit,
            PolicyId::FinalNewline,
        ]
    );
    assert_eq!(defaults.policies, PolicyConfig::defaults());
    assert_eq!(defaults.concurrency, ConcurrencyConfig::defaults());
    assert!(!defaults.concurrency.hooks.concurrent_commits);
    assert_eq!(
        defaults.concurrency.index_lock.unproven_owner_timeout_ms,
        DEFAULT_UNPROVEN_OWNER_TIMEOUT_MS
    );
    assert_eq!(DEFAULT_UNPROVEN_OWNER_TIMEOUT_MS, 1_000);
    assert_eq!(
        defaults.concurrency.landing.reserve_after_lost_races,
        DEFAULT_RESERVE_AFTER_LOST_RACES
    );
    assert_eq!(DEFAULT_RESERVE_AFTER_LOST_RACES, 1);
    assert_eq!(
        defaults.policies.markdown_autofix.rules,
        vec![MarkdownRule::LfsImageUrl]
    );
    assert_eq!(
        defaults.policies.markdown_autofix.exclude,
        Vec::<String>::new()
    );
    assert!(defaults.policies.forbidden_strings.builtin_rules);
}
