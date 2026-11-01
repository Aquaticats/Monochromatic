//! Rendered tests of the focus ring component alone (D117), on a small fixture window that places it around a pill, a circle, a
//! rounded rectangle, a square-cornered box and a box inside a clipping parent.
//!
//! What: the fixture draws `FocusRing` from `ui/focus.slint` with the ring off and on, and the same pixel checks as the player
//! tests read the bands. The clipping parent is drawn twice: with the ring inside it, as a control that shows the clip cutting a
//! ring, and with the ring in a sibling layer above it, which must stay whole.
//! Why: the real window has few places where a parent clips a ring, so the property "a ring in a layer above the clip is never
//! cut" is proved here, where a clip is placed on purpose.

use slint::ComponentHandle;

use crate::ui_focus_ring_tests::{Case, RingReport, case, check_ring};
use crate::ui_render_support::{Frame, RoundedBox, draw_frame, install_software_platform};

slint::slint! {
    import { FocusRing } from "../ui/focus.slint";
    import { Theme } from "../ui/theme.slint";
    export { Theme }

    export component RingFixture inherits Window {
        width: 360px;
        height: 200px;
        // A mid gray backdrop, so the white outer ring contrasts with what is under it in the light scheme too.
        background: #6a6a6a;
        in property <bool> on: false;

        // A pill, 100 by 40.
        Rectangle { x: 20px; y: 20px; width: 100px; height: 40px; border-radius: 20px; background: #cccccc; }
        FocusRing { x: 20px; y: 20px; width: 100px; height: 40px; round: true; active: root.on; }
        // A circle, 40 px across.
        Rectangle { x: 150px; y: 20px; width: 40px; height: 40px; border-radius: 20px; background: #cccccc; }
        FocusRing { x: 150px; y: 20px; width: 40px; height: 40px; round: true; active: root.on; }
        // A rounded rectangle, 12 px corners.
        Rectangle { x: 20px; y: 100px; width: 100px; height: 60px; border-radius: 12px; background: #cccccc; }
        FocusRing { x: 20px; y: 100px; width: 100px; height: 60px; radius: 12px; active: root.on; }
        // A box with square corners.
        Rectangle { x: 150px; y: 100px; width: 60px; height: 40px; background: #cccccc; }
        FocusRing { x: 150px; y: 100px; width: 60px; height: 40px; active: root.on; }
        // A box at the corner of a clipping parent, with the ring in a layer above the clip.
        Rectangle {
            x: 240px; y: 10px; width: 50px; height: 50px; clip: true;
            Rectangle { width: 50px; height: 50px; background: #cccccc; }
        }
        FocusRing { x: 240px; y: 10px; width: 50px; height: 50px; active: root.on; }
        // The same box with the ring inside the clipping parent: the control, where the clip cuts the ring.
        Rectangle {
            x: 240px; y: 100px; width: 50px; height: 50px; clip: true;
            Rectangle { width: 50px; height: 50px; background: #cccccc; }
            FocusRing { width: 50px; height: 50px; active: root.on; }
        }
    }
}

/// What: `fixture_frames` draws the fixture with the ring off and on, and gives the two frames and the roles of the scheme.
fn fixture_frames() -> (Frame, Frame, ([u8; 3], [u8; 3])) {
    let window = install_software_platform();
    let fixture = RingFixture::new().expect("the fixture builds");
    fixture.global::<Theme>().set_scheme_override(1);
    fixture.window().set_size(slint::PhysicalSize::new(360, 200));
    fixture.show().expect("the fixture shows");
    let off = draw_frame(&window, 360, 200);
    fixture.set_on(true);
    let on = draw_frame(&window, 360, 200);
    let theme = fixture.global::<Theme>();
    let (primary, on_primary) = (theme.get_primary(), theme.get_on_primary());
    let colors = ([primary.red(), primary.green(), primary.blue()], [on_primary.red(), on_primary.green(), on_primary.blue()]);
    return (on, off, colors);
}

/// What: `check` checks one shape of the fixture and gives the counts.
fn check(on: &Frame, off: &Frame, colors: ([u8; 3], [u8; 3]), shape: RoundedBox) -> Result<RingReport, String> {
    let options = Case { far: false, coverage: true, ..case("fixture", "", (0.0, 0.0), 0.0) };
    return check_ring(on, off, shape, colors, false, &options);
}

#[test]
fn the_ring_component_hugs_a_pill_a_circle_a_rounded_rectangle_and_a_square_box() {
    let (on, off, colors) = fixture_frames();
    for (name, shape) in [
        ("pill", RoundedBox::new(20.0, 20.0, 100.0, 40.0, 20.0)),
        ("circle", RoundedBox::new(150.0, 20.0, 40.0, 40.0, 20.0)),
        ("rounded rectangle", RoundedBox::new(20.0, 100.0, 100.0, 60.0, 12.0)),
        ("square box", RoundedBox::new(150.0, 100.0, 60.0, 40.0, 0.0)),
    ] {
        let report = check(&on, &off, colors, shape).unwrap_or_else(|message| panic!("{name}: {message}"));
        assert!(report.near > 20 && report.outer > 8, "{name}: both bands were judged, {} and {}", report.near, report.outer);
        assert!(report.partial >= 4, "{name}: the coverage of curved or cornered pixels was judged, {} pixels", report.partial);
    }
}

#[test]
fn a_ring_in_a_layer_above_a_clipping_parent_stays_whole_and_one_inside_it_is_cut() {
    let (on, off, colors) = fixture_frames();
    let above = check(&on, &off, colors, RoundedBox::new(240.0, 10.0, 50.0, 50.0, 0.0));
    assert!(above.is_ok(), "the ring above the clip is whole: {:?}", above.err());
    let inside = check(&on, &off, colors, RoundedBox::new(240.0, 100.0, 50.0, 50.0, 0.0));
    assert!(inside.is_err(), "the control: the clip cuts a ring that is inside it, so the bands are not found");
}
