//! The pull side of the store:
//!  what a full or unchanged pull answer does to the stored sets,
//!  and which result identifier the next pull request carries.

/// The store and the types of its records.
use super::{DiagnosticStore, StoredSet};
/// Pull answers carry their freshness;
///  their items are converted later.
use crate::language::diagnostics::Freshness;
/// Sets are keyed by server process and stamped with the text they were accepted for.
use crate::language::identity::{DocumentStamp, ServerIdentity};
/// The protocol's diagnostic record and each server's column unit.
use helix_lsp::{OffsetEncoding, lsp};

/// What:
///  Forget the pull result identifier of `server`.
/// Why:
///  A set replaced,
///  removed,
///  or re-stamped for a server makes its earlier identifier obsolete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// resultIds = resultIds.filter(([known]) => !same(known, server));
/// ```
pub(super) fn forget_result_id(
    result_ids: &mut Vec<(ServerIdentity, String)>,
    server: &ServerIdentity,
) {
    let held = std::mem::take(result_ids);
    for entry in held {
        if &entry.0 != server {
            result_ids.push(entry);
        }
    }
}

/// The pull-answer methods of the store.
impl DiagnosticStore {
    /// What:
    ///  Store a full pull answer;
    ///  false when it answers another revision or file.
    ///       `Option<String>` is the result identifier the server may attach.
    /// Why:
    ///  A pull answer is fenced like every other reply:
    ///  only the displayed text counts.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// pulledFull(server, encoding, stamp, resultId: string | undefined, items): boolean
    /// ```
    pub(crate) fn pulled_full(
        &mut self,
        server: &ServerIdentity,
        encoding: OffsetEncoding,
        stamp: DocumentStamp,
        result_id: Option<String>,
        items: Vec<lsp::Diagnostic>,
    ) -> bool {
        let outdated = match self.document.as_ref() {
            Some(tracked) => tracked.stamp != stamp,
            None => true,
        };
        if outdated {
            return false;
        }
        let held = std::mem::take(&mut self.sets);
        for set in held {
            if !set.pulled || &set.server != server {
                self.sets.push(set);
            }
        }
        forget_result_id(&mut self.result_ids, server);
        if let Some(identifier) = result_id {
            self.result_ids.push((server.clone(), identifier));
        }
        self.sets.push(StoredSet {
            server: server.clone(),
            pulled: true,
            stamp,
            freshness: Freshness::Pulled,
            encoding,
            items,
        });
        return true;
    }

    /// The server said its previous pull answer still holds:
    ///  re-stamp it for the displayed text.
    pub(crate) fn pulled_unchanged(
        &mut self,
        server: &ServerIdentity,
        stamp: DocumentStamp,
        result_id: String,
    ) -> bool {
        let outdated = match self.document.as_ref() {
            Some(tracked) => tracked.stamp != stamp,
            None => true,
        };
        if outdated {
            return false;
        }
        // `iter_mut` hands out modifiable borrows so the stored set can be re-stamped in place.
        for set in self.sets.iter_mut() {
            if set.pulled && &set.server == server {
                set.stamp = stamp;
                forget_result_id(&mut self.result_ids, server);
                self.result_ids.push((server.clone(), result_id));
                return true;
            }
        }
        return false;
    }

    /// What:
    ///  The identifier to send with the next pull request,
    ///  or nothing.
    /// Why:
    ///  It is offered only while the set it refers to is still stored;
    ///  otherwise an
    ///      "unchanged" answer would leave nothing to show.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// previousResultId(server: ServerIdentity): string | undefined
    /// ```
    pub(crate) fn previous_result_id(&self, server: &ServerIdentity) -> Option<String> {
        let mut stored = false;
        for set in self.sets.iter() {
            if set.pulled && &set.server == server {
                stored = true;
                break;
            }
        }
        if !stored {
            // `None` is the "nothing" variant of `Option`.
            return None;
        }
        for (known, identifier) in &self.result_ids {
            if known == server {
                return Some(identifier.clone());
            }
        }
        return None;
    }
}
