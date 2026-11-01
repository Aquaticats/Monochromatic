//! Rendered tests of the track menu: the ring on a menu row (D117) and the card's own color and boundary (D119).
//!
//! What: the real window opens the menu with the Menu key on the software renderer, draws it with a row highlighted and without,
//! and reads the pixels against the row's box from the element tree (a popup's elements report positions relative to the popup,
//! so the card's corner is added); a second test reads the card's color and edge.
//! Why: a menu row is as wide as its card, so the ring is outside the card's edge, and neither the popup nor the card may clip
//! it; and the card is told apart from the page by its outline, not by a fill, so the outline and the color are facts of the
//! pixels.

use slint::ComponentHandle;

use crate::ui_focus_ring_tests::{Case, case, check_pair, roles_of, tab_to};
use crate::ui_render_support::{Frame, Rendered};
use crate::ui_test_support::tap;

/// What: `open_menu` opens the menu of the ringed track row with the Menu key and waits for the card to finish fading in.
fn open_menu(rendered: &Rendered) {
    tab_to(rendered, "track-list");
    tap(&rendered.app, "Menu");
    rendered.wait(400);
    assert!(rendered.app.get_menu_open(), "the Menu key opened the menu");
}

/// What: `frames_with_ring` draws the window with the menu ring on a row and with no row highlighted.
fn frames_with_ring(rendered: &Rendered, row: i32) -> (Frame, Frame) {
    rendered.app.set_menu_ring(row);
    rendered.wait(150);
    let on = rendered.frame();
    rendered.app.set_menu_ring(-1);
    rendered.wait(150);
    let off = rendered.frame();
    return (on, off);
}

#[test]
fn a_menu_row_ring_hugs_the_row_with_2_px_corners_and_reaches_past_the_card_edge_unclipped() {
    let rendered = Rendered::new(1200, 900);
    open_menu(&rendered);
    let origin = (rendered.app.get_menu_card_x(), rendered.app.get_menu_card_y());
    for dark in [false, true] {
        rendered.set_dark(dark);
        for row in 0..6_usize {
            let (on, off) = frames_with_ring(&rendered, i32::try_from(row).unwrap_or(0));
            let menu_row = Case { origin, nth: Some(row), ..case("menu", "TrackMenu::row-card", (0.0, 0.0), 0.0) };
            check_pair(&rendered, &menu_row, &on, &off, if dark { "dark" } else { "light" });
        }
    }
}

#[test]
fn the_menu_card_is_the_pure_canvas_color_with_a_one_pixel_outline_variant_edge() {
    let rendered = Rendered::new(1200, 900);
    open_menu(&rendered);
    let (card_x, card_y) = (rendered.app.get_menu_card_x(), rendered.app.get_menu_card_y());
    let width = rendered.app.get_menu_card_width();
    let height = rendered.app.get_menu_card_height();
    for (dark, canvas) in [(false, [0xFF_u8; 3]), (true, [0x00_u8; 3])] {
        rendered.set_dark(dark);
        rendered.wait(150);
        let frame = rendered.frame();
        let outline = {
            let color = rendered.app.global::<crate::Theme>().get_outline_variant();
            [color.red(), color.green(), color.blue()]
        };
        // The heading's end is empty card: no text reaches the last 16 px of a row.
        let (probe_x, probe_y) = ((card_x + width - 8.0) as usize, (card_y + 12.0) as usize);
        assert_eq!(frame.at(probe_x, probe_y), canvas, "the card is exactly the canvas color (dark: {dark})");
        // The left edge, in the middle of the card, is one column of the outline-variant role.
        let middle = (card_y + height / 2.0) as usize;
        let columns: Vec<usize> = (card_x as usize - 2..card_x as usize + 3).filter(|x| return frame.at(*x, middle) == outline).collect();
        assert_eq!(columns.len(), 1, "one column of the edge is the outline-variant role {outline:?} (dark: {dark}), found at {columns:?}");
        let (primary, _on_primary) = roles_of(&rendered);
        assert_ne!(outline, primary);
    }
}

#[test]
fn a_menu_opened_by_the_pointer_shows_no_ring_and_a_key_then_shows_it() {
    use slint::LogicalPosition;
    use slint::platform::{PointerEventButton, WindowEvent};
    let rendered = Rendered::new(1200, 900);
    // A right click on the first track row of the right pane opens the menu with no row highlighted: `:focus-visible` applies
    // to keyboard focus only.
    let position = LogicalPosition::new(700.0, 80.0);
    rendered.app.window().dispatch_event(WindowEvent::PointerMoved { position });
    rendered.app.window().dispatch_event(WindowEvent::PointerPressed { position, button: PointerEventButton::Right });
    rendered.app.window().dispatch_event(WindowEvent::PointerReleased { position, button: PointerEventButton::Right });
    rendered.wait(400);
    assert!(rendered.app.get_menu_open(), "a right click opened the menu");
    assert_eq!(rendered.app.get_menu_ring(), -1, "no row is highlighted after a pointer open");
    tap(&rendered.app, "Down");
    assert_eq!(rendered.app.get_menu_ring(), 0, "a key then highlights the first row");
}
