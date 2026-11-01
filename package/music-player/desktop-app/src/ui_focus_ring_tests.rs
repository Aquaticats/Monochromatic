//! Rendered tests of the focus ring on the player page (D117): where the ring is, what it hugs, and what it leaves alone.
//!
//! What: each test draws the real window on the software renderer twice with the focus where it is, once with the keyboard ring
//! on and once with it off, then reads the pixels against the element's box from the element tree. The ring must be exactly
//! two bands flush outside the element: the 2 px band next to it in the `primary` role and the 1 px band around that in the
//! `on-primary` role, each edge being the element grown by the widths inside it (with each corner's radius grown the same, a
//! square corner becoming a 2 px one). Every pixel outside the bands must be identical in the two frames, which includes every
//! pixel inside the element, so focus adds no fill, tint or size change.
//! Why: D117 describes the ring by its pixels (two solid rings, flush, hugging, not clipped, no other change), so a test that
//! reads pixels is the only one that sees a ring that a parent clipped or a fill that came back with focus.
//! Note: element boxes come from `SLINT_EMIT_DEBUG_INFO=1` ids, which the repo test task sets.

use i_slint_backend_testing::ElementHandle;
use slint::ComponentHandle;

use crate::ui_render_support::{Frame, Rendered, RoundedBox, box_of};
use crate::ui_test_support::{focus_label, tap};

/// Counts what `check_ring` judged, so a test can see that it was not vacuous.
pub(crate) struct RingReport {
    /// Pixels judged to be in the near ring (2 px, `primary`).
    pub near: usize,
    /// Pixels judged to be in the outer ring (1 px, `on-primary`).
    pub outer: usize,
    /// Pixels judged to be outside both rings and compared with the frame without the ring.
    pub unchanged: usize,
    /// Pixels of either ring that are exactly the ring's color (the straight parts; curves are covered in part).
    pub exact: usize,
    /// Pixels the outer ring covers in part whose coverage was compared with the shape.
    pub partial: usize,
}

/// What: `roles_of` gives the `primary` and `on-primary` roles of the window's current scheme as `[red, green, blue]` each.
pub(crate) fn roles_of(rendered: &Rendered) -> ([u8; 3], [u8; 3]) {
    let theme = rendered.app.global::<crate::Theme>();
    let (primary, on_primary) = (theme.get_primary(), theme.get_on_primary());
    return ([primary.red(), primary.green(), primary.blue()], [on_primary.red(), on_primary.green(), on_primary.blue()]);
}

/// What: `color_distance` is the sum of the channel differences of two colors.
fn color_distance(a: [u8; 3], b: [u8; 3]) -> i32 {
    return a.iter().zip(b.iter()).map(|(left, right)| return (i32::from(*left) - i32::from(*right)).abs()).sum();
}

/// What: `share_toward` tells how far along the line from one color to another a pixel lies, 0 at the first and 1 at the second.
/// Why: a pixel that a curve covers in part is a blend of what was there and the ring's color, so how far along it lies is how
/// much the ring covers it.
fn share_toward(pixel: [u8; 3], from: [u8; 3], to: [u8; 3]) -> f32 {
    let (mut along, mut span) = (0.0_f32, 0.0_f32);
    for channel in 0..3 {
        let step = f32::from(to[channel]) - f32::from(from[channel]);
        along += (f32::from(pixel[channel]) - f32::from(from[channel])) * step;
        span += step * step;
    }
    return if span == 0.0 { 1.0 } else { along / span };
}

/// How many sub-pixel samples per side `check_ring` takes in each pixel to tell how much of it a ring covers.
const SAMPLES: usize = 4;

