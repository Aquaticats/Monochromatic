//! Rendered tests of the list defects: a name wider than its pane ends in an ellipsis (OV1), the playing row carries a marker
//! in the primary role (VD6), and the scrollbar is drawn from Theme roles (VD1).
//!
//! What: each test draws the real window with Slint's software renderer into a pixel buffer and reads pixels.
//! Why: whether text ends in an ellipsis or is cut at the box edge is a property of the pixels, and no element of the window
//! reports it, so a test of the element tree cannot tell a fixed list from a broken one.
//! Note: the platform is installed per test thread, as the testing backend does, so these tests run beside the others.

use std::rc::Rc;

use music_player::command::Update;
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, WindowAdapter};
use slint::{ComponentHandle, PlatformError, Rgb8Pixel};

use crate::AppWindow;
use crate::ui_screen::{project, update_state};
use crate::ui_screen_view::ScreenState;

/// What: `SoftwarePlatform` hands one software window to every window the test creates.
/// Why: Slint needs a platform to create a window, and this one draws into memory.
struct SoftwarePlatform {
    /// The window every `AppWindow` of the test uses.
    window: Rc<MinimalSoftwareWindow>,
}

/// The platform of the rendered tests.
impl Platform for SoftwarePlatform {
    /// What: `create_window_adapter` gives the shared software window.
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        return Ok(self.window.clone());
    }
}

/// What: `Frame` is one rendered picture of the window.
struct Frame {
    /// Width in pixels.
    width: usize,
    /// Pixels row by row.
    pixels: Vec<Rgb8Pixel>,
}

/// Methods of `Frame`.
impl Frame {
    /// What: `at` gives the pixel at a column and row as `[red, green, blue]`.
    fn at(&self, x: usize, y: usize) -> [u8; 3] {
        let pixel = self.pixels[y * self.width + x];
        return [pixel.r, pixel.g, pixel.b];
    }

    /// What: `ink_rows` counts the rows of a rectangle that hold a pixel clearly different from the pixel at its top left.
    /// Why: text drawn on a flat background is the pixels unlike the background, and the rows they fill tell a full glyph
    /// from the dots of an ellipsis.
    fn ink_rows(&self, left: usize, right: usize, top: usize, bottom: usize) -> usize {
        let background = self.at(left, top);
        return (top..bottom)
            .filter(|row| {
                return (left..right).any(|column| {
                    let pixel = self.at(column, *row);
                    return pixel.iter().zip(background.iter()).any(|(a, b)| return a.abs_diff(*b) > 60);
                });
            })
            .count();
    }
}

/// More methods of `Frame`.
impl Frame {
    /// What: `rightmost_ink` gives the column of the right-most pixel in a rectangle that is clearly different from the pixel
    /// at its top left, or None when the rectangle holds nothing but background.
    /// Why: where a line of text ends tells whether it ends in a whole glyph, a cut one or the dots of an ellipsis.
    fn rightmost_ink(&self, left: usize, right: usize, top: usize, bottom: usize) -> Option<usize> {
        let background = self.at(left, top);
        return (left..right).rev().find(|column| {
            return (top..bottom).any(|row| {
                let pixel = self.at(*column, row);
                return pixel.iter().zip(background.iter()).any(|(a, b)| return a.abs_diff(*b) > 60);
            });
        });
    }
}

/// What: `render_window` builds the window on a software renderer, loads a queue, and draws one frame.
/// Why: the pixels are what the user sees, in the light scheme the window tests use.
fn render_window(width: usize, height: usize, names: &[String], playing: Option<usize>) -> Frame {
    return render_window_with(width, height, names, playing, |_app| {});
}

/// What: `render_window_with` is `render_window` that lets the test change the window once the queue is loaded.
/// Why: Search rows are set on the window directly.
fn render_window_with(width: usize, height: usize, names: &[String], playing: Option<usize>, setup: impl FnOnce(&AppWindow)) -> Frame {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(SoftwarePlatform { window: window.clone() })).expect("no platform is installed yet on this thread");
    update_state(|state| {
        *state = ScreenState::default();
    });
    crate::ui_settings::replace_saver(|_track_rows, _playing| return Ok(()));
    crate::ui_settings::replace_auto_play_saver(|_enabled| return Ok(()));
    let app = AppWindow::new().expect("the window builds on the software renderer");
    app.window().set_size(slint::PhysicalSize::new(u32::try_from(width).unwrap_or(1), u32::try_from(height).unwrap_or(1)));
    app.global::<crate::Theme>().set_scheme_override(1);
    app.show().expect("the window shows");
    let paths: Vec<std::path::PathBuf> = names.iter().map(|name| return std::path::PathBuf::from("/nonexistent-library").join(name)).collect();
    crate::ui_screen::apply_update(&app, &Update::Library(paths));
    crate::ui_screen::apply_update(&app, &Update::Queue(names.to_vec()));
    if let Some(index) = playing {
        crate::ui_screen::apply_update(&app, &Update::NowPlaying { index: Some(index), name: names[index].clone(), duration: 200.0 });
    }
    project(&app);
    setup(&app);
    slint::platform::update_timers_and_animations();
    let mut pixels = vec![Rgb8Pixel::default(); width * height];
    // The first draw builds what the properties asked for (a page that has just become visible, a list that has just been given
    // rows), and the second draws it.
    for _ in 0..3 {
        slint::platform::update_timers_and_animations();
        window.request_redraw();
        window.draw_if_needed(|renderer| {
            renderer.render(&mut pixels, width);
        });
    }
    return Frame { width, pixels };
}

/// What: `one_folder` names `count` tracks of one folder.
fn one_folder(count: usize) -> Vec<String> {
    return (0..count).map(|number| return format!("Big/Track {number:05}.mp3")).collect();
}

