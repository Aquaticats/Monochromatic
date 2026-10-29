//! Presenting a frame to the window:
//!  the scene and offsets a presentation carries,
//!  the gutter's wrap indicator,
//!  and the setters that run after the state borrow ends.

/// The window,
///  the source state,
///  the find presentation,
///  and the annotation readings.
use crate::native::{
    AppWindow, State, annotate, find::present::present, rows, ui::SourceSelection, viewport::reach,
};
/// The rectangles a frame hands to the window.
use ide_app::shaped_text::ReadingRect;
/// The scene the GPU draws into a texture,
/// and the scenes of a tile that is drawn across several.
use ide_app::tile_scene::{TileScene, TileStack};
/// Toolkit models carry owned source presentation data.
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
/// UI callbacks share state without cross-thread synchronization.
use std::{cell::RefCell, rc::Rc};

/// What:
///  Width of the cell the gutter's wrap indicator is painted in,
/// in logical pixels.
/// Why:
///  One space advance of the source font is 9 px;
/// the cell leaves the mark some room at its right edge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SIGN_WIDTH = 12;
/// ```
const SIGN_WIDTH: f32 = 12.0;

/// Copy a frame's rectangles into the window's row type.
/// Selection and find rectangles reach the markup as model rows.
fn model_rows(rectangles: &[ReadingRect]) -> Vec<SourceSelection> {
    let mut rows = Vec::new();
    for rect in rectangles {
        rows.push(SourceSelection {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        });
    }
    return rows;
}

/// What:
///  What one presentation hands the window:
/// a new scene for the GPU,
/// the offsets,
/// and the frame's diagnostics.
/// Why:
///  The tile swap and the synchronous path present through the same code,
/// so they cannot disagree on order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Frame = { scene?: Scene[]; matchesChanged: boolean; scale: number; scrolled?: number; sign?: Scene };
/// ```
pub(in crate::native) struct Frame {
    /// Scenes of a newly built tile,
    /// one for each texture of its stack,
    /// when one was swapped in.
    pub(in crate::native) scene: Option<TileStack>,
    /// Whether the match rectangles changed and must be handed over again.
    pub(in crate::native) matches_changed: bool,
    /// Physical pixels per logical pixel,
    /// to size the image.
    pub(in crate::native) scale: f32,
    /// Scroll offset that settling the rows asked the window to take.
    pub(in crate::native) scrolled: Option<f32>,
    /// Line the language surface belongs to.
    pub(in crate::native) anchor_line: i32,
    /// A failure to start or queue a tile,
    /// shown with the other diagnostics.
    pub(in crate::native) notice: Option<String>,
    /// The gutter's wrap indicator,
    /// drawn again after a change of display scale.
    pub(in crate::native) sign: Option<TileScene>,
}

/// What:
///  Build the scene of the gutter's wrap indicator for display scale `factor` when it has not been built for it yet.
///  The symbol is chosen once,
/// from the fonts.
/// Why:
///  The picture is set in the shaper the source text uses,
/// so the coverage check is about what is shown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ensureSign(current: State, factor: number): Scene | undefined;
/// ```
pub(super) fn ensure_sign(current: &mut State, factor: f32) -> Option<TileScene> {
    if current.sign_scale == Some(factor.to_bits()) {
        return None;
    }
    let State {
        shaper,
        sign,
        sign_scale,
        ..
    } = current;
    let chosen = match sign {
        Some(known) => *known,
        None => {
            let picked = shaper.choose_sign(factor);
            tracing::debug!(?picked, "chose the wrap indicator");
            *sign = Some(picked);
            picked
        }
    };
    match shaper.sign_scene(chosen, factor, SIGN_WIDTH) {
        Ok(scene) => {
            *sign_scale = Some(factor.to_bits());
            return Some(scene);
        }
        Err(error) => {
            tracing::error!(%error, "wrap indicator could not be drawn");
            return None;
        }
    }
}