/// What: `check_ring` compares the frame with the ring and the frame without it against the element's box.
/// Why: the near band (between the element and the element grown by 2 px) must be the near color, the outer band (up to 3 px)
/// the outer color, and nothing else may differ. The renderer anti-aliases curves, so a pixel is judged only where the shapes say
/// it is wholly one thing: wholly inside the near band (exactly the near color), wholly inside the outer band (exactly the outer
/// color), or touched by neither band (identical to the frame without the ring). `inside` is the one case where the rings are on
/// the element's own edge: the outer ring is then the outermost 1 px of the box and the near ring the 2 px inside it. `far`
/// also compares every pixel away from the element, which a text field cannot offer (its ring does not turn off with the
/// keyboard ring, so the other frame has another ring elsewhere).
pub(crate) fn check_ring(on: &Frame, off: &Frame, element: RoundedBox, colors: ([u8; 3], [u8; 3]), inside: bool, options: &Case) -> Result<RingReport, String> {
    let (near_color, outer_color) = colors;
    let far = options.far;
    let (edge_element, edge_near, edge_outer) = if inside {
        (element, element.grown(-1.0), element.grown(-3.0))
    } else {
        (element, element.grown(2.0), element.grown(3.0))
    };
    // Which band a sample point is in: 1 near, 2 outer, 0 neither.
    let band_of = |px: f32, py: f32| -> u8 {
        if inside {
            if edge_element.distance(px, py) <= 0.0 && edge_near.distance(px, py) > 0.0 {
                return 2;
            }
            if edge_near.distance(px, py) <= 0.0 && edge_outer.distance(px, py) > 0.0 {
                return 1;
            }
            return 0;
        }
        if edge_element.distance(px, py) > 0.0 && edge_near.distance(px, py) <= 0.0 {
            return 1;
        }
        if edge_near.distance(px, py) > 0.0 && edge_outer.distance(px, py) <= 0.0 {
            return 2;
        }
        return 0;
    };
    let near_box = edge_element.grown(6.0);
    let (left, top) = (near_box.x.floor().max(0.0) as usize, near_box.y.floor().max(0.0) as usize);
    let (right, bottom) = (((near_box.x + near_box.width).ceil() as usize).min(on.width), ((near_box.y + near_box.height).ceil() as usize).min(on.height));
    let mut report = RingReport { near: 0, outer: 0, unchanged: 0, exact: 0, partial: 0 };
    for y in 0..on.height {
        for x in 0..on.width {
            let (cx, cy) = (x as f32 + 0.5, y as f32 + 0.5);
            if options.skip.iter().any(|(skip_left, skip_top, skip_right, skip_bottom)| return cx >= *skip_left && cx < *skip_right && cy >= *skip_top && cy < *skip_bottom) {
                continue;
            }
            if x < left || x >= right || y < top || y >= bottom {
                if far && on.at(x, y) != off.at(x, y) {
                    return Err(format!("pixel {x},{y} is far from the element and focus changed it from {:?} to {:?}", off.at(x, y), on.at(x, y)));
                }
                continue;
            }
            let (mut near_samples, mut outer_samples) = (0, 0);
            for sy in 0..SAMPLES {
                for sx in 0..SAMPLES {
                    let (px, py) = (x as f32 + (sx as f32 + 0.5) / SAMPLES as f32, y as f32 + (sy as f32 + 0.5) / SAMPLES as f32);
                    match band_of(px, py) {
                        1 => near_samples += 1,
                        2 => outer_samples += 1,
                        _ => {}
                    }
                }
            }
            let all = SAMPLES * SAMPLES;
            let pixel = on.at(x, y);
            // The renderer only approximates the coverage of an arc, so on a curve a pixel need only be nearer to its band's color
            // than to what was there before, while on a straight edge it must be exactly the band's color.
            let curved = edge_outer.near_a_curve(cx, cy, 2.0) || edge_element.near_a_curve(cx, cy, 2.0);
            let (before, tolerance) = (off.at(x, y), 12);
            if near_samples == all || outer_samples == all {
                let (band, band_color) = if near_samples == all { ("near", near_color) } else { ("outer", outer_color) };
                if near_samples == all {
                    report.near += 1;
                } else {
                    report.outer += 1;
                }
                if curved {
                    if outer_samples == all && color_distance(band_color, before) < 60 {
                        continue;
                    }
                    if share_toward(pixel, before, band_color) < 0.25 {
                        return Err(format!("pixel {x},{y} is on a curve in the {band} ring and is {pixel:?}, less than a quarter of the way from {before:?} to {band_color:?}"));
                    }
                } else {
                    if color_distance(pixel, band_color) > tolerance {
                        return Err(format!("pixel {x},{y} is wholly in the {band} ring and is {pixel:?}, not {band_color:?}"));
                    }
                    report.exact += 1;
                }
            } else if options.coverage && near_samples == 0 && outer_samples > 0 && color_distance(outer_color, before) > 60 {
                // A pixel the outer ring covers in part: how far it moved from what was there toward the ring's color is the share
                // of the pixel the ring covers. This is what tells a corner of the right radius from a sharper or a rounder one.
                let expected = outer_samples as f32 / all as f32;
                let share = share_toward(pixel, before, outer_color);
                report.partial += 1;
                if (share - expected).abs() > 0.3 {
                    return Err(format!("pixel {x},{y} is {:.0} percent covered by the outer ring but shows {:.0} percent ({pixel:?} from {before:?} toward {outer_color:?})", expected * 100.0, share * 100.0));
                }
            } else if near_samples + outer_samples == 0 && !curved {
                report.unchanged += 1;
                if pixel != before {
                    return Err(format!("pixel {x},{y} is outside the rings and focus changed it from {before:?} to {pixel:?}"));
                }
            }
        }
    }
    if report.near < 20 || report.outer < 8 {
        return Err(format!("a ring band holds too few whole pixels (near {}, outer {}): the ring is not drawn", report.near, report.outer));
    }
    return Ok(report);
}

