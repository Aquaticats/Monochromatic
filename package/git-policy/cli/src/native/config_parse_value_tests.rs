//! What:
//!  Rejection controls for invalid values inside known configuration keys.
//! Why:
//!  A wrong type,
//!  an out-of-range number,
//!  an unknown policy ID or an unknown option
//!      must fail with a message naming the key,
//!  and must reject the whole document.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(() => parseConfig('{"landing":{"reserveAfterLostRaces":-1}}')).toThrow(/landing/);
//! ```

/// Import the parser under test and the rejection helpers shared with the structural controls.
use super::parse_config;
use super::tests::{assert_rejections, rejection};

/// Concurrency sections reject wrong shapes,
///  unknown nested keys and out-of-range numbers.
#[test]
fn invalid_concurrency_values_are_rejected_by_key() {
    assert_rejections(&[
        (
            r#"{ "hooks": [] }"#,
            "Configuration key hooks must be an object, found an array.",
        ),
        (
            r#"{ "indexLock": 1000 }"#,
            "Configuration key indexLock must be an object, found a number.",
        ),
        (
            r#"{ "landing": "1" }"#,
            "Configuration key landing must be an object, found a string.",
        ),
        (
            r#"{ "hooks": { "concurrent": true } }"#,
            "Unknown configuration key: hooks.concurrent. The only accepted key under hooks is concurrentCommits.",
        ),
        (
            r#"{ "indexLock": { "timeoutMs": 1 } }"#,
            "Unknown configuration key: indexLock.timeoutMs. The only accepted key under indexLock is unprovenOwnerTimeoutMs.",
        ),
        (
            r#"{ "landing": { "reserveAfterLostRaces": 1, "extra": 1 } }"#,
            "Unknown configuration key: landing.extra. The only accepted key under landing is reserveAfterLostRaces.",
        ),
        (
            r#"{ "hooks": { "unprovenOwnerTimeoutMs": 1 } }"#,
            "Unknown configuration key: hooks.unprovenOwnerTimeoutMs.",
        ),
        (
            r#"{ "hooks": { "concurrentCommits": "true" } }"#,
            "Configuration key hooks.concurrentCommits must be true or false, found a string.",
        ),
        (
            r#"{ "hooks": { "concurrentCommits": 1 } }"#,
            "Configuration key hooks.concurrentCommits must be true or false, found a number.",
        ),
        (
            r#"{ "indexLock": { "unprovenOwnerTimeoutMs": -1 } }"#,
            "Configuration key indexLock.unprovenOwnerTimeoutMs must be a whole number from 0 to 9007199254740991, found -1.",
        ),
        (
            r#"{ "indexLock": { "unprovenOwnerTimeoutMs": 0.5 } }"#,
            "Configuration key indexLock.unprovenOwnerTimeoutMs must be a whole number from 0",
        ),
        (
            r#"{ "indexLock": { "unprovenOwnerTimeoutMs": 9007199254740992 } }"#,
            "Configuration key indexLock.unprovenOwnerTimeoutMs must be a whole number from 0",
        ),
        (
            r#"{ "indexLock": { "unprovenOwnerTimeoutMs": "1000" } }"#,
            "Configuration key indexLock.unprovenOwnerTimeoutMs must be a whole number from 0 to 9007199254740991, found a string.",
        ),
        (
            r#"{ "landing": { "reserveAfterLostRaces": 0 } }"#,
            "Configuration key landing.reserveAfterLostRaces must be a whole number from 1 to 9007199254740991, found 0.",
        ),
        (
            r#"{ "landing": { "reserveAfterLostRaces": true } }"#,
            "Configuration key landing.reserveAfterLostRaces must be a whole number from 1",
        ),
    ]);
}

