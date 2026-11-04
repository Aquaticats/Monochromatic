//! What:
//!  Controls for environment lookup and `GIT_CONFIG_COUNT` parsing,
//!  checked against real Git.
//! Why:
//!  The overlay may only append after the entries Git 2.56.0 itself would read,
//!  so the
//!      count must be accepted and rejected exactly where Git accepts and rejects it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parseConfigCount('+1')).toBe(1n);
//! ```

/// Import the functions under test,
///  the shared environment builder and native string types.
use super::tests::environment;
use super::{COUNT_VARIABLE, environment_value, parse_config_count};
use std::ffi::{OsStr, OsString};
use std::process::Command;

/// Lookup follows `getenv`:
///  exact name,
///  first entry wins,
///  absence is distinct from empty.
#[test]
fn environment_lookup_matches_getenv() {
    let pairs: Vec<(OsString, OsString)> = environment(&[
        ("A", "first"),
        ("A", "second"),
        ("EMPTY", ""),
        ("a", "lowercase"),
    ]);
    assert_eq!(
        environment_value(pairs.as_slice(), "A"),
        Some(OsString::from("first"))
    );
    assert_eq!(
        environment_value(pairs.as_slice(), "EMPTY"),
        Some(OsString::new())
    );
    assert_eq!(
        environment_value(pairs.as_slice(), "a"),
        Some(OsString::from("lowercase"))
    );
    assert_eq!(environment_value(pairs.as_slice(), "B"), None);
    assert_eq!(environment_value(pairs.as_slice(), ""), None);
}

/// Accepted spellings convert to their count.
#[test]
fn config_count_accepts_what_strtoul_and_git_accept() {
    for (value, expected) in [
        ("", 0),
        ("0", 0),
        ("7", 7),
        ("007", 7),
        (" 2", 2),
        ("\t\n\x0b\x0c\r 3", 3),
        ("+4", 4),
        (" +5", 5),
        ("-0", 0),
        ("2147483647", 2_147_483_647),
    ] {
        assert_eq!(
            parse_config_count(OsStr::new(value)),
            Some(expected),
            "{value:?}"
        );
    }
}

/// Every spelling Git reports as bogus or too large is rejected.
#[test]
fn config_count_rejects_what_git_rejects() {
    for value in [
        " ",
        "+",
        "-",
        "x",
        "1x",
        "2 ",
        "1.5",
        "0x10",
        "+-1",
        "- 1",
        "1 2",
        "-1",
        "-2147483648",
        "2147483648",
        "4294967296",
        "18446744073709551615",
        "18446744073709551616",
        "99999999999999999999999999",
        "\u{a0}1",
        "\u{661}",
    ] {
        assert_eq!(parse_config_count(OsStr::new(value)), None, "{value:?}");
    }
}

/// Real Git 2.56.0 accepts and rejects exactly the same count spellings.
#[test]
fn native_git_agrees_on_count_validity() {
    for value in [
        "",
        "0",
        "1",
        " 1",
        "+1",
        "-0",
        "007",
        "\t2",
        "-1",
        "1x",
        "2 ",
        " ",
        "+",
        "x",
        "1.5",
        "0x10",
        "2147483647",
        "2147483648",
        "99999999999999999999999999",
    ] {
        let output = Command::new("/usr/bin/git")
            .env_clear()
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
            .env(COUNT_VARIABLE, value)
            .args(["config", "--get", "core.lockfilepid"])
            .current_dir("/")
            .output()
            .expect("native Git count probe");
        let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
        let git_accepts: bool =
            !stderr.contains("bogus count") && !stderr.contains("too many entries");
        assert_eq!(
            parse_config_count(OsStr::new(value)).is_some(),
            git_accepts,
            "{value:?}: {stderr}"
        );
    }
}
