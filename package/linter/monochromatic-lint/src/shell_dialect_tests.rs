//! What:
//!  Controls for the dialect table,
//!  the construct list and the `shell:` value classifier.
//! Why:
//!  A wrong table entry flags a PowerShell backtick as command substitution or lets an unlisted
//!  interpreter escape the ban.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('shell dialect', () => { /* suffix, applies, classify */ });
//! ```

/// Import the items under test.
use super::{CONSTRUCTS, Construct, ShellClass, ShellDialect, classify_shell};

/// Every dialect's suffix reads back to the same dialect.
#[test]
fn suffixes_round_trip() {
    for dialect in [
        ShellDialect::Posix,
        ShellDialect::Fish,
        ShellDialect::PowerShell,
        ShellDialect::Cmd,
    ] {
        let name: String = format!("ci.yml/3{}", dialect.suffix());
        assert_eq!(ShellDialect::from_filename(name.as_str()), Some(dialect));
    }
    assert_eq!(ShellDialect::from_filename("ci.yml/3.rs"), None);
    assert_eq!(ShellDialect::from_filename("ci.yml"), None);
}

/// The eight rule identifiers are distinct and live in the `shell/` namespace.
#[test]
fn rule_ids_are_distinct() {
    let mut seen: Vec<&str> = Vec::new();
    for construct in CONSTRUCTS {
        let id: &str = construct.rule_id();
        assert!(id.starts_with("shell/no-"), "{id}");
        assert!(!seen.contains(&id), "{id} repeats");
        seen.push(id);
        assert!(!construct.token().is_empty());
    }
    assert_eq!(seen.len(), 8);
}

/// The table follows the cited manuals:
///  fish has no backtick substitution,
///  PowerShell has neither backtick substitution nor `|&`,
///  and `cmd` has only four operators.
#[test]
fn dialect_table_matches_the_manuals() {
    for construct in CONSTRUCTS {
        assert!(construct.applies_to(ShellDialect::Posix));
    }
    assert!(!Construct::Backtick.applies_to(ShellDialect::Fish));
    assert!(Construct::PipeStderr.applies_to(ShellDialect::Fish));
    assert!(!Construct::Backtick.applies_to(ShellDialect::PowerShell));
    assert!(!Construct::PipeStderr.applies_to(ShellDialect::PowerShell));
    assert!(Construct::DollarParen.applies_to(ShellDialect::PowerShell));
    assert!(Construct::Semicolon.applies_to(ShellDialect::PowerShell));
    for construct in [
        Construct::Semicolon,
        Construct::DollarParen,
        Construct::Backtick,
        Construct::PipeStderr,
    ] {
        assert!(!construct.applies_to(ShellDialect::Cmd), "{construct:?}");
    }
    for construct in [
        Construct::Pipe,
        Construct::AndChain,
        Construct::OrChain,
        Construct::Ampersand,
    ] {
        assert!(construct.applies_to(ShellDialect::Cmd), "{construct:?}");
    }
}

/// Listed shells select their dialect,
///  with directories,
///  `.exe`,
///  `env` and templates ignored.
#[test]
fn shells_select_their_dialect() {
    let cases: [(&str, ShellDialect); 9] = [
        ("bash", ShellDialect::Posix),
        ("bash -e {0}", ShellDialect::Posix),
        ("/usr/bin/zsh {0}", ShellDialect::Posix),
        ("env bash {0}", ShellDialect::Posix),
        ("env -i FOO=1 sh -c {0}", ShellDialect::Posix),
        ("fish", ShellDialect::Fish),
        ("pwsh", ShellDialect::PowerShell),
        ("\"C:\\Program Files\\PowerShell\\7\\pwsh.exe\" -File {0}", ShellDialect::PowerShell),
        ("CMD /D /E:ON /V:ON /S /C \"CALL \"{0}\"\"", ShellDialect::Cmd),
    ];
    for (value, dialect) in cases {
        assert_eq!(classify_shell(value), ShellClass::Dialect(dialect), "{value}");
    }
}

/// Known non-shell interpreters are out of scope.
#[test]
fn interpreters_are_out_of_scope() {
    for value in ["node {0}", "python", "python3.12 {0}", "env node {0}", "deno run {0}"] {
        assert_eq!(classify_shell(value), ShellClass::OutOfScope, "{value}");
    }
}

/// Expressions,
///  empty values and unlisted interpreters fail closed.
#[test]
fn unknown_values_fail_closed() {
    for value in [
        "${{ matrix.shell }}",
        "",
        "   ",
        "env",
        "elvish {0}",
        "nu {0}",
        "C:\\Program Files\\PowerShell\\7\\pwsh.exe -File {0}",
        "\"\" {0}",
    ] {
        assert!(
            matches!(classify_shell(value), ShellClass::Unknown(_)),
            "{value:?}"
        );
    }
}
