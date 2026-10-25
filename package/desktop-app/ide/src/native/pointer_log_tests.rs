//! The debug records of a press,
//!  a drag,
//!  and a release carry the line the press selected,
//!  the point the handler received,
//!  and the offsets of the view,
//!  so a verification reads them from the log and needs no screenshot.

/// The production window that receives the pointer events.
use super::AppWindow;
/// The complete reader fixture shared with the find tests.
use super::find_tests::{Reader, reader};
/// Where source text starts in the window,
///  derived in `sidebar_tests` from the tree,
///  divider,
///  and gutter widths.
use super::sidebar_tests::TEXT_LEFT;
/// Window events as a seat delivers them,
///  and a point in logical window pixels.
use slint::{
    ComponentHandle, LogicalPosition,
    platform::{PointerEventButton, WindowEvent},
};
/// Disposable project files.
use std::fs;
/// A scoped writer the records go to.
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};
/// The same pipeline the application installs,
///  under a scoped subscriber.
use tracing_subscriber::fmt::MakeWriter;

/// Source rows start below the 32 px file label.
const TEXT_TOP: f32 = 32.0;

/// Logical pixels of one code row.
const ROW: f32 = 24.0;

/// Bytes written by the scoped subscriber.
#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);

impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().expect("capture").extend_from_slice(bytes);
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> io::Result<()> {
        return Ok(());
    }
}

impl<'writer> MakeWriter<'writer> for Capture {
    type Writer = Capture;

    fn make_writer(&'writer self) -> Self::Writer {
        return self.clone();
    }
}

/// Run `action` under a subscriber that writes debug records and return the lines it wrote.
fn logged(action: impl FnOnce()) -> Vec<String> {
    let capture = Capture::default();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_ansi(false)
        .with_writer(capture.clone())
        .finish();
    tracing::subscriber::with_default(subscriber, action);
    let text = String::from_utf8(capture.0.lock().expect("capture").clone()).expect("log text");
    return text.lines().map(str::to_owned).collect();
}

/// The number after `name=` in a record.
fn field(record: &str, name: &str) -> f32 {
    let start = record
        .find(&format!(" {name}="))
        .unwrap_or_else(|| panic!("no `{name}` in {record}"))
        + name.len()
        + 2;
    let rest = &record[start..];
    let end = rest.find(' ').unwrap_or(rest.len());
    return rest[..end]
        .parse()
        .unwrap_or_else(|_| panic!("`{name}` is not a number in {record}"));
}

/// The one record of `message` among `lines`.
fn record<'a>(lines: &'a [String], message: &str) -> &'a str {
    let found: Vec<&String> = lines
        .iter()
        .filter(|line| line.contains(&format!("{message} y=")))
        .collect();
    assert_eq!(
        found.len(),
        1,
        "expected one `{message}` record in {lines:#?}"
    );
    return found[0];
}

/// A reader over a file of `lines` lines, `line N` on line N.
fn numbered(lines: usize) -> (tempfile::TempDir, Reader) {
    let fixture = tempfile::tempdir().expect("disposable pointer log project");
    let mut text = String::new();
    for number in 1..=lines {
        text.push_str(&format!("line {number}\n"));
    }
    fs::write(fixture.path().join("log.txt"), text).expect("pointer log fixture");
    let source = reader(fixture.path(), "log.txt");
    return (fixture, source);
}

/// The window point over text row `row` (counted from 0) at the offset the window has now.
fn point(window: &AppWindow, row: usize) -> LogicalPosition {
    return LogicalPosition::new(
        TEXT_LEFT + 20.0,
        TEXT_TOP + row as f32 * ROW + ROW / 2.0 + window.get_scroll_y(),
    );
}

/// Press the left button at a window point.
fn press(window: &AppWindow, position: LogicalPosition) {
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
}

/// Release the left button at a window point.
fn release(window: &AppWindow, position: LogicalPosition) {
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
}

