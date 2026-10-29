//! Tree rows that git ignores,
//!  in both color schemes:
//!  the name is drawn at 0.8 of its ink,
//!  nothing else of the row changes,
//!  and the contrast is measured on rendered pixels and printed.

/// A window with every production binding over a disposable project.
use super::find_tests::reader;
/// Polling for the tree the reader fills in the background.
use super::navigation_tests::row;
/// Rendered frames and single pixels,
///  shared with the sidebar paint tests.
use super::sidebar_paint_tests::{frame, pixel};
/// The shared window fixture with a visible tree,
///  and pointer movement.
use super::sidebar_tests::{fixture, motion, settle};
/// The scheme switch the desktop-settings watcher makes,
///  and color bytes.
use super::theme_tests::{rgba, switch};
/// Generated window and row types from the shipped markup.
use super::{AppWindow, ui::TreeEntry};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::{NAVIGATION_UNREACHED, Waiting};
/// The row elements an assistive tool is given.
use i_slint_backend_testing::{AccessibleRole, ElementQuery};
/// The WCAG 2 arithmetic of the selected-ink rule,
///  and its choice of ink.
use ide_app::selection_ink::{contrast, legible_ink, luminance};
/// The toolkit's scheme enum,
///  reached through its unstable re-export module.
use slint::private_unstable_api::re_exports::ColorScheme;
/// Window ownership,
///  toolkit models,
///  and pixel types.
use slint::{ComponentHandle, Model, ModelRc, Rgba8Pixel, SharedPixelBuffer, VecModel};
/// The window accepts row models only behind a shared pointer.
use std::{fs, path::Path, rc::Rc};

/// The opacity the user asked for.
const OPACITY: f32 = 0.8;

/// Rows are 48px tall under the 32px project label.
const fn top(index: usize) -> usize {
    return 32 + 48 * index;
}

/// What:
///  `[u8; 4]` is red,
///  green,
///  blue,
///  alpha;
///  the answer is the lightest and the darkest pixel inside the left,
///  right,
///  top,
///  bottom bounds by WCAG luminance.
/// Why:
///  Glyph cores carry the ink;
///  the extreme that differs from the fill is what the text was drawn in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function extremes(frame: Frame, bounds: [number, number, number, number]): [Color, Color];
/// ```
fn extremes(frame: &SharedPixelBuffer<Rgba8Pixel>, bounds: [usize; 4]) -> ([u8; 4], [u8; 4]) {
    let first = pixel(frame, bounds[0], bounds[2]);
    let mut lightest = [first.r, first.g, first.b, first.a];
    let mut darkest = lightest;
    for y in bounds[2]..bounds[3] {
        for x in bounds[0]..bounds[1] {
            let found = pixel(frame, x, y);
            let bytes = [found.r, found.g, found.b, found.a];
            if luminance(bytes) > luminance(lightest) {
                lightest = bytes;
            }
            if luminance(bytes) < luminance(darkest) {
                darkest = bytes;
            }
        }
    }
    return (lightest, darkest);
}

/// The drawn ink of a part:
///  the extreme that lies farther from the fill in luminance.
fn strongest(frame: &SharedPixelBuffer<Rgba8Pixel>, bounds: [usize; 4], fill: [u8; 4]) -> [u8; 4] {
    let (lightest, darkest) = extremes(frame, bounds);
    return if contrast(lightest, fill) >= contrast(darkest, fill) {
        lightest
    } else {
        darkest
    };
}

/// The fill of a row away from its text.
fn fill(frame: &SharedPixelBuffer<Rgba8Pixel>, index: usize) -> [u8; 4] {
    let found = pixel(frame, 230, top(index) + 24);
    return [found.r, found.g, found.b, found.a];
}

/// Name glyph bounds of a depth-0 row.
fn name(index: usize) -> [usize; 4] {
    return [32, 150, top(index) + 12, top(index) + 36];
}

/// Badge or arrow glyph bounds of a depth-0 row.
fn mark(index: usize) -> [usize; 4] {
    return [8, 32, top(index) + 12, top(index) + 36];
}

/// The model of the measurement:
///  the same name plain and ignored,
///  a selected pair,
///  a directory pair,
///  and a badge pair.
fn rows(window: &AppWindow) {
    let model = vec![
        tree_entry("same.ts", false, false, false, ""),
        tree_entry("same.ts", true, false, false, ""),
        tree_entry("sel.ts", false, true, false, "1"),
        tree_entry("sel.ts", true, true, false, "1"),
        tree_entry("dir", false, false, true, ""),
        tree_entry("dir", true, false, true, ""),
        tree_entry("same.ts", false, false, false, "2"),
        tree_entry("same.ts", true, false, false, "2"),
    ];
    window.set_tree_entries(ModelRc::from(Rc::new(VecModel::from(model))));
}

