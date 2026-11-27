//! What a lifecycle event and a pushed set do to the stored sets:
//!  a file displayed or closed,
//!  a reload and the holds it opens,
//!  one judged `publishDiagnostics` notification,
//!  and a server process that ended.

/// The store's own hold type.
use super::holds::Hold;
/// Dropping a server's pull result identifier is shared with the pull answers.
use super::pulled::forget_result_id;
/// The store these methods extend,
///  the push verdict the caller logs,
///  how a set's freshness is recorded,
///  the document it tracks,
///  the set it stores,
///  and the set a hold keeps.
use super::{
    DiagnosticStore,
    Freshness,
    Kept,
    PushVerdict,
    StoredSet,
    Tracked,
};
/// The server process a set belongs to,
///  and the text it was accepted for.
use crate::language::identity::{
    DocumentStamp,
    ServerIdentity,
};
/// Each server's column unit,
///  and the protocol's diagnostic record.
use helix_lsp::{
    OffsetEncoding,
    lsp,
};

/// What a lifecycle event and a pushed set do to the stored sets.
impl DiagnosticStore {
    /// Create an empty store with no displayed document.
    pub(crate) fn new() -> Self {
        return Self {
            document: None,
            sets: Vec::new(),
            holds: Vec::new(),
            result_ids: Vec::new(),
            serial: 0,
        };
    }

    /// A file is displayed:
    ///  forget everything about the previous one.
    pub(crate) fn open(
        &mut self,
        stamp: DocumentStamp,
        version: i32,
    ) {
        self.close();
        // `Some(...)` wraps the record in the "value present" variant of `Option`.
        self.document = Some(Tracked {
            stamp,
            version,
        });
    }

    /// No file is displayed:
    ///  a file switch or close clears every set,
    ///  hold,
    ///  and result identifier.
    pub(crate) fn close(&mut self) {
        self.document = None;
        self.sets.clear();
        self.holds.clear();
        self.result_ids.clear();
    }

    /// What:
    ///  Record an accepted reload and return its number.
    ///  `&[ServerIdentity]` lends a list
    ///       of the servers that hold the document open.
    /// Why:
    ///  Every pushed set of the previous revision is invalid at once,
    ///  and each server's
    ///      unversioned pushes are kept back until it has demonstrably processed the change or
    ///      the fixed delay passed.
    ///  Sets kept for the previous revision are dropped with its holds.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reload(stamp: DocumentStamp, version: number, servers: ServerIdentity[]): number {
    ///   this.sets = this.sets.filter(set => set.pulled);
    ///   this.holds = servers.map(server => ({ server, serial: ++this.serial }));
    /// }
    /// ```
    pub(crate) fn reload(
        &mut self,
        stamp: DocumentStamp,
        version: i32,
        servers: &[ServerIdentity],
    ) -> u64 {
        self.serial += 1;
        self.document = Some(Tracked {
            stamp,
            version,
        });
        // What: `retain` keeps only the items for which the closure returns true; `|set|` is a
        //       closure parameter, like `set =>` in an arrow function.
        // Why: Pulled sets stay stored but unshown, because an "unchanged" answer may re-stamp them.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.sets = this.sets.filter(set => set.pulled);
        // ```
        let held = std::mem::take(&mut self.sets);
        for set in held {
            if set.pulled {
                self.sets.push(set);
            }
        }
        self.holds.clear();
        for server in servers {
            self.holds.push(Hold {
                // `clone` copies the identity so the hold owns its own value.
                server: server.clone(),
                serial: self.serial,
                kept: None,
            });
        }
        return self.serial;
    }

    /// What:
    ///  Judge one `publishDiagnostics` notification for the displayed file and store it when
    ///       accepted.
    ///  `version` is the optional document version the server attached;
    ///       `line_count` is the number of lines of the displayed text.
    /// Why:
    ///  A versioned set is exact;
    ///  an unversioned one can only be shown conservatively.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// push(server, encoding, version: number | undefined, items, lineCount: number): PushVerdict
    /// ```
    pub(crate) fn push(
        &mut self,
        server: &ServerIdentity,
        encoding: OffsetEncoding,
        version: Option<i32>,
        items: Vec<lsp::Diagnostic>,
        line_count: usize,
    ) -> PushVerdict {
        // `let Some(x) = option else { ... }` binds the inner value or leaves the function.
        let Some(tracked) = self.document.as_ref() else {
            return PushVerdict::NoDocument;
        };
        let stamp = tracked.stamp;
        let freshness = match version {
            Some(sent) => {
                if sent != tracked.version {
                    return PushVerdict::WrongVersion;
                }
                Freshness::Versioned
            }
            None => {
                for item in &items {
                    // What: `usize::try_from` converts the protocol's `u32`; `is_ok_and` is true only
                    //       for a successful conversion whose value passes the closure's test.
                    // Why: A range starting past the last line belongs to another revision.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // if (item.range.start.line >= lineCount) return 'lineOutOfRange';
                    // ```
                    let exists = match usize::try_from(item.range.start.line) {
                        Ok(line) => line < line_count,
                        Err(_) => false,
                    };
                    if !exists {
                        return PushVerdict::LineOutOfRange;
                    }
                }
                // What: `iter_mut().find(...)` lends the server's open hold for changing, if any.
                // Why: During the hold the set is kept, replacing an earlier kept one, and shown
                //      when the hold ends. The line check already ran against the displayed text,
                //      which is the text the hold belongs to.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const hold = this.holds.find(hold => same(hold.server, server));
                // if (hold) { hold.kept = { encoding, items }; return 'held'; }
                // ```
                let mut open_hold = None;
                for (position, hold) in self.holds.iter().enumerate() {
                    if &hold.server == server {
                        open_hold = Some(position);
                        break;
                    }
                }
                if let Some(position) = open_hold {
                    self.holds[position].kept = Some(Kept {
                        encoding,
                        items,
                    });
                    return PushVerdict::Held;
                }
                Freshness::Unversioned
            }
        };
        // A versioned set accepted during the hold is newer than anything the hold kept.
        for hold in self.holds.iter_mut() {
            if &hold.server == server {
                hold.kept = None;
            }
        }
        // Any later set from the same server replaces its earlier pushed set.
        let held = std::mem::take(&mut self.sets);
        for set in held {
            if set.pulled || &set.server != server {
                self.sets.push(set);
            }
        }
        self.sets.push(StoredSet {
            server: server.clone(),
            pulled: false,
            stamp,
            freshness,
            encoding,
            items,
        });
        if freshness == Freshness::Versioned {
            return PushVerdict::AcceptedVersioned;
        }
        return PushVerdict::AcceptedUnversioned;
    }

    /// A server process ended:
    ///  its sets,
    ///  hold,
    ///  and result identifier are removed.
    pub(crate) fn server_exited(
        &mut self,
        server: &ServerIdentity,
    ) {
        let sets = std::mem::take(&mut self.sets);
        for set in sets {
            if &set.server != server {
                self.sets.push(set);
            }
        }
        let holds = std::mem::take(&mut self.holds);
        for hold in holds {
            if &hold.server != server {
                self.holds.push(hold);
            }
        }
        forget_result_id(&mut self.result_ids, server);
    }
}
