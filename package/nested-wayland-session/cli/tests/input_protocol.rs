//! The key,
//!  click,
//!  drag,
//!  scroll,
//!  drop-file,
//!  and session verbs added in 0.4.0 parse exactly and fail whole on bad input.

/// The wire parser is the same interface the Unix control socket consumes.
use std::path::PathBuf;

use nested_wayland_session::protocol::{parse_command, Command, KeyAction, PointerButton};

/// A key command with a name and an action.
fn key(name: &str, action: KeyAction) -> Result<Command, String> {
    return Ok(Command::Key { name: name.to_string(), action });
}

/// Press,
///  release,
///  tap,
///  and hold are the four actions,
///  and tap is the default.
#[test]
fn key_actions_include_hold() {
    assert_eq!(parse_command("key f10"), key("f10", KeyAction::Tap));
    assert_eq!(parse_command("key f10 tap"), key("f10", KeyAction::Tap));
    assert_eq!(parse_command("key Menu press"), key("Menu", KeyAction::Press));
    assert_eq!(parse_command("key ctrl+shift+f5 release"), key("ctrl+shift+f5", KeyAction::Release));
    assert_eq!(parse_command("key right hold 1500"), key("right", KeyAction::Hold { millis: 1500 }));
    assert_eq!(parse_command("key code:68"), key("code:68", KeyAction::Tap));
    assert_eq!(parse_command("key ctrl++"), key("ctrl++", KeyAction::Tap));
}

/// A hold needs a positive whole duration within the bound,
///  and nothing may follow the action.
#[test]
fn key_hold_rejects_bad_durations_and_extra_words() {
    for command in [
        "key a hold",
        "key a hold 0",
        "key a hold -5",
        "key a hold 1.5",
        "key a hold soon",
        "key a hold 600001",
        "key a hold 100 200",
        "key a tap now",
        "key a wiggle",
        "key",
    ] {
        assert!(parse_command(command).is_err(), "{command}");
    }
}

/// A click takes a button,
///  a count,
///  an interval,
///  and held modifiers in any order after the coordinates.
#[test]
fn click_options_parse() {
    assert_eq!(
        parse_command("click 5 6 right count=2 mods=ctrl+shift interval=120"),
        Ok(Command::Click {
            x: 5.0,
            y: 6.0,
            button: PointerButton::Right,
            count: 2,
            interval_ms: 120,
            modifiers: "ctrl+shift".to_string(),
        }),
    );
    assert_eq!(
        parse_command("click 5 6 mods=alt"),
        Ok(Command::Click {
            x: 5.0,
            y: 6.0,
            button: PointerButton::Left,
            count: 1,
            interval_ms: 80,
            modifiers: "alt".to_string(),
        }),
    );
}

/// Bad click options fail whole and name the verb.
#[test]
fn click_options_reject_bad_input() {
    for command in [
        "click 1 2 count=0",
        "click 1 2 count=11",
        "click 1 2 count=two",
        "click 1 2 interval=-1",
        "click 1 2 interval=9999",
        "click 1 2 mods=nokey",
        "click 1 2 mods=",
        "click 1 2 speed=3",
        "click 1 2 count=2 count=3",
        "click 1 2 left right",
        "click 1 2 left; quit",
    ] {
        let message = parse_command(command).expect_err(command);
        assert!(message.starts_with("click") || message.starts_with("unknown button"), "{command}: {message}");
    }
}

/// A drag has both end points,
///  a button,
///  steps,
///  a step delay,
///  and modifiers.
#[test]
fn drag_parses_end_points_and_options() {
    assert_eq!(
        parse_command("drag 10 20 110 20"),
        Ok(Command::Drag {
            from: (10.0, 20.0),
            to: (110.0, 20.0),
            button: PointerButton::Left,
            steps: 10,
            step_ms: 16,
            modifiers: String::new(),
        }),
    );
    assert_eq!(
        parse_command("drag 1 2 3 4 middle steps=3 step-ms=0 mods=shift"),
        Ok(Command::Drag {
            from: (1.0, 2.0),
            to: (3.0, 4.0),
            button: PointerButton::Middle,
            steps: 3,
            step_ms: 0,
            modifiers: "shift".to_string(),
        }),
    );
    for command in ["drag", "drag 1 2 3", "drag 1 2 3 x", "drag 1 2 3 4 steps=0", "drag 1 2 3 4 steps=1001", "drag 1 2 3 inf"] {
        assert!(parse_command(command).is_err(), "{command}");
    }
}

