//! Rendered tests of the focus ring on the pages other than the player (D117): Search, Settings, the template editor, the
//! first-run pane, the toasts and the track menu, and of the menu card's own color (D119).
//!
//! What: the same pixel checks as `ui_focus_ring_tests`, one call per focusable element of each page, in the light and the dark
//! scheme: the ring is the 2 px `primary` band and the 1 px `on-primary` band flush outside the element (or on the edge of an
//! element as wide as its pane), and nothing else differs when the ring is on.
//! Why: each page builds its own elements, and a fill or a clip that a page keeps would show only there.

use i_slint_backend_testing::ElementHandle;
use slint::{ModelRc, VecModel};

use crate::ui_focus_ring_tests::{Case, case, check_focused, check_pair, run_case, tab_to};
use crate::ui_render_support::{Rendered, box_of};
use crate::ui_test_support::{chord, tap};

/// What: `open_search` opens the Search page with Ctrl+F and waits for its fade to end.
fn open_search(rendered: &Rendered) {
    chord(&rendered.app, &["Ctrl"], "f");
    rendered.wait(300);
    assert_eq!(rendered.app.get_route(), 1, "Ctrl+F opened Search");
}

/// What: `open_settings` opens Settings with Ctrl+Alt+S and waits for its fade to end.
fn open_settings(rendered: &Rendered) {
    chord(&rendered.app, &["Ctrl", "Alt"], "s");
    rendered.wait(300);
    assert_eq!(rendered.app.get_route(), 2, "Ctrl+Alt+S opened Settings");
}

#[test]
fn the_search_back_and_clear_rings_are_circles_flush_outside_the_40_px_circle() {
    let rendered = Rendered::new(850, 900);
    open_search(&rendered);
    rendered.app.set_search_query("abc".into());
    rendered.wait(50);
    run_case(&rendered, &case("search-back", "SearchPage::back-stop", (4.0, 4.0), 20.0));
    run_case(&rendered, &case("search-clear", "SearchPage::clear-stop", (4.0, 4.0), 20.0));
}

#[test]
fn the_search_result_ring_is_just_inside_the_rounded_shape_of_the_row() {
    let rendered = Rendered::new(850, 900);
    let segment = |text: &str| return crate::SearchSegment { text: text.into(), emphasized: false };
    let rows: Vec<crate::SearchRow> = (0..4)
        .map(|index| {
            return crate::SearchRow {
                kind: 1,
                title_segments: ModelRc::new(VecModel::from(vec![segment(&format!("Result {index}"))])),
                parent_segments: ModelRc::new(VecModel::from(vec![segment("Folder")])),
                has_parent: true,
                current: false,
                playing: false,
                spoken: "".into(),
                index,
            };
        })
        .collect();
    rendered.app.set_search_rows(ModelRc::new(VecModel::from(rows)));
    open_search(&rendered);
    for ring in [0, 2] {
        rendered.app.set_search_ring(ring);
        rendered.wait(50);
        // The query field keeps the typing focus, and its caret blinks, so the header is not judged.
        // The scrollbar's 12 px column at the right edge is drawn above the rows, but its 4 px track starts where the row's shape
        // ends (8 px from the edge), so the shape's right edge is judged and only the pixels past it are skipped.
        let skip = vec![(0.0, 0.0, 850.0, 73.0), (842.0, 0.0, 850.0, 900.0)];
        // The shape is the row's rounded box (D126): its 21.3 px corners are the ring's corners.
        let row = Case { inside: true, skip, nth: Some(usize::try_from(ring).unwrap_or(0)), radii: [21.3; 4], ..case("search-result", "SearchPage::result-shape", (0.0, 0.0), 0.0) };
        check_focused(&rendered, &row);
    }
}

#[test]
fn the_settings_back_ring_is_a_circle_and_a_row_ring_follows_the_group_corners_in_the_single_form() {
    let rendered = Rendered::new(443, 900);
    open_settings(&rendered);
    // The first row of the group is rounded 14 px at the top and 2 px at the joint below it.
    let first = Case { radii: [14.0, 14.0, 2.0, 2.0], ..case("settings-list", "SettingsPage::row-box", (0.0, 0.0), 0.0) };
    check_focused(&rendered, &first);
    run_case(&rendered, &case("settings-back", "SettingsPage::back-stop", (0.0, 0.0), 28.0));
}

#[test]
fn the_selected_settings_row_ring_follows_the_21_px_pill_in_the_two_pane_form() {
    let rendered = Rendered::new(1200, 900);
    open_settings(&rendered);
    check_focused(&rendered, &case("settings-list", "SettingsPage::row-box", (0.0, 0.0), 21.0));
}

