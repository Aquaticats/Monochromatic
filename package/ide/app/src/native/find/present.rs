//! Derive the find bar's count text and the active match from accepted results and the selection.

/// The displayed document,
///  its identity,
///  and the accepted results live in one source state.
use crate::native::State;
/// A match is a source character range.
use ide_app::find::FindRange;
/// Count text and active-match rules are pure library functions with their own tests.
use ide_app::find_navigation::{FindStatus, active, positioned_matches, status};

/// What:
///  `Option<T>` is a value or nothing;
///  here "nothing" has a meaning for each field.
/// Why:
///  Rendering decides in one place what the bar says and which match is drawn as active.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Presented = { status?: FindStatus; active?: FindRange };
/// ```
pub(in crate::native) struct Presented {
    /// `None` keeps the previous count text while results for a new file or revision are pending.
    pub(in crate::native) status: Option<FindStatus>,
    /// The match exactly covered by the selection,
    ///  drawn as the selection with a heavier boundary.
    pub(in crate::native) active: Option<FindRange>,
}

/// Describe find for the displayed document without changing any state.
pub(in crate::native) fn present(current: &State) -> Presented {
    if current.find.is_none() {
        // What: `Some(FindStatus::default())` wraps an empty status; `None` means no active match.
        // Why: A closed bar or empty find text shows no count and no highlights.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { status: { label: '', detail: '', noMatch: false }, active: undefined };
        // ```
        return Presented {
            status: Some(FindStatus::default()),
            active: None,
        };
    }
    // What: `let ... else` extracts matches whose positions describe this document, or leaves early.
    // Why: Results for another file or revision must not be counted or drawn.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const matches = positionedMatches(current.find, file, revision); if (!matches) return {};
    // ```
    let Some(matches) = positioned_matches(
        &current.find,
        current.file_generation,
        current.document.revision(),
    ) else {
        return Presented {
            status: None,
            active: None,
        };
    };
    let position = current.document.position();
    let start = position.anchor.min(position.head);
    let end = position.anchor.max(position.head);
    let index = active(&matches.ranges, start, end);
    let mut range = None;
    if let Some(found) = index {
        range = Some(matches.ranges[found]);
    }
    return Presented {
        status: Some(status(matches, index)),
        active: range,
    };
}
