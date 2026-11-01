//! A tall window over a file whose wrapped lines the background measurement has not reached:
//!  the tile is cut to the raster limit instead of refused,
//!  the text stays on screen,
//!  and no window size or display scale produces a raster past the limit.
//! Nothing here waits on the window thread:
//! every wait advances timers and lets the workers answer.

/// The paint inputs of a tile that a windowless build needs,
/// and the tile's raster width rule.
use super::render::{tile_cap, viewport_of};
/// The map rules a measurement ends in.
use super::rows::{absorb, settle};
/// Exact window sizes for the text width.
use super::sidebar_tests::{TEXT_LEFT, minimap_beside};
/// The worker's pass gate,
/// the tile that is wanted,
/// and the placement of the view.
use super::{
    State,
    tile::{TileBounds, requested},
    tile_build::PASS_GATE,
    viewport::{holds as window_holds, place, wrap_width},
};
/// The open reader and the pixels of the shown image.
use super::{
    tile_reads::{shown, tile_size, tile_textures},
    wrap_tests::{long_text, measured, open},
};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::Waiting;
/// The code row the raster edges are rounded to,
/// and the limit one raster may not pass.
use ide_app::{
    row_map::CODE_ROW,
    shaped_text::{stack_fits, texture_dimension, texture_fits},
};
/// Toolkit events.
use slint::{
    ComponentHandle, LogicalSize,
    platform::{WindowEvent, update_timers_and_animations},
};

/// Lines of several lengths,
/// from a few words to several rows at any text width of a window,
/// in a fixed pseudo-random order.
pub(super) fn mixed(count: usize) -> String {
    let mut text = String::new();
    let mut seed: u64 = 12345;
    for number in 0..count {
        seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let words = match (seed >> 33) % 10 {
            0..=4 => 3,
            5..=6 => 20,
            7 => 60,
            8 => 120,
            _ => 250,
        };
        text.push_str(&format!(
            "const line_{number} = call({});\n",
            "word ".repeat(words)
        ));
    }
    return text;
}

/// Hold the worker's measurement of the whole text still until the returned sender is dropped,
/// so the vertical map counts every line it has not been told about as one row.
pub(super) fn hold_the_pass() -> std::sync::mpsc::Sender<()> {
    let (release, gate) = std::sync::mpsc::channel();
    *PASS_GATE.lock().expect("pass gate") = Some(gate);
    return release;
}

/// Wait until no tile is queued or on the worker.
pub(super) fn settled_tile(reader: &super::find_tests::Reader) {
    let mut waiting = Waiting::slow();
    loop {
        update_timers_and_animations();
        {
            let current = reader.source.borrow();
            if current.tile_job.is_none() && current.scroll_wanted.is_none() {
                return;
            }
        }
        waiting.step("the tile worker did not finish the re-center");
    }
}

/// Report a window with a text area `text_width` wide and `height` tall to the toolkit,
/// without drawing a frame:
/// the headless renderer cannot draw a window this tall.
pub(super) fn resize_without_frame(
    reader: &super::find_tests::Reader,
    text_width: f32,
    height: f32,
) {
    reader.window.window().dispatch_event(WindowEvent::Resized {
        size: LogicalSize::new(TEXT_LEFT + text_width + minimap_beside(text_width), height),
    });
    update_timers_and_animations();
}

