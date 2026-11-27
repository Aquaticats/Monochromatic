//! Rebuilding the window's view models from the screen state.
//!
//! What:
//! `project` writes every list,
//! picker and deck value from one read of the state,
//! `track_row` converts one plain row into the window's row type,
//! `apply_scroll_request` moves the track list for one pending request,
//! and `model_of` wraps a vector in a shared Slint model.
//! Why:
//! the window must never mix models from two projections,
//! so every write goes through one function,
//! split from `src/ui_screen.rs` to stay within the line budget.

/// What:
///      `use std::rc::Rc` brings the shared ownership type that a Slint model is held in.
/// Why:
///      A model is shared between the window and Rust,
///  so it cannot be a plain `Box`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS objects are garbage collected; no reference-counted box is needed.
/// ```
use std::rc::Rc;

/// What:
///      `use music_player::template_model::DEFAULT_TRACK_TEMPLATE` brings the template a
///           track row falls back to.
/// Why:
///      A row's supporting line is evaluated from the open template,
///  or from this default when none applies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DEFAULT_TRACK_TEMPLATE } from "music_player/template_model";
/// ```
use music_player::template_model::DEFAULT_TRACK_TEMPLATE;
/// What:
///      `use music_player::track_supporting_line::supporting_line_from_template` brings
///           the evaluation of a row's supporting line.
/// Why:
///      The projection shows each track's length and peak through that one function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { supporting_line_from_template } from "music_player/track_supporting_line";
/// ```
use music_player::track_supporting_line::supporting_line_from_template;

/// What:
///      `use slint::{...}` brings the model types and the shared string that fill the
///           window.
/// Why:
///      A Slint list property takes a model of shared strings,
///  so the projection builds exactly those values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ModelRc, SharedString, VecModel } from "slint";
/// ```
use slint::{
    ModelRc,
    SharedString,
    VecModel,
};

/// What:
///      `use crate::ui_screen_scroll::{...}` brings the scroll request and its decision.
/// Why:
///      The projection is where a pending scroll request is finally applied.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ScrollRequest, next_scroll_request, scroll_offset_for } from "./ui_screen_scroll";
/// ```
use crate::ui_screen_scroll::{
    ScrollRequest,
    next_scroll_request,
    scroll_offset_for,
};
/// What:
///      `use crate::ui_screen_view::{...}` brings the screen state and the view it
///           projects.
/// Why:
///      The projection reads the state,
///  asks the pure module for the view,
///  and writes that view into the window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ScreenState, ScreenView, view } from "./ui_screen_view";
/// ```
use crate::ui_screen_view::{
    ScreenState,
    ScreenView,
    view,
};
/// What:
///      `use crate::{...}` brings the window and the generated row and deck types.
/// Why:
///      Every written property is typed by one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { AppWindow, DeckState, FolderName, FolderRailCell, TrackListRow } from "./main";
/// ```
use crate::{
    AppWindow,
    DeckState,
    FolderName,
    FolderRailCell,
    TrackListRow,
};
/// What:
///      `use slint::ComponentHandle` brings the trait whose methods reach the window's
///           global values.
/// Why:
///      The deck state is a Slint global,
///  and `global` is a trait method,
///  so the trait must be in scope although no line names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { ComponentHandle } from "slint";
/// ```
use slint::ComponentHandle;

/// What:
///      `use super::{read_screen, read_state, write_screen}` brings the state accessors
///           of the parent module.
/// Why:
///      A projection reads the state twice and writes it once,
///  and those three accessors are the parent's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { read_screen, read_state, write_screen } from "./ui_screen";
/// ```
use super::{
    read_screen,
    read_state,
    write_screen,
};

/// What:
/// `apply_scroll_request` moves the track list for one pending request by writing its offset and bumping the tick.
/// Why:
/// the offset is written before the tick,
/// so the list reads the new offset when the tick changes;
/// without a
/// request the list keeps its scroll position.
fn apply_scroll_request(
    app: &AppWindow,
    pending: Option<ScrollRequest>,
    rows: &[crate::ui_screen_rows::ListRow],
) {
    let Some(request) = pending else {
        return;
    };
    app.set_scroll_offset(scroll_offset_for(request, rows));
    app.set_scroll_tick(app.get_scroll_tick() + 1);
}