/// One tree entry with only the fields these tests vary.
fn tree_entry(
    label: &str,
    ignored: bool,
    selected: bool,
    directory: bool,
    recency: &str,
) -> TreeEntry {
    return TreeEntry {
        label: label.into(),
        ignored,
        selected,
        directory,
        recency: recency.into(),
        ..TreeEntry::default()
    };
}

/// Largest per-channel difference between two colors.
fn spread(left: [u8; 4], right: [u8; 4]) -> u8 {
    let mut widest = 0;
    for channel in 0..3 {
        widest = widest.max(left[channel].abs_diff(right[channel]));
    }
    return widest;
}

/// Where each of the plain and ignored rows is expected to differ:
///  `ink * 0.8 + fill * 0.2` per channel.
fn blended(ink: [u8; 4], fill: [u8; 4]) -> [u8; 4] {
    let mut result = [255; 4];
    for channel in 0..3 {
        let value = f32::from(ink[channel]) * OPACITY + f32::from(fill[channel]) * (1.0 - OPACITY);
        result[channel] = value.round() as u8;
    }
    return result;
}

/// Measure one state:
///  returns the contrast of the plain row's name,
///  and of the ignored row's name,
///  against the fill that row has.
fn name_pair(
    window: &AppWindow,
    scheme: &str,
    state: &str,
    plain: usize,
    dimmed: usize,
) -> (f32, f32) {
    let shown = frame(window);
    let plain_fill = fill(&shown, plain);
    let dimmed_fill = fill(&shown, dimmed);
    let plain_ink = strongest(&shown, name(plain), plain_fill);
    let dimmed_ink = strongest(&shown, name(dimmed), dimmed_fill);
    let plain_ratio = contrast(plain_ink, plain_fill);
    let dimmed_ratio = contrast(dimmed_ink, dimmed_fill);
    println!(
        "ignored-row contrast: {scheme} {state}: fill #{:02X}{:02X}{:02X}, plain name {plain_ratio:.2}:1 \
         (#{:02X}{:02X}{:02X}), ignored name {dimmed_ratio:.2}:1 (#{:02X}{:02X}{:02X})",
        dimmed_fill[0],
        dimmed_fill[1],
        dimmed_fill[2],
        plain_ink[0],
        plain_ink[1],
        plain_ink[2],
        dimmed_ink[0],
        dimmed_ink[1],
        dimmed_ink[2]
    );
    // Same glyphs, same fill: the ignored name is the plain name's ink mixed 0.8 to 0.2 with the fill.
    // Antialiasing and 8-bit rounding allow a few levels.
    let expected = blended(plain_ink, dimmed_fill);
    assert!(
        spread(dimmed_ink, expected) <= 6,
        "{scheme} {state}: the ignored name is not the plain ink at 0.8 opacity over the fill: \
         drew #{:02X}{:02X}{:02X}, expected about #{:02X}{:02X}{:02X}",
        dimmed_ink[0],
        dimmed_ink[1],
        dimmed_ink[2],
        expected[0],
        expected[1],
        expected[2]
    );
    // The repository's own floor for ink on a fill (the selected-ink rule asserts the same 3:1).
    assert!(
        dimmed_ratio >= 3.0,
        "{scheme} {state}: the ignored name reaches only {dimmed_ratio:.2}:1 against its fill"
    );
    return (plain_ratio, dimmed_ratio);
}

/// Everything of the row except its name looks the same ignored and plain:
///  the directory arrow,
///  and the slot badge.
fn marks_unchanged(window: &AppWindow, scheme: &str) {
    let shown = frame(window);
    for (plain, dimmed, what) in [(4, 5, "directory arrow"), (6, 7, "slot badge")] {
        let plain_ink = strongest(&shown, mark(plain), fill(&shown, plain));
        let dimmed_ink = strongest(&shown, mark(dimmed), fill(&shown, dimmed));
        assert!(
            spread(plain_ink, dimmed_ink) <= 2,
            "{scheme}: the {what} of an ignored row is dimmed: plain #{:02X}{:02X}{:02X}, ignored #{:02X}{:02X}{:02X}",
            plain_ink[0],
            plain_ink[1],
            plain_ink[2],
            dimmed_ink[0],
            dimmed_ink[1],
            dimmed_ink[2]
        );
    }
}

