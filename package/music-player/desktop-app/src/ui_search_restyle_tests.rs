//! Rendered tests of the Search page restyle (D126): the canvas header, the outline-variant divider, the rounded result row with its
//! tonal icon circle, the keyboard ring just inside the row's shape, the one status block of the five empty and no-result states,
//! and what the restyle leaves alone (highlight corners, the D58 columns, the result copy).
//!
//! What: each test builds the real `AppWindow` on Slint's software renderer, opens Search with Ctrl+F, reads element boxes from the
//! element tree and probes pixels of the drawn frame against the theme roles of the window's scheme, in light and in dark.
//! Why: the restyle is pure markup (`ui/search.slint`), so only the composed window can show a header that is still the old band, a
//! divider in the wrong role, a row ring that is a square, or a status block that sits where it did before.
//! Note: element lookup by id needs `SLINT_EMIT_DEBUG_INFO=1` at build time, which the repo test task sets.

use i_slint_backend_testing::{AccessibleRole, ElementHandle};
use music_player::search_results::SearchState;
use slint::platform::WindowEvent;
use slint::{ComponentHandle, LogicalPosition, ModelRc, VecModel};

use crate::ui_render_support::{Frame, Rendered, box_of};
use crate::ui_search_model::search_view_of;
use crate::ui_test_support::chord;

/// Header height of the Search page, in logical pixels.
const HEADER_HEIGHT: f32 = 72.0;
/// Top of the body: the header and its 1 px divider.
const BODY_TOP: f32 = 73.0;
/// Corner radius of a result row's shape.
const ROW_RADIUS: f32 = 21.3;
/// Distance from the page edge to the content on both layouts (the caller never sets another inset).
const CONTENT_START_X: f32 = 16.0;

/// What: `Colors` holds the roles of one scheme that these tests compare pixels with, as `[red, green, blue]` each.
struct Colors {
    canvas: [u8; 3],
    surface_high: [u8; 3],
    outline: [u8; 3],
    outline_variant: [u8; 3],
    secondary_container: [u8; 3],
    on_secondary_container: [u8; 3],
    primary: [u8; 3],
    on_primary: [u8; 3],
    on_surface: [u8; 3],
    match_fill: [u8; 3],
}

/// What: `rgb` turns a Slint color into `[red, green, blue]`.
fn rgb(color: slint::Color) -> [u8; 3] {
    return [color.red(), color.green(), color.blue()];
}

/// What: `colors_of` reads the roles of the window's current scheme from its `Theme` global.
fn colors_of(rendered: &Rendered) -> Colors {
    let theme = rendered.app.global::<crate::Theme>();
    return Colors {
        canvas: rgb(theme.get_canvas()),
        surface_high: rgb(theme.get_surface_high()),
        outline: rgb(theme.get_outline()),
        outline_variant: rgb(theme.get_outline_variant()),
        secondary_container: rgb(theme.get_secondary_container()),
        on_secondary_container: rgb(theme.get_on_secondary_container()),
        primary: rgb(theme.get_primary()),
        on_primary: rgb(theme.get_on_primary()),
        on_surface: rgb(theme.get_on_surface()),
        match_fill: rgb(theme.get_match_fill()),
    };
}

/// What: `distance` is the sum of the channel differences of two colors.
fn distance(a: [u8; 3], b: [u8; 3]) -> i32 {
    return a.iter().zip(b.iter()).map(|(left, right)| return (i32::from(*left) - i32::from(*right)).abs()).sum();
}

/// What: `segment` builds one plain or emphasized piece of a result line.
fn segment(text: &str, emphasized: bool) -> crate::SearchSegment {
    return crate::SearchSegment { text: text.into(), emphasized };
}