#[test]
fn a_title_wider_than_the_pane_ends_in_an_ellipsis_and_not_in_a_cut_letter() {
    // Capital H is full height in every column, so a letter cut at the box edge fills the rows of the whole strip, while the
    // dots of an ellipsis fill only the few rows at the baseline.
    let mut names = one_folder(3);
    names[0] = format!("Big/{}.mp3", "H".repeat(300));
    let frame = render_window(1200, 800, &names, None);
    // The first row of the right pane starts at y 56; its title line is 24 px high from 12 px below the row top, and the
    // title's box ends 16 px inside the pane's right edge, at x 1184. The strip is the last 10 px of that box.
    let rows = frame.ink_rows(1174, 1184, 56 + 12, 56 + 12 + 24);
    assert!(rows <= 4, "the end of a long title is the dots of an ellipsis, {rows} rows are inked in its last 10 px");
}

#[test]
fn a_title_that_fits_leaves_the_end_of_its_box_empty() {
    let frame = render_window(1200, 800, &one_folder(3), None);
    assert_eq!(frame.ink_rows(1174, 1184, 56 + 12, 56 + 12 + 24), 0, "nothing is drawn at the end of a short title's box");
}

#[test]
fn a_cut_title_is_cut_at_a_whole_letter_before_the_ellipsis() {
    let mut names = one_folder(3);
    names[0] = format!("Big/{}.mp3", "H".repeat(300));
    let frame = render_window(1200, 800, &names, None);
    // Somewhere before the ellipsis the letters are drawn at full height.
    let letters = frame.ink_rows(616, 700, 56 + 12, 56 + 12 + 24);
    assert!(letters >= 10, "the start of the title is drawn in full letters, {letters} rows");
}

#[test]
fn the_playing_row_is_marked_in_the_primary_role_on_its_left_edge() {
    let names = one_folder(5);
    let frame = render_window(1200, 800, &names, Some(1));
    // The light baseline primary role is #455481. The marker is the first 4 px of the playing row, so on the pane's left edge
    // exactly one row's height (48 px) carries it, and 5 px further in the same rows carry the row fill instead.
    let primary = [0x45, 0x54, 0x81];
    let marked: Vec<usize> = (56..800).filter(|row| return frame.at(601, *row) == primary).collect();
    assert_eq!(marked.len(), 48, "one row of 48 px carries the marker in the primary role, found {} rows", marked.len());
    assert!(marked.iter().all(|row| return frame.at(606, *row) != primary), "and it is only 4 px wide");
}

#[test]
fn the_scrollbar_is_drawn_from_theme_roles_with_a_thumb_and_a_track() {
    let names = one_folder(3000);
    let frame = render_window(1200, 800, &names, None);
    // The thumb sits at the top of the track at rest: 4 px wide in the middle of the 12 px column, in the outline role
    // (light baseline #757780); below it the track is the outline-variant role (#C5C6D0).
    assert_eq!(frame.at(1194, 60), [0x75, 0x77, 0x80], "the thumb is the outline role");
    assert_eq!(frame.at(1194, 400), [0xC5, 0xC6, 0xD0], "the track is the outline-variant role");
    assert_ne!(frame.at(1190, 400), frame.at(1194, 400), "the track is 4 px wide, not the whole grab zone");
}

/// What: `ends_in_dots` tells whether the line of text in a rectangle ends in the dots of an ellipsis.
/// Why: the last 10 px of a line that is cut in a letter hold the letter's strokes over most of the glyph's height, while the
/// ellipsis has ink only in its few baseline rows.
fn ends_in_dots(frame: &Frame, left: usize, right: usize, top: usize, bottom: usize) -> bool {
    let Some(end) = frame.rightmost_ink(left, right, top, bottom) else {
        return false;
    };
    return frame.ink_rows(end.saturating_sub(9), end + 1, top, bottom) <= 4;
}

#[test]
fn a_folder_name_wider_than_the_picker_ends_in_an_ellipsis() {
    let names = vec![format!("{}/01 Song.mp3", "H".repeat(200)), "Abba/01 Song.mp3".to_owned()];
    let frame = render_window(1200, 800, &names, None);
    // The picker's names start right of the 48 px rail. The first heading is 38 px tall, so the long name's line starts at
    // y 56 + 38 + 48 (the first name under the first heading, which is Abba) when sorted before it.
    let mut found = false;
    for line_top in (56..400).step_by(48) {
        if ends_in_dots(&frame, 300, 596, line_top + 12, line_top + 36) {
            found = true;
        }
    }
    assert!(found, "a line of the picker ends in an ellipsis for a name 200 letters long");
}

#[test]
fn a_search_result_title_wider_than_the_page_ends_in_an_ellipsis() {
    use slint::{ModelRc, VecModel};
    let names = one_folder(3);
    let frame = render_window_with(1200, 800, &names, None, |app| {
        let segment = |text: &str| return crate::SearchSegment { text: text.into(), emphasized: false };
        let rows = vec![crate::SearchRow {
            kind: 1,
            title_segments: ModelRc::new(VecModel::from(vec![segment(&"H".repeat(300))])),
            parent_segments: ModelRc::new(VecModel::from(vec![segment("Folder")])),
            has_parent: true,
            current: false,
            playing: false,
            spoken: "".into(),
            index: 0,
        }];
        app.set_search_rows(ModelRc::new(VecModel::from(rows)));
        app.set_route(1);
    });
    // The first result row starts under the 72 px header; its title is the upper line of the row.
    assert!(ends_in_dots(&frame, 700, 1186, 72 + 12, 72 + 36), "the long title ends in an ellipsis");
}
