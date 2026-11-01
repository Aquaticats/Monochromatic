// What:  Unit tests for the control-protocol parser and response formatter.
// Why:   The parser is a text -> command boundary; per the repo's boundary-testing rule
//        these cases include adversarial input (shell metacharacters, path traversal,
//        missing/extra/non-numeric arguments) to prove the grammar is exact and that
//        `type`/`screenshot` payloads are passed through verbatim (their real destination
//        is keysym injection and a filesystem path the caller owns, never a shell).
//
// In TS you'd write (pseudocode):
// ```ts
// describe("protocol", () => { /* cases below */ });
// ```

use super::{format_response, parse_command, ButtonAction, Command, KeyAction, PointerButton, Response};

#[test]
fn ping_and_quit_parse() {
    assert_eq!(parse_command("ping"), Ok(Command::Ping));
    // Trailing CRLF is stripped.
    assert_eq!(parse_command("quit\r\n"), Ok(Command::Quit));
}

#[test]
fn blank_and_unknown_lines_error() {
    assert!(parse_command("").is_err());
    assert!(parse_command("   ").is_err());
    assert!(parse_command("\n").is_err());
    assert!(parse_command("frobnicate 1 2").is_err());
}

#[test]
fn screenshot_preserves_path_including_spaces_and_traversal() {
    assert_eq!(
        parse_command("screenshot /tmp/a b.png"),
        Ok(Command::Screenshot("/tmp/a b.png".into()))
    );
    // Traversal tokens are preserved verbatim: the caller owns the target path.
    assert_eq!(
        parse_command("screenshot ../../etc/passwd"),
        Ok(Command::Screenshot("../../etc/passwd".into()))
    );
    assert!(parse_command("screenshot").is_err());
    assert!(parse_command("screenshot    ").is_err());
}

#[test]
fn type_payload_is_verbatim_including_metacharacters() {
    assert_eq!(
        parse_command("type hello world"),
        Ok(Command::Type("hello world".to_string()))
    );
    // Shell metacharacters are just characters here; they are never interpreted by a
    // shell, only mapped to keysyms (unsupported ones are skipped at injection time).
    assert_eq!(
        parse_command("type a; rm -rf / && echo $HOME | cat"),
        Ok(Command::Type("a; rm -rf / && echo $HOME | cat".to_string()))
    );
    // Interior tabs survive; only trailing CR/LF is stripped.
    assert_eq!(
        parse_command("type tab\there\r\n"),
        Ok(Command::Type("tab\there".to_string()))
    );
    // `type ` (trailing space) and bare `type` both mean "type nothing".
    assert_eq!(parse_command("type "), Ok(Command::Type(String::new())));
    assert_eq!(parse_command("type"), Ok(Command::Type(String::new())));
}

#[test]
fn click_parses_coordinates_and_optional_button() {
    assert_eq!(
        parse_command("click 10 20"),
        Ok(Command::Click {
            x: 10.0,
            y: 20.0,
            button: PointerButton::Left,
            count: 1,
            interval_ms: 80,
            modifiers: String::new(),
        })
    );
    assert_eq!(
        parse_command("click 1.5 2.5 right"),
        Ok(Command::Click {
            x: 1.5,
            y: 2.5,
            button: PointerButton::Right,
            count: 1,
            interval_ms: 80,
            modifiers: String::new(),
        })
    );
    assert!(parse_command("click 10").is_err());
    assert!(parse_command("click a b").is_err());
    assert!(parse_command("click 1 2 sideways").is_err());
    assert!(parse_command("click").is_err());
}

#[test]
fn key_parses_name_and_action() {
    assert_eq!(
        parse_command("key enter"),
        Ok(Command::Key {
            name: "enter".to_string(),
            action: KeyAction::Tap
        })
    );
    assert_eq!(
        parse_command("key a press"),
        Ok(Command::Key {
            name: "a".to_string(),
            action: KeyAction::Press
        })
    );
    assert_eq!(
        parse_command("key space release"),
        Ok(Command::Key {
            name: "space".to_string(),
            action: KeyAction::Release
        })
    );
    assert!(parse_command("key").is_err());
    assert!(parse_command("key a wiggle").is_err());
}

#[test]
fn resize_validates_dimensions() {
    assert_eq!(
        parse_command("resize 800 600"),
        Ok(Command::Resize {
            width: 800,
            height: 600
        })
    );
    assert!(parse_command("resize 0 600").is_err());
    assert!(parse_command("resize -1 600").is_err());
    assert!(parse_command("resize 800").is_err());
    assert!(parse_command("resize wide tall").is_err());
}