/// What: `result_rows` builds `count` track rows whose title starts with an emphasized `Lan` and whose second line is a folder.
fn result_rows(count: i32) -> Vec<crate::SearchRow> {
    return (0..count)
        .map(|index| {
            return crate::SearchRow {
                kind: if index % 2 == 0 { 1 } else { 0 },
                title_segments: ModelRc::new(VecModel::from(vec![segment("Lan", true), segment(&format!("tern {index}"), false)])),
                parent_segments: ModelRc::new(VecModel::from(vec![segment("Aster Vale", false)])),
                has_parent: true,
                current: false,
                playing: false,
                spoken: format!("Lantern {index}, {}, Aster Vale", if index % 2 == 0 { "Track" } else { "Folder" }).into(),
                index,
            };
        })
        .collect();
}

/// What: `open_search` opens the Search page with Ctrl+F and waits for its fade to end.
fn open_search(rendered: &Rendered) {
    chord(&rendered.app, &["Ctrl"], "f");
    rendered.wait(300);
    assert_eq!(rendered.app.get_route(), 1, "Ctrl+F opened Search");
}

/// What: `search_with_rows` builds a window of a size, gives it result rows and opens Search.
fn search_with_rows(width: usize, height: usize, count: i32) -> Rendered {
    let rendered = Rendered::new(width, height);
    rendered.app.set_search_rows(ModelRc::new(VecModel::from(result_rows(count))));
    open_search(&rendered);
    return rendered;
}

/// What: `boxes` lists the boxes of the elements with an id that have a size, in tree order.
fn boxes(rendered: &Rendered, id: &str) -> Vec<(f32, f32, f32, f32)> {
    return ElementHandle::find_by_element_id(&rendered.app, id)
        .map(|element| return box_of(&element))
        .filter(|(_x, _y, width, _height)| return *width > 0.0)
        .collect();
}

/// What: `one_box` is the box of the only drawn element with an id; more or fewer is a failure.
fn one_box(rendered: &Rendered, id: &str) -> (f32, f32, f32, f32) {
    let found = boxes(rendered, id);
    assert_eq!(found.len(), 1, "exactly one drawn {id}: {found:?}");
    return found[0];
}

/// What: `page_box` is the box of the Search page.
fn page_box(rendered: &Rendered) -> (f32, f32, f32, f32) {
    let found: Vec<(f32, f32, f32, f32)> = ElementHandle::find_by_element_type_name(&rendered.app, "SearchPage")
        .map(|element| return box_of(&element))
        .filter(|(_x, _y, width, _height)| return *width > 0.0)
        .collect();
    assert_eq!(found.len(), 1, "one Search page is drawn: {found:?}");
    return found[0];
}

/// What: `pixel` reads the frame at a position given in logical pixels, flooring both coordinates.
fn pixel(frame: &Frame, x: f32, y: f32) -> [u8; 3] {
    return frame.at(x.floor() as usize, y.floor() as usize);
}

/// What: `in_both_schemes` runs a check once in the light scheme and once in the dark one, with the roles of each.
fn in_both_schemes(rendered: &Rendered, check: impl Fn(&Colors, &str)) {
    for dark in [false, true] {
        rendered.set_dark(dark);
        rendered.wait(150);
        check(&colors_of(rendered), if dark { "dark" } else { "light" });
    }
    rendered.set_dark(false);
}

// ---- Header and divider ----