/// Unknown policy IDs,
///  including traversal-shaped and namespace near misses,
///  list the shipped IDs.
#[test]
fn unknown_policy_ids_are_rejected_with_the_shipped_list() {
    let listed: &str = "Shipped policies: require-root, linked-worktree-only, branch-worktree-only, \
                        add-explicit, final-newline, markdown/autofix, mono/forbidden-root-context, \
                        mono/dependent-version-bump, security/forbidden-strings.";
    for id in [
        "forbidden-strings",
        "security/forbidden-string",
        "Security/forbidden-strings",
        "../security/forbidden-strings",
        "security/forbidden-strings/",
        "my-plugin/custom",
        "",
        "require root",
        "--no-enforce-require-root",
    ] {
        let source: String = format!(r#"{{ "policies": {{ "{id}": "off" }} }}"#);
        let message: String = rejection(source.as_str());
        assert_eq!(
            message,
            format!("Unknown policy ID: {id}. {listed}"),
            "{source}"
        );
    }
}

/// Settings of the wrong shape or with an unknown severity word are rejected by key.
#[test]
fn invalid_policy_settings_are_rejected_by_key() {
    assert_rejections(&[
        (
            r#"{ "policies": [] }"#,
            "Configuration key policies must be an object mapping policy IDs to settings, found an array.",
        ),
        (
            r#"{ "policies": { "require-root": "fatal" } }"#,
            "Configuration key policies.require-root has an invalid severity \"fatal\"; use \"off\", \"warn\" or \"error\".",
        ),
        (
            r#"{ "policies": { "require-root": "Error" } }"#,
            "Configuration key policies.require-root has an invalid severity \"Error\"",
        ),
        (
            r#"{ "policies": { "require-root": "" } }"#,
            "Configuration key policies.require-root has an invalid severity \"\"",
        ),
        (
            r#"{ "policies": { "require-root": 2 } }"#,
            "Configuration key policies.require-root must be a severity string, found a number.",
        ),
        (
            r#"{ "policies": { "require-root": true } }"#,
            "Configuration key policies.require-root must be a severity string, found a boolean.",
        ),
        (
            r#"{ "policies": { "require-root": { "severity": "error" } } }"#,
            "Configuration key policies.require-root must be a severity string, found an object.",
        ),
        (
            r#"{ "policies": { "markdown/autofix": 1 } }"#,
            "Configuration key policies.markdown/autofix must be a severity string, [severity, options] or an options object, found a number.",
        ),
        (
            r#"{ "policies": { "require-root": ["error", {}] } }"#,
            "Configuration key policies.require-root does not accept options; write the severity alone",
        ),
        (
            r#"{ "policies": { "mono/dependent-version-bump": ["error"] } }"#,
            "Configuration key policies.mono/dependent-version-bump does not accept options",
        ),
        (
            r#"{ "policies": { "markdown/autofix": [] } }"#,
            "Configuration key policies.markdown/autofix must be a severity or exactly [severity, options]; found an array of 0 items.",
        ),
        (
            r#"{ "policies": { "markdown/autofix": ["warn"] } }"#,
            "found an array of 1 items.",
        ),
        (
            r#"{ "policies": { "security/forbidden-strings": ["warn", {}, {}] } }"#,
            "Configuration key policies.security/forbidden-strings must be a severity or exactly [severity, options]; found an array of 3 items.",
        ),
        (
            r#"{ "policies": { "markdown/autofix": [{}, "warn"] } }"#,
            "Configuration key policies.markdown/autofix[0] must be a string, found an object.",
        ),
        (
            r#"{ "policies": { "markdown/autofix": ["loud", {}] } }"#,
            "Configuration key policies.markdown/autofix[0] has an invalid severity \"loud\"",
        ),
        (
            r#"{ "policies": { "markdown/autofix": ["warn", []] } }"#,
            "Configuration key policies.markdown/autofix[1] must be an options object, found an array.",
        ),
        (
            r#"{ "policies": { "security/forbidden-strings": ["warn", "x"] } }"#,
            "Configuration key policies.security/forbidden-strings[1] must be an options object, found a string.",
        ),
    ]);
}

/// Option keys outside each policy's schema,
///  including program-selecting ones,
///  are rejected.
#[test]
fn invalid_policy_options_are_rejected_by_key() {
    assert_rejections(&[
        (
            r#"{ "policies": { "security/forbidden-strings": ["error", { "executable": "./scanner" }] } }"#,
            "Configuration key policies.security/forbidden-strings[1].executable is retired",
        ),
        (
            r#"{ "policies": { "security/forbidden-strings": ["error", { "rules": [] }] } }"#,
            "Unknown configuration key: policies.security/forbidden-strings[1].rules. Accepted options: builtinRules, rulesFile.",
        ),
        (
            r#"{ "policies": { "security/forbidden-strings": ["error", { "rulesFile": 1 }] } }"#,
            "Configuration key policies.security/forbidden-strings[1].rulesFile must be a file name, or an array of file names and objects with \"path\" and an optional \"optional\", found a number.",
        ),
        (
            r#"{ "policies": { "security/forbidden-strings": ["error", { "rulesFile": [] }] } }"#,
            "Configuration key policies.security/forbidden-strings[1].rulesFile must name at least one file",
        ),
        (
            r#"{ "policies": { "security/forbidden-strings": ["error", { "rulesFile": ["a.txt", 1] }] } }"#,
            "Configuration key policies.security/forbidden-strings[1].rulesFile[1] must be a file name or an object with \"path\" and an optional \"optional\", found a number.",
        ),
        (
            r#"{ "policies": { "security/forbidden-strings": ["error", { "rulesFile": ["a.txt", "b/../c.txt"] }] } }"#,
            "Configuration key policies.security/forbidden-strings[1].rulesFile[1] must name a file relative to the repository's top level that stays inside it, but the value has a . or .. component.",
        ),
        (
            r#"{ "policies": { "security/forbidden-strings": ["error", { "rulesFile": [{ "path": "/etc/rules.txt" }] }] } }"#,
            "Configuration key policies.security/forbidden-strings[1].rulesFile[0].path must name a file relative to the repository's top level that stays inside it, but the value starts with /.",
        ),
        (
            r#"{ "policies": { "security/forbidden-strings": ["error", { "rulesFile": [{ "optional": true }] }] } }"#,
            "Configuration key policies.security/forbidden-strings[1].rulesFile[0] must have a \"path\" key naming the rules file.",
        ),
        (
            r#"{ "policies": { "security/forbidden-strings": ["error", { "rulesFile": [{ "path": "a.txt", "optionnal": true }] }] } }"#,
            "Unknown configuration key: policies.security/forbidden-strings[1].rulesFile[0].optionnal. Accepted keys: path, optional.",
        ),
        (
            r#"{ "policies": { "security/forbidden-strings": ["error", { "rulesFile": [{ "path": "a.txt", "optional": "yes" }] }] } }"#,
            "Configuration key policies.security/forbidden-strings[1].rulesFile[0].optional must be true or false, found a string.",
        ),
        (
            r#"{ "policies": { "security/forbidden-strings": ["error", { "rulesFile": [{ "path": "a.txt", "path": "b.txt" }] }] } }"#,
            "Configuration key policies.security/forbidden-strings[1].rulesFile[0].path is defined more than once",
        ),
        (
            r#"{ "policies": { "security/forbidden-strings": ["error", { "rulesFile": ["a.txt", { "path": "a.txt", "optional": true }] }] } }"#,
            "Configuration key policies.security/forbidden-strings[1].rulesFile[1] names the same file as policies.security/forbidden-strings[1].rulesFile[0]",
        ),
        (
            r#"{ "policies": { "security/forbidden-strings": ["error", { "rulesFile": "../outside.txt" }] } }"#,
            "Configuration key policies.security/forbidden-strings[1].rulesFile must name a file relative to the repository's top level that stays inside it, but the value has a . or .. component.",
        ),
        (
            r#"{ "policies": { "security/forbidden-strings": { "rulesFile": "/etc/rules.txt" } } }"#,
            "Configuration key policies.security/forbidden-strings.rulesFile must name a file relative to the repository's top level that stays inside it, but the value starts with /.",
        ),
        (
            r#"{ "policies": { "security/forbidden-strings": ["error", { "builtinRules": "yes" }] } }"#,
            "Configuration key policies.security/forbidden-strings[1].builtinRules must be true or false, found a string.",
        ),
        (
            r#"{ "policies": { "markdown/autofix": ["warn", { "command": ["node", "x.ts"] }] } }"#,
            "Configuration key policies.markdown/autofix[1].command is retired",
        ),
        (
            r#"{ "policies": { "markdown/autofix": ["warn", { "builtinRules": true }] } }"#,
            "Unknown configuration key: policies.markdown/autofix[1].builtinRules. Accepted options: rules, exclude.",
        ),
        (
            r#"{ "policies": { "markdown/autofix": ["warn", { "rules": "lfs-image-url" }] } }"#,
            "Configuration key policies.markdown/autofix[1].rules must be an array of strings, found a string.",
        ),
        (
            r#"{ "policies": { "markdown/autofix": ["warn", { "rules": [] }] } }"#,
            "Configuration key policies.markdown/autofix[1].rules must name at least one rule",
        ),
        (
            r#"{ "policies": { "markdown/autofix": ["warn", { "rules": ["lfs-image-url", "semantic-line-breaks"] }] } }"#,
            "Configuration key policies.markdown/autofix[1].rules[1] names an unknown Markdown rule \"semantic-line-breaks\"",
        ),
        (
            r#"{ "policies": { "markdown/autofix": ["warn", { "rules": ["--fix"] }] } }"#,
            "Configuration key policies.markdown/autofix[1].rules[0] names an unknown Markdown rule \"--fix\"",
        ),
        (
            r#"{ "policies": { "markdown/autofix": ["warn", { "rules": ["lfs-image-url", "lfs-image-url"] }] } }"#,
            "Configuration key policies.markdown/autofix[1].rules[1] repeats the Markdown rule \"lfs-image-url\"",
        ),
        (
            r#"{ "policies": { "markdown/autofix": ["warn", { "rules": [1] }] } }"#,
            "Configuration key policies.markdown/autofix[1].rules[0] must be a string, found a number.",
        ),
        (
            r#"{ "policies": { "markdown/autofix": ["warn", { "exclude": "package/ssg/" }] } }"#,
            "Configuration key policies.markdown/autofix[1].exclude must be an array of strings, found a string.",
        ),
        (
            r#"{ "policies": { "markdown/autofix": ["warn", { "exclude": ["a", ["b"]] }] } }"#,
            "Configuration key policies.markdown/autofix[1].exclude[1] must be a string, found an array.",
        ),
    ]);
}

/// A rejected later key never leaks earlier settings:
///  the whole document fails.
#[test]
fn any_invalid_key_rejects_the_whole_document() {
    assert!(
        parse_config(r#"{ "hooks": { "concurrentCommits": true }, "landing": { "reserveAfterLostRaces": 0 } }"#)
            .is_err()
    );
    assert!(
        parse_config(r#"{ "policies": { "final-newline": "error", "unknown": "off" } }"#).is_err()
    );
}
