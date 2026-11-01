//! Which pushed and pulled diagnostic sets describe the displayed revision.

/// Conversion and grouping live beside the public types.
use super::{Diagnostic, DiagnosticsSnapshot, Freshness, SourceGroup, add_to_group, convert};
/// Sets are keyed by server process and stamped with the text they were accepted for.
use crate::language::identity::{DocumentStamp, ServerIdentity};
/// Conversion happens against the rope of the displayed revision.
use helix_core::Rope;
/// The protocol's diagnostic record and each server's column unit.
use helix_lsp::{OffsetEncoding, lsp};
/// Dropping a server's pull result identifier is shared with the pull answers.
use pulled::forget_result_id;

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
    fn names(&self, hold: &Hold) -> bool {
        return match self {
            Ending::Server(server) => &hold.server == *server,
            Ending::Serial(serial) => hold.serial == *serial,
        };
    }
}

/// Orders source groups by their source text.
fn by_source(left: &SourceGroup, right: &SourceGroup) -> std::cmp::Ordering {
    return left.source.cmp(&right.source);
}

/// Orders diagnostics by their start,
///  then their end.
fn by_range(left: &Diagnostic, right: &Diagnostic) -> std::cmp::Ordering {
    return (left.start, left.end).cmp(&(right.start, right.end));
}

/// An open hold on one server's unversioned pushes.
struct Hold {
    /// Server whose unversioned sets are kept back.
    server: ServerIdentity,
    /// Number of the reload that opened the hold;
    ///  `u64` never wraps in practice.
    serial: u64,
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
    kept: Option<Kept>,
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

/// Store operations,
///  each corresponding to one lifecycle or protocol event.
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
    pub(crate) fn open(&mut self, stamp: DocumentStamp, version: i32) {
        self.close();
        // `Some(...)` wraps the record in the "value present" variant of `Option`.
        self.document = Some(Tracked { stamp, version });
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
        self.document = Some(Tracked { stamp, version });
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
    ///  End every hold for which `ends` returns true,
    ///  and show the set each kept.
    ///       `impl Fn(&Hold) -> bool` is any function that judges a borrowed hold;
    ///  `drain` empties
    ///       the list and hands out each hold by value.
    ///  Returns how many holds ended.
    /// Why:
    ///  A hold ends when the server answers for the new text or when its fixed delay
    ///      passes.
    ///  Either way its kept set is the server's latest set for this revision.
    ///  A set
    ///      that arrived after it never meets it here:
    ///  an accepted push removes the kept set.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// endHolds(ends: (hold: Hold) => boolean): number {
    ///   const ending = this.holds.filter(ends);
    ///   this.holds = this.holds.filter(hold => !ends(hold));
    ///   for (const hold of ending) if (hold.kept) this.storePushed(hold.server, hold.kept, 'unversioned');
    ///   return ending.length;
    /// }
    /// ```
    fn end_holds(&mut self, ending: Ending<'_>) -> usize {
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
    pub(crate) fn is_held(&self, server: &ServerIdentity) -> bool {
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
    pub(crate) fn answered(&mut self, server: &ServerIdentity, stamp: DocumentStamp) -> bool {
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
    pub(crate) fn hold_expired(&mut self, serial: u64) -> bool {
        return self.end_holds(Ending::Serial(serial)) > 0;
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
                    self.holds[position].kept = Some(Kept { encoding, items });
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
    pub(crate) fn server_exited(&mut self, server: &ServerIdentity) {
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

    /// What:
    ///  Build the latest-value snapshot against the displayed text,
    ///  or nothing when no file
    ///       is displayed.
    ///  Only sets stamped with the displayed revision take part.
    /// Why:
    ///  Sets of an earlier revision are removed from display,
    ///  never carried forward.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// snapshot(text: Rope): DiagnosticsSnapshot | undefined
    /// ```
    pub(crate) fn snapshot(&self, text: &Rope) -> Option<DiagnosticsSnapshot> {
        // The trailing `?` returns `None` when no document is tracked.
        let tracked = self.document.as_ref()?;
        let mut groups: Vec<SourceGroup> = Vec::new();
        for set in &self.sets {
            if set.stamp != tracked.stamp {
                continue;
            }
            for raw in &set.items {
                // `if let Some(x) = ...` runs the block only when conversion produced a value.
                if let Some(item) = convert(raw, text, set.encoding, &set.server, set.freshness) {
                    // What: `as_deref()` turns `Option<String>` into `Option<&str>`;
                    //       `unwrap_or("")` substitutes empty text for "nothing".
                    // Why: A server that names no source still gets one group.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // addToGroup(groups, raw.source ?? '', item);
                    // ```
                    add_to_group(&mut groups, raw.source.as_deref().unwrap_or(""), item);
                } else {
                    tracing::debug!(server = %set.server.name, "dropped a diagnostic outside the displayed text");
                }
            }
        }
        // What: `sort_by` orders in place; `cmp` compares two values and returns their ordering.
        // Why: A stable order keeps equal snapshots equal, so unchanged state is not republished.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // groups.sort((left, right) => left.source.localeCompare(right.source));
        // ```
        groups.sort_by(by_source);
        for group in groups.iter_mut() {
            group.items.sort_by(by_range);
        }
        return Some(DiagnosticsSnapshot {
            stamp: tracked.stamp,
            groups,
        });
    }
}

/// What a pull answer does to the stored sets,
///  and which result identifier the next pull request carries.
mod pulled;