/// What: `check_header_and_divider` probes the header band and the divider of a window of one size in both schemes.
fn check_header_and_divider(width: usize) {
    let rendered = search_with_rows(width, 900, 4);
    let (page_x, _page_y, page_width, _page_height) = page_box(&rendered);
    // A point of the header away from Back, the text, the caret and Clear.
    let probe_x = page_x + page_width * 0.6;
    let divider = one_box(&rendered, "SearchPage::divider");
    assert_eq!((divider.1, divider.3), (HEADER_HEIGHT, 1.0), "the divider is 1 px high directly under the 72 px header: {divider:?}");
    assert_eq!((divider.0, divider.2), (page_x, page_width), "the divider spans the page: {divider:?}");
    in_both_schemes(&rendered, |colors, scheme| {
        let frame = rendered.frame();
        for y in [2.0, 36.0, 70.0, 71.0] {
            assert_eq!(pixel(&frame, probe_x, y), colors.canvas, "{scheme} {width}: the header at y {y} is the canvas");
        }
        assert_ne!(colors.canvas, colors.surface_high, "{scheme}: the old band color differs from the canvas, so the probe can tell");
        assert_eq!(pixel(&frame, probe_x, HEADER_HEIGHT + 0.5), colors.outline_variant, "{scheme} {width}: the divider is outline-variant");
        assert_ne!(colors.outline_variant, colors.outline, "{scheme}: the old divider color differs, so the probe can tell");
        assert_eq!(pixel(&frame, probe_x, HEADER_HEIGHT + 1.5), colors.canvas, "{scheme} {width}: the divider is one pixel high");
        assert_eq!(pixel(&frame, page_x + 1.0, HEADER_HEIGHT + 0.5), colors.outline_variant, "{scheme} {width}: it starts at the page's left edge");
        assert_eq!(pixel(&frame, page_x + page_width - 1.0, HEADER_HEIGHT + 0.5), colors.outline_variant, "{scheme} {width}: it ends at the page's right edge");
    });
}

#[test]
fn the_header_is_the_canvas_and_the_divider_is_one_outline_variant_pixel_on_the_inner_panel() {
    check_header_and_divider(1200);
}

#[test]
fn the_header_is_the_canvas_and_the_divider_is_one_outline_variant_pixel_on_the_cover() {
    check_header_and_divider(443);
}

// ---- Result row: shape, circle and glyph ----

/// What: `check_row_geometry` checks the shape and the circle of the first row of a window of one size.
fn check_row_geometry(width: usize) {
    let rendered = search_with_rows(width, 900, 4);
    let (page_x, _page_y, page_width, _page_height) = page_box(&rendered);
    let shapes = boxes(&rendered, "SearchPage::result-shape");
    assert_eq!(shapes.len(), 4, "four rows draw four shapes: {shapes:?}");
    let shape = shapes[0];
    assert_eq!(shape.0, page_x + CONTENT_START_X - 8.0, "the shape starts 8 before the content");
    assert_eq!(shape.0 + shape.2, page_x + page_width - 8.0, "the shape ends 8 after the end inset, where the scrollbar track starts");
    assert_eq!((shape.1, shape.3), (BODY_TOP, 72.0), "the first shape is a 72 px row under the divider: {shape:?}");
    assert_eq!(shapes[1].1, BODY_TOP + 72.0, "rows follow each other with no gap");
    let circle = boxes(&rendered, "SearchPage::row-circle")[0];
    assert_eq!((circle.2, circle.3), (40.0, 40.0), "the icon circle is 40 by 40: {circle:?}");
    assert_eq!(circle.0, page_x + CONTENT_START_X + 4.0, "the circle sits 4 inside its 48 px slot");
    assert_eq!(circle.1, shape.1 + 16.0, "the circle is centered in the 72 px row");
    // D58: the Back target, the circle and the glyph share one center x, and the title starts where the query text does.
    let back = boxes(&rendered, "SearchPage::back-touch")[0];
    let glyph = boxes(&rendered, "SearchPage::row-glyph")[0];
    assert_eq!((glyph.2, glyph.3), (24.0, 24.0), "the glyph is drawn at 24: {glyph:?}");
    assert_eq!(circle.0 + 20.0, back.0 + back.2 / 2.0, "the circle's center is the Back target's center");
    assert_eq!(glyph.0 + 12.0, back.0 + back.2 / 2.0, "the glyph's center is the Back target's center");
    assert_eq!(glyph.1 + 12.0, circle.1 + 20.0, "the glyph is centered in the circle vertically");
    let field = one_box(&rendered, "SearchPage::field");
    let titles = boxes(&rendered, "SearchPage::title-text");
    assert_eq!(titles[0].0, field.0, "the title starts where the query text starts");
    assert_eq!(titles[0].0, circle.0 - 4.0 + 48.0, "the title starts 48 after the content start, with no extra gap");
}