#[test]
fn drop_file_parses_path_and_optional_coordinates() {
    // Path only: coordinates default to None (window centre chosen at run time).
    assert_eq!(
        parse_command("drop-file /tmp/hello.txt"),
        Ok(Command::DropFile {
            paths: vec!["/tmp/hello.txt".into()],
            x: None,
            y: None
        })
    );
    // Path plus both coordinates.
    assert_eq!(
        parse_command("drop-file /tmp/hello.txt 100 40"),
        Ok(Command::DropFile {
            paths: vec!["/tmp/hello.txt".into()],
            x: Some(100.0),
            y: Some(40.0)
        })
    );
    // Traversal tokens in the path are preserved verbatim; the caller owns the target.
    assert_eq!(
        parse_command("drop-file ../../etc/passwd"),
        Ok(Command::DropFile {
            paths: vec!["../../etc/passwd".into()],
            x: None,
            y: None
        })
    );
    // Missing path and a lone coordinate error. More words are more paths, not an error.
    assert!(parse_command("drop-file").is_err());
    assert!(parse_command("drop-file /tmp/a 100").is_err());
}

#[test]
fn responses_format_to_wire_form() {
    assert_eq!(format_response(&Response::Ok), "ok");
    assert_eq!(
        format_response(&Response::OkWith("data here".to_string())),
        "ok data here"
    );
    assert_eq!(
        format_response(&Response::Err("bad thing".to_string())),
        "err bad thing"
    );
}

#[test]
fn record_parses_dir_fps_format_and_stop() {
    assert_eq!(
        parse_command("record /tmp/frames"),
        Ok(Command::Record {
            dir: "/tmp/frames".into(),
            fps: 60.0,
            format: "png".to_string()
        })
    );
    assert_eq!(
        parse_command("record /tmp/f 30 bmp"),
        Ok(Command::Record {
            dir: "/tmp/f".into(),
            fps: 30.0,
            format: "bmp".to_string()
        })
    );
    assert_eq!(parse_command("record stop"), Ok(Command::RecordStop));
    // Missing directory, non-numeric fps, and extra tokens are rejected.
    assert!(parse_command("record").is_err());
    assert!(parse_command("record /tmp/f notanumber").is_err());
    assert!(parse_command("record stop extra").is_err());
    assert!(parse_command("record /tmp/f 60 png extra").is_err());
}

/// `move` takes two finite coordinates and nothing else.
#[test]
fn move_parses_coordinates_and_rejects_bad_input() {
    assert_eq!(parse_command("move 10 20"), Ok(Command::Move { x: 10.0, y: 20.0 }));
    assert_eq!(parse_command("move 1.5 -2"), Ok(Command::Move { x: 1.5, y: -2.0 }));
    assert!(parse_command("move 10").is_err());
    assert!(parse_command("move 10 20 30").is_err());
    assert!(parse_command("move 10 20 quit").is_err());
    assert!(parse_command("move a 2").is_err());
    assert!(parse_command("move NaN 2").is_err());
    assert!(parse_command("move 1 inf").is_err());
}

/// `button` names an action, then an optional button that defaults to left.
#[test]
fn button_parses_action_and_defaults_to_left() {
    assert_eq!(parse_command("button down"), Ok(Command::Button { action: ButtonAction::Down, button: PointerButton::Left }));
    assert_eq!(parse_command("button up right"), Ok(Command::Button { action: ButtonAction::Up, button: PointerButton::Right }));
    assert_eq!(parse_command("button down middle"), Ok(Command::Button { action: ButtonAction::Down, button: PointerButton::Middle }));
    assert!(parse_command("button").is_err());
    assert!(parse_command("button press").is_err());
    assert!(parse_command("button down purple").is_err());
    assert!(parse_command("button down left extra").is_err());
}

/// The verbs without arguments reject any trailing token instead of ignoring it.
#[test]
fn argumentless_taskbar_and_window_verbs_reject_trailing_tokens() {
    assert_eq!(parse_command("title"), Ok(Command::Title));
    assert_eq!(parse_command("app-id"), Ok(Command::AppId));
    assert_eq!(parse_command("launcher"), Ok(Command::Launcher));
    assert_eq!(parse_command("launcher-clear"), Ok(Command::LauncherClear));
    assert!(parse_command("title extra").is_err());
    assert!(parse_command("app-id x").is_err());
    assert!(parse_command("launcher 1").is_err());
    assert!(parse_command("launcher-clear now").is_err());
}

/// `launcher-log` takes an optional count of at least one, defaulting to 64.
#[test]
fn launcher_log_parses_optional_count() {
    assert_eq!(parse_command("launcher-log"), Ok(Command::LauncherLog { count: 64 }));
    assert_eq!(parse_command("launcher-log 5"), Ok(Command::LauncherLog { count: 5 }));
    assert!(parse_command("launcher-log 0").is_err());
    assert!(parse_command("launcher-log -1").is_err());
    assert!(parse_command("launcher-log x").is_err());
    assert!(parse_command("launcher-log 5 6").is_err());
}

/// An empty title answers `ok ` with the trailing space the protocol documents.
#[test]
fn empty_payload_keeps_its_trailing_space() {
    assert_eq!(format_response(&Response::OkWith(String::new())), "ok ");
}