/// What: `tab_to` presses Tab until the named element holds focus, at most sixteen times.
pub(crate) fn tab_to(rendered: &Rendered, label: &str) {
    for _ in 0..16 {
        if focus_label(&rendered.app) == label {
            return;
        }
        tap(&rendered.app, "Tab");
    }
    assert_eq!(focus_label(&rendered.app), label, "Tab never reached {label}");
}

/// What: `element_for_ring` finds the element of the tree whose box, shrunk by `inset` on each side, is the box the ring hugs.
/// Why: the window builds both layouts and a lot of rows, and a hidden copy keeps a stale box. The ring's own pixels say which
/// copy is on screen: the one whose box grown by 2 px (or exactly, for an `inside` ring) is the box of the differing pixels.
pub(crate) fn element_for_ring(rendered: &Rendered, id: &str, inset: (f32, f32), origin: (f32, f32), differing: (usize, usize, usize, usize), inside: bool) -> RoundedBox {
    // The differing pixels hold the near ring always and the outer ring only where `on-primary` differs from the surface under
    // it (it equals the canvas in the light scheme), so the element's box is either 2 px or 3 px inside the differing box.
    let reaches: [f32; 2] = if inside { [-1.0, 0.0] } else { [2.0, 3.0] };
    let wanted = (differing.0 as f32, differing.1 as f32, (differing.2 - differing.0) as f32, (differing.3 - differing.1) as f32);
    let found: Vec<(f32, f32, f32, f32)> = ElementHandle::find_by_element_id(&rendered.app, id)
        .map(|element| return box_of(&element))
        .map(|(x, y, width, height)| return (x + origin.0 + inset.0, y + origin.1 + inset.1, width - 2.0 * inset.0, height - 2.0 * inset.1))
        .filter(|(x, y, width, height)| {
            return reaches.iter().any(|reach| {
                return (x - reach - wanted.0).abs() < 0.6
                    && (y - reach - wanted.1).abs() < 0.6
                    && (width + 2.0 * reach - wanted.2).abs() < 0.6
                    && (height + 2.0 * reach - wanted.3).abs() < 0.6;
            });
        })
        .collect();
    assert!(!found.is_empty(), "no element {id} (inset {inset:?}) has a box that the ring of {wanted:?} hugs");
    let (x, y, width, height) = found[0];
    return RoundedBox::new(x, y, width, height, 0.0);
}

