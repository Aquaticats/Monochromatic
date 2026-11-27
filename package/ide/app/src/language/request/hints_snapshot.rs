//! The latest-value hint snapshot:
//!  every server's stored hints,
//!  merged in the order the interface draws them.

/// One hint as a server sent it,
///  and the snapshot the interface draws.
use super::super::hints::{
    HintsSnapshot,
    InlayHint,
};
/// The session whose stored hints these are.
use super::super::session::Session;

/// Orders inlay hints by the character offset they are drawn before.
fn by_position(
    left: &InlayHint,
    right: &InlayHint,
) -> std::cmp::Ordering {
    return left.position.cmp(&right.position);
}

/// What:
///  Merge the stored hints of every server into the latest-value snapshot,
///  or nothing when
///       no server has answered for the displayed text.
/// Why:
///  The interface draws one ordered list regardless of how many servers contributed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hintsSnapshot(session: Session): HintsSnapshot | undefined
/// ```
pub(in crate::language) fn hints_snapshot(session: &Session) -> Option<HintsSnapshot> {
    // The trailing `?` returns `None` when no document is displayed.
    let document = session.document.as_ref()?;
    if session.hints.is_empty() {
        return None;
    }
    let mut merged: Vec<InlayHint> = Vec::new();
    for (_, hints) in &session.hints {
        // `extend` appends copies of one server's hints.
        merged.extend(hints.iter().cloned());
    }
    // `sort_by` is stable: equal positions keep server order.
    merged.sort_by(by_position);
    return Some(HintsSnapshot {
        stamp: document.stamp,
        first_line: session.hint_lines.0,
        last_line: session.hint_lines.1,
        hints: merged,
    });
}
