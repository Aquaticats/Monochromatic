//! The minimap is a zoomed-out reflection of the window:
//!  every pixel of its picture is the average of the window's own pixels it stands for,
//!  read from a snapshot of the rendered window and from the minimap's texture,
//!  for plain text,
//! a selection,
//! find matches,
//! wrapped lines,
//! and diagnostics with their hints,
//!  at display scale one and two.
//!  This is what tells a reflection from a re-drawing.
//! Nothing here waits on the window thread:
//! every wait advances timers and lets the workers answer.

/// The window's snapshot and its toolkit colors.
use super::annotation_paint_tests::{fixture_reader, frame};
/// Hints and diagnostics over the fixture,
/// and the position of the text.
use super::annotation_tests::{FIXTURE, annotate};
/// The reader fixture and the keys.
use super::find_tests::{Reader, chord, settle, type_text};
/// The geometry of the text now shown.
use super::render::render;
/// The window's metrics.
use super::sidebar_tests::{DIVIDER, GUTTER, HEADER, resize};
/// The minimap's texture,
/// read back from the GPU.
use super::tile_reads::minimap_pixels;
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::Waiting;
/// The geometry the picture follows.
use ide_app::document::{ReadingAffinity, ReadingPosition};
use ide_app::minimap::Geometry;
/// Toolkit events.
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent, update_timers_and_animations},
};

/// The fixture's three lines followed by code,
/// some of it long enough to wrap in the text area.
fn text() -> String {
    let mut text = String::from(FIXTURE);
    for number in 0..24 {
        if number % 5 == 2 {
            text.push_str(&format!(
                "    let value_{number} = compute({number}, \"a long argument that has to wrap in this narrow window\", Some(other.field), [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);\n"
            ));
        } else {
            text.push_str(&format!(
                "fn item_{number}(value: u32) -> u32 {{ value + {number} }}\n"
            ));
        }
    }
    return text;
}

/// A reader on the text at display scale `scale`,
/// with its first minimap picture on screen.
/// The headless window renders into a buffer of 726,000 pixels,
/// 1100 to a line,
/// which the default window of 1100 by 660 pixels fills at scale one;
/// at scale two the window is made 960 by 640 pixels.
fn opened(scale: f32) -> (tempfile::TempDir, Reader) {
    let (directory, reader) = fixture_reader(&text());
    let window = &reader.window;
    if scale != 1.0 {
        resize(window, 480.0, 320.0);
        window
            .window()
            .dispatch_event(WindowEvent::ScaleFactorChanged {
                scale_factor: scale,
            });
        update_timers_and_animations();
    }
    picture_current(&reader, 0);
    return (directory, reader);
}

/// Wait until the syntax classes have arrived,
/// then until a picture newer than the `shown` pictures before is on
/// screen for the request that was posted last,
/// with the text,
/// the classes,
/// and the rows the window has now.
fn picture_current(reader: &Reader, shown: u64) {
    {
        let mut waiting = Waiting::slow();
        while reader.source.borrow().styles.is_empty() {
            waiting.step("the highlighting did not arrive");
        }
    }
    settle();
    settle();
    let mut waiting = Waiting::slow();
    loop {
        {
            let current = reader.source.borrow();
            let styles = std::sync::Arc::as_ptr(&current.styles).cast::<()>() as usize;
            let current_picture = match current.minimap.shown {
                Some(picture) => {
                    picture.styles == styles
                        && picture.rows == current.row_key
                        && picture.stamp.revision == current.document.revision()
                }
                None => false,
            };
            if current_picture
                && current.minimap.posted == current.minimap.shown
                && current.minimap.images_set > shown
            {
                break;
            }
        }
        waiting.step("the minimap picture did not follow the change");
    }
    settle();
    super::overscan_tile_tests::settle_tile(&reader.source);
    update_timers_and_animations();
}

/// How many pictures the window has set.
fn pictures(reader: &Reader) -> u64 {
    return reader.source.borrow().minimap.images_set;
}

/// What a comparison of the picture with the window found.
struct Found {
    /// The largest difference of any channel of any compared pixel.
    worst: u8,
    /// The mean difference over compared pixel channels.
    mean: f64,
    /// Pixels of the picture compared.
    counted: usize,
    /// Pixels of the picture that are not the background.
    inked: usize,
}

/// The overlap of the intervals `[a, b)` and `[c, d)`.
fn overlap(a: f64, b: f64, c: f64, d: f64) -> f64 {
    return (b.min(d) - a.max(c)).max(0.0);
}

/// What:
///  Compare every pixel of the minimap's picture with the box average of the window pixels it stands for.
/// Why:
///  The picture's pixels are straight alpha (how the toolkit holds an image);
/// laid over the window's background it must equal the average of the
///  opaque window pixels,
///  because compositing over a constant background is linear.
///  Rows where the caret stands are left out,
///  and so is any picture row whose source rows reach outside the text.
fn compare(reader: &Reader) -> Found {
    return compare_at(reader, (0, 0));
}