/// What: `Case` describes one focusable element and the shape its ring must hug.
pub(crate) struct Case<'a> {
    /// Name the focus model reports while the element holds focus.
    pub label: &'a str,
    /// Qualified id of the element in the tree whose box the ring hugs.
    pub id: &'a str,
    /// How far the shape the ring hugs is inside that element's box, as `(x, y)` on each side.
    pub inset: (f32, f32),
    /// Corner radii of the shape, top left, top right, bottom right, bottom left.
    pub radii: [f32; 4],
    /// True for an element as wide as its container, whose ring is on its own edge.
    pub inside: bool,
    /// False when the frame without the ring has another ring elsewhere (a text field keeps its ring when the keyboard ring is
    /// off), so only the pixels near the element are compared.
    pub far: bool,
    /// Boxes `(left, top, right, bottom)` whose pixels are not judged: the notch a label cuts in a ring, a text caret that blinks.
    pub skip: Vec<(f32, f32, f32, f32)>,
    /// Where the element tree's origin lies in the window: zero for the window, the card's corner for a popup (a popup's elements
    /// report positions relative to the popup).
    pub origin: (f32, f32),
    /// True to compare how much of each partly covered outer-ring pixel the ring covers with the shape. The renderer's
    /// antialiasing of long curves is not linear in coverage, so this is for the fixture's corners (small radii) only.
    pub coverage: bool,
    /// The position among the elements with this id that have a size, when the ring's own pixels cannot tell which one is meant
    /// (a text caret blinks in the same frame pair); None finds it by the ring.
    pub nth: Option<usize>,
}

/// What: `run_case` focuses an element, then in the light and in the dark scheme draws the two frames, finds the element the ring
/// hugs and checks the ring.
/// Why: one call per element keeps each test to the facts that differ, and both schemes prove the roles, not fixed colors.
pub(crate) fn run_case(rendered: &Rendered, case: &Case) -> Vec<RingReport> {
    tab_to(rendered, case.label);
    return check_focused(rendered, case);
}

/// What: `check_focused` checks the ring of the element that holds focus now, in both schemes.
pub(crate) fn check_focused(rendered: &Rendered, case: &Case) -> Vec<RingReport> {
    let mut reports = Vec::new();
    for dark in [false, true] {
        rendered.set_dark(dark);
        let (on, off) = rendered.ring_frames();
        reports.push(check_pair(rendered, case, &on, &off, if dark { "dark" } else { "light" }));
    }
    rendered.set_dark(false);
    return reports;
}

/// What: `check_pair` finds the element the ring of a frame pair hugs and checks the pair against it.
pub(crate) fn check_pair(rendered: &Rendered, case: &Case, on: &Frame, off: &Frame, scheme: &str) -> RingReport {
    let found = if let Some(nth) = case.nth {
        let (x, y, width, height) = ElementHandle::find_by_element_id(&rendered.app, case.id)
            .map(|element| return box_of(&element))
            .map(|(x, y, width, height)| return (x + case.origin.0, y + case.origin.1, width, height))
            .filter(|(_x, _y, width, _height)| return *width > 0.0)
            .nth(nth)
            .unwrap_or_else(|| panic!("{}: no element {} number {nth} has a size", case.label, case.id));
        RoundedBox::new(x + case.inset.0, y + case.inset.1, width - 2.0 * case.inset.0, height - 2.0 * case.inset.1, 0.0)
    } else {
        let differing = on.differing_box(off).unwrap_or_else(|| panic!("{} ({scheme}): the ring is not drawn: both frames are equal", case.label));
        element_for_ring(rendered, case.id, case.inset, case.origin, differing, case.inside)
    };
    let element = RoundedBox { radii: case.radii, ..found };
    let colors = roles_of(rendered);
    return check_ring(on, off, element, colors, case.inside, case).unwrap_or_else(|message| panic!("{} {} ({scheme}): {message}", case.label, case.id));
}

/// What: `case` makes the common case: a ring outside the element, all corners the same.
pub(crate) fn case<'a>(label: &'a str, id: &'a str, inset: (f32, f32), radius: f32) -> Case<'a> {
    return Case { label, id, inset, radii: [radius; 4], inside: false, far: true, skip: Vec::new(), origin: (0.0, 0.0), coverage: false, nth: None };
}

#[test]
fn the_open_pill_ring_is_a_pill_flush_outside_the_pill() {
    let rendered = Rendered::new(850, 900);
    let reports = run_case(&rendered, &case("open-pill", "OpenPill::pill", (0.0, 0.0), 20.0));
    for report in reports {
        assert!(report.near > 150 && report.outer > 60, "a pill ring has long bands, {} and {} whole pixels", report.near, report.outer);
        assert!(report.unchanged > 1000, "the pixels around it were compared, {}", report.unchanged);
    }
}