/// A window so tall that the tile of three views passes the raster limit,
/// over wrapped lines no measurement has reached,
/// at a place far from the top:
/// the tile the window chose from the map's one row for each line is several times taller once its lines are shaped.
/// The text is painted,
/// the window names no failure,
/// and the raster holds the view.
#[test]
fn a_tall_window_over_unmeasured_wrapped_lines_paints_text_and_names_no_failure() {
    for height in [3000.0_f32, 4100.0, 4100.0] {
        let (_project, opened) = open(&mixed(6000));
        let release = hold_the_pass();
        let window = &opened.window;
        resize_without_frame(&opened, 1330.0, height);
        settled_tile(&opened);
        window.set_scroll_y(-30_000.0);
        update_timers_and_animations();
        settled_tile(&opened);
        assert_eq!(
            window.get_error_message().to_string(),
            "",
            "height {height}: the window shows a failure"
        );
        {
            let current = opened.source.borrow();
            let view = current.shaped.as_ref().expect("a shown frame");
            assert!(
                stack_fits(u64::from(view.width), u64::from(view.height)),
                "height {height}: the raster passes the limit"
            );
            let offset = -window.get_scroll_y();
            assert!(
                (offset - current.offset).abs() < 1.0,
                "height {height}: the window is at {offset} and the state at {}",
                current.offset
            );
            let top = view.origin;
            let bottom = view.origin + view.height as f32 / view.viewport.scale;
            assert!(
                top <= offset + 0.5 && offset + window.get_viewport_height() <= bottom + 0.5,
                "height {height}: the raster [{top}, {bottom}] does not hold the view at {offset}"
            );
        }
        let mut inked = false;
        for pixel in shown(window, &opened.source) {
            if pixel[3] > 0 {
                inked = true;
                break;
            }
        }
        assert!(inked, "height {height}: the text area is blank");
        drop(release);
        window.hide().expect("close wrap window");
    }
}

/// What `tile_for` found.
struct Built {
    /// The raster is within the stack of textures a tile may be.
    fits: bool,
    /// The shaper had to cut the clip the window asked for.
    cut: bool,
    /// The raster holds the view,
    /// or no raster can.
    holds: bool,
    /// One raster can hold the view.
    holdable: bool,
    /// The window accepts the tile as holding the view,
    /// which ends its request for another.
    accepted: bool,
    /// The numbers behind the answers,
    /// for a failure message.
    detail: String,
}

/// The tile the window asks for at `offset` in a view `text_width` by `height` at display `scale`,
/// shaped on the shaper the window uses,
/// with the map as the window holds it before its measurement of the whole text (`measured` false)
/// or after (`measured` true);
/// the answer is whether the raster is within the stack of textures a tile may be,
/// whether the shaper had to cut the clip the window asked for,
/// and whether the raster holds the view.
fn tile_for(
    text: &str,
    scale: f32,
    text_width: f32,
    height: f32,
    offset: f32,
    measured: bool,
) -> Built {
    let mut current = State::new(text, None);
    current.scale = scale;
    place(&mut current, offset, text_width, height);
    if measured {
        let lines = current.document.text().len_lines();
        let found = {
            let State {
                shaper,
                document,
                styles,
                width,
                ..
            } = &mut current;
            shaper.measure_lines(document, 0..lines, scale, Some(*width), styles)
        };
        absorb(&mut current, found.lines);
        settle(&mut current, scale, (offset, text_width, height), true);
    }
    let asked = requested(&current);
    let viewport = viewport_of(asked, scale);
    let view = {
        let State {
            shaper,
            document,
            styles,
            row_map,
            ..
        } = &mut current;
        shaper.prepare_rows(document, viewport, styles, row_map)
    };
    let fits = stack_fits(u64::from(view.width), u64::from(view.height));
    let cut = view.viewport.clip != asked.clip;
    let cap = tile_cap(current.width, scale, height);
    let view_top = current.offset.max(0.0);
    let top = view.origin;
    let bottom = view.origin + view.height as f32 / scale;
    // A raster holds the view when it reaches both its edges, or the ends of the text.
    let holds = (top <= view_top + 0.5 || top <= 0.0)
        && (view_top + height <= bottom + 0.5 || bottom >= current.row_map.height() - 0.5);
    // A view that does not fit a raster of the cap cannot be held by any.
    let holdable = height + 2.0 * CODE_ROW <= (cap / CODE_ROW).floor() * CODE_ROW;
    let detail = format!(
        "raster [{top}, {bottom}], view [{view_top}, {}], cap {cap}, text {}, clip {:?}, asked {:?}",
        view_top + height,
        current.row_map.height(),
        view.viewport.clip,
        asked.clip
    );
    let built = TileBounds {
        clip: view.viewport.clip,
        ..asked
    };
    let accepted = window_holds(&current, built, view_top, height);
    return Built {
        fits,
        cut,
        holds: holds || !holdable,
        holdable,
        accepted,
        detail,
    };
}

