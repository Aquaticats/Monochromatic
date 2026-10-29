//! A live system color-scheme change repaints source ink,
//!  find overlays,
//!  the tree,
//!  the find bar,
//! and the divider,
//!  through the same toolkit call the desktop-settings portal watcher makes.

/// Real key events,
///  the full production reader,
///  and bounded waits shared with the find tests.
use super::find_tests::{Reader, chord, reader, status_for, type_text};
/// Shared rendering entry point,
///  used here only to draw a cold reference image.
use super::{
    AppWindow, render,
    tile_reads::{same_buffers, tile_pixels},
};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::Waiting;
/// Selected-text ink is chosen from the drawn selection fill,
///  as production rendering does.
use ide_app::selection_ink::legible_ink;
/// What:
///  `ColorScheme` is the toolkit's scheme enum (`Unknown`,
///  `Dark`,
///  `Light`);
///  `WindowInner` is the
/// toolkit's internal side of a window,
///  reached through its unstable re-export module.
/// Why:
///  Slint 1.18.1's portal watcher applies a scheme with `SlintContext::set_color_scheme`
/// (`i-slint-backend-winit-1.18.1/xdg_desktop_settings.rs:117`);
///  the headless backend has no portal,
/// so the test makes that same call on the window's context.
/// Gotcha:
///  This module is not stable API;
///  a toolkit upgrade can rename it and break only this test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ColorScheme, windowInternals } from 'slint/private';
/// ```
use slint::private_unstable_api::re_exports::{ColorScheme, WindowInner};
/// Frames,
///  pixels,
///  and toolkit colors are read back from the rendered window.
use slint::{
    Color, ComponentHandle, Model, Rgba8Pixel, SharedPixelBuffer,
    platform::{Key, update_timers_and_animations},
};
/// Fixture files are written into a disposable project directory.
use std::fs;

/// What:
///  `[u8; 4]` is a fixed array of four bytes (red,
///  green,
///  blue,
///  alpha);
///  siblings `Vec<u8>` and `&[u8]`.
/// Why:
///  The keyword ink pair pinned in `src/code_colors.rs` `ink` for role 1,
///  dark then light.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const KEYWORD_DARK = [199, 146, 234, 255];
/// ```
const KEYWORD_DARK: [u8; 4] = [199, 146, 234, 255];
/// Light-scheme keyword ink from the same table.
const KEYWORD_LIGHT: [u8; 4] = [118, 54, 164, 255];

/// Height of the file-label row above the source column and of the project row above the tree.
const HEADER: usize = 32;
/// Width of the divider's layout cell,
///  which is its line:
///  the pixel column right after the sidebar.
const DIVIDER: usize = 1;
/// Width of the line-number gutter at the left edge of the source column.
const GUTTER: usize = super::sidebar_tests::GUTTER as usize;
/// Height of the open find bar's single row at the bottom of the source column.
const FIND_BAR: usize = 56;

/// Apply a system color scheme exactly where the portal watcher applies it,
///  then run change handlers.
pub(super) fn switch(window: &AppWindow, scheme: ColorScheme) {
    // What: `from_pub` borrows the toolkit's internal window behind the public handle; `context()`
    // borrows the process-wide toolkit context that every window reads its color scheme from.
    // Why: This is the one value `Palette.color-scheme` follows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // windowInternals(window).context.setColorScheme(scheme);
    // ```
    WindowInner::from_pub(window.window())
        .context()
        .set_color_scheme(scheme);
    update_timers_and_animations();
}

/// Copy a toolkit color into the four bytes the source raster uses.
pub(super) fn rgba(color: Color) -> [u8; 4] {
    return [color.red(), color.green(), color.blue(), color.alpha()];
}

/// What:
///  `SharedPixelBuffer<Rgba8Pixel>` is a frame of four-byte pixels;
///  `expect` fails the test with
/// the message when the image has no pixel data.
/// Why:
///  The tile's texture is the only place source ink exists;
///  it is read back from the GPU after a frame.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sourcePixels(reader: Reader): Frame;
/// ```
fn source_pixels(reader: &Reader) -> SharedPixelBuffer<Rgba8Pixel> {
    return tile_pixels(&reader.window, &reader.source).expect("source image pixels");
}

/// Whether any fully covered source pixel has exactly this ink;
///  covered pixels carry the ink unblended.
fn has_ink(pixels: &SharedPixelBuffer<Rgba8Pixel>, ink: [u8; 4]) -> bool {
    // What: A `for` loop that stops at the first pixel with the ink.
    // Why: One fully covered pixel of a role proves that role was painted with this ink.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return pixels.some(pixel => [pixel.r, pixel.g, pixel.b, pixel.a].join() === ink.join());
    // ```
    for pixel in pixels.as_slice() {
        if [pixel.r, pixel.g, pixel.b, pixel.a] == ink {
            return true;
        }
    }
    return false;
}

/// The lightness of the frame's pixel at column `x` and row `y`,
///  for a frame `width` pixels wide.
fn lightness_at(frame: &SharedPixelBuffer<Rgba8Pixel>, width: usize, x: usize, y: usize) -> f32 {
    return lightness(frame.as_slice()[y * width + x]);
}