/// `compare`,
/// with the source read `shift` pixels to the right and down of where it should be.
fn compare_at(reader: &Reader, shift: (i64, i64)) -> Found {
    let window = &reader.window;
    let scale = f64::from(window.window().scale_factor());
    let snapshot = frame(window);
    let image = minimap_pixels(window, &reader.source).expect("picture pixels");
    let first_row = window.get_minimap_picture_top() as usize;
    let (out_width, out_height) = (image.width() as usize, image.height() as usize);
    let state = reader.source.borrow();
    let content = f64::from(state.row_map.height());
    let wrap = f64::from(state.width);
    drop(state);
    let view = f64::from(window.get_viewport_height());
    let geometry = Geometry::new(content as f32, view as f32);
    let ratio = f64::from(geometry.scale());
    let columns = snapshot.width() as usize;
    // The source column ends at the window's right edge, with the minimap beside the text area.
    // The sidebar gives way in a narrow window, so its shown width is read from the window.
    let text_left = f64::from(window.get_sidebar_width()) + f64::from(DIVIDER) + f64::from(GUTTER);
    let left = text_left * scale + shift.0 as f64;
    let top = f64::from(HEADER) * scale + shift.1 as f64;
    let background =
        snapshot.as_slice()[(top as usize + 2) * columns + (left + wrap * scale) as usize - 2];
    let source_per_column = wrap * scale / out_width as f64;
    let caret_top = f64::from(window.get_caret_y()) * scale;
    let caret_bottom = caret_top + 24.0 * scale;
    let mut found = Found {
        worst: 0,
        mean: 0.0,
        counted: 0,
        inked: 0,
    };
    let mut total = 0.0;
    for row in 0..out_height {
        let from = (first_row + row) as f64 / ratio;
        let to = (first_row + row + 1) as f64 / ratio;
        // Only rows whose source lies in the window's view, and not under the caret, can be compared.
        if to > content * scale
            || to > view * scale
            || (to > caret_top - 1.0 && from < caret_bottom + 1.0)
        {
            continue;
        }
        for column in 0..out_width {
            let x0 = column as f64 * source_per_column;
            let x1 = x0 + source_per_column;
            let mut sums = [0.0_f64; 3];
            for y in (from.floor() as usize)..(to.ceil() as usize) {
                let wy = overlap(y as f64, y as f64 + 1.0, from, to);
                if wy <= 0.0 {
                    continue;
                }
                for x in (x0.floor() as usize)..(x1.ceil() as usize) {
                    let wx = overlap(x as f64, x as f64 + 1.0, x0, x1);
                    if wx <= 0.0 {
                        continue;
                    }
                    let at = (top as usize + y) * columns + left as usize + x;
                    let pixel = snapshot.as_slice()[at];
                    let weight = wx * wy / (source_per_column / ratio);
                    sums[0] += weight * f64::from(pixel.r);
                    sums[1] += weight * f64::from(pixel.g);
                    sums[2] += weight * f64::from(pixel.b);
                }
            }
            let picture = image.as_slice()[row * out_width + column];
            // The texture holds straight alpha.
            let covered = f64::from(picture.a) / 255.0;
            let behind = 1.0 - covered;
            let shown = [
                f64::from(picture.r) * covered + f64::from(background.r) * behind,
                f64::from(picture.g) * covered + f64::from(background.g) * behind,
                f64::from(picture.b) * covered + f64::from(background.b) * behind,
            ];
            for channel in 0..3 {
                let difference = (shown[channel] - sums[channel]).abs();
                if difference > 25.0 {
                    eprintln!(
                        "WORSTPX row {row} col {column} ch {channel} shown {:.1} avg {:.1} alpha {} bg {}",
                        shown[channel], sums[channel], picture.a, background.r
                    );
                }
                found.worst = found.worst.max(difference.round() as u8);
                total += difference;
            }
            found.counted += 1;
            if picture.a > 2 {
                found.inked += 1;
            }
        }
    }
    found.mean = total / (found.counted as f64 * 3.0);
    return found;
}

/// The picture is the reflection the window gives,
/// with a bound on how far any pixel strays.
fn assert_reflection(reader: &Reader, what: &str) -> Found {
    let found = compare(reader);
    println!(
        "{what}: {} pixels compared, {} with ink, worst difference {}, mean {:.3}",
        found.counted, found.inked, found.worst, found.mean
    );
    assert!(
        found.counted > 500,
        "{what}: only {} pixels compared",
        found.counted
    );
    assert!(
        found.inked > 100,
        "{what}: the picture holds only {} inked pixels",
        found.inked
    );
    // The window and the picture are drawn by the same renderer from the same items, the picture scaled down.
    // Measured 2026-10-20 with the picture aligned (a shift of one pixel in any direction is far worse),
    // the same on every run: the worst pixel at scale 1 is 13 for plain text and find matches, 19 for diagnostics
    // and 32 for a selection; at scale 2 it is 41, 41, 49 and 54; the mean is at most 0.45, 0.68, 0.41 and 0.39
    // at scale 1 and 0.53, 0.78, 0.57 and 0.47 at scale 2.
    // The total ink of the picture and of the window agree within 0.1 to 1.4 percent;
    // what remains is where the ink of a glyph edge falls inside a pixel.
    // The bounds sit a little above those measurements, per scale.
    let scale = reader.window.window().scale_factor();
    let (worst, mean) = if scale > 1.5 { (58, 0.9) } else { (36, 0.8) };
    assert!(
        found.worst <= worst,
        "{what}: a pixel of the minimap differs by {} from the average of the window's pixels",
        found.worst
    );
    assert!(
        found.mean <= mean,
        "{what}: the minimap differs by {:.3} on average from the average of the window's pixels",
        found.mean
    );
    return found;
}