/// Window sizes and display scales,
/// with the map before and after the measurement of the whole text:
/// the raster of the tile is never past the limit,
/// and after the measurement it holds the view whenever one raster can.
/// The guard is seen working in at least one case,
/// so the test cannot pass because no case reaches it.
#[test]
fn no_window_size_or_scale_makes_a_raster_past_the_limit() {
    let text = mixed(500);
    let mut cuts_before_measurement = 0;
    let mut holdable_cases = 0;
    for scale in [0.5_f32, 1.0, 1.25, 1.5, 1.75, 2.0, 3.0] {
        for text_width in [160.0_f32, 1330.0, 3000.0] {
            // A window is held to the device's texture side in physical pixels, and its raster is as wide as its
            // text area: a width the device cannot make a texture of is not a window.
            if ((text_width + 22.0) * scale).ceil() > texture_dimension() as f32 {
                continue;
            }
            for height in [660.0_f32, 3500.0, 12_000.0] {
                for offset in [0.0_f32, 14_000.0] {
                    let label =
                        format!("scale {scale} width {text_width} height {height} offset {offset}");
                    let stale = tile_for(&text, scale, text_width, height, offset, false);
                    assert!(
                        stale.fits,
                        "{label}: a raster past the limit before the measurement"
                    );
                    if stale.cut {
                        cuts_before_measurement += 1;
                    }
                    let found = tile_for(&text, scale, text_width, height, offset, true);
                    assert!(
                        found.fits,
                        "{label}: a raster past the limit after the measurement"
                    );
                    assert!(
                        found.holds,
                        "{label}: the raster does not hold the view: {}",
                        found.detail
                    );
                    assert!(
                        found.accepted,
                        "{label}: the window would not accept the tile it asked for: {}",
                        found.detail
                    );
                    if found.holdable {
                        holdable_cases += 1;
                    }
                }
            }
        }
    }
    assert!(
        cuts_before_measurement > 0,
        "no case made the shaper cut a clip: the test does not reach the guard"
    );
    assert!(holdable_cases > 0, "no case had a view one raster can hold");
}

/// A window nearly as tall as the tallest stack of textures a tile may be,
/// over wrapped text:
/// the tile the worker builds is cut to the raster limit,
/// the window accepts it as holding the view,
/// and the measurement pass finishes.
/// The view's margin of a quarter view height on each side passed the cap,
/// so the window discarded every tile the worker built
/// and asked for the same tile again,
/// and the pass never finished (issue 1785).
/// The heights come from `tile_cap` and not from fixed sizes:
/// the cap grew when tiles became stacks of textures,
/// and heights of 10,000 to 14,000 px then left more than a quarter view of room on each side,
/// so the margin was never cut and the test passed with the cut removed.
/// Each height is checked to leave less room than the margin asks for,
/// and the text to be taller than the tile:
/// a tile that reaches an end of the text is held on that side whatever the margin.
#[test]
fn a_tall_window_over_wrapped_text_finishes_the_measurement_pass() {
    let text_width = 1328.0_f32;
    for share in [0.85_f32, 0.95] {
        let (_project, opened) = open(&long_text(1500));
        let scale = opened.window.window().scale_factor();
        let cap = tile_cap(wrap_width(text_width).expect("a width"), scale, 1.0e9);
        let height = (cap * share).floor();
        resize_without_frame(&opened, text_width, height);
        {
            // The margin the window asks for is a quarter of the view, and the cap leaves room for less.
            let current = opened.source.borrow();
            let view = opened.window.get_viewport_height();
            let room = (tile_cap(current.width, current.scale, view) - view) / 2.0 - 2.0 * CODE_ROW;
            assert!(
                0.0 < room && room < 0.25 * view,
                "height {height}: a view {view} px tall leaves {room} px of room, not under a quarter view: the test does not reach the guard"
            );
        }
        let mut waiting = Waiting::slow();
        while !measured(&opened.source, scale) {
            waiting.step(format_args!(
                "height {height}: the measurement pass did not finish"
            ));
        }
        update_timers_and_animations();
        assert_eq!(
            opened.window.get_error_message().to_string(),
            "",
            "height {height}: the window shows a failure"
        );
        {
            let current = opened.source.borrow();
            let view = current.shaped.as_ref().expect("a shown frame");
            assert!(
                stack_fits(u64::from(view.width), u64::from(view.height)),
                "height {height}: the raster passes the limit"
            );
            assert!(
                current.row_map.height() > cap,
                "height {height}: the text is {} px tall, not past the cap of {cap} px: the test does not reach the guard",
                current.row_map.height()
            );
        }
        opened.window.hide().expect("close wrap window");
    }
}

