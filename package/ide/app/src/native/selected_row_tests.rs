//! Selected rows of the tree,
//!  the search results,
//!  and the location list in both color schemes:
//! their text and marks use the ink chosen from the selection fill,
//!  and the contrast is measured
//! on rendered pixels and printed.

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
use super::{
    AppWindow,
    ui::{ReferenceEntry, SearchEntry, TreeEntry},
};
/// The rule under test and the WCAG 2 arithmetic it is built on.
use ide_app::selection_ink::{contrast, legible_ink, luminance};
/// What:
///  `ColorScheme` is the toolkit's scheme enum (`Unknown`,
///  `Dark`,
///  `Light`),
///  reached through its
/// unstable re-export module.
/// Why:
///  The palette's own selection ink differs between the schemes while the selection fill does not.
/// Gotcha:
///  This module is not stable API;
///  a toolkit upgrade can rename it and break only these tests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ColorScheme } from 'slint/private';
/// ```
use slint::private_unstable_api::re_exports::ColorScheme;
/// Window ownership,
///  toolkit models,
///  and pixel types.
use slint::{ComponentHandle, ModelRc, Rgba8Pixel, SharedPixelBuffer, VecModel};
/// What:
///  `Rc` is a shared pointer for one thread (sibling `Arc` works across threads).
/// Why:
///  The window accepts row models only behind a shared pointer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = T;
/// ```
use std::rc::Rc;

/// What:
///  `[u8; 4]` is a fixed array of four bytes,
///  red,
///  green,
///  blue,
///  alpha (siblings `Vec<u8>`,
///  growable,
/// and `&[u8]`,
///  borrowed);
///  `Rgba8Pixel` is the frame's own four-byte pixel record.
/// Why:
///  The contrast arithmetic takes byte arrays,
///  and frames hold pixel records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bytes(pixel: Pixel): [number, number, number, number];
/// ```
fn bytes(found: Rgba8Pixel) -> [u8; 4] {
    return [found.r, found.g, found.b, found.a];
}

/// What:
///  `bounds` is left,
///  right,
///  top,
///  bottom in whole pixels;
///  the answer is a pair of byte arrays:
/// the lightest and the darkest pixel inside the bounds by WCAG luminance.
/// Why:
///  Glyph cores carry the ink;
///  whichever extreme differs from the fill is what the text was drawn in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function extremes(frame: Frame, bounds: [number, number, number, number]): [Color, Color];
/// ```
fn extremes(frame: &SharedPixelBuffer<Rgba8Pixel>, bounds: [usize; 4]) -> ([u8; 4], [u8; 4]) {
    let mut lightest = bytes(pixel(frame, bounds[0], bounds[2]));
    let mut darkest = lightest;
    for y in bounds[2]..bounds[3] {
        for x in bounds[0]..bounds[1] {
            let found = bytes(pixel(frame, x, y));
            if luminance(found) > luminance(lightest) {
                lightest = found;
            }
            if luminance(found) < luminance(darkest) {
                darkest = found;
            }
        }
    }
    return (lightest, darkest);
}

/// What:
///  A record naming one drawn part of a selected row:
///  a label for messages,
///  the bounds its glyphs
/// are in,
///  and the pixel position where the row shows only its fill.
/// Why:
///  Every part of every selected row gets the same measurement and the same assertion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Part = { name: string; glyphs: [number, number, number, number]; fill: [number, number] };
/// ```
struct Part {
    /// What is measured,
    ///  for messages and the printed record.
    name: &'static str,
    /// Left,
    ///  right,
    ///  top,
    ///  bottom of the glyphs,
    ///  away from the row's boundary.
    glyphs: [usize; 4],
    /// A pixel of the row that holds only its fill.
    fill: [usize; 2],
}

