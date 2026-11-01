//! Runtime output-scale parsing covers its boundaries before any surface is told a scale.

/// The wire parser is the same interface the Unix control socket consumes.
use nested_wayland_session::{
    protocol::{parse_command, Command},
    screen_geometry::OutputScale,
};

/// Parse a scale the test knows is valid;
///  a parse failure stops the test with its message.
fn scale(text: &str) -> OutputScale {
    return OutputScale::parse(text).expect(text);
}

/// The verb takes exactly the values the `--scale` startup option takes.
#[test]
fn scale_values_match_the_startup_option() {
    assert_eq!(parse_command("scale 1.25").expect("parse_command accepts a well-formed scale line"), Command::Scale(scale("1.25")));
    assert_eq!(parse_command("scale 2").expect("parse_command accepts a well-formed scale line"), Command::Scale(scale("2")));
    // A trailing line terminator belongs to the socket framing, not to the value.
    assert_eq!(parse_command("scale 1.5\r\n").expect("parse_command accepts a well-formed scale line"), Command::Scale(scale("1.5")));
    // Runs of separators collapse like every other token verb.
    assert_eq!(parse_command("scale \t 1").expect("parse_command accepts a well-formed scale line"), Command::Scale(OutputScale::ONE));
}

/// The verb itself is known,
///  so a malformed value names the verb instead of "unknown command".
#[test]
fn scale_rejects_missing_unknown_and_extra_values() {
    for command in [
        "scale",
        "scale ",
        "scale 0",
        "scale 1.333",
        "scale 3.5",
        "scale -2",
        "scale NaN",
        "scale inf",
        "scale 125%",
        "scale 1,25",
        "scale 1.25 2",
        "scale 1.25; quit",
        "scale 1.25\nquit",
        "scale 1.25\0",
        "scale \"2\"",
        "scale ../2",
    ] {
        // expect_err extracts the failure message and stops the test when parsing succeeded.
        let message = parse_command(command).expect_err(command);
        assert!(message.contains("scale"), "{command:?}: {message}");
        assert!(!message.starts_with("unknown command"), "{command:?}: {message}");
    }
}

/// Lookalike spellings stay separate verbs and never change the output scale.
#[test]
fn scale_lookalike_verbs_are_unknown_commands() {
    for command in ["scale=2", "--scale 2", "output-scale 2", "Scale 2", "scale-factor 2"] {
        let message = parse_command(command).expect_err(command);
        assert!(message.starts_with("unknown command"), "{command:?}: {message}");
    }
}