/// The name of an ignored row is drawn at 0.8 of its ink in every state and both schemes,
///  its arrow and badge are not,
///  and the contrast of each state is printed.
#[test]
fn ignored_names_are_drawn_at_eighty_percent_with_measured_contrast() {
    let shared = fixture(6);
    let window = &shared.window;
    rows(window);
    for (scheme, label) in [(ColorScheme::Dark, "dark"), (ColorScheme::Light, "light")] {
        switch(window, scheme);
        window.invoke_focus_source();
        motion(window, 800.0, 500.0);
        settle(window);
        let (plain, dimmed) = name_pair(window, label, "plain", 0, 1);
        assert!(
            dimmed < plain,
            "{label}: the ignored name is not weaker than the plain one"
        );
        marks_unchanged(window, label);
        // The pointer over the plain row and over the ignored row.
        motion(window, 230.0, top(0) as f32 + 24.0);
        name_pair(window, label, "pointer over the plain row", 0, 1);
        motion(window, 230.0, top(1) as f32 + 24.0);
        name_pair(window, label, "pointer over the ignored row", 0, 1);
        motion(window, 800.0, 500.0);
        // Keyboard focus on the ignored row.
        window.invoke_focus_tree();
        window.set_tree_focused_row(1);
        name_pair(window, label, "keyboard focus on the ignored row", 0, 1);
        window.invoke_focus_source();
        // The selected pair: the ink is chosen from the selection fill.
        let selected = name_pair(window, label, "selected", 2, 3);
        motion(window, 230.0, top(3) as f32 + 24.0);
        let hovered = name_pair(
            window,
            label,
            "selected, pointer over the ignored row",
            2,
            3,
        );
        motion(window, 800.0, 500.0);
        window.invoke_focus_tree();
        window.set_tree_focused_row(3);
        let focused = name_pair(
            window,
            label,
            "selected, keyboard focus on the ignored row",
            2,
            3,
        );
        window.invoke_focus_source();
        let ink = legible_ink(
            rgba(window.get_selection_fill().color()),
            rgba(window.get_selected_foreground().color()),
        );
        println!(
            "ignored-row contrast: {label} summary: plain {plain:.2}:1 to ignored {dimmed:.2}:1; selected {:.2}:1 \
             to ignored {:.2}:1 (pointer {:.2}:1, focus {:.2}:1); selection ink #{:02X}{:02X}{:02X}",
            selected.0, selected.1, hovered.1, focused.1, ink[0], ink[1], ink[2]
        );
    }
    window.hide().expect("close ignored-row window");
}

/// A disposable work tree for the tests that run the real reader:
///  the extension is chosen so a user's global excludes file,
/// which the reader also reads,
/// cannot match it.
fn project(root: &Path) {
    fs::create_dir(root.join(".git")).expect("git folder");
    fs::write(
        root.join(".gitignore"),
        "*.zz-ignored\nbuild/\n!kept.zz-ignored\n",
    )
    .expect("rule file");
    for name in [
        "a.zz-ignored",
        "kept.zz-ignored",
        "plain.txt",
        "build/inner.txt",
        "sub/deep.zz-ignored",
        "sub/deep.txt",
    ] {
        let path = root.join(name);
        fs::create_dir_all(path.parent().expect("parent")).expect("fixture folder");
        fs::write(path, "x\n").expect("fixture file");
    }
}

/// Whether the model row with this label is marked ignored.
fn flagged(window: &AppWindow, label: &str) -> Option<bool> {
    let index = row(window, label)?;
    return match window.get_tree_entries().row_data(index as usize) {
        Some(entry) => Some(entry.ignored),
        None => None,
    };
}

/// How many accessible list items carry exactly this label.
fn labelled(window: &AppWindow, label: &str) -> usize {
    let wanted = label.to_string();
    let mut count = 0;
    for candidate in ElementQuery::from_root(window)
        .match_accessible_role(AccessibleRole::ListItem)
        .find_all()
    {
        let named = match candidate.accessible_label() {
            Some(text) => text == wanted.as_str(),
            None => false,
        };
        if named {
            count += 1;
        }
    }
    return count;
}