#[test]
fn a_result_row_is_a_flat_rounded_shape_with_a_40_px_circle_in_its_48_px_slot_on_the_inner_panel() {
    check_row_geometry(1200);
}

#[test]
fn a_result_row_is_a_flat_rounded_shape_with_a_40_px_circle_in_its_48_px_slot_on_the_cover() {
    check_row_geometry(443);
}

#[test]
fn the_row_circle_is_secondary_container_with_an_on_secondary_container_glyph_and_the_row_is_flat_on_the_canvas() {
    let rendered = search_with_rows(443, 900, 4);
    let circle = boxes(&rendered, "SearchPage::row-circle")[0];
    let shape = boxes(&rendered, "SearchPage::result-shape")[0];
    in_both_schemes(&rendered, |colors, scheme| {
        let frame = rendered.frame();
        // The left part of the circle, away from the 24 px glyph.
        assert_eq!(pixel(&frame, circle.0 + 3.0, circle.1 + 20.0), colors.secondary_container, "{scheme}: the circle's fill");
        // The corner of the circle's box is outside a circle.
        assert_eq!(pixel(&frame, circle.0 + 1.0, circle.1 + 1.0), colors.canvas, "{scheme}: the circle is round, not a square");
        assert_eq!(pixel(&frame, circle.0 - 2.0, circle.1 + 20.0), colors.canvas, "{scheme}: nothing outside the circle");
        let glyph_pixels = (0..40)
            .flat_map(|dy| return (0..40).map(move |dx| return (dx, dy)))
            .filter(|(dx, dy)| return distance(pixel(&frame, circle.0 + *dx as f32, circle.1 + *dy as f32), colors.on_secondary_container) <= 12)
            .count();
        assert!(glyph_pixels > 20, "{scheme}: the glyph is drawn in on-secondary-container, {glyph_pixels} pixels");
        // The row is flat on the canvas at rest: the shape's middle, between the title and the end, is the canvas.
        assert_eq!(pixel(&frame, shape.0 + shape.2 - 40.0, shape.1 + 36.0), colors.canvas, "{scheme}: the row has no fill at rest");
        assert_eq!(pixel(&frame, shape.0 - 4.0, shape.1 + 36.0), colors.canvas, "{scheme}: nothing in the margin before the shape");
    });
}

// ---- Row states: ring and hover ----

