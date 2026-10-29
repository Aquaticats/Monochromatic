//! Native image updates and source reading-state presentation.

/// Virtual-row texts and diagnostic marks are positioned against the frame's shaped rows.
/// Child modules can borrow their parent's private state and generated UI.
use super::{AppWindow, State, rewrap, rows, tile, tile_build::build_tile, viewport::place};
/// Native palette values retain syntax and selection contrast.
use ide_app::code_colors::CodeColors;
/// Selected-text ink is chosen from the selection fill,
///  not from the color scheme.
use ide_app::selection_ink::{legible_ink, luminance};
/// Toolkit images and models carry owned source presentation data.
use slint::{ComponentHandle, SharedString};
/// UI callbacks share state without cross-thread synchronization.
use std::{cell::RefCell, rc::Rc, time::Instant};

/// Measuring the lines of a requested tile and keeping the view still.
mod measure;
/// The paint inputs of a tile and the geometry they are painted at.
mod paint;
/// Presenting a frame to the window.
mod present;

/// Measuring the requested tile before it is built.
use measure::measure_requested;
/// The paint inputs and tile geometry that the tile worker and the tests also use.
pub(super) use paint::{Paint, job, stamp_for, tile_cap, viewport_of};
/// The gutter's wrap indicator,
///  painted before the frame is presented.
use present::ensure_sign;
/// The frame a tile swap or a render hands to the window.
pub(super) use present::{Frame, show};

/// Convert a toolkit palette color to raster input without losing alpha.
fn rgba(color: slint::Color) -> [u8; 4] {
    return [color.red(), color.green(), color.blue(), color.alpha()];
}

/// What:
///  `[u8; 4]` is a fixed array of four bytes,
///  red,
///  green,
///  blue,
///  alpha;
///  `Color::from_argb_u8` builds a toolkit color from them,
/// alpha first,
/// and `Brush::from` wraps it as the fill type window properties hold.
/// Why:
///  Inks chosen by native code are handed to the markup as window properties.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function brush([red, green, blue, alpha]: Color): Brush;
/// ```
fn brush(color: [u8; 4]) -> slint::Brush {
    return slint::Brush::from(slint::Color::from_argb_u8(
        color[3], color[0], color[1], color[2],
    ));
}

/// The palette of the source glyphs and selection,
/// from the window's current colors.
pub(super) fn code_colors(window: &AppWindow) -> CodeColors {
    // The palette's selection ink follows the color scheme while its selection fill does not,
    // so the ink is chosen from the fill that is actually drawn behind the selected glyphs.
    return CodeColors {
        foreground: rgba(window.get_source_foreground().color()),
        selected: legible_ink(
            rgba(window.get_selection_fill().color()),
            rgba(window.get_selected_foreground().color()),
        ),
        dark: window.get_dark_scheme(),
    };
}