/// A press on source row 7 writes a record that names line 8, with the point and the offsets of the view;
///  the first drag move is the drag start, a later one is not, and the release names the line it went up on.
#[test]
fn a_press_a_drag_and_a_release_name_the_line_they_selected() {
    let (_fixture, source) = numbered(200);
    let window = &source.window;
    let start = point(window, 7);
    let target = point(window, 12);
    let lines = logged(|| {
        press(window, start);
        window
            .window()
            .dispatch_event(WindowEvent::PointerMoved { position: target });
        window
            .window()
            .dispatch_event(WindowEvent::PointerMoved { position: target });
        release(window, target);
    });
    let pressed = record(&lines, "source pointer press");
    assert_eq!(field(pressed, "line_number"), 8.0, "{pressed}");
    assert_eq!(field(pressed, "line"), 7.0, "{pressed}");
    assert_eq!(field(pressed, "offset"), 0.0, "{pressed}");
    assert_eq!(field(pressed, "drawn_offset"), 0.0, "{pressed}");
    assert!(pressed.contains(" held=false"), "{pressed}");
    assert_eq!(
        field(pressed, "y"),
        7.0 * ROW + ROW / 2.0,
        "the point the handler received is in the text space of the drawn rows: {pressed}"
    );
    assert!(
        pressed.contains(" column=") && pressed.contains(" x="),
        "{pressed}"
    );
    let started = record(&lines, "source pointer drag start");
    assert_eq!(field(started, "line_number"), 13.0, "{started}");
    let dragged = lines
        .iter()
        .filter(|line| line.contains("source pointer drag y="))
        .count();
    assert_eq!(
        dragged, 1,
        "the second move is a drag, not a start: {lines:#?}"
    );
    let released = record(&lines, "source pointer release");
    assert_eq!(field(released, "line_number"), 13.0, "{released}");
    assert_eq!(field(released, "line"), 12.0, "{released}");
    // The records come in the order of the events.
    let order = |message: &str| {
        lines
            .iter()
            .position(|line| line.contains(&format!("{message} y=")))
            .expect("record")
    };
    assert!(order("source pointer press") < order("source pointer drag start"));
    assert!(order("source pointer drag") < order("source pointer release"));
    window.hide().expect("close pointer log window");
}

/// On a scrolled view the logged line is the line the logged offset and point name,
///  `(y + drawn_offset) / row`,
///  whether the view is held or not.
#[test]
fn the_logged_line_is_the_line_the_logged_offset_and_point_name() {
    let (_fixture, source) = numbered(400);
    let window = &source.window;
    window.set_scroll_y(-(120.0 * ROW));
    let lines = logged(|| {
        press(window, point(window, 130));
        release(window, point(window, 130));
    });
    let pressed = record(&lines, "source pointer press");
    // No tile covers the far offset, so the view is held: the real and the drawn offset differ, and a mapping through
    // the real offset would name another line than the one drawn there.
    assert!(pressed.contains(" held=true"), "{pressed}");
    assert_ne!(
        field(pressed, "offset"),
        field(pressed, "drawn_offset"),
        "{pressed}"
    );
    // The pointer stands 12 px into the row `130` of the text at the offset the window had, so its place in the
    // view is that distance from the top of the text area; the drawn offset moves the text under it.
    assert_eq!(
        field(pressed, "view_y"),
        130.0 * ROW + ROW / 2.0 - 120.0 * ROW,
        "{pressed}"
    );
    let named = ((field(pressed, "view_y") + field(pressed, "drawn_offset")) / ROW).floor();
    assert_eq!(field(pressed, "line"), named, "{pressed}");
    assert_eq!(field(pressed, "line_number"), named + 1.0, "{pressed}");
    assert_eq!(
        field(pressed, "offset"),
        120.0 * ROW,
        "the real offset is the window's scroll: {pressed}"
    );
    let released = record(&lines, "source pointer release");
    assert_eq!(
        field(released, "line_number"),
        field(pressed, "line_number"),
        "{released}"
    );
    window.hide().expect("close pointer log window");
}

/// No record is written, and no point is looked up for one, when the debug level is off.
#[test]
fn nothing_is_written_when_debug_logging_is_off() {
    let (_fixture, source) = numbered(50);
    let window = &source.window;
    let capture = Capture::default();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .with_ansi(false)
        .with_writer(capture.clone())
        .finish();
    tracing::subscriber::with_default(subscriber, || {
        press(window, point(window, 3));
        release(window, point(window, 3));
    });
    assert!(capture.0.lock().expect("capture").is_empty());
    window.hide().expect("close pointer log window");
}
