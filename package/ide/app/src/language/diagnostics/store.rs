//! Which pushed and pulled diagnostic sets describe the displayed revision.

/// How a set's freshness was established.
use super::Freshness;
/// Sets are keyed by server process and stamped with the text they were accepted for.
use crate::language::identity::{
    DocumentStamp,
    ServerIdentity,
};
/// Each server's column unit,
///  and the protocol's diagnostic record a set keeps unconverted.
use helix_lsp::{
    OffsetEncoding,
    lsp,
};
/// The hold that keeps one server's unversioned pushes back.
use holds::Hold;

/// The holds on a server's unversioned pushes.
mod holds;
/// What a pull answer does to the stored sets.
mod pulled;
/// What a lifecycle event and a pushed set do to the stored sets.
mod pushes;
/// The latest-value snapshot of the displayed text.
mod snapshot;

/// What:
///  A closed set of names for what happened to one pushed set.
/// Why:
///  The worker logs the reason,
///  and tests assert the exact rule that applied.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PushVerdict = 'acceptedVersioned' | 'acceptedUnversioned' | 'wrongVersion'
///                  | 'held' | 'lineOutOfRange' | 'noDocument';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PushVerdict {
    /// The set's version equals the displayed revision's protocol version.
    AcceptedVersioned,
    /// No version was sent,
    ///  no hold is open,
    ///  and every range starts on an existing line.
    AcceptedUnversioned,
    /// The set names another protocol version of the document.
    WrongVersion,
    /// No version was sent and the server has not yet answered anything since the last reload:
    /// the set is kept,
    ///  unshown,
    ///  and shown when the hold ends unless a newer set arrived first.
    Held,
    /// No version was sent and a range starts past the last line of the displayed text.
    LineOutOfRange,
    /// No file is displayed.
    NoDocument,
}

/// The displayed document as the store needs it.
struct Tracked {
    /// File generation and revision of the displayed text.
    stamp: DocumentStamp,
    /// Protocol version of that revision.
    ///  `i32` is the signed 32-bit integer the protocol
    /// mandates (siblings:
    ///  `u32`,
    ///  `i64`).
    version: i32,
}

/// One accepted set from one server on one channel.
struct StoredSet {
    /// Server process that sent it.
    server: ServerIdentity,
    /// True for an answer to a pull request;
    ///  false for a pushed notification.
    pulled: bool,
    /// Text the set was accepted for.
    stamp: DocumentStamp,
    /// How its freshness was established.
    freshness: Freshness,
    /// Column unit of the sending server.
    encoding: OffsetEncoding,
    /// What:
    ///  `Vec<lsp::Diagnostic>` keeps the protocol records unconverted.
    /// Why:
    ///  A later "unchanged" pull answer re-stamps the set for a new revision,
    ///  and the ranges
    ///      must then be converted against that revision's text.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// items: lsp.Diagnostic[];
    /// ```
    items: Vec<lsp::Diagnostic>,
}

/// The latest unversioned set one server pushed while its hold was open.
struct Kept {
    /// Column unit of the sending server.
    encoding: OffsetEncoding,
    /// What:
    ///  The protocol records,
    ///  unconverted,
    ///  as `Vec<lsp::Diagnostic>` (a growable list).
    /// Why:
    ///  They are converted when shown,
    ///  against the text of the revision the hold belongs to.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// items: lsp.Diagnostic[];
    /// ```
    items: Vec<lsp::Diagnostic>,
}

/// What:
///  The store's whole state.
///  `Option<Tracked>` is "a tracked document,
///  or nothing".
/// Why:
///  Push and pull arrive on different paths at different times;
///  one owner decides which
///      sets may be shown together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class DiagnosticStore { document?: Tracked; sets: StoredSet[] = []; holds: Hold[] = [];
///                         resultIds: [ServerIdentity, string][] = []; serial = 0; }
/// ```
pub(crate) struct DiagnosticStore {
    /// Displayed document,
    ///  absent when no file is displayed.
    document: Option<Tracked>,
    /// At most one pushed and one pulled set per server process.
    sets: Vec<StoredSet>,
    /// Servers whose unversioned pushes are kept back for now,
    ///  with the latest set each pushed.
    holds: Vec<Hold>,
    /// Result identifiers servers attached to their last full pull answer.
    result_ids: Vec<(ServerIdentity, String)>,
    /// Count of reloads,
    ///  used to tell a late hold timer from a current one.
    serial: u64,
}