#[test]
fn the_keyboard_ring_follows_the_rounded_corners_just_inside_the_shape() {
    let rendered = search_with_rows(443, 900, 4);
    rendered.app.set_search_ring(1);
    rendered.wait(100);
    let shape = boxes(&rendered, "SearchPage::result-shape")[1];
    let (left, top, right, bottom) = (shape.0, shape.1, shape.0 + shape.2, shape.1 + shape.3);
    let (middle_x, middle_y) = ((left + right) / 2.0, (top + bottom) / 2.0);
    for dark in [false, true] {
        rendered.set_dark(dark);
        let colors = colors_of(&rendered);
        let (on, off) = rendered.ring_frames();
        let scheme = if dark { "dark" } else { "light" };
        // The near ring is the 2 px band inside the 1 px outer ring (the outer ring equals the canvas in light, so it proves
        // nothing there): the two rows of pixels under the shape's top edge are `primary` with the ring and the canvas without.
        for y in [top + 1.5, top + 2.5, bottom - 2.5, bottom - 1.5] {
            assert_eq!(pixel(&on, middle_x, y), colors.primary, "{scheme}: the near ring at y {y}");
            assert_eq!(pixel(&off, middle_x, y), colors.canvas, "{scheme}: no ring before the keyboard shows it, at y {y}");
        }
        for x in [left + 1.5, left + 2.5, right - 2.5, right - 1.5] {
            assert_eq!(pixel(&on, x, middle_y), colors.primary, "{scheme}: the near ring at x {x}");
        }
        assert_eq!(pixel(&on, middle_x, top + 0.5), colors.on_primary, "{scheme}: the outer ring is on the shape's own edge");
        // Inside the ring, the row is as it was.
        assert_eq!(pixel(&on, left + 4.5, middle_y), pixel(&off, left + 4.5, middle_y), "{scheme}: no fill or tint inside the ring");
        // Nothing outside the shape: the margin on both sides stays the canvas.
        assert_eq!(pixel(&on, left - 1.5, middle_y), colors.canvas, "{scheme}: nothing before the shape");
        assert_eq!(pixel(&on, right + 6.5, middle_y), colors.canvas, "{scheme}: nothing after the shape (past the scrollbar's 4 px track)");
        // The ring follows the 21.3 px corners: a square ring would paint the corner pixel and leave the arc's 45 degree point empty.
        assert_eq!(pixel(&on, left + 1.5, top + 1.5), colors.canvas, "{scheme}: the corner pixel is outside the rounded shape");
        let on_arc = pixel(&on, left + 7.5, top + 7.5);
        assert!(distance(on_arc, colors.primary) <= 24, "{scheme}: the near ring passes the arc's 45 degree point, {on_arc:?}");
        let radius_at_arc = (ROW_RADIUS - 7.5) * 2.0_f32.sqrt();
        assert!((18.3..=20.3).contains(&radius_at_arc), "the probe lies in the near ring's band of radii, {radius_at_arc}");
    }
    rendered.set_dark(false);
}

#[test]
fn the_hover_layer_is_clipped_to_the_rounded_shape() {
    let rendered = search_with_rows(443, 900, 4);
    let shape = boxes(&rendered, "SearchPage::result-shape")[0];
    let (left, top, right) = (shape.0, shape.1, shape.0 + shape.2);
    let rest = rendered.frame();
    let position = LogicalPosition::new(right - 60.0, top + 36.0);
    rendered.app.window().dispatch_event(WindowEvent::PointerMoved { position });
    rendered.wait(250);
    let hovered = rendered.frame();
    let colors = colors_of(&rendered);
    assert_eq!(pixel(&rest, right - 60.0, top + 36.0), colors.canvas, "the row is the canvas at rest");
    assert_ne!(pixel(&hovered, right - 60.0, top + 36.0), colors.canvas, "the hover layer tints the shape");
    assert_ne!(pixel(&hovered, left + 4.5, top + 36.0), colors.canvas, "the layer reaches the shape's left edge on the straight part");
    assert_eq!(pixel(&hovered, left + 1.5, top + 1.5), colors.canvas, "the layer does not reach the corner pixel outside the arc");
    assert_eq!(pixel(&hovered, left + 3.5, top + 3.5), colors.canvas, "the layer stays inside the arc");
    assert_eq!(pixel(&hovered, left - 2.5, top + 36.0), colors.canvas, "the layer stays out of the margin before the shape");
    assert_eq!(pixel(&hovered, right + 6.5, top + 36.0), colors.canvas, "the layer stays out of the margin after the shape");
    assert_eq!(pixel(&hovered, right - 60.0, top + 72.0 + 36.0), colors.canvas, "the next row stays at rest");
}

// ---- Status block ----

/// What: `StatusCase` is one of the five states of the empty and no-result page and what it shows.
struct StatusCase {
    name: &'static str,
    state: SearchState,
    complete: bool,
    has_detail: bool,
}

