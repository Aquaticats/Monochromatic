//! The window's own checks on every reply,
//!  after the handle's fence.
//!
//! The handle's fence moves only when an open or reload command was queued,
//!  so for one tick a
//! reply for the previous text can pass it.
//!  These checks compare against the document the window
//! actually shows and against the request the window is waiting for.

/// The request the window waits for.
use super::Pending;
/// Identities and replies.
use ide_app::language::{identity::DocumentStamp, reply::LanguageReply};

/// What:
///  Whether a reply may be applied,
///  and if not,
///  which identity moved on.
///  A plain `enum`.
/// Why:
///  Logs and tests must say which rule dropped a reply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Verdict = 'apply' | 'otherRequest' | 'otherText';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Verdict {
    /// The reply answers the waiting request about the displayed text.
    Apply,
    /// The reply answers another request:
    ///  an earlier one,
    ///  or one of another kind.
    OtherRequest,
    /// The reply describes text that is no longer displayed:
    ///  another file or an earlier revision.
    OtherText,
}

/// What:
///  Decide whether `reply` may be applied for `pending` while `displayed` is shown.
///       `&Pending` and `&LanguageReply` are lent read-only;
///  `DocumentStamp` is copied.
/// Why:
///  A late hover must not appear after the next one was asked for,
///  and a definition must not
///      move the caret by offsets of a revision the reload already replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function verdict(pending: Pending, displayed: DocumentStamp, reply: LanguageReply): Verdict {
///   if (pending.number !== reply.request || pending.action.kind !== reply.kind) return 'otherRequest';
///   if (!equal(reply.stamp, displayed) || !equal(pending.stamp, displayed)) return 'otherText';
///   return 'apply';
/// }
/// ```
pub(super) fn verdict(
    pending: &Pending,
    displayed: DocumentStamp,
    reply: &LanguageReply,
) -> Verdict {
    // `Some(reply.request)` wraps the number so it compares with the optional pending number.
    if pending.number != Some(reply.request) || pending.action.kind() != reply.kind {
        return Verdict::OtherRequest;
    }
    if reply.stamp != displayed || pending.stamp != displayed {
        return Verdict::OtherText;
    }
    return Verdict::Apply;
}