/// What:
///  `window: &AppWindow` lends the window;
///  `scheme` and `part` name what is measured;
///  the answer
/// is an `f32` (a 32-bit float;
///  sibling `f64`),
///  the contrast ratio of the chosen ink on the drawn fill.
/// Why:
///  The ink is the rule's choice for the declared selection fill.
///  On the fill that is actually
/// drawn,
///  which a focused or hovered tree row tints,
///  that ink must still reach the rule's 3:1,
/// the glyphs must show it,
///  and nothing may be drawn in the opposite ink.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function measure(window: AppWindow, scheme: string, part: Part): number;
/// ```
fn measure(window: &AppWindow, scheme: &str, part: &Part) -> f32 {
    let shown = frame(window);
    let ink = legible_ink(
        rgba(window.get_selection_fill().color()),
        rgba(window.get_selected_foreground().color()),
    );
    let fill = bytes(pixel(&shown, part.fill[0], part.fill[1]));
    let (lightest, darkest) = extremes(&shown, part.glyphs);
    // The glyph extreme on the ink's side of the fill, and the one on the opposite side.
    let (toward, away) = if luminance(ink) > luminance(fill) {
        (lightest, darkest)
    } else {
        (darkest, lightest)
    };
    let declared = contrast(ink, fill);
    let rendered = contrast(toward, fill);
    println!(
        "selected-row contrast: {scheme} {}: fill #{:02X}{:02X}{:02X}, ink #{:02X}{:02X}{:02X}, \
         ink on fill {declared:.2}:1, strongest glyph pixel {rendered:.2}:1",
        part.name, fill[0], fill[1], fill[2], ink[0], ink[1], ink[2]
    );
    assert!(
        declared >= 3.0,
        "{scheme} {}: the chosen ink reaches only {declared:.2}:1 on the drawn fill",
        part.name
    );
    // Thin glyphs are antialiased, so their strongest pixel is weaker than the ink itself; the opposite
    // ink would leave this value at 1.
    assert!(
        rendered >= 2.0,
        "{scheme} {}: selected row text is not drawn in the ink chosen from the selection fill \
         (strongest glyph pixel {rendered:.2}:1)",
        part.name
    );
    assert!(
        contrast(away, fill) < 1.3,
        "{scheme} {}: part of the selected row is drawn in the opposite ink",
        part.name
    );
    return declared;
}

/// What:
///  `state` names a tinted state of the selected tree row;
///  `plain` is the contrast on the untinted fill.
/// Why:
///  Hover and keyboard focus tint the row.
///  The tint must be visible,
///  and it must move the fill away
/// from the ink,
///  so the text never loses contrast in those states.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tinted(window: AppWindow, scheme: string, state: string, plain: number): void;
/// ```
fn tinted(window: &AppWindow, scheme: &str, state: &str, plain: f32) {
    for part in &TREE {
        let shown = measure(window, &format!("{scheme}, tree row {state},"), part);
        assert!(
            shown > plain,
            "{scheme}: the {state} tint lowered the contrast of the selected row's text from {plain:.2}:1 \
             to {shown:.2}:1, or did not tint the row"
        );
    }
}

/// What:
///  `vec![...]` builds an array;
///  `..TreeEntry::default()` fills every field not named;
/// `.into()` converts a string literal to Slint's string type;
///  `ModelRc::from(Rc::new(VecModel::from(..)))`
/// wraps the array as the shared model type the window accepts.
/// Why:
///  The second row is a selected file with a slot badge,
///  the first an expanded directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// window.treeEntries = [{ label: 'src', directory: true, expanded: true }, { label: 'selected.ts', selected: true }];
/// ```
fn tree(window: &AppWindow) {
    let rows = vec![
        TreeEntry {
            label: "src".into(),
            directory: true,
            expanded: true,
            ..TreeEntry::default()
        },
        TreeEntry {
            label: "selected.ts".into(),
            depth: 1,
            selected: true,
            recency: "1".into(),
            ..TreeEntry::default()
        },
        TreeEntry {
            label: "other.ts".into(),
            depth: 1,
            ..TreeEntry::default()
        },
    ];
    window.set_tree_entries(ModelRc::from(Rc::new(VecModel::from(rows))));
}