/// What: `status_cases` lists the five states of the empty and no-result page, each from the Rust view builder.
fn status_cases() -> Vec<StatusCase> {
    return vec![
        StatusCase { name: "empty query", state: SearchState::EmptyQuery, complete: true, has_detail: true },
        StatusCase { name: "no match", state: SearchState::NoMatch("zzzz".to_string()), complete: true, has_detail: true },
        StatusCase { name: "unavailable", state: SearchState::Unavailable, complete: true, has_detail: true },
        StatusCase { name: "empty inventory", state: SearchState::EmptyInventory, complete: true, has_detail: false },
        StatusCase { name: "still scanning", state: SearchState::NoMatch("zzzz".to_string()), complete: false, has_detail: true },
    ];
}

/// What: `show_status` gives an open window the copy that the view builder makes for a state, and waits for it to draw.
fn show_status(rendered: &Rendered, case: &StatusCase) {
    let view = search_view_of(&case.state, None, false, case.complete);
    assert!(view.rows.is_empty(), "{}: a status state has no rows", case.name);
    rendered.app.set_search_status_title(view.status_title.as_str().into());
    rendered.app.set_search_status_detail(view.status_detail.as_str().into());
    rendered.wait(50);
}

/// What: `status_window` builds a window of a size and opens Search on it with the empty-query copy.
/// Why: a thread installs one platform, so a test builds one window and shows its states one after another.
fn status_window(width: usize, height: usize) -> Rendered {
    let rendered = Rendered::new(width, height);
    open_search(&rendered);
    return rendered;
}

/// What: `check_status_block` checks the geometry of every state's block on a window of one size.
fn check_status_block(width: usize, compact: bool) {
    let rendered = status_window(width, 900);
    for case in status_cases() {
        show_status(&rendered, &case);
        let (page_x, _page_y, page_width, _page_height) = page_box(&rendered);
        assert_eq!(page_x > 0.0, !compact, "{}: the window is the {} layout", case.name, if compact { "cover" } else { "inner" });
        let circle = one_box(&rendered, "SearchPage::status-circle-box");
        let glyph = one_box(&rendered, "SearchPage::status-glyph-path");
        let title = one_box(&rendered, "SearchPage::status-title-text");
        assert_eq!((circle.2, circle.3), (72.0, 72.0), "{}: the circle is 72 by 72", case.name);
        assert_eq!(circle.1, BODY_TOP + 72.0, "{}: the block starts 72 under the divider's bottom edge", case.name);
        let expected_x = if compact { page_x + (page_width - 72.0) / 2.0 } else { page_x + CONTENT_START_X };
        assert_eq!(circle.0, expected_x, "{}: the circle is centered on the cover and at the content start on the inner panel", case.name);
        assert_eq!((glyph.2, glyph.3), (32.0, 32.0), "{}: the magnifier is 32 by 32", case.name);
        assert_eq!((glyph.0 + 16.0, glyph.1 + 16.0), (circle.0 + 36.0, circle.1 + 36.0), "{}: the magnifier is centered in the circle", case.name);
        assert_eq!(title.1, circle.1 + 72.0 + 24.0, "{}: the title is 24 under the circle", case.name);
        assert_eq!(title.3, 24.0, "{}: the title line is 24 high", case.name);
        assert_eq!(title.0, page_x + 16.0, "{}: the title starts at the block's left edge", case.name);
        assert_eq!(title.2, page_width - 32.0, "{}: the title is as wide as the block", case.name);
        let details = boxes(&rendered, "SearchPage::status-detail-text");
        assert_eq!(details.len(), usize::from(case.has_detail), "{}: the support line is drawn only when the state has one", case.name);
        if case.has_detail {
            let gap = if compact { 8.0 } else { 12.0 };
            assert_eq!(details[0].1, title.1 + 24.0 + gap, "{}: the support line is {gap} under the title", case.name);
            assert_eq!(details[0].3, 20.0, "{}: the support line is 20 high", case.name);
        }
    }
}

