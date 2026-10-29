//! Real-wheel parsing covers boundaries before any synthetic input is emitted.

/// The wire parser is the same interface the Unix control socket consumes.
use nested_wayland_session::protocol::{parse_command, Command};

/// Positive and negative notch counts preserve direction and coordinates.
#[test]
fn wheel_fields_are_explicit() {
    assert_eq!(parse_command("wheel 120.5 300 0 1").expect("parse_command accepts a well-formed wheel line"),
        Command::Wheel { x: 120.5, y: 300.0, horizontal: 0.0, vertical: 1.0, modifiers: String::new() });
    assert_eq!(parse_command("wheel 10 20 -2 -3").expect("parse_command accepts a well-formed wheel line"),
        Command::Wheel { x: 10.0, y: 20.0, horizontal: -2.0, vertical: -3.0, modifiers: String::new() });
}

/// Fractional notches and held modifiers parse exactly.
#[test]
fn wheel_accepts_fractional_notches_and_modifiers() {
    assert_eq!(parse_command("wheel 1 2 0 0.25").expect("parse_command accepts a well-formed wheel line"),
        Command::Wheel { x: 1.0, y: 2.0, horizontal: 0.0, vertical: 0.25, modifiers: String::new() });
    assert_eq!(parse_command("wheel 1 2 -1.5 0 mods=ctrl+shift").expect("parse_command accepts a well-formed wheel line"),
        Command::Wheel { x: 1.0, y: 2.0, horizontal: -1.5, vertical: 0.0, modifiers: "ctrl+shift".to_string() });
}

/// Malformed commands never reach pointer injection.
#[test]
fn wheel_rejects_missing_extra_and_nonfinite_values() {
    for command in [
        "wheel", "wheel 1 2", "wheel 1 2 0",
        "wheel NaN 2 0 1", "wheel 1 inf 0 1",
        "wheel 1 2 0 20000", "wheel 1 2 0 NaN", "wheel 1 2 0 1 mods=bogus", "wheel 1 2 0 1 speed=2",
        "wheel 1 2 0 1 mods=ctrl mods=alt", "wheel 1 2 0 1 mods=",
        "wheel 1 2 0 1 extra", "wheel 1 2 0 1\nquit",
    ] {
        assert!(parse_command(command).is_err(), "{command}");
    }
}
