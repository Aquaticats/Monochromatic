//! The window's models and setters for the vertical mapping:
//!  the persistent row models,
//!  updating them row by row,
//!  and handing a collected `Placement` to the window.

/// The `Placement` collected while the state was borrowed.
use super::Placement;
/// The window that receives the models and the anchor.
use crate::native::AppWindow;
/// `Model` gives a list its `row_count` and row access,
///  `ModelRc` hands a model to the window,
///  and `VecModel` is the toolkit's growable list model.
use slint::{Model, ModelRc, VecModel};
/// `Rc` shares the persistent model between the state and the window.
use std::rc::Rc;

/// What:
///  Make `model` hold exactly `wanted`,
///  changing only rows that differ.
///  `T` is the row type;
///  `Copy` lets a
///       row be duplicated like a number and `PartialEq` lets two rows be compared.
/// Why:
///  Updating in place changes numbers,
///  positions,
///  and letters while scrolling without rebuilding elements.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function update<T>(model: ArrayModel<T>, wanted: T[]): void;
/// ```
fn update<T: Copy + PartialEq + 'static>(model: &VecModel<T>, wanted: &[T]) {
    for (index, row) in wanted.iter().enumerate() {
        if index >= model.row_count() {
            model.push(*row);
        } else if model.row_data(index) != Some(*row) {
            model.set_row_data(index, *row);
        }
    }
    while model.row_count() > wanted.len() {
        model.remove(model.row_count() - 1);
    }
}

/// What:
///  Hand a [`Placement`] to the window.
///  The line-number and severity-mark models are updated row by row.
/// Why:
///  The window draws a number and,
///  where a diagnostic starts,
///  a letter on each materialized code row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function present(window: AppWindow, placement: Placement): void;
/// ```
pub(in crate::native) fn present(window: &AppWindow, placement: Placement) {
    window.set_content_extent(placement.extent);
    update(&placement.top_model, &placement.tops);
    update(&placement.number_model, &placement.numbers);
    update(&placement.mark_model, &placement.marks);
    // What: `Rc::clone` copies the pointer; the window compares model pointers, so handing over the same
    //       model again changes nothing.
    // Why: The first render installs the model; later ones only confirm it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // window.rowTops = current.rowTops;
    // ```
    window.set_row_tops(ModelRc::from(Rc::clone(&placement.top_model)));
    window.set_row_numbers(ModelRc::from(Rc::clone(&placement.number_model)));
    window.set_row_marks(ModelRc::from(Rc::clone(&placement.mark_model)));
    // `try_from` turns the count into the toolkit's 32-bit integer; a count past its range reads as its largest.
    window.set_line_count(i32::try_from(placement.lines).unwrap_or(i32::MAX));
    let (top, height) = placement.anchor;
    window.set_language_anchor_y(top);
    window.set_language_anchor_height(height);
}

/// A fresh row-top model for a new source state.
pub(in crate::native) fn top_model() -> Rc<VecModel<f32>> {
    return Rc::new(VecModel::from(Vec::new()));
}

/// A fresh model of integers for a new source state:
///  line numbers and severity marks.
pub(in crate::native) fn mark_model() -> Rc<VecModel<i32>> {
    return Rc::new(VecModel::from(Vec::new()));
}