#[test]
fn the_five_status_states_share_one_block_of_a_72_px_circle_a_title_and_a_support_line_on_the_cover() {
    check_status_block(443, true);
}

#[test]
fn the_five_status_states_share_one_block_of_a_72_px_circle_a_title_and_a_support_line_on_the_inner_panel() {
    check_status_block(1200, false);
}

#[test]
fn the_status_circle_is_secondary_container_with_an_on_secondary_container_magnifier() {
    let rendered = status_window(443, 900);
    show_status(&rendered, &status_cases()[0]);
    let circle = one_box(&rendered, "SearchPage::status-circle-box");
    in_both_schemes(&rendered, |colors, scheme| {
        let frame = rendered.frame();
        assert_eq!(pixel(&frame, circle.0 + 3.0, circle.1 + 36.0), colors.secondary_container, "{scheme}: the circle's fill");
        assert_eq!(pixel(&frame, circle.0 + 1.0, circle.1 + 1.0), colors.canvas, "{scheme}: the circle is round");
        let glyph_pixels = (0..72)
            .flat_map(|dy| return (0..72).map(move |dx| return (dx, dy)))
            .filter(|(dx, dy)| return distance(pixel(&frame, circle.0 + *dx as f32, circle.1 + *dy as f32), colors.on_secondary_container) <= 12)
            .count();
        assert!(glyph_pixels > 40, "{scheme}: the magnifier is drawn in on-secondary-container, {glyph_pixels} pixels");
        let title = one_box(&rendered, "SearchPage::status-title-text");
        let ink = (0..title.2 as usize)
            .flat_map(|dx| return (0..title.3 as usize).map(move |dy| return (dx, dy)))
            .filter(|(dx, dy)| return distance(pixel(&frame, title.0 + *dx as f32, title.1 + *dy as f32), colors.on_surface) <= 12)
            .count();
        assert!(ink > 20, "{scheme}: the title is drawn in on-surface, {ink} pixels");
    });
}

/// What: `title_after_wheel` shows the empty-query state in a window of one height, turns the wheel over the block and gives the
/// title's y before and after.
fn title_after_wheel(height: usize) -> (f32, f32) {
    let rendered = status_window(443, height);
    show_status(&rendered, &status_cases()[0]);
    let before = one_box(&rendered, "SearchPage::status-title-text");
    let position = LogicalPosition::new(200.0, 150.0);
    rendered.app.window().dispatch_event(WindowEvent::PointerMoved { position });
    rendered.app.window().dispatch_event(WindowEvent::PointerScrolled { position, delta_x: 0.0, delta_y: -200.0 });
    rendered.wait(400);
    let after = one_box(&rendered, "SearchPage::status-title-text");
    return (before.1, after.1);
}

#[test]
fn the_status_block_scrolls_when_the_window_is_shorter_than_the_block() {
    // The block ends 220 px under the divider (at y 293), and the title (y 241 to 265) is still on screen in a 270 px window, so
    // the wheel can scroll by the 23 px of block that is below the window.
    let (before, after) = title_after_wheel(270);
    assert!((after - (before - 23.0)).abs() < 1.0, "the wheel moves the block up by what hangs below the window: {before} to {after}");
}

#[test]
fn the_status_block_stays_put_when_it_fits() {
    let (before, after) = title_after_wheel(900);
    assert_eq!(after, before, "a block that fits does not move");
}

// ---- What the restyle leaves alone ----

