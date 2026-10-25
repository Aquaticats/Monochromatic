//! The step of a tick that asks for hover information where the pointer has rested,
//!  and that finds the character to ask about in the view that is drawn when the request is built.

/// The state,
///  the request record,
///  and the places the step reads.
use super::{Action, Language, Pending, pointer, surface};
/// The window and the source.
use crate::native::{AppWindow, State};
/// The identity of the displayed text.
use ide_app::language::identity::DocumentStamp;
/// What:
///  `Rc<RefCell<T>>` is the window's shared,
///  borrow-checked state.
/// Why:
///  The step reads the displayed rows and changes the popup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = { current: T };
/// ```
use std::{cell::RefCell, rc::Rc};

/// What:
///  Ask for hover information where the pointer has rested long enough.
/// Why:
///  Nothing is asked while the list,
///  the search overlay,
///  or a note is shown.
///  The character is found now,
///  from the pointer's distance from the top of the view and the offset that is drawn,
///  because the toolkit sends no pointer event for text that scrolls under a still pointer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rest(window, source, language, displayed: DocumentStamp): void
/// ```
pub(super) fn rest(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    language: &mut Language,
    displayed: DocumentStamp,
) {
    // A view that moved since the last tick (a wheel ease, a jump, rows arriving) puts other text under the pointer,
    //  so a rest that was not asked for yet starts again.
    language.rest.follow(pointer::View {
        offset: window.get_drawn_offset(),
        extent: window.get_content_extent(),
    });
    let blocked = window.get_search_open()
        || matches!(
            language.shown,
            surface::Shown::List { .. } | surface::Shown::Popup { note: true, .. }
        );
    if blocked {
        return;
    }
    if language.rest.idle() && matches!(language.shown, surface::Shown::Popup { pointer: true, .. })
    {
        language.shown = surface::dismiss(
            window,
            &language.shown,
            "the pointer rests over no character",
        );
    }
    let Some(spot) = language.rest.due() else {
        return;
    };
    // The character is resolved now,
    //  against the rows and the offset that are drawn,
    //  not where the pointer last moved.
    let offset = window.get_drawn_offset();
    let resolved = pointer::character_at(&source.borrow(), spot.below_top + offset, spot.x);
    tracing::debug!(
        ?spot,
        offset,
        ?resolved,
        "the resting pointer was resolved against the displayed view"
    );
    let Some(character) = resolved else {
        // Nothing is asked and nothing is marked asked:
        //  the rest is now one over no character,
        //  which a later move of the view starts again.
        language.rest.found_nothing();
        return;
    };
    language.rest.asked = true;
    // A character outside the displayed text cannot be asked about.
    if character > source.borrow().document.text().len_chars() {
        return;
    }
    language.hover = Some(Pending {
        action: Action::PointerHover,
        stamp: displayed,
        position: character,
        number: None,
        outcomes: Vec::new(),
        complete: false,
    });
}