/// Fully covered source pixels inside the active match's selection,
///  two pixels in from its edges.
fn selected_inks(reader: &Reader, pixels: &SharedPixelBuffer<Rgba8Pixel>) -> Vec<[u8; 4]> {
    let current = reader.source.borrow();
    // What: `as_ref()` borrows the optional shaped view; `expect` fails the test when it is absent.
    // Why: The selection rectangles come from the same shaped rows that produced the image.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const view = current.shaped ?? fail('shaped source');
    // ```
    let view = current.shaped.as_ref().expect("shaped source");
    let rectangle = view.selections.first().expect("active match selection");
    let width = pixels.width() as usize;
    // What: `as usize` turns a logical float into a pixel index; the headless window has scale 1.
    // Why: Edge pixels blend selected and ordinary ink, so only interior pixels are compared.
    let left = rectangle.x as usize + 2;
    let right = (rectangle.x + rectangle.width) as usize - 2;
    let top = (rectangle.y - current.first as f32 * 24.0) as usize;
    let mut found = Vec::new();
    for y in top..top + 24 {
        for x in left..right {
            let pixel = pixels.as_slice()[y * width + x];
            if pixel.a == 255 {
                found.push([pixel.r, pixel.g, pixel.b, pixel.a]);
            }
        }
    }
    return found;
}

/// WCAG relative-luminance approximation of one rendered pixel,
///  from 0 (black) to 1 (white).
fn lightness(pixel: Rgba8Pixel) -> f32 {
    return (0.2126 * f32::from(pixel.r)
        + 0.7152 * f32::from(pixel.g)
        + 0.0722 * f32::from(pixel.b))
        / 255.0;
}

/// Mean,
///  darkest,
///  and lightest pixel of a window region given as `[left, right, top, bottom]`.
pub(super) fn region(frame: &SharedPixelBuffer<Rgba8Pixel>, bounds: [usize; 4]) -> (f32, f32, f32) {
    let width = frame.width() as usize;
    let mut total = 0.0;
    let mut darkest: f32 = 1.0;
    let mut lightest: f32 = 0.0;
    for y in bounds[2]..bounds[3] {
        for x in bounds[0]..bounds[1] {
            let value = lightness(frame.as_slice()[y * width + x]);
            total += value;
            darkest = darkest.min(value);
            lightest = lightest.max(value);
        }
    }
    let count = ((bounds[1] - bounds[0]) * (bounds[3] - bounds[2])) as f32;
    return (total / count, darkest, lightest);
}

/// The displayed source image equals a cold render,
///  and selected glyphs use the ink chosen from the fill.
fn assert_current(reader: &Reader, name: &str) -> SharedPixelBuffer<Rgba8Pixel> {
    let window = &reader.window;
    let live = source_pixels(reader);
    // A cold render after forgetting the previous frame is what this scheme looks like from startup.
    reader.source.borrow_mut().frame_stamp = None;
    render(window, &reader.source);
    let cold = source_pixels(reader);
    assert!(
        same_buffers(&live, &cold),
        "{name}: the live source image differs from a cold render; the palette change did not re-rasterize"
    );
    let selected = legible_ink(
        rgba(window.get_selection_fill().color()),
        rgba(window.get_selected_foreground().color()),
    );
    let inks = selected_inks(reader, &live);
    assert!(
        !inks.is_empty(),
        "{name}: no fully covered selected glyph pixels"
    );
    let mut all_chosen = true;
    for ink in inks.iter() {
        if *ink != selected {
            all_chosen = false;
        }
    }
    assert!(
        all_chosen,
        "{name}: selected glyphs are not painted with the ink chosen from the selection fill"
    );
    return live;
}

/// Check one scheme:
///  current source image,
///  the scheme's keyword ink,
///  overlays,
///  and window chrome.
fn assert_scheme(reader: &Reader, dark: bool) -> SharedPixelBuffer<Rgba8Pixel> {
    let window = &reader.window;
    let name = if dark { "dark" } else { "light" };
    assert_eq!(
        window.get_dark_scheme(),
        dark,
        "{name}: Palette.color-scheme did not change"
    );
    let live = assert_current(reader, name);
    let (own, other) = if dark {
        (KEYWORD_DARK, KEYWORD_LIGHT)
    } else {
        (KEYWORD_LIGHT, KEYWORD_DARK)
    };
    assert!(
        has_ink(&live, own),
        "{name}: keyword ink for this scheme is missing"
    );
    assert!(
        !has_ink(&live, other),
        "{name}: keyword ink of the other scheme remains"
    );
    assert!(
        window.get_source_matches().row_count() >= 1,
        "{name}: match rectangles disappeared"
    );
    let frame = window.window().take_snapshot().expect("themed frame");
    let sidebar = window.get_sidebar_width() as usize;
    let source = sidebar + DIVIDER;
    let height = frame.height() as usize;
    let width = frame.width() as usize;
    let tree = region(&frame, [0, sidebar, HEADER, height]);
    let find_bar = region(&frame, [source, width, height - FIND_BAR + 2, height]);
    for (part, (mean, darkest, lightest)) in [("tree", tree), ("find bar", find_bar)] {
        if dark {
            assert!(
                mean < 0.3 && lightest > 0.5,
                "{part} did not follow dark: mean {mean}, lightest {lightest}"
            );
        } else {
            assert!(
                mean > 0.7 && darkest < 0.5,
                "{part} did not follow light: mean {mean}, darkest {darkest}"
            );
        }
    }
    // The idle divider line is the column right after the sidebar; four columns further is gutter background.
    let line = lightness_at(&frame, width, sidebar, height / 2);
    let cell = lightness_at(&frame, width, sidebar + 4, height / 2);
    // The non-active match's left border is one pixel left of its first character, here at mid-row.
    let found = window
        .get_source_matches()
        .row_data(0)
        .expect("visible match");
    let border = lightness_at(
        &frame,
        width,
        source + GUTTER + found.x as usize - 1,
        HEADER + found.y as usize + 12,
    );
    let page = lightness_at(&frame, width, source + 4, HEADER + 12);
    for (part, mark, behind) in [("divider line", line, cell), ("match border", border, page)] {
        if dark {
            assert!(
                behind < 0.3 && mark > behind + 0.1,
                "{part} did not follow dark: {mark} on {behind}"
            );
        } else {
            assert!(
                behind > 0.7 && mark < behind - 0.1,
                "{part} did not follow light: {mark} on {behind}"
            );
        }
    }
    return live;
}

