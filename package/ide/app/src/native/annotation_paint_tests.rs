//! Rendered annotation pixels in both schemes with measured contrast,
//!  repaints only for visible changes,
//!  a
//! scroll range that reaches hints and messages,
//!  and virtual rows that keep their place at another display scale.
//! Which pixels move when rows arrive is pinned in `annotation_stability_tests`.

/// The production window.
use super::AppWindow;
/// The production setter and the stamp of the displayed text.
use super::annotate::{displayed, set_annotations};
/// Snapshot builders,
///  the fixture,
///  the painted-record counts,
///  and the text origin shared with the behavior tests.
use super::annotation_tests::{FIXTURE, TEXT_LEFT, TEXT_TOP, annotate, painted as shown, problem};
/// The complete reader fixture.
use super::find_tests::{Reader, reader};
/// The size of the tile's texture,
/// read after a frame.
use super::tile_reads::tile_size;
/// Snapshot records.
use ide_app::language::diagnostics::{DiagnosticsSnapshot, Severity, SourceGroup};
/// The heights virtual rows are built from.
use ide_app::virtual_row::{BLOCK_GAP, ROW_HEIGHT};
/// What:
///  `ColorScheme` is the toolkit's scheme enum;
///  `WindowInner` is the toolkit's internal side of a window,
///       reached through its unstable re-export module.
/// Why:
///  Slint 1.18.1's portal watcher applies a scheme with `set_color_scheme` on the window's context;
///  the
///      headless backend has no portal,
///  so the test makes the same call.
/// Gotcha:
///  This module is not stable API;
///  a toolkit upgrade can rename it and break only these tests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ColorScheme, windowInternals } from 'slint/private';
/// ```
use slint::private_unstable_api::re_exports::{ColorScheme, WindowInner};
/// Frames,
///  pixels,
///  and toolkit colors are read back from the rendered window;
///  a scale change arrives as a
/// window event.
use slint::{
    Color, ComponentHandle, Rgba8Pixel, SharedPixelBuffer,
    platform::{WindowEvent, update_timers_and_animations},
};
/// Disposable fixtures and shared snapshot pointers.
use std::{fs, sync::Arc};

/// Apply a system color scheme exactly where the portal watcher applies it,
///  then run change handlers.
pub(super) fn switch(window: &AppWindow, scheme: ColorScheme) {
    WindowInner::from_pub(window.window())
        .context()
        .set_color_scheme(scheme);
    update_timers_and_animations();
}

/// The rendered window frame.
pub(super) fn frame(window: &AppWindow) -> SharedPixelBuffer<Rgba8Pixel> {
    return window.window().take_snapshot().expect("window frame");
}

/// One frame pixel at whole logical coordinates;
///  the headless window has scale one.
pub(super) fn pixel(frame: &SharedPixelBuffer<Rgba8Pixel>, x: f32, y: f32) -> Rgba8Pixel {
    let index = y as usize * frame.width() as usize + x as usize;
    return frame.as_slice()[index];
}

/// WCAG 2 relative luminance of an opaque pixel.
fn luminance(pixel: Rgba8Pixel) -> f32 {
    return 0.2126 * linear(pixel.r) + 0.7152 * linear(pixel.g) + 0.0722 * linear(pixel.b);
}

/// One color channel as linear light,
///  from the sRGB encoding.
fn linear(channel: u8) -> f32 {
    let value = f32::from(channel) / 255.0;
    if value <= 0.04045 {
        return value / 12.92;
    }
    return ((value + 0.055) / 1.055).powf(2.4);
}

/// The address of the shaped rows the reader last painted;
///  a repaint replaces the rows and so changes the address.
fn painted(reader: &Reader) -> *const ide_app::shaped_text::ShapedRow {
    return reader
        .source
        .borrow()
        .shaped
        .as_ref()
        .expect("shaped")
        .rows
        .as_ptr();
}

/// The logical height of every block of virtual rows the reader holds.
fn heights(reader: &Reader) -> Vec<f32> {
    let source = reader.source.borrow();
    let mut found: Vec<f32> = Vec::new();
    for block in source.blocks.iter() {
        found.push(block.height());
    }
    return found;
}