#[test]
fn the_match_highlight_keeps_square_corners_and_the_result_copy_is_unchanged() {
    let rendered = search_with_rows(443, 900, 2);
    let titles = boxes(&rendered, "SearchPage::title-text");
    // The first two pieces of the first row are the emphasized `Lan` and the plain rest.
    let highlight = titles[0];
    in_both_schemes(&rendered, |colors, scheme| {
        let frame = rendered.frame();
        let (left, top) = (highlight.0.round(), highlight.1.round());
        let (right, bottom) = ((highlight.0 + highlight.2).round() - 1.0, (highlight.1 + highlight.3).round() - 1.0);
        for (x, y) in [(left, top), (right, top), (left, bottom), (right, bottom)] {
            assert_eq!(pixel(&frame, x, y), colors.match_fill, "{scheme}: the highlight fills its corner pixel at {x},{y}, so it is square");
        }
    });
    let labels: Vec<String> = ElementHandle::find_by_element_id(&rendered.app, "SearchPage::title-text")
        .filter(|element| return box_of(element).2 > 0.0)
        .filter_map(|element| return element.accessible_label().map(|label| return label.to_string()))
        .collect();
    assert_eq!(labels, vec!["Lan", "tern 0", "Lan", "tern 1"], "the result copy is the pieces the model made, in order");
    let parents: Vec<String> = ElementHandle::find_by_element_id(&rendered.app, "SearchPage::parent-text")
        .filter(|element| return box_of(element).2 > 0.0)
        .filter_map(|element| return element.accessible_label().map(|label| return label.to_string()))
        .collect();
    assert_eq!(parents, vec!["Aster Vale", "Aster Vale"], "the second line is unchanged");
}

// ---- Each result row is one named button (D77) ----

/// What: `row_taps` lists the tap targets of the result rows that are on screen, in tree order.
fn row_taps(rendered: &Rendered) -> Vec<ElementHandle> {
    return ElementHandle::find_by_element_id(&rendered.app, "SearchPage::row-touch")
        .filter(|element| return box_of(element).2 > 0.0)
        .collect();
}

#[test]
fn every_result_row_is_a_named_button_that_opens_its_result() {
    let rendered = Rendered::new(443, 900);
    // Row 0 is a track and the current one; row 1 is a folder.
    let rows: Vec<crate::SearchRow> = result_rows(2)
        .into_iter()
        .map(|row| return crate::SearchRow { current: row.index == 0, ..row })
        .collect();
    rendered.app.set_search_rows(ModelRc::new(VecModel::from(rows)));
    open_search(&rendered);
    let opened = std::rc::Rc::new(std::cell::RefCell::new(Vec::<i32>::new()));
    let sink = std::rc::Rc::clone(&opened);
    rendered.app.on_search_open(move |index| sink.borrow_mut().push(index));
    let taps = row_taps(&rendered);
    assert_eq!(taps.len(), 2, "one tap target per result row");
    assert!(
        taps.iter().all(|tap| return tap.accessible_role() == Some(AccessibleRole::Button)),
        "each result row is exposed as a button"
    );
    let labels: Vec<String> = taps.iter().map(|tap| return tap.accessible_label().unwrap_or_default().to_string()).collect();
    assert_eq!(labels, vec!["Lantern 0, Track, Aster Vale", "Lantern 1, Folder, Aster Vale"], "name, kind and parent folder");
    assert_eq!(taps[0].accessible_checkable(), Some(true), "a track can be the current one");
    assert_eq!(taps[0].accessible_checked(), Some(true), "the current track reads as checked");
    assert_ne!(taps[1].accessible_checkable(), Some(true), "a folder is never the current track");
    for tap in &taps {
        tap.invoke_accessible_default_action();
    }
    assert_eq!(*opened.borrow(), vec![0, 1], "the default action of each row opens that row's result");
}

#[test]
fn a_result_row_adds_no_second_label_for_its_title_pieces() {
    let rendered = search_with_rows(443, 900, 2);
    let taps = row_taps(&rendered);
    let titled: Vec<String> = taps.iter().filter_map(|tap| return tap.accessible_label()).map(|label| return label.to_string()).collect();
    assert_eq!(titled.len(), 2, "only the row buttons carry the spoken label");
    assert!(titled.iter().all(|label| return label.matches("Lantern").count() == 1), "the name is spoken once: {titled:?}");
}
