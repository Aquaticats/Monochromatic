//! What:
//!  Controls for the `rulesFile` value check:
//!  every refusal,
//!  its order and its words,
//!       and the names that stay inside the repository.
//! Why:
//!  A value that escaped the repository would let configuration choose a file
//!      anywhere on the machine;
//!  a refused ordinary name would block a valid setting.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(() => checkRulesFile('../x')).toThrow('dot-component');
//! ```

/// Import the module under test.
use super::{RulesFileRefusal, check_rules_file, rules_file_refusal_reason};

/// Relative names of non-empty,
///  non-dot components are accepted.
#[test]
fn names_inside_the_repository_are_accepted() {
    for name in [
        "rules.txt",
        ".cache/forbidden-strings.rules.txt",
        "a/b/c",
        "..hidden",
        ".hidden/...",
        "1:digit-is-not-a-drive",
        "C",
        "name with spaces/and\u{e9}",
        "trailing:colon:",
    ] {
        assert_eq!(check_rules_file(name), Ok(()), "{name}");
    }
}

/// Each way out of the repository,
///  or out of a portable name,
///  is refused with its reason,
/// the first reason in a fixed order.
#[test]
fn names_that_could_leave_the_repository_are_refused() {
    let cases: [(&str, RulesFileRefusal); 15] = [
        ("", RulesFileRefusal::Empty),
        ("/etc/rules.txt", RulesFileRefusal::Absolute),
        ("/", RulesFileRefusal::Absolute),
        ("C:/rules.txt", RulesFileRefusal::Drive),
        ("c:rules.txt", RulesFileRefusal::Drive),
        ("dir\\rules.txt", RulesFileRefusal::Backslash),
        ("\\\\server\\share", RulesFileRefusal::Backslash),
        ("rules\0.txt", RulesFileRefusal::Nul),
        ("a//b", RulesFileRefusal::EmptyComponent),
        ("dir/", RulesFileRefusal::EmptyComponent),
        ("..", RulesFileRefusal::DotComponent),
        ("../outside.txt", RulesFileRefusal::DotComponent),
        ("a/../../outside.txt", RulesFileRefusal::DotComponent),
        ("./rules.txt", RulesFileRefusal::DotComponent),
        ("a/./b", RulesFileRefusal::DotComponent),
    ];
    for (name, refusal) in cases {
        assert_eq!(check_rules_file(name), Err(refusal), "{name:?}");
    }
    // Order: absolute before backslash, drive before an empty component, NUL before dots.
    assert_eq!(check_rules_file("/a\\b"), Err(RulesFileRefusal::Absolute));
    assert_eq!(check_rules_file("C://x"), Err(RulesFileRefusal::Drive));
    assert_eq!(check_rules_file("a\0/../x"), Err(RulesFileRefusal::Nul));
}

/// Every refusal has its own words.
#[test]
fn every_refusal_is_explained() {
    let expected: [(RulesFileRefusal, &str); 7] = [
        (RulesFileRefusal::Empty, "is empty"),
        (RulesFileRefusal::Absolute, "starts with /"),
        (RulesFileRefusal::Drive, "starts with a drive letter"),
        (
            RulesFileRefusal::Backslash,
            "contains a backslash; use / between directories",
        ),
        (RulesFileRefusal::Nul, "contains a NUL character"),
        (
            RulesFileRefusal::EmptyComponent,
            "has an empty component (two slashes in a row or a trailing slash)",
        ),
        (RulesFileRefusal::DotComponent, "has a . or .. component"),
    ];
    for (refusal, words) in expected {
        assert_eq!(rules_file_refusal_reason(refusal), words, "{refusal:?}");
    }
}