#[test]
fn the_search_and_settings_rings_are_circles_flush_outside_the_40_px_circle() {
    let rendered = Rendered::new(850, 900);
    for label in ["search", "settings"] {
        run_case(&rendered, &case(label, "IconTarget::stop", (4.0, 4.0), 20.0));
    }
}

#[test]
fn the_transport_rings_are_circles_flush_outside_the_40_px_circles() {
    let rendered = Rendered::new(850, 900);
    for (label, id) in [("previous", "TransportDeck::prev-stop"), ("play", "TransportDeck::play-stop"), ("next", "TransportDeck::next-stop")] {
        run_case(&rendered, &case(label, id, (4.0, 4.0), 20.0));
    }
}

#[test]
fn the_seek_bar_ring_is_a_pill_around_its_48_px_target() {
    let rendered = Rendered::new(850, 900);
    run_case(&rendered, &case("seek", "TransportDeck::seek-stop", (0.0, 0.0), 24.0));
}

#[test]
fn the_folder_trigger_ring_is_a_pill_flush_outside_the_40_px_pill_of_the_folded_layout() {
    let rendered = Rendered::new(443, 900);
    run_case(&rendered, &case("folder-trigger", "CoverTopRow::trigger", (0.0, 4.0), 20.0));
}

#[test]
fn the_folder_list_ring_is_a_square_cornered_box_with_2_px_corners_around_the_name() {
    let rendered = Rendered::new(850, 900);
    run_case(&rendered, &case("folder-list", "FolderPickerPane::name-row", (0.0, 0.0), 0.0));
}

#[test]
fn the_track_list_ring_is_on_the_edge_of_a_row_as_wide_as_its_pane() {
    let rendered = Rendered::new(850, 900);
    let row = Case { inside: true, ..case("track-list", "TrackListPane::item", (0.0, 0.0), 0.0) };
    run_case(&rendered, &row);
}

/// What: `mode_ring_case` checks the ring on each of the four segments of the mode group at a window width.
/// Why: the group is one row, two rows of two or four rows depending on the width, and each layout rounds different corners.
fn mode_ring_case(width: usize, corners: [[f32; 4]; 4]) {
    let rendered = Rendered::new(width, 900);
    tab_to(&rendered, "mode-group");
    for (mode, radii) in corners.into_iter().enumerate() {
        rendered.app.set_playback_mode(i32::try_from(mode).unwrap_or(0));
        // The segments ease their width over 150 ms when the selection changes.
        rendered.wait(250);
        let id = format!("TransportDeck::seg-{mode}");
        check_focused(&rendered, &Case { radii, ..case("mode-group", &id, (0.0, 0.0), 0.0) });
    }
}

#[test]
fn the_mode_ring_hugs_the_selected_segment_with_its_own_corners_in_one_row() {
    // Wide enough for the group in one row: the two ends are pills and the inside corners are square, so the ring on a middle
    // segment is a rectangle with 2 px corners.
    mode_ring_case(1200, [[24.0, 0.0, 0.0, 24.0], [0.0; 4], [0.0; 4], [0.0, 24.0, 24.0, 0.0]]);
}

#[test]
fn the_mode_ring_hugs_the_selected_segment_with_its_own_corners_in_two_rows() {
    // Two rows of two: each segment rounds only the corner that is a corner of the group.
    mode_ring_case(400, [[20.0, 0.0, 0.0, 0.0], [0.0, 20.0, 0.0, 0.0], [0.0, 0.0, 0.0, 20.0], [0.0, 0.0, 20.0, 0.0]]);
}

#[test]
fn the_mode_ring_hugs_the_selected_segment_with_its_own_corners_in_four_rows() {
    // The narrowest window stacks the four segments: the top one rounds its upper corners and the bottom one its lower corners.
    mode_ring_case(360, [[20.0, 20.0, 0.0, 0.0], [0.0; 4], [0.0; 4], [0.0, 0.0, 20.0, 20.0]]);
}