/// Render shared shaped rows,
///  releasing state before any Slint setter can reenter.
///
/// A tile whose only change is the reader's scroll is built by the tile worker,
///  and the displayed tile stays on screen until its result is swapped in.
/// A new wrap width is taken here,
/// within an allowance (`rewrap`):
/// the lines near the view are broken again,
///  the tile is painted for the new width,
/// and the frame that follows shows it;
///  a width whose lines do not fit the allowance is built by the tile worker like a scroll re-center.
/// Every other change builds the requested tile here,
/// as before.
pub(super) fn render(window: &AppWindow, state: &Rc<RefCell<State>>) {
    #[cfg(debug_assertions)]
    {
        state.borrow_mut().renders += 1;
    }
    let factor = window.window().scale_factor();
    let colors = code_colors(window);
    // Selected rows of the tree, the search results, and the location list sit on the same fill
    // as selected source text, so they are drawn in the same chosen ink.
    window.set_selected_row_ink(brush(colors.selected));
    // What: `luminance` is the WCAG lightness of the chosen ink, from 0 (black) to 1 (white); the
    //       `if ... else` picks one of two opaque colors as a value.
    // Why: A hovered or focused selected row is tinted. The tint must move the fill away from the ink,
    //      black under light ink and white under dark ink, or it would lower the contrast of the row's text.
    let tint = if luminance(colors.selected) > 0.5 {
        [0, 0, 0, 255]
    } else {
        [255, 255, 255, 255]
    };
    window.set_selected_row_tint(brush(tint));
    // The window names the line a language surface belongs to; the map places it.
    let anchor_line = window.get_language_anchor_line();
    let view_now = (
        -window.get_scroll_y(),
        window.get_viewport_width(),
        window.get_viewport_height(),
    );
    let mut current = state.borrow_mut();
    // The size limit of a tile depends on the display scale, so a new scale asks for the tile to be placed again.
    if current.scale != factor {
        current.scale = factor;
        let (offset, width, height) = view_now;
        place(&mut current, offset, width, height);
    }
    let sign = ensure_sign(&mut current, factor);
    // What: A new wrap width the viewport just reported is taken here, within an allowance, or left to the worker.
    // Why: The frame that follows must not show the old width's text in a window of the new width.
    let mut allowance = rewrap::begin(&mut current);
    let mut rewrap_started = None;
    if allowance.is_some() {
        rewrap_started = Some((
            Instant::now(),
            current.shaper.shaped_rows(),
            current.shaper.rebroken_rows(),
        ));
    }
    // No frame is painted against a map that no longer describes the displayed text and its annotations.
    // The step for a new width installs the blocks even while the reader scrolls: the frame needs them now.
    let mut scrolled = rows::settle(&mut current, factor, view_now, allowance.is_some());
    // The lines of the tile for the new width are measured and the view kept still, each line paid for first.
    let mut measured_here = None;
    if let Some(spending) = allowance.as_mut() {
        let outcome = measure_requested(&mut current, factor, view_now, scrolled, Some(spending));
        scrolled = outcome.scrolled;
        if !outcome.complete {
            rewrap::defer(&mut current, "the lines of the tile are over the allowance");
        } else if !rewrap::raster_fits(&current, outcome.bounds, factor) {
            rewrap::defer(
                &mut current,
                "the raster of the settled tile is too large for one frame",
            );
        } else {
            measured_here = Some(outcome.bounds);
        }
    }
    if measured_here.is_none() {
        allowance = None;
    }
    let paint = Paint::new(window, &current);
    let requested = tile::requested(&current);
    let shown = current.shown;
    // The displayed tile's own inputs decide whether the frame changed; the requested window is judged separately.
    let shown_stamp = stamp_for(&current, shown, &paint);
    let mut same_inputs = current.frame_stamp.as_ref() == Some(&shown_stamp);
    // A new wrap width moves the blocks at once, while the displayed tile keeps the ones it was painted with until the
    // worker's tile for the new width arrives; that difference alone does not make the displayed tile stale.
    let differs_only_in_annotations = match current.frame_stamp.as_ref() {
        Some(held) => held.same_except_annotations(&shown_stamp),
        None => false,
    };
    if !same_inputs && shown.width != requested.width && differs_only_in_annotations {
        same_inputs = true;
    }
    let mut scene = None;
    let mut matches_changed = false;
    let mut notice = None;
    // What a new width cost: when it began, the lines and work charged, the lines shaped and broken again, and the tile.
    let mut rewrapped = None;
    if measured_here.is_none() && same_inputs && requested == shown {
        // What: Destructuring lends the cached frame for change and the map for reading at once.
        // Why: The materialized lines are unchanged, but rows above them may not be: the frame then keeps
        //      its pixels and only moves, and its find rectangles are handed over at their new place.
        let State {
            shaped, row_map, ..
        } = &mut *current;
        if let Some(view) = shaped.as_mut()
            && view.rebase(row_map)
        {
            matches_changed = true;
        }
    } else if measured_here.is_none() && same_inputs && current.scroll_wanted == Some(requested) {
        // A scroll re-center or a new wrap width: the worker builds the requested tile, and the displayed tile
        // stays on screen meanwhile, moved to where the map now puts it.
        let State {
            shaped, row_map, ..
        } = &mut *current;
        if let Some(view) = shaped.as_mut()
            && view.rebase(row_map)
        {
            matches_changed = true;
        }
        if current.tile_job != Some(requested) {
            let queued = job(&current, requested, &paint);
            if let Err(error) = tile::submit(window, state, &mut current, queued) {
                tracing::error!(%error, "source tile request failed");
                notice = Some(format!("{error:#}"));
            }
        }
    } else {
        // Any other change builds the requested tile here, and any tile still queued for the worker is dropped.
        let build_started = Instant::now();
        tile::cancel(&mut current);
        // The map learns the heights of the lines the tile holds first, so the tile and the map agree;
        // a new width has had that done already, within its allowance.
        let settled = match measured_here {
            Some(bounds) => bounds,
            None => {
                let outcome = measure_requested(&mut current, factor, view_now, scrolled, None);
                scrolled = outcome.scrolled;
                outcome.bounds
            }
        };
        let built_job = job(&current, settled, &paint);
        let result = build_tile(&mut current.shaper, built_job);
        match result {
            Ok(built) => {
                #[cfg(debug_assertions)]
                {
                    current.tiles_painted += 1;
                    current.rows_shaped += built.rows_shaped;
                }
                tracing::debug!(
                    elapsed_us = build_started.elapsed().as_micros() as u64,
                    first = settled.first,
                    count = settled.count,
                    clipped = settled.clip.is_some(),
                    "window thread measured and built a tile"
                );
                if let (Some(spending), Some((at, shaped, rebroken))) =
                    (allowance.as_ref(), rewrap_started)
                {
                    #[cfg(debug_assertions)]
                    {
                        current.rewrapped += 1;
                        current.rewrap_spent = (spending.lines(), spending.work());
                    }
                    rewrapped = Some((
                        at,
                        (spending.lines(), spending.work()),
                        (
                            current.shaper.shaped_rows() - shaped,
                            current.shaper.rebroken_rows() - rebroken,
                        ),
                        settled,
                    ));
                }
                // The shaper may have cut the frame to the raster limit: what is shown, and so what is wanted next,
                // is the clip it was really painted with.
                current.shown = built.bounds;
                current.clip = built.bounds.clip;
                let mut view = built.view;
                view.rebase(&current.row_map);
                current.shaped = Some(view);
                current.frame_stamp = Some(built.stamp);
                scene = Some(built.scene);
                // The first shown tile starts the worker, so a later scroll re-center does not wait for its start-up.
                if let Err(error) = tile::warm(window, state, &mut current) {
                    tracing::error!(%error, "source tile worker could not start");
                    notice = Some(format!("{error:#}"));
                }
                matches_changed = true;
            }
            Err(error) => {
                // What: The displayed picture and its rows stay as they are, and the next render builds again.
                // Why: A refused tile must not blank the source: the reader keeps the text last painted, with the
                //      failure named beside it.
                tracing::error!(%error, "source raster failed");
                current.frame_stamp = None;
                drop(current);
                window.set_error_message(SharedString::from(format!(
                    "Cannot render source: {error}"
                )));
                return;
            }
        }
    }
    // The worker measures the whole text at the wrap width in the background.
    tile::sync_pass(window, state, &mut current, factor);
    let frame = Frame {
        scene,
        matches_changed,
        scale: factor,
        scrolled,
        anchor_line,
        notice,
        sign,
    };
    drop(current);
    show(window, state, frame);
    // The minimap picture follows the text, the wrap width, and the rows; a plain scroll step changes none of them.
    super::minimap::note(window, state);
    // The whole step the window thread took for a new width, from the allowance to the last setter of the frame.
    if let Some((at, (charged_lines, charged_work), (shaped, rebroken), settled)) = rewrapped {
        tracing::debug!(
            elapsed_us = at.elapsed().as_micros() as u64,
            first = settled.first,
            count = settled.count,
            width = settled.width,
            charged_lines,
            charged_work,
            shaped,
            rebroken,
            "window thread re-wrapped for a new width"
        );
    }
}

/// Redraw cached source at a new system appearance or display scale without requiring user input.
pub(super) fn bind_appearance(window: &AppWindow, shared: &Rc<RefCell<State>>) {
    let theme_state = Rc::clone(shared);
    let theme_window = window.as_weak();
    window.on_theme_changed(move || {
        if let Some(active) = theme_window.upgrade() {
            render(&active, &theme_state);
        }
    });
    let scale_state = Rc::clone(shared);
    let scale_window = window.as_weak();
    window.on_scale_changed(move || {
        if let Some(active) = scale_window.upgrade() {
            render(&active, &scale_state);
        }
    });
}
