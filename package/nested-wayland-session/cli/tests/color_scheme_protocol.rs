//! Runtime appearance parsing covers its boundaries before any private-bus traffic.

/// The wire parser is the same interface the Unix control socket consumes.
use nested_wayland_session::{
    protocol::{parse_command, Command},
    ColorSchemePreference,
};

/// Runtime values are exactly the names the `--color-scheme` startup option accepts.
#[test]
fn color_scheme_values_match_the_startup_option() {
    assert_eq!(
        parse_command("color-scheme dark").expect("parse_command accepts a well-formed color-scheme line"),
        Command::ColorScheme(ColorSchemePreference::Dark),
    );
    // A trailing line terminator belongs to the socket framing, not to the value.
    assert_eq!(
        parse_command("color-scheme light\r\n").expect("parse_command accepts a well-formed color-scheme line"),
        Command::ColorScheme(ColorSchemePreference::Light),
    );
    // Runs of separators collapse like every other token verb.
    assert_eq!(
        parse_command("color-scheme \t dark").expect("parse_command accepts a well-formed color-scheme line"),
        Command::ColorScheme(ColorSchemePreference::Dark),
    );
}

/// The verb itself is known,
///  so a malformed value names the verb instead of "unknown command".
#[test]
fn color_scheme_rejects_missing_unknown_and_extra_values() {
    for command in [
        "color-scheme",
        "color-scheme ",
        "color-scheme system",
        "color-scheme Dark",
        "color-scheme 1",
        "color-scheme ../dark",
        "color-scheme dark light",
        "color-scheme dark; quit",
        "color-scheme dark\nquit",
        "color-scheme dark\0",
        "color-scheme \"dark\"",
    ] {
        // expect_err extracts the failure message and stops the test when parsing succeeded.
        let message = parse_command(command).expect_err(command);
        assert!(message.contains("color-scheme"), "{command:?}: {message}");
        assert!(!message.starts_with("unknown command"), "{command:?}: {message}");
    }
}

/// Lookalike spellings stay separate verbs and never reach the private portal.
#[test]
fn color_scheme_lookalike_verbs_are_unknown_commands() {
    for command in ["color-scheme=dark", "colorscheme dark", "color_scheme dark", "--color-scheme dark"] {
        let message = parse_command(command).expect_err(command);
        assert!(message.starts_with("unknown command"), "{command:?}: {message}");
    }
}