#[test]
fn the_template_editor_page_rings_follow_the_back_circle_and_the_reset_pill() {
    let rendered = Rendered::new(443, 900);
    open_settings(&rendered);
    tap(&rendered.app, "Enter");
    rendered.wait(300);
    assert_eq!(rendered.app.get_route(), 3, "Enter opened the editor page");
    rendered.app.set_editor_can_reset(true);
    rendered.wait(50);
    run_case(&rendered, &case("editor-back", "TemplateEditor::back-stop", (0.0, 0.0), 28.0));
    run_case(&rendered, &case("reset", "TemplateEditor::reset-stop", (0.0, 0.0), 24.0));
}

#[test]
fn the_template_text_field_ring_follows_its_4_px_outline_and_leaves_the_outline_alone() {
    let rendered = Rendered::new(443, 900);
    open_settings(&rendered);
    tap(&rendered.app, "Enter");
    rendered.wait(300);
    tab_to(&rendered, "template-field");
    // The field shows its ring for any focus, so the frame without it is the one with the focus on the next element: the pixels
    // far from the field then differ by that element's ring and are not compared. The label's patch breaks the ring where the
    // label sits, so the top of the ring behind it is not judged.
    let (on, _same) = rendered.ring_frames();
    tap(&rendered.app, "Tab");
    let (_moved, off) = rendered.ring_frames();
    let label = ElementHandle::find_by_element_id(&rendered.app, "TemplateEditor::template-label")
        .map(|label| return box_of(&label))
        .find(|(_x, _y, width, _h)| return *width > 0.0)
        .expect("the label of the field");
    let input = ElementHandle::find_by_element_id(&rendered.app, "TemplateEditor::field-input")
        .map(|input| return box_of(&input))
        .find(|(_x, _y, width, _h)| return *width > 0.0)
        .expect("the text of the field");
    // The caret and the selection live inside the field, and only the focused field has them.
    let skip = vec![
        (label.0 - 6.0, label.1 - 12.0, label.0 + label.2 + 6.0, label.1 + label.3 + 12.0),
        (input.0, input.1, input.0 + input.2, input.1 + input.3),
    ];
    let field = Case { far: false, skip, nth: Some(0), ..case("template-field", "TemplateEditor::field-box", (0.0, 0.0), 4.0) };
    check_pair(&rendered, &field, &on, &off, "light");
}

#[test]
fn the_first_run_button_rings_follow_the_filled_and_the_outlined_pill() {
    let rendered = Rendered::new(850, 900);
    rendered.app.set_first_run_state(1);
    rendered.app.set_first_run_title("Choose your music".into());
    rendered.app.set_first_run_body("Pick the folder that holds it.".into());
    rendered.app.set_first_run_primary_label("Choose folder".into());
    rendered.app.set_first_run_secondary_labels(ModelRc::new(VecModel::from(vec!["Not now".into(), "Learn more".into()])));
    rendered.wait(100);
    run_case(&rendered, &case("first-run-primary", "FirstRunPane::primary-pill", (0.0, 0.0), 24.0));
    run_case(&rendered, &case("first-run-secondary-0", "FirstRunPane::secondary-pill", (0.0, 0.0), 24.0));
    run_case(&rendered, &case("first-run-secondary-1", "FirstRunPane::secondary-pill", (0.0, 0.0), 24.0));
}

#[test]
fn the_toast_button_rings_are_pills_and_circles_inside_the_card_for_an_error_and_an_undo() {
    let rendered = Rendered::new(850, 900);
    let item = |id: &str, text: &str, kind: i32, label: &str| {
        return crate::FeedbackItem { id: id.into(), text: text.into(), kind, action_label: label.into() };
    };
    let items = vec![item("a", "Could not play the file", 0, "Retry"), item("b", "Moved to the trash", 1, "Undo")];
    rendered.app.set_feedback_items(ModelRc::new(VecModel::from(items)));
    rendered.wait(300);
    // The error card comes first, then the undo card; each has an action pill and a dismiss circle.
    for turn in 0..2 {
        run_case(&rendered, &case("feedback-action", "FeedbackOverlay::action-target", (0.0, 4.0), 20.0));
        run_case(&rendered, &case("feedback-dismiss", "FeedbackOverlay::dismiss-target", (4.0, 4.0), 20.0));
        if turn == 0 {
            // Tab on to the second card's action.
            tap(&rendered.app, "Tab");
        }
    }
}
