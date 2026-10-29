//! The window's reply checks on fabricated replies:
//!  request identity and displayed text.

/// The checks under test and the request record they compare against.
use super::{
    Action, Pending,
    guard::{Verdict, verdict},
};
/// What:
///  `Rope` is Helix's character-indexed text buffer.
/// Why:
///  The store groups what it accepts by the lines of the displayed text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Rope } from 'helix-core';
/// ```
use helix_core::Rope;
/// The snapshot store the poll fills and the source renderer reads.
use ide_app::annotation::Annotations;
/// Replies,
///  identities,
///  and snapshots as the worker produces them.
use ide_app::language::{
    diagnostics::DiagnosticsSnapshot,
    hints::HintsSnapshot,
    identity::{DocumentStamp, ServerIdentity},
    reply::{LanguageReply, RequestKind, RequestOutcome},
};
/// Snapshots arrive behind shared pointers.
use std::sync::Arc;

/// A hover request numbered 7 for file 3,
///  revision 2.
fn waiting() -> Pending {
    return Pending {
        action: Action::Hover,
        stamp: DocumentStamp {
            file: 3,
            revision: 2,
        },
        position: 5,
        number: Some(7),
        outcomes: Vec::new(),
        complete: false,
    };
}

/// A reply to request `number` of `kind` about `stamp`.
fn reply(number: u64, kind: RequestKind, stamp: DocumentStamp) -> LanguageReply {
    return LanguageReply {
        request: number,
        stamp,
        server: Some(ServerIdentity {
            name: "scripted-ls".to_string(),
            instance: 1,
        }),
        kind,
        position: 5,
        remaining: 0,
        outcome: RequestOutcome::Empty,
    };
}

/// The answer to the waiting request about the displayed text is applied.
#[test]
fn reply_to_the_waiting_request_about_the_displayed_text_is_applied() {
    let pending = waiting();
    let answer = reply(7, RequestKind::Hover, pending.stamp);
    assert_eq!(verdict(&pending, pending.stamp, &answer), Verdict::Apply);
}

/// A late reply to an earlier request,
///  or a reply of another kind,
///  is never applied.
#[test]
fn reply_to_another_request_is_not_applied() {
    let pending = waiting();
    let earlier = reply(6, RequestKind::Hover, pending.stamp);
    assert_eq!(
        verdict(&pending, pending.stamp, &earlier),
        Verdict::OtherRequest,
        "a reply to an earlier request was applied"
    );
    let other_kind = reply(7, RequestKind::Definition, pending.stamp);
    assert_eq!(
        verdict(&pending, pending.stamp, &other_kind),
        Verdict::OtherRequest,
        "a reply of another kind was applied"
    );
    let unsent = Pending {
        number: None,
        ..waiting()
    };
    assert_eq!(
        verdict(&unsent, pending.stamp, &earlier),
        Verdict::OtherRequest,
        "a reply was applied to a request that was never sent"
    );
}

/// A reply about a revision a reload replaced,
///  or about another file,
///  is never applied,
/// even when the handle's fence has not caught up yet.
#[test]
fn reply_about_text_no_longer_displayed_is_not_applied() {
    let pending = waiting();
    let answer = reply(7, RequestKind::Hover, pending.stamp);
    let reloaded = DocumentStamp {
        file: 3,
        revision: 3,
    };
    assert_eq!(
        verdict(&pending, reloaded, &answer),
        Verdict::OtherText,
        "a reply for the previous revision was applied after a reload"
    );
    let switched = DocumentStamp {
        file: 4,
        revision: 0,
    };
    assert_eq!(
        verdict(&pending, switched, &answer),
        Verdict::OtherText,
        "a reply for the previous file was applied after a file switch"
    );
}

/// An empty hints snapshot for `stamp`.
fn hints_for(stamp: DocumentStamp) -> Arc<HintsSnapshot> {
    return Arc::new(HintsSnapshot {
        stamp,
        first_line: 0,
        last_line: 1,
        hints: Vec::new(),
    });
}

/// An empty diagnostics snapshot for `stamp`.
fn diagnostics_for(stamp: DocumentStamp) -> Arc<DiagnosticsSnapshot> {
    return Arc::new(DiagnosticsSnapshot {
        stamp,
        groups: Vec::new(),
    });
}

/// Snapshots are stored only for the displayed text and handed out only for the stamp asked with.
#[test]
fn snapshots_for_other_text_are_not_stored_or_handed_out() {
    let displayed = DocumentStamp {
        file: 3,
        revision: 2,
    };
    let older = DocumentStamp {
        file: 3,
        revision: 1,
    };
    let mut store = Annotations::default();
    // `Rope::from` copies the literal into a text buffer; `&text` lends it to the store.
    let text = Rope::from("one line");
    assert!(
        !store.accept_hints(displayed, &text, hints_for(older)),
        "hints for an earlier revision were stored"
    );
    assert!(
        !store.accept_diagnostics(displayed, &text, diagnostics_for(older)),
        "diagnostics for an earlier revision were stored"
    );
    assert!(store.accept_hints(older, &text, hints_for(older)));
    assert!(store.accept_diagnostics(older, &text, diagnostics_for(older)));
    assert!(
        store.hints(displayed).is_none(),
        "hints for an earlier revision were handed out"
    );
    assert!(
        store.diagnostics(displayed).is_none(),
        "diagnostics for an earlier revision were handed out"
    );
    assert!(store.hints(older).is_some());
    assert!(store.diagnostics(older).is_some());
}