/// WCAG 2 contrast ratio of two opaque pixels.
pub(super) fn contrast(first: Rgba8Pixel, second: Rgba8Pixel) -> f32 {
    let (lighter, darker) = (
        luminance(first).max(luminance(second)),
        luminance(first).min(luminance(second)),
    );
    return (lighter + 0.05) / (darker + 0.05);
}

/// Whether a frame pixel equals a toolkit color within rounding.
pub(super) fn near(pixel: Rgba8Pixel, color: Color) -> bool {
    return pixel.r.abs_diff(color.red()) <= 2
        && pixel.g.abs_diff(color.green()) <= 2
        && pixel.b.abs_diff(color.blue()) <= 2;
}

/// Whether a frame pixel is at least half way from `background` to a toolkit color,
///  in every channel that differs between the two.
fn mostly(pixel: Rgba8Pixel, color: Color, background: Rgba8Pixel) -> bool {
    let channels = [
        (pixel.r, color.red(), background.r),
        (pixel.g, color.green(), background.g),
        (pixel.b, color.blue(), background.b),
    ];
    let mut half = true;
    for (shown, ink, behind) in channels {
        let whole = f32::from(ink.abs_diff(behind));
        let done = f32::from(shown.abs_diff(behind));
        if whole > 8.0 && done < whole / 2.0 {
            half = false;
        }
    }
    return half;
}

/// A reader over the shared fixture in a fresh disposable project;
///  the directory lives as long as the reader.
pub(super) fn fixture_reader(text: &str) -> (tempfile::TempDir, Reader) {
    let directory = tempfile::tempdir().expect("disposable paint project");
    fs::write(directory.path().join("main.rs"), text).expect("paint fixture");
    let opened = reader(directory.path(), "main.rs");
    return (directory, opened);
}

/// The strongest contrast against `background` among the pixels of the window rectangle starting at (`left`,
/// `top`),
///  `width` by `height` logical pixels.
fn strongest(
    shown: &SharedPixelBuffer<Rgba8Pixel>,
    origin: (f32, f32),
    size: (f32, f32),
    background: Rgba8Pixel,
) -> (Rgba8Pixel, f32) {
    let mut best = background;
    for dy in 0..size.1 as usize {
        for dx in 0..size.0 as usize {
            let candidate = pixel(shown, origin.0 + dx as f32, origin.1 + dy as f32);
            if contrast(candidate, background) > contrast(best, background) {
                best = candidate;
            }
        }
    }
    return (best, contrast(best, background));
}

/// In both schemes the hint row is painted in the hint ink and every message row in the ink of its severity,
/// each at least 4.5:1 against the background,
///  and the error underline is drawn in the error ink;
///  contrast is
/// measured on rendered pixels.
#[test]
fn rows_and_marks_render_in_both_schemes_with_measured_contrast() {
    let mut inks = Vec::new();
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        let (_directory, reader) = fixture_reader(FIXTURE);
        let window = &reader.window;
        switch(window, scheme);
        annotate(&reader);
        update_timers_and_animations();
        let shown = frame(window);
        let error = window.get_error_ink();
        let source = reader.source.borrow();
        let view = source.shaped.as_ref().expect("shaped");
        let rows = view.annotations.as_ref().expect("annotation frame");
        // The last line is empty and has no block: its row shows the plain background.
        let empty = TEXT_TOP + source.row_map.code_top(3) + 12.0;
        let background = pixel(&shown, TEXT_LEFT + 700.0, empty);
        let mut least: f32 = f32::MAX;
        for text in &rows.texts {
            let top = TEXT_TOP + source.row_map.code_top(text.line) - text.rise;
            let (ink, ratio) = strongest(
                &shown,
                (TEXT_LEFT + text.x, top),
                (text.width, ROW_HEIGHT),
                background,
            );
            println!(
                "contrast {scheme:?}: line {} row rising {} ({:?}): ink {ink:?} vs background {background:?} = {ratio:.2}",
                text.line, text.rise, text.severity
            );
            least = least.min(ratio);
            // A message row's strongest pixel is its severity's ink; a hint row's is the hint ink over the background.
            if text.severity == Some(Severity::Error) {
                assert!(
                    near(ink, error),
                    "{scheme:?}: an error row is not in the error ink"
                );
            }
        }
        assert!(
            least >= 4.5,
            "{scheme:?}: a virtual row reaches only {least:.2}:1"
        );
        // Underline pixels under `area(2, 3)` on line 0, in the band below the baseline of its code row.
        let code = TEXT_TOP + source.row_map.code_top(0);
        let view_left = view.rows[0].caret_x(12, 1.0);
        let mut underline = 0;
        for y in 17..24 {
            for x in 0..60 {
                // A thin wave is anti-aliased: a pixel counts when it is at least half way from the background to the ink.
                if mostly(
                    pixel(&shown, TEXT_LEFT + view_left + x as f32, code + y as f32),
                    error,
                    background,
                ) {
                    underline += 1;
                }
            }
        }
        println!(
            "contrast {scheme:?}: least row contrast {least:.2}; error underline pixels {underline}"
        );
        assert!(
            underline > 20,
            "{scheme:?}: the error underline was not drawn in the error ink"
        );
        inks.push(error);
    }
    assert_ne!(inks[0], inks[1], "the error ink does not follow the scheme");
}