/// A touchpad scroll has a point,
///  a travel,
///  and optional steps.
#[test]
fn scroll_parses_travel_and_steps() {
    assert_eq!(
        parse_command("scroll 100 200 0 -48.5 steps=4 step-ms=8"),
        Ok(Command::Scroll { x: 100.0, y: 200.0, dx: 0.0, dy: -48.5, steps: 4, step_ms: 8, modifiers: String::new() }),
    );
    for command in ["scroll", "scroll 1 2 3", "scroll 1 2 3 NaN", "scroll 1 2 3 4 left", "scroll 1 2 3 4 steps=0"] {
        assert!(parse_command(command).is_err(), "{command}");
    }
}

/// A drop-file path with spaces is quoted,
///  several paths make one drop,
///  and the position follows `at`.
#[test]
fn drop_file_quotes_spaces_and_takes_several_paths() {
    assert_eq!(
        parse_command("drop-file \"/music/My Album/a b.mp3\""),
        Ok(Command::DropFile { paths: vec![PathBuf::from("/music/My Album/a b.mp3")], x: None, y: None }),
    );
    assert_eq!(
        parse_command("drop-file \"/m/a b.mp3\" /m/c.mp3 \"/m/q\\\"uote.mp3\" at 100 40"),
        Ok(Command::DropFile {
            paths: vec![PathBuf::from("/m/a b.mp3"), PathBuf::from("/m/c.mp3"), PathBuf::from("/m/q\"uote.mp3")],
            x: Some(100.0),
            y: Some(40.0),
        }),
    );
    assert_eq!(
        parse_command("drop-file /m/a.mp3 /m/b.mp3"),
        Ok(Command::DropFile { paths: vec![PathBuf::from("/m/a.mp3"), PathBuf::from("/m/b.mp3")], x: None, y: None }),
    );
}

/// The earlier `PATH X Y` form still works,
///  and a quoted `at` is a file name.
#[test]
fn drop_file_keeps_the_earlier_form_and_quoted_keywords() {
    assert_eq!(
        parse_command("drop-file /tmp/a.mp3 100 40"),
        Ok(Command::DropFile { paths: vec![PathBuf::from("/tmp/a.mp3")], x: Some(100.0), y: Some(40.0) }),
    );
    assert_eq!(
        parse_command("drop-file /m/x \"at\" 1 2"),
        Ok(Command::DropFile {
            paths: vec![PathBuf::from("/m/x"), PathBuf::from("at"), PathBuf::from("1"), PathBuf::from("2")],
            x: None,
            y: None,
        }),
    );
}

/// Unterminated quotes,
///  text after a closing quote,
///  an empty list,
///  and a bad position fail whole.
#[test]
fn drop_file_rejects_malformed_lines() {
    for command in [
        "drop-file",
        "drop-file \"/m/a b.mp3",
        "drop-file \"/m/a\"b",
        "drop-file at 1 2",
        "drop-file /m/a at x y",
        "drop-file /m/a at NaN 2",
        "drop-file /m/a 1",
    ] {
        assert!(parse_command(command).is_err(), "{command}");
    }
}

/// The session verbs parse with and without arguments.
#[test]
fn session_verbs_parse() {
    assert_eq!(parse_command("repeat-info"), Ok(Command::RepeatInfo(None)));
    assert_eq!(parse_command("repeat-info 30 400"), Ok(Command::RepeatInfo(Some((30, 400)))));
    assert_eq!(parse_command("focus"), Ok(Command::Focus));
    assert_eq!(parse_command("windows"), Ok(Command::Windows));
    assert_eq!(parse_command("display"), Ok(Command::Display));
    assert_eq!(parse_command("cursor"), Ok(Command::Cursor));
    assert_eq!(parse_command("gpu"), Ok(Command::Gpu));
    assert_eq!(parse_command("a11y"), Ok(Command::Accessibility));
    assert_eq!(parse_command("close"), Ok(Command::CloseWindow(None)));
    assert_eq!(parse_command("close 1"), Ok(Command::CloseWindow(Some(1))));
    for command in ["repeat-info 30", "repeat-info -1 200", "repeat-info 30 20000", "repeat-info 2000 200", "repeat-info a b", "focus now", "cursor now", "gpu now", "a11y tree", "close x", "close 1 2"] {
        assert!(parse_command(command).is_err(), "{command}");
    }
}
