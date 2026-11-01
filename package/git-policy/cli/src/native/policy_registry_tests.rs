//! What:
//!  Controls for the compiled-in policy registry.
//! Why:
//!  Policy IDs,
//!  order and defaults are a stable contract for configuration and JSONL.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(POLICY_REGISTRY.map(p => p.name)).toEqual([...]);
//! ```

/// Import the registry and its lookups from the module under test.
use super::{
    POLICY_REGISTRY, PolicyId, Severity, policy_by_name, policy_descriptor, severity_from_name,
    severity_name,
};

/// The registry lists exactly the shipped IDs,
///  built-ins first,
///  in execution order.
#[test]
fn registry_order_and_names_are_stable() {
    let mut names: Vec<&str> = Vec::<&str>::new();
    for descriptor in POLICY_REGISTRY {
        names.push(descriptor.name);
    }
    assert_eq!(
        names,
        vec![
            "require-root",
            "linked-worktree-only",
            "branch-worktree-only",
            "add-explicit",
            "final-newline",
            "markdown/autofix",
            "mono/forbidden-root-context",
            "mono/dependent-version-bump",
            "security/forbidden-strings",
        ]
    );
}

/// Every policy's default severity is pinned to the incumbent definition's `defaultSeverity`.
/// Sources:
///  `src/policy-engine/{require-root,linked-worktree,branch-worktree,add-explicit,final-newline}-policy.ts`,
/// `src/optional/markdown-lint/index.ts`,
///  `src/optional/repository-policy/index.ts`,
/// `src/optional/repository-policy/dependent-version-bump-policy.ts`,
///  `src/optional/forbidden-strings/index.ts`.
#[test]
fn every_identity_resolves_to_its_declared_row() {
    // (identity, name, incumbent defaultSeverity, warnSafe, accepts options, off unless listed)
    let expected: [(PolicyId, &str, Severity, bool, bool, bool); 9] = [
        (
            PolicyId::RequireRoot,
            "require-root",
            Severity::Error,
            false,
            false,
            false,
        ),
        (
            PolicyId::LinkedWorktreeOnly,
            "linked-worktree-only",
            Severity::Error,
            false,
            false,
            false,
        ),
        (
            PolicyId::BranchWorktreeOnly,
            "branch-worktree-only",
            Severity::Error,
            true,
            false,
            false,
        ),
        (
            PolicyId::AddExplicit,
            "add-explicit",
            Severity::Error,
            false,
            false,
            false,
        ),
        (
            PolicyId::FinalNewline,
            "final-newline",
            Severity::Warn,
            true,
            false,
            false,
        ),
        (
            PolicyId::MarkdownAutofix,
            "markdown/autofix",
            Severity::Warn,
            true,
            true,
            true,
        ),
        (
            PolicyId::ForbiddenRootContext,
            "mono/forbidden-root-context",
            Severity::Error,
            true,
            false,
            true,
        ),
        (
            PolicyId::DependentVersionBump,
            "mono/dependent-version-bump",
            Severity::Error,
            false,
            false,
            true,
        ),
        (
            PolicyId::ForbiddenStrings,
            "security/forbidden-strings",
            Severity::Error,
            false,
            true,
            true,
        ),
    ];
    assert_eq!(
        expected.len(),
        POLICY_REGISTRY.len(),
        "every registry row is pinned"
    );
    for (id, name, default_severity, warn_safe, accepts_options, off_unless_listed) in expected {
        let descriptor = policy_descriptor(id);
        assert_eq!(descriptor.id, id);
        assert_eq!(descriptor.name, name);
        assert_eq!(descriptor.default_severity, default_severity, "{name}");
        assert_eq!(descriptor.warn_safe, warn_safe, "{name}");
        assert_eq!(descriptor.accepts_options, accepts_options, "{name}");
        assert_eq!(descriptor.off_unless_listed, off_unless_listed, "{name}");
        assert_eq!(policy_by_name(name), Some(descriptor));
    }
    // No incumbent default is off. Whether a policy the configuration does not name runs
    // at all is decided by `off_unless_listed`, pinned row by row in the list here.
    for descriptor in POLICY_REGISTRY {
        assert_ne!(
            descriptor.default_severity,
            Severity::Off,
            "{}",
            descriptor.name
        );
    }
    let mut seen: usize = 0;
    for first in POLICY_REGISTRY {
        for second in POLICY_REGISTRY {
            if first.id == second.id || first.name == second.name {
                seen += 1;
            }
        }
    }
    assert_eq!(
        seen,
        POLICY_REGISTRY.len(),
        "identities and names are unique"
    );
}

/// Near-miss spellings never resolve to a shipped policy.
#[test]
fn unknown_policy_names_are_absent() {
    for name in [
        "",
        "Require-Root",
        "require-root ",
        "require",
        "require-root/",
        "forbidden-strings",
        "security/forbidden-strings/extra",
        "../security/forbidden-strings",
        "mono\\forbidden-root-context",
        "final-newline\0",
    ] {
        assert_eq!(policy_by_name(name), None, "{name:?}");
    }
}

/// Only the three documented words are severities,
///  and each prints back unchanged.
#[test]
fn severities_round_trip_and_reject_other_spellings() {
    for (name, severity) in [
        ("off", Severity::Off),
        ("warn", Severity::Warn),
        ("error", Severity::Error),
    ] {
        assert_eq!(severity_from_name(name), Some(severity));
        assert_eq!(severity_name(severity), name);
    }
    for name in [
        "", "Off", "ERROR", "warning", "err", " error", "error\n", "2",
    ] {
        assert_eq!(severity_from_name(name), None, "{name:?}");
    }
}