/// The selected tree row is the second 48px row under the 32px project label,
///  at depth 1:
/// its badge column starts at 24 and its name at 48.
const TREE: [Part; 2] = [
    Part {
        name: "tree name",
        glyphs: [48, 160, 92, 116],
        fill: [220, 104],
    },
    Part {
        name: "tree slot badge",
        glyphs: [24, 48, 92, 116],
        fill: [220, 104],
    },
];

/// The first search result row spans rows 154 to 217 of the centered panel;
///  its path line starts at
/// row 160 and its detail line at row 186,
///  both from column 170.
const SEARCH: [Part; 2] = [
    Part {
        name: "search path",
        glyphs: [170, 320, 162, 182],
        fill: [800, 186],
    },
    Part {
        name: "search detail and preview",
        glyphs: [170, 420, 188, 208],
        fill: [800, 186],
    },
];

/// Every selected row draws its name,
///  its marks,
///  and its secondary text in the ink chosen from the
/// selection fill,
///  in both schemes,
///  whether or not its list has keyboard focus or the pointer.
#[test]
fn selected_rows_use_the_ink_chosen_from_the_fill_with_measured_contrast() {
    let shared = fixture(6);
    // What: `&shared.window` borrows the window out of the fixture record.
    // Why: The helpers take a borrowed window, and the fixture keeps owning it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const window = shared.window;
    // ```
    let window = &shared.window;
    tree(window);
    for (scheme, label) in [(ColorScheme::Dark, "dark"), (ColorScheme::Light, "light")] {
        switch(window, scheme);
        // The source has keyboard focus and the pointer rests over the source.
        window.invoke_focus_source();
        motion(window, 800.0, 500.0);
        settle(window);
        let mut plain = 0.0;
        for part in &TREE {
            plain = measure(window, &format!("{label}, tree unfocused,"), part);
        }
        // The pointer over the selected row tints it.
        motion(window, 230.0, 104.0);
        tinted(window, label, "hovered", plain);
        motion(window, 800.0, 500.0);
        // Keyboard focus on the selected row tints it and adds a boundary.
        window.invoke_focus_tree();
        window.set_tree_focused_row(1);
        tinted(window, label, "focused", plain);
        window.invoke_focus_source();
        // The location list beside the first source line: its second row is selected and has a detail.
        let places = vec![
            ReferenceEntry {
                label: "src/other.ts:3".into(),
                detail: "".into(),
            },
            ReferenceEntry {
                label: "lib/selected.d.ts:12".into(),
                detail: "Outside project".into(),
            },
        ];
        window.set_references_title("2 references".into());
        window.set_references_entries(ModelRc::from(Rc::new(VecModel::from(places))));
        window.set_references_selected(1);
        window.set_references_open(true);
        settle(window);
        // What: `as usize` converts a logical pixel position to a whole pixel index.
        // Why: The list reports where it was placed; its rows are 48px tall under a 32px title,
        // and start 4px inside the list, which starts where the source text starts, at column 313.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const top = Math.trunc(window.referencesY) + 32 + 48;
        // ```
        let top = window.get_references_y() as usize + 32 + 48;
        let places_parts = [
            Part {
                name: "location label",
                glyphs: [325, 520, top + 12, top + 36],
                fill: [600, top + 24],
            },
            Part {
                name: "location detail",
                glyphs: [660, 780, top + 12, top + 36],
                fill: [600, top + 24],
            },
        ];
        for focus in ["unfocused", "focused"] {
            if focus == "focused" {
                window.invoke_focus_references();
            }
            for part in &places_parts {
                measure(window, &format!("{label}, location list {focus},"), part);
            }
        }
        window.set_references_open(false);
        window.invoke_focus_source();
        // The search overlay with one selected result that has a detail and a preview.
        let results = vec![SearchEntry {
            path: "src/selected.ts".into(),
            detail: "Line 12".into(),
            preview: "const selected = 1;".into(),
        }];
        window.set_search_entries(ModelRc::from(Rc::new(VecModel::from(results))));
        window.set_search_selected(0);
        window.set_search_open(true);
        for part in &SEARCH {
            measure(window, &format!("{label}, search results,"), part);
        }
        window.set_search_open(false);
    }
    window.hide().expect("close selected-row window");
}