/// What:
///  The answer is a pair:
///  the disposable directory (`TempDir`,
///  deleted when dropped) and the reader.
/// Why:
///  The directory must outlive the reader's file workers,
///  so the caller keeps both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function themedReader(): [TempDir, Reader];
/// ```
fn themed_reader() -> (tempfile::TempDir, Reader) {
    let fixture = tempfile::tempdir().expect("disposable theme project");
    let text = "const label = \"needle and needle\";\nconst count = 1;\n";
    fs::write(fixture.path().join("theme.ts"), text).expect("theme fixture");
    // A second, unselected tree row is drawn in ordinary ink rather than selection ink.
    fs::write(fixture.path().join("notes.txt"), "notes").expect("second tree row");
    let reader = reader(fixture.path(), "theme.ts");
    let window = &reader.window;
    {
        let mut waiting = Waiting::eventually();
        while !(!reader.source.borrow().styles.is_empty()) {
            waiting.step("TypeScript highlighting did not arrive");
        }
    }
    {
        let mut waiting = Waiting::eventually();
        while !(window.get_tree_entries().row_count() == 2) {
            waiting.step("the project tree did not list both files");
        }
    }
    chord(window, Key::Control, "f");
    type_text(window, "needle");
    status_for(window, "needle", "1/2");
    return (fixture, reader);
}

/// Dark,
///  light,
///  and dark again repaint everything without input,
///  and return to identical source pixels.
#[test]
fn live_color_scheme_repaints_source_overlays_tree_find_bar_and_divider() {
    // `_fixture` keeps the directory alive until the test ends.
    let (_fixture, reader) = themed_reader();
    let window = &reader.window;
    switch(window, ColorScheme::Dark);
    let first_dark = assert_scheme(&reader, true);
    switch(window, ColorScheme::Light);
    let light = assert_scheme(&reader, false);
    assert!(
        !same_buffers(&light, &first_dark),
        "the light source image equals the dark one"
    );
    switch(window, ColorScheme::Dark);
    let second_dark = assert_scheme(&reader, true);
    assert!(
        same_buffers(&second_dark, &first_dark),
        "returning to dark did not restore the dark source image"
    );
    window.hide().expect("close theme window");
}

/// An accent change re-tints the selection fill without flipping the scheme;
///  the source image stays current.
///
/// The portal watcher applies an accent with `SlintContext::set_accent_color`
/// (`i-slint-backend-winit-1.18.1/xdg_desktop_settings.rs:139`),
///  which no scheme-flip handler observes.
#[test]
fn accent_color_change_keeps_the_source_image_current() {
    let (_fixture, reader) = themed_reader();
    let window = &reader.window;
    for scheme in [ColorScheme::Dark, ColorScheme::Light] {
        switch(window, scheme);
        let dark = window.get_dark_scheme();
        // Saturated, pale, neutral, and near-black accents, each a different hue and lightness.
        for accent in [
            [255, 215, 0],
            [144, 238, 144],
            [255, 255, 255],
            [220, 20, 60],
            [20, 20, 20],
        ] {
            let before = rgba(window.get_selection_fill().color());
            WindowInner::from_pub(window.window())
                .context()
                .set_accent_color(Color::from_rgb_u8(accent[0], accent[1], accent[2]));
            update_timers_and_animations();
            let fill = rgba(window.get_selection_fill().color());
            let name = format!("accent {accent:?}, fill {fill:?}");
            assert_ne!(
                fill, before,
                "{name}: the accent did not re-tint the selection fill"
            );
            assert_eq!(
                window.get_dark_scheme(),
                dark,
                "{name}: an accent change flipped the scheme"
            );
            assert_current(&reader, &name);
        }
    }
    window.hide().expect("close accent window");
}
