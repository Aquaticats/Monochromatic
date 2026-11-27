//! The holds on a server's unversioned pushes:
//!  which of them an answer or a timer ends,
//!  and the set each kept for the revision it belongs to.

/// The store these methods extend,
///  the accepted set a hold shows when it ends,
///  and how that set's freshness is recorded.
use super::{
    DiagnosticStore,
    Freshness,
    Kept,
    StoredSet,
};
/// The server process whose pushes are kept back,
///  and the text the store is showing.
use crate::language::identity::{
    DocumentStamp,
    ServerIdentity,
};

/// What:
///  Which holds `end_holds` ends:
///  those of one server,
///  or those started by one reload.
/// Why:
///  The two callers name their holds differently,
///  and a named choice carries the value the test needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Ending = { kind: 'server'; server: ServerIdentity } | { kind: 'serial'; serial: number };
/// ```
enum Ending<'a> {
    /// The holds on this server.
    Server(
        /// The server whose holds end.
        &'a ServerIdentity,
    ),
    /// The holds started by the reload with this number.
    Serial(
        /// The number of the reload.
        u64,
    ),
}

/// Which holds a choice names.
impl Ending<'_> {
    /// Whether `hold` is one of the holds this choice names.
    fn names(
        &self,
        hold: &Hold,
    ) -> bool {
        return match self {
            Ending::Server(server) => &hold.server == *server,
            Ending::Serial(serial) => hold.serial == *serial,
        };
    }
}

/// An open hold on one server's unversioned pushes.
pub(super) struct Hold {
    /// Server whose unversioned sets are kept back.
    pub(super) server: ServerIdentity,
    /// Number of the reload that opened the hold;
    ///  `u64` never wraps in practice.
    pub(super) serial: u64,
    /// What:
    ///  The latest unversioned set the server pushed during the hold,
    ///  or nothing.
    ///       `Option<Kept>` is "a kept set,
    ///  or nothing".
    /// Why:
    ///  A set pushed during the hold may be the server's only set for the new text,
    ///  for
    ///      example from a server that pushes once per change.
    ///  Dropping it would leave the
    ///      displayed file without diagnostics until the server's next push;
    ///  it is shown when
    ///      the hold ends instead.
    ///  A hold belongs to one revision,
    ///  because every reload replaces
    ///      all holds,
    ///  so a kept set is always shown against the text it arrived for.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// kept?: Kept;
    /// ```
    pub(super) kept: Option<Kept>,
}

/// The hold operations,
///  which one ends when,
///  and what a hold kept.
impl DiagnosticStore {
    fn end_holds(
        &mut self,
        ending: Ending<'_>,
    ) -> usize {
        let Some(tracked) = self.document.as_ref() else {
            self.holds.clear();
            return 0;
        };
        let stamp = tracked.stamp;
        let mut ended = 0;
        // `Vec::new()` creates an empty list that collects the holds that stay open.
        let mut open: Vec<Hold> = Vec::new();
        for hold in self.holds.drain(..) {
            if !ending.names(&hold) {
                open.push(hold);
                continue;
            }
            ended += 1;
            // `if let Some(x) = ...` runs the block only when the hold kept a set.
            if let Some(kept) = hold.kept {
                tracing::debug!(server = %hold.server.name, "showing the set pushed during the hold that just ended");
                let held = std::mem::take(&mut self.sets);
                for set in held {
                    if set.pulled || set.server != hold.server {
                        self.sets.push(set);
                    }
                }
                self.sets.push(StoredSet {
                    server: hold.server,
                    pulled: false,
                    stamp,
                    freshness: Freshness::Unversioned,
                    encoding: kept.encoding,
                    items: kept.items,
                });
            }
        }
        self.holds = open;
        return ended;
    }

    /// True while the server's unversioned pushes are being kept back.
    ///  Only tests ask;
    ///  `push`
    /// finds the hold itself,
    ///  because it also changes it.
    #[cfg(test)]
    pub(crate) fn is_held(
        &self,
        server: &ServerIdentity,
    ) -> bool {
        // `iter().any(...)` is `Array.prototype.some`; `&self.holds` is only read.
        for hold in self.holds.iter() {
            if &hold.server == server {
                return true;
            }
        }
        return false;
    }

    /// What:
    ///  The server answered a request sent for `stamp`;
    ///  returns true when that ended a hold.
    /// Why:
    ///  An answer to a request sent after the change shows the server processed the change,
    ///      provided servers handle messages in order,
    ///  which the protocol does not promise.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// answered(server: ServerIdentity, stamp: DocumentStamp): boolean
    /// ```
    pub(crate) fn answered(
        &mut self,
        server: &ServerIdentity,
        stamp: DocumentStamp,
    ) -> bool {
        // What: `as_ref()` borrows the value inside the `Option`; `is_none_or` is true for
        //       "nothing" or when the closure accepts the value.
        // Why: An answer about an older revision proves nothing about the latest change.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.document?.stamp !== stamp) return false;
        // ```
        let outdated = match self.document.as_ref() {
            Some(tracked) => tracked.stamp != stamp,
            None => true,
        };
        if outdated {
            return false;
        }
        // The holds of this server end; `&hold.server == server` compares the borrowed identities.
        return self.end_holds(Ending::Server(server)) > 0;
    }

    /// The fixed delay after reload number `serial` passed;
    ///  returns true when a hold ended.
    pub(crate) fn hold_expired(
        &mut self,
        serial: u64,
    ) -> bool {
        return self.end_holds(Ending::Serial(serial)) > 0;
    }
}