/// A 2000 px tall window at display scales 1 and 2 on a device whose textures are at most 8192 px on a side
/// (the harness device is limited to that):
/// the tile is stacked from textures that are each within the limit on both sides,
/// together hold the view with overscan,
/// and are drawn with text.
/// A tile of three views is 6,048 px at scale 1 and 12,096 px at scale 2:
/// one texture at the first,
/// two at the second.
#[test]
fn a_2000_px_window_gets_a_tile_of_textures_within_the_devices_limit_at_both_scales() {
    for scale in [1.0_f32, 2.0] {
        let (_project, opened) = open(&mixed(6000));
        let window = &opened.window;
        window
            .window()
            .dispatch_event(WindowEvent::ScaleFactorChanged {
                scale_factor: scale,
            });
        resize_without_frame(&opened, 1100.0, 2000.0);
        settled_tile(&opened);
        // Until the measurement pass has finished the map counts each unmeasured line as one row, and a tile
        // placed by it is cut to the bound of the stack and not to three views, however many textures that is.
        super::wrap_tests::settled(&opened);
        window.set_scroll_y(-30_000.0);
        update_timers_and_animations();
        settled_tile(&opened);
        let (width, height) = tile_size(window, &opened.source);
        let textures = tile_textures(window, &opened.source);
        println!(
            "TILE scale {scale}: window 2000 px tall, {} textures {textures:?} of {width} x {height} px in all, device limit {}",
            textures.len(),
            texture_dimension()
        );
        let mut together = 0;
        for (across, down) in &textures {
            assert!(
                *across <= texture_dimension() && *down <= texture_dimension(),
                "scale {scale}: the texture {across}x{down} passes the device's {} px limit",
                texture_dimension()
            );
            assert!(
                texture_fits(u64::from(*across), u64::from(*down)),
                "scale {scale}: the texture {across}x{down} passes the limit of one texture"
            );
            together += down;
        }
        assert_eq!(
            together, height,
            "scale {scale}: the textures do not add up"
        );
        assert_eq!(
            textures.len(),
            if scale > 1.5 { 2 } else { 1 },
            "scale {scale}: {textures:?}"
        );
        assert!(
            stack_fits(u64::from(width), u64::from(height)),
            "scale {scale}: the tile {width}x{height} passes the limit of the stack"
        );
        {
            let current = opened.source.borrow();
            let view = current.shaped.as_ref().expect("a shown frame");
            assert_eq!((view.width, view.height), (width, height));
            let offset = -window.get_scroll_y();
            let top = view.origin;
            let bottom = view.origin + view.height as f32 / view.viewport.scale;
            assert!(
                top <= offset + 0.5 && offset + window.get_viewport_height() <= bottom + 0.5,
                "scale {scale}: the tile [{top}, {bottom}] does not hold the view at {offset}"
            );
            println!(
                "TILE scale {scale}: overscan {:.0} px above and {:.0} px below the view",
                offset - top,
                bottom - offset - window.get_viewport_height()
            );
        }
        let mut inked = false;
        for pixel in shown(window, &opened.source) {
            if pixel[3] > 0 {
                inked = true;
                break;
            }
        }
        assert!(inked, "scale {scale}: the text area is blank");
        window.hide().expect("close wrap window");
    }
}