/// A snapshot whose only change lies outside the materialized rows does not repaint;
///  one inside does;
/// handing back the same snapshots does nothing.
#[test]
fn only_visible_annotation_changes_repaint() {
    let mut text = String::new();
    for index in 0..200 {
        text.push_str(&format!("line {index}: value\n"));
    }
    let (_directory, reader) = fixture_reader(&text);
    let window = &reader.window;
    let original = painted(&reader);
    let stamp = displayed(&reader.source.borrow());
    // Character offset of line 150, far below the materialized rows.
    let mut far = 0;
    for line in text.lines().take(150) {
        far += line.chars().count() + 1;
    }
    let outside = Some(Arc::new(DiagnosticsSnapshot {
        stamp,
        groups: vec![SourceGroup {
            source: "rustc".to_string(),
            items: vec![problem(far, far + 4, Severity::Error, "E1", "far below")],
        }],
    }));
    set_annotations(window, &reader.source, None, outside.clone());
    assert_eq!(
        painted(&reader),
        original,
        "a change outside the materialized rows repainted the source"
    );
    assert_eq!(shown(&reader), (0, 0, 0));
    assert_eq!(
        reader.source.borrow().row_map.block_height(150),
        BLOCK_GAP + ROW_HEIGHT,
        "the far diagnostic must still take its row in the vertical mapping (positive control)"
    );
    let inside = Some(Arc::new(DiagnosticsSnapshot {
        stamp,
        groups: vec![SourceGroup {
            source: "rustc".to_string(),
            items: vec![
                problem(far, far + 4, Severity::Error, "E1", "far below"),
                problem(16, 20, Severity::Warning, "W1", "near"),
            ],
        }],
    }));
    set_annotations(window, &reader.source, None, inside.clone());
    let repainted = painted(&reader);
    assert_ne!(repainted, original, "a visible change did not repaint");
    assert_eq!(shown(&reader), (0, 1, 1));
    set_annotations(window, &reader.source, None, inside);
    assert_eq!(painted(&reader), repainted, "the same snapshots repainted");
}

/// At twice the display scale every virtual row keeps its logical place:
///  blocks are as tall,
///  hints stand at
/// the caret x of their positions at that scale,
///  and the image is twice as large in physical pixels.
#[test]
fn display_scale_keeps_virtual_rows_in_place() {
    let (_directory, reader) = fixture_reader(FIXTURE);
    let window = &reader.window;
    annotate(&reader);
    update_timers_and_animations();
    let before = heights(&reader);
    let image = tile_size(window, &reader.source);
    let scale = window.window().scale_factor();
    window
        .window()
        .dispatch_event(WindowEvent::ScaleFactorChanged {
            scale_factor: scale * 2.0,
        });
    update_timers_and_animations();
    assert_eq!(
        heights(&reader),
        before,
        "blocks changed their logical height"
    );
    let doubled = tile_size(window, &reader.source);
    assert_eq!(doubled.1, image.1 * 2);
    assert_eq!(doubled.0, image.0 * 2);
    let source = reader.source.borrow();
    let view = source.shaped.as_ref().expect("shaped");
    assert_eq!(view.viewport.scale, scale * 2.0);
    let rows = view.annotations.as_ref().expect("annotation frame");
    // The first hint annotates character 9 of line 0.
    assert_eq!(rows.texts[0].x, view.rows[0].caret_x(9, scale * 2.0));
    assert_eq!(rows.texts[0].rise, 3.0 * ROW_HEIGHT);
}