/// Present the displayed frame:
/// its scene and offsets,
/// its gutter,
/// rectangles,
/// and caret,
/// then the other readings.
/// Each setter runs after the state borrow ends.
pub(in crate::native) fn show(window: &AppWindow, state: &Rc<RefCell<State>>, frame: Frame) {
    let Frame {
        scene,
        matches_changed,
        scale: factor,
        scrolled,
        anchor_line,
        notice,
        sign,
    } = frame;
    let mut current = state.borrow_mut();
    let found = present(&current);
    let view = current.shaped.as_ref().expect("painted source view");
    let mut matches = None;
    if matches_changed {
        matches = Some(model_rows(&view.matches));
    }
    let document = &current.document;
    let caret = view.caret(document);
    let selections = model_rows(&view.selections);
    let position = document.position();
    let revision = document.revision();
    let selected = document.selected_text();
    // The tile starts at the top of the first materialized line's block.
    let origin = view.origin;
    let problems = annotate::problems(&current);
    let updated_source = if current.presented_revision != Some(revision) {
        current.presented_revision = Some(revision);
        Some(current.document.text().to_string())
    } else {
        None
    };
    let mut notices = Vec::new();
    if let Some(message) = &current.navigation_error {
        notices.push(message.as_str());
    }
    if let Some(message) = &current.file_error {
        notices.push(message.as_str());
    }
    if let Some(message) = &current.syntax_error {
        notices.push(message.as_str());
    }
    if let Some(message) = &notice {
        notices.push(message.as_str());
    }
    let diagnostic = notices.join("\n");
    #[cfg(debug_assertions)]
    if scene.is_some() {
        current.images_set += 1;
    }
    let placement = rows::measure(&current, anchor_line);
    let (hold_top, hold_bottom) = reach(&current);
    let gpu = current.gpu.clone();
    drop(current);

    // The GPU draws the scenes into their textures just before the next frame; the image on the window follows
    // the texture, and a redraw is asked for because a new scene may change no property the window watches.
    if let Some(tile) = scene {
        window.set_image_width(tile.width as f32 / factor);
        window.set_image_height(tile.height as f32 / factor);
        gpu.submit_tile(window, tile);
        window.window().request_redraw();
    }
    if let Some(mark) = sign {
        window.set_wrap_sign_width(SIGN_WIDTH);
        gpu.submit_sign(mark);
        window.window().request_redraw();
    }
    // Rows above the tile can change without a repaint; the image and the line numbers follow the map.
    // The scroll extent is set before the offset, so the old extent cannot clamp the new offset.
    window.set_image_y(origin);
    // The reach of the displayed tile arrives with the tile and its origin, so the view is never drawn against a
    // tile other than the one whose picture the frame samples.
    window.set_hold_top(hold_top);
    window.set_hold_bottom(hold_bottom);
    rows::present(window, placement);
    if let Some(offset) = scrolled {
        window.set_scroll_y(-offset);
    }
    window.set_error_message(SharedString::from(diagnostic));
    window.set_source_selections(ModelRc::from(Rc::new(VecModel::from(selections))));
    window.set_selection_is_match(found.active.is_some());
    if let Some(marks) = matches {
        window.set_source_matches(ModelRc::from(Rc::new(VecModel::from(marks))));
    }
    annotate::present_problems(window, problems);
    if let Some(status) = found.status {
        window.set_find_status(SharedString::from(status.label));
        window.set_find_status_detail(SharedString::from(status.detail));
        window.set_find_no_match(status.no_match);
    }
    window.set_caret_x(caret.x);
    window.set_caret_y(caret.y);
    if let Some(source) = updated_source {
        window.set_source_text(SharedString::from(source));
    }
    window.set_selected_text(SharedString::from(selected));
    window.set_selection_anchor(position.anchor as i32);
    window.set_selection_head(position.head as i32);
    tracing::debug!(
        revision,
        anchor = position.anchor,
        head = position.head,
        "source reading state presented"
    );
}