/// A fingerprint of the picture on screen.
fn fingerprint(reader: &Reader) -> Vec<u8> {
    let image = minimap_pixels(&reader.window, &reader.source).expect("picture pixels");
    let mut bytes = Vec::new();
    for pixel in image.as_slice() {
        bytes.extend_from_slice(&[pixel.r, pixel.g, pixel.b, pixel.a]);
    }
    return bytes;
}

/// Plain text,
/// with wrapped lines,
/// at both display scales.
#[test]
fn the_minimap_of_plain_wrapped_text_is_the_average_of_the_windows_pixels() {
    for scale in [1.0_f32, 2.0] {
        let (_directory, reader) = opened(scale);
        let wrapped = {
            let state = reader.source.borrow();
            let mut any = false;
            for line in 0..state.row_map.lines() {
                if state.row_map.rows_of(line) > 1 {
                    any = true;
                }
            }
            any
        };
        assert!(wrapped, "the fixture has a wrapped line");
        assert_reflection(&reader, &format!("plain text at scale {scale}"));
        // Positive control: the comparison tells a reflection from one a few pixels off.
        let shifted = compare_at(&reader, (3, 0));
        assert!(
            shifted.worst > 12 && shifted.mean > 1.5,
            "a window read three pixels aside still matched the picture (worst {}, mean {:.3})",
            shifted.worst,
            shifted.mean
        );
        reader.window.hide().expect("close minimap window");
    }
}

/// A selection across several lines shows in the picture as it does in the window.
#[test]
fn the_minimap_of_a_selection_is_the_average_of_the_windows_pixels() {
    for scale in [1.0_f32, 2.0] {
        let (_directory, reader) = opened(scale);
        let before = fingerprint(&reader);
        let shown = pictures(&reader);
        {
            let mut state = reader.source.borrow_mut();
            let end = state.document.text().line_to_char(14) + 20;
            state.document.select(ReadingPosition {
                anchor: 30,
                head: end,
                viewport: 0,
                affinity: ReadingAffinity::Downstream,
            });
        }
        render(&reader.window, &reader.source);
        picture_current(&reader, shown);
        assert_ne!(
            fingerprint(&reader),
            before,
            "the selection changed nothing in the picture"
        );
        assert_reflection(&reader, &format!("a selection at scale {scale}"));
        reader.window.hide().expect("close minimap window");
    }
}

/// The matches of an in-file find,
/// and the active match with its boundary,
/// show as they do in the window.
#[test]
fn the_minimap_of_find_matches_is_the_average_of_the_windows_pixels() {
    for scale in [1.0_f32, 2.0] {
        let (_directory, reader) = opened(scale);
        let before = fingerprint(&reader);
        let shown = pictures(&reader);
        chord(&reader.window, Key::Control, "f");
        type_text(&reader.window, "value");
        // Typing selects the first match by itself once the worker's reply for the whole query arrives, and the
        // selection spells the query only then (a reply for a shorter prefix selects the prefix). An Enter pressed
        // after that reply steps on to the second match and one pressed before it does not, so the test presses
        // none and waits for the first match of the whole query.
        {
            let mut waiting = Waiting::eventually();
            while !(reader.window.get_selection_is_match()
                && reader.window.get_find_status().starts_with("1/")
                && reader.window.get_selected_text().to_lowercase() == "value")
            {
                waiting.step("the find did not reach its first match");
            }
        }
        picture_current(&reader, shown);
        assert_ne!(
            fingerprint(&reader),
            before,
            "the matches changed nothing in the picture"
        );
        assert_reflection(&reader, &format!("find matches at scale {scale}"));
        reader.window.hide().expect("close minimap window");
    }
}

/// Hints,
/// a diagnostic's underline and its message rows show as they do in the window.
#[test]
fn the_minimap_of_diagnostics_and_hints_is_the_average_of_the_windows_pixels() {
    for scale in [1.0_f32, 2.0] {
        let (_directory, reader) = opened(scale);
        let before = fingerprint(&reader);
        let shown = pictures(&reader);
        annotate(&reader);
        picture_current(&reader, shown);
        assert_ne!(
            fingerprint(&reader),
            before,
            "the annotations changed nothing in the picture"
        );
        assert_reflection(&reader, &format!("annotations at scale {scale}"));
        reader.window.hide().expect("close minimap window");
    }
}