/// The real reader marks rows from the project's rule files:
///  the model carries the flag,
///  and each ignored row says so in its accessible label while a plain row does not.
#[test]
fn ignored_rows_are_flagged_in_the_model_and_named_in_the_accessibility_tree() {
    let fixture = tempfile::tempdir().expect("disposable project");
    project(fixture.path());
    let shown = reader(fixture.path(), "plain.txt");
    let window = &shown.window;
    {
        let mut waiting = Waiting::eventually();
        while !(row(window, "sub").is_some()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    for name in ["build", "sub"] {
        window.invoke_tree_activate(row(window, name).expect("directory row"));
        {
            let mut waiting = Waiting::eventually();
            while !(row(
                window,
                if name == "build" {
                    "inner.txt"
                } else {
                    "deep.txt"
                },
            )
            .is_some())
            {
                waiting.step(NAVIGATION_UNREACHED);
            }
        }
    }
    settle(window);
    // File pattern, negation, directory pattern, and a plain file.
    assert_eq!(flagged(window, "a.zz-ignored"), Some(true));
    assert_eq!(flagged(window, "kept.zz-ignored"), Some(false));
    assert_eq!(flagged(window, "plain.txt"), Some(false));
    assert_eq!(flagged(window, "build"), Some(true));
    assert_eq!(
        flagged(window, "inner.txt"),
        Some(true),
        "inside an ignored directory"
    );
    assert_eq!(flagged(window, "deep.zz-ignored"), Some(true));
    assert_eq!(flagged(window, "deep.txt"), Some(false));
    // The accessibility tree says it a second way: the row's label.
    for (label, count) in [
        ("a.zz-ignored, ignored by git", 1),
        ("build, ignored by git", 1),
        ("inner.txt, ignored by git", 1),
        ("deep.zz-ignored, ignored by git", 1),
        ("kept.zz-ignored", 1),
        ("plain.txt", 1),
        ("deep.txt", 1),
        // A plain row is never named with the suffix, and an ignored row never without it.
        ("plain.txt, ignored by git", 0),
        ("a.zz-ignored", 0),
        ("build", 0),
    ] {
        assert_eq!(
            labelled(window, label),
            count,
            "{count} list item(s) should be labelled {label:?}"
        );
    }
    window.hide().expect("close ignored-row window");
}

/// A `.gitignore` changed on disk while the window runs changes the rows of the root and of an open folder
///  without reopening the project,
///  and a folder outside any work tree dims nothing.
#[test]
fn a_changed_gitignore_dims_and_undims_rows_while_the_window_runs() {
    let fixture = tempfile::tempdir().expect("disposable project");
    project(fixture.path());
    let shown = reader(fixture.path(), "plain.txt");
    let window = &shown.window;
    {
        let mut waiting = Waiting::eventually();
        while !(row(window, "sub").is_some()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    window.invoke_tree_activate(row(window, "sub").expect("sub row"));
    {
        let mut waiting = Waiting::eventually();
        while !(row(window, "deep.txt").is_some()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    {
        let mut waiting = Waiting::eventually();
        while !(flagged(window, "deep.zz-ignored") == Some(true)) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    assert_eq!(flagged(window, "plain.txt"), Some(false));
    assert_eq!(flagged(window, "deep.txt"), Some(false));
    // The test, never the IDE, rewrites the rule file: every .txt is now ignored, and the extension no longer is.
    fs::write(
        fixture.path().join(".gitignore"),
        "*.txt\n# rules changed while the window runs\n",
    )
    .expect("changed rule file");
    {
        let mut waiting = Waiting::eventually();
        while !(flagged(window, "plain.txt") == Some(true)) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    {
        let mut waiting = Waiting::eventually();
        while !(flagged(window, "deep.txt") == Some(true)) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    {
        let mut waiting = Waiting::eventually();
        while !(flagged(window, "deep.zz-ignored") == Some(false)) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    {
        let mut waiting = Waiting::eventually();
        while !(flagged(window, "a.zz-ignored") == Some(false)) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    // And back.
    fs::write(fixture.path().join(".gitignore"), "*.zz-ignored\n").expect("restored rule file");
    {
        let mut waiting = Waiting::eventually();
        while !(flagged(window, "plain.txt") == Some(false)) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    {
        let mut waiting = Waiting::eventually();
        while !(flagged(window, "deep.zz-ignored") == Some(true)) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    window.hide().expect("close ignored-row window");
}

/// Outside any work tree a `.gitignore` is only a file:
///  no row is flagged and no label carries the suffix.
#[test]
fn outside_a_work_tree_no_row_is_flagged_or_named_ignored() {
    let fixture = tempfile::tempdir().expect("disposable project");
    project(fixture.path());
    fs::remove_dir(fixture.path().join(".git")).expect("remove the git folder");
    let shown = reader(fixture.path(), "plain.txt");
    let window = &shown.window;
    {
        let mut waiting = Waiting::eventually();
        while !(row(window, "a.zz-ignored").is_some()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    settle(window);
    for label in [
        "a.zz-ignored",
        "kept.zz-ignored",
        "plain.txt",
        "build",
        "sub",
    ] {
        assert_eq!(flagged(window, label), Some(false), "{label} is flagged");
    }
    assert_eq!(labelled(window, "a.zz-ignored"), 1);
    assert_eq!(labelled(window, "a.zz-ignored, ignored by git"), 0);
    window.hide().expect("close ignored-row window");
}
