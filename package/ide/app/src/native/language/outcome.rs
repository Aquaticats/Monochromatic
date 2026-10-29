//! What a finished request does:
//!  navigate,
//!  list,
//!  show hover content,
//!  or explain why not.
//!
//! editord's Ctrl+B goes to the definition and,
//!  when the caret already is at it,
//!  finds its
//! references;
//!  one reference is opened directly,
//!  several are listed
//! (`package-paused/desktop-daemon/editord/src/client/app/lsp-goto-cursor.ts`,
//! `app/lsp-references.ts`).
//!  Several definitions are listed the same way here.

/// The state,
///  the action and request records,
///  and the helpers for each kind of result.
use super::{Action, Language, Pending, hover_text, message, surface, targets};
/// The window,
///  the source,
///  and the navigation that opens other files.
use crate::native::{AppWindow, State, navigation::Navigation};
/// Replies' results.
use ide_app::language::reply::{RequestOutcome, Target};
/// What:
///  `Rc<RefCell<T>>` is the window's shared,
///  borrow-checked state.
/// Why:
///  Results change the caret,
///  the popup,
///  and the list of this window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = { current: T };
/// ```
use std::{cell::RefCell, rc::Rc};

/// What:
///  Show a note for an explicit action,
///  and for a resting pointer only when the file's language support itself is missing or failing,
///  once for each displayed file;
///  otherwise only log the reason for a resting pointer.
/// Why:
///  Hovering is passive;
///  editord shows nothing when hover fails,
///  and a failure at one position is not worth interrupting the pointer for.
///  A file without working language support fails at every position,
///  and nothing else on the way to the pointer says why,
///  so the first rest on it names the cause.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function explain(window, source, language, pending: Pending, sentence: string): void
/// ```
pub(super) fn note(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    language: &mut Language,
    pending: &Pending,
    sentence: &str,
) {
    if !pending.action.explicit() {
        // What: Compare the displayed file's generation with the one already explained, then ask whether
        //       the answers are about the language support itself.
        // Why: The explanation is once per file; an empty answer or a server still starting is not one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const due = language.explained !== pending.stamp.file && statesSupportProblem(pending.outcomes, language.status);
        // ```
        let due = language.explained != Some(pending.stamp.file)
            && message::states_support_problem(&pending.outcomes, &language.status);
        if due {
            tracing::info!(
                sentence,
                "a resting pointer explains why hover is unavailable"
            );
            language.explained = Some(pending.stamp.file);
            // The popup follows the pointer like hover content and ends like it, but is drawn as a note.
            language.shown =
                surface::popup(window, source, pending.position, sentence, (true, true));
            return;
        }
        tracing::debug!(sentence, "pointer hover produced nothing to show");
        // A pointer popup describing another character must not stay after an empty answer.
        if matches!(language.shown, surface::Shown::Popup { pointer: true, .. }) {
            language.shown = surface::dismiss(window, &language.shown, "nothing to hover here");
        }
        return;
    }
    tracing::info!(action = ?pending.action, sentence, "language action cannot be satisfied");
    language.shown = surface::popup(window, source, pending.position, sentence, (true, false));
}

/// What:
///  The single target a definition reply names when that is the caret's own line.
/// Why:
///  editord's "already at definition":
///  the same file and the same line as the request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function atDefinition(targets: Target[], line: number): boolean
/// ```
fn at_definition(found: &[Target], line: usize) -> bool {
    // A slice pattern: exactly one element, which must be an openable target.
    let [Target::Open(only)] = found else {
        return false;
    };
    return only.same_document && only.line == line;
}

/// What:
///  Navigate to one target or list several;
///  explain a target that cannot be opened.
/// Why:
///  One place handles definitions,
///  references,
///  and a choice from the list alike.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function locations(window, source, navigation, language, pending, found: Target[]): void
/// ```
fn locations(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    navigation: &Rc<RefCell<Navigation>>,
    language: &mut Language,
    pending: &Pending,
    found: Vec<Target>,
) {
    if found.len() == 1 {
        if let Err(sentence) = targets::open(window, source, navigation, &found[0]) {
            note(window, source, language, pending, &sentence);
        }
        return;
    }
    let noun = if pending.action == Action::References {
        "references"
    } else {
        "definitions"
    };
    let title = format!("{} {noun}", found.len());
    let rows = targets::entries(&found, &language.root);
    language.shown = surface::list(window, source, pending.position, &title, found, rows);
}

/// What:
///  Act on a finished request.
///  `pending` is moved in;
///  it is complete.
/// Why:
///  Locations navigate or list,
///  hover content shows,
///  and everything else is explained.
///      A definition at the caret's own line asks for references,
///  as editord's Ctrl+B does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finish(window, source, navigation, language, pending: Pending): void
/// ```
pub(super) fn finish(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    navigation: &Rc<RefCell<Navigation>>,
    language: &mut Language,
    pending: Pending,
) {
    let mut found: Vec<Target> = Vec::new();
    let mut hover = None;
    for (_, outcome) in &pending.outcomes {
        match outcome {
            // `extend` appends copies of every target of this server's answer.
            RequestOutcome::Locations(each) => found.extend(each.iter().cloned()),
            RequestOutcome::Hover(text) if hover.is_none() => hover = Some(text.clone()),
            _ => {}
        }
    }
    tracing::debug!(action = ?pending.action, locations = found.len(), hover = hover.is_some(), "language request finished");
    // A resting pointer's answer never replaces the location list or a note shown since it was asked.
    if pending.action == Action::PointerHover
        && matches!(
            language.shown,
            surface::Shown::List { .. } | surface::Shown::Popup { note: true, .. }
        )
    {
        tracing::debug!("dropped a pointer hover answer under a list or a note");
        return;
    }
    if let Some(content) = hover {
        let text = hover_text::plain(&content);
        let pointer = pending.action == Action::PointerHover;
        language.shown = surface::popup(window, source, pending.position, &text, (false, pointer));
        return;
    }
    if found.is_empty() {
        let sentence = message::explain(pending.action, &pending.outcomes, &language.status);
        note(window, source, language, &pending, &sentence);
        return;
    }
    let line = source
        .borrow()
        .document
        .text()
        .char_to_line(pending.position);
    if pending.action == Action::Definition && at_definition(&found, line) {
        tracing::debug!(line, "already at the definition; asking for references");
        // `Some(Pending { .. })` replaces the finished request with its references request.
        language.action = Some(Pending {
            action: Action::References,
            number: None,
            outcomes: Vec::new(),
            complete: false,
            ..pending
        });
        return;
    }
    locations(window, source, navigation, language, &pending, found);
}

/// What:
///  Act on the list choice at `index`:
///  close the list and open its location.
/// Why:
///  Enter and a click both choose;
///  an unopenable row explains itself instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function choose(window, source, navigation, language, index: number): void
/// ```
pub(super) fn choose(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    navigation: &Rc<RefCell<Navigation>>,
    language: &mut Language,
    index: i32,
) {
    let surface::Shown::List {
        targets: listed,
        origin,
    } = &language.shown
    else {
        return;
    };
    // `usize::try_from` refuses a negative index instead of wrapping it.
    let Ok(row) = usize::try_from(index) else {
        return;
    };
    let Some(target) = listed.get(row).cloned() else {
        return;
    };
    let position = origin.caret;
    language.shown = surface::dismiss(window, &language.shown, "a location was chosen");
    if let Err(sentence) = targets::open(window, source, navigation, &target) {
        language.shown = surface::popup(window, source, position, &sentence, (true, false));
    }
}