/// What:
/// `project` rebuilds the window's view models from the state and writes them to the window.
/// Why:
/// every list,
/// picker and deck value comes from one projection,
/// so the window never mixes stale models.
pub(crate) fn project(app: &AppWindow) {
    let started = std::time::Instant::now();
    let (screen, request): (ScreenView, Option<ScrollRequest>) = {
        let mut state = write_screen();
        // The template text is copied so the supporting line does not keep `state` borrowed while the
        // scroll decision below needs it mutably.
        let template = match state.track_rows_template.clone() {
            Some(text) => text,
            None => DEFAULT_TRACK_TEMPLATE.to_string(),
        };
        let screen = {
            let supporting_line = supporting_line_from_template(template.as_str());
            view(&state, &supporting_line)
        };
        let request = next_scroll_request(&mut state, &screen);
        (screen, request)
    };
    // With no folder page (reading, empty or without audio) the folded header would show only a caret (EM5): it shows
    // the library's name instead. The two-pane layout keeps its own heading, which names the library already.
    let header_title: String = if screen.folder_title.is_empty() {
        read_screen().library_heading()
    } else {
        screen.folder_title.clone()
    };
    app.set_folded_title(header_title.as_str().into());
    app.set_folder_title(screen.folder_title.as_str().into());
    let view_ms = started.elapsed().as_millis();
    app.set_track_rows(model_of(screen.rows.iter().map(track_row).collect()));
    crate::ui_lists::publish_track_layout(app, &screen.rows);
    apply_scroll_request(app, request, &screen.rows);
    let mut rail_cells: Vec<FolderRailCell> = Vec::with_capacity(screen.picker.cells.len());
    for cell in &screen.picker.cells {
        rail_cells.push(FolderRailCell {
            key: cell.key.as_str().into(),
            label: cell.label.as_str().into(),
            section_start: cell.section_start,
        });
    }
    app.set_rail_cells(model_of(rail_cells));
    let mut folder_names: Vec<FolderName> = Vec::with_capacity(screen.picker.names.len());
    for row in &screen.picker.names {
        folder_names.push(FolderName {
            name: row.name.as_str().into(),
            section_start: row.section_start,
            heading: row.heading.as_str().into(),
        });
    }
    app.set_folder_names(model_of(folder_names));
    app.set_current_folder(screen.folder_title.as_str().into());
    app.set_current_cell(screen.picker.current_cell.as_str().into());
    {
        let mut state = write_screen();
        state.shown_rows = screen.rows.clone();
        state.shown_names = screen.picker.names.clone();
        state.shown_folder = screen.folder_title.clone();
    }
    crate::ui_lists::sync_rings(app);
    app.set_library_name(read_screen().library_heading().into());
    app.set_deck_title(screen.deck_title.as_str().into());
    app.set_deck_folder(screen.deck_folder.as_str().into());
    app.set_deck_subtitle(screen.deck_subtitle.as_str().into());
    app.set_shuffle_folder_label(screen.shuffle_folder_label.as_str().into());
    // D1 shortens a long name in the middle; the deck draws the head with an ellipsis and then the last three characters.
    let (shuffle_head, shuffle_tail) =
        music_player::non_spacing_mark::split_tail_clusters(&screen.shuffle_folder_label, 3);
    app.set_shuffle_folder_head(shuffle_head.as_str().into());
    app.set_shuffle_folder_tail(shuffle_tail.as_str().into());
    app.global::<DeckState>()
        .set_disabled(read_state(ScreenState::deck_disabled));
    app.set_picker_open(read_screen().picker_open);
    app.set_route(read_screen().route.to_int());
    crate::ui_overlays::project(app);
    crate::ui_settings::project_pages(app);
    tracing::debug!(
        rows = screen.rows.len(),
        view_ms,
        total_ms = started.elapsed().as_millis(),
        "the library screen was projected"
    );
}

/// What:
/// `track_row` converts one plain list row into the window's `TrackListRow`.
/// Why:
/// the window struct carries Slint strings,
/// which the pure row model does not.
fn track_row(row: &crate::ui_screen_rows::ListRow) -> TrackListRow {
    return TrackListRow {
        kind: row.kind,
        title: SharedString::from(row.title.as_str()),
        supporting: SharedString::from(row.supporting.as_str()),
        current: row.current,
        index: row.index,
    };
}

/// What:
/// `model_of` wraps a vector in a shared Slint model.
/// Why:
/// Slint list properties take `ModelRc` values,
/// and the rows are rebuilt on every projection.
pub(crate) fn model_of<T: Clone + 'static>(items: Vec<T>) -> ModelRc<T> {
    return Rc::new(VecModel::from(items)).into();
}
