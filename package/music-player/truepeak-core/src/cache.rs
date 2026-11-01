//! The persistent decision cache,
//!  backed by Turso (behind the `service` feature).
//!
//! One row per decision,
//!  keyed by the source fingerprint and the full cache identity tuple
//! (policy,
//!  meter,
//!  decoder stack,
//!  and schema version),
//!  so a row is reused only when its
//! whole production environment matches.
//!  Writes upsert,
//!  but an exact decision is never
//! overwritten by a probe estimate for the same key,
//!  because the exact decision is strictly
//! better evidence.
//!  The whole module compiles only with the `service` feature,
//!  so the
//! meter-only build never pulls Turso or Tokio.

/// What:
///      `use crate::decision::{Decision, DecisionKind};`.
///  The value stored and its tag.
/// Why:
///       Rows are read into and written from `Decision`s.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Decision, DecisionKind } from "./decision";
/// ```
use crate::decision::{Decision, DecisionKind};

/// What:
///      `use crate::policy::{CacheIdentity, SCHEMA_VERSION};`.
///  The four-part identity a row must
///           match,
///  and the current row-and-key version.
/// Why:
///       The identity supplies the non-fingerprint half of the primary key;
///  the version drives
///           the stale-row cleanup on open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CacheIdentity, SCHEMA_VERSION } from "./policy";
/// ```
use crate::policy::{CacheIdentity, SCHEMA_VERSION};

/// What:
///      `use std::collections::HashSet;`.
///  A set of owned `u64` fingerprints.
/// Why:
///       `exact_fingerprints` returns every fingerprint whose decision is exact,
///  for a
///           warming sweep's bulk skip-check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HashSet = Set<bigint>;
/// ```
use std::collections::HashSet;

/// What:
///      `use std::fmt;`.
///  The formatting module,
///  for the error `Display`.
/// Why:
///       `CacheError` renders a human message for logs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no import; TS interpolates strings directly
/// ```
use std::fmt;

/// What:
///      `use turso::{Builder, Connection};`.
///  The Turso database builder and an open
///           connection.
///  A stored cell is a `turso::Value` (`Integer(i64)`,
///  `Real(f64)`,
///           `Text`,
///  `Blob`,
///  `Null`),
///  reached through `row.get_value` rather than imported.
/// Why:
///       `Builder` opens the file and `Connection` runs statements.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Builder, Connection } from "turso";
/// ```
use turso::{Builder, Connection};

/// What:
///      `const CREATE_TABLE: &str = "...";`.
///  The decisions table:
///  the fingerprint and
///           the four identity columns form the primary key.
///  `&str` (sibling `String`) is a
///           baked-in literal.
/// Why:
///       A row is reusable only when all five key columns match.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CREATE_TABLE = "CREATE TABLE IF NOT EXISTS decisions (...)";
/// ```
const CREATE_TABLE: &str = "CREATE TABLE IF NOT EXISTS decisions (\
    fingerprint INTEGER NOT NULL, policy_id INTEGER NOT NULL, meter_id INTEGER NOT NULL, \
    decoder_stack_id INTEGER NOT NULL, schema_version INTEGER NOT NULL, kind INTEGER NOT NULL, \
    gain REAL NOT NULL, measured_peak REAL NOT NULL, duration_secs REAL NOT NULL, \
    PRIMARY KEY (fingerprint, policy_id, meter_id, decoder_stack_id, schema_version))";

/// What:
///      `const SELECT_DECISION: &str = "...";`.
///  Read the four decision columns for one
///           exact key.
///  `&str` literal.
/// Why:
///       A cache hit returns the stored gain and its evidence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SELECT_DECISION = "SELECT kind, gain, ... WHERE fingerprint = ?1 AND ...";
/// ```
const SELECT_DECISION: &str = "SELECT kind, gain, measured_peak, duration_secs FROM decisions \
    WHERE fingerprint = ?1 AND policy_id = ?2 AND meter_id = ?3 AND decoder_stack_id = ?4 \
    AND schema_version = ?5";

/// What:
///      `const UPSERT_DECISION: &str = "...";`.
///  Insert or update,
///  but the `WHERE` on the
///           update clause skips overwriting an exact row (kind 0 or 2) with a probe (kind
///           1).
///  `&str` literal.
/// Why:
///       An exact decision must never be downgraded to a probe estimate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UPSERT_DECISION = "INSERT ... ON CONFLICT(...) DO UPDATE SET ... WHERE ...";
/// ```
const UPSERT_DECISION: &str = "INSERT INTO decisions (fingerprint, policy_id, meter_id, \
    decoder_stack_id, schema_version, kind, gain, measured_peak, duration_secs) \
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) \
    ON CONFLICT (fingerprint, policy_id, meter_id, decoder_stack_id, schema_version) \
    DO UPDATE SET kind = excluded.kind, gain = excluded.gain, \
    measured_peak = excluded.measured_peak, duration_secs = excluded.duration_secs \
    WHERE NOT (decisions.kind IN (0, 2) AND excluded.kind = 1)";

/// What:
///      `const DELETE_DECISION: &str = "...";`.
///  Remove the row for one exact key,
///  across all five key columns.
///  `&str` literal.
/// Why:
///       A re-analysis drops one track's decision so the next resolve measures it again,
///  without touching any other track or identity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DELETE_DECISION = "DELETE FROM decisions WHERE fingerprint = ?1 AND ...";
/// ```
const DELETE_DECISION: &str = "DELETE FROM decisions WHERE fingerprint = ?1 AND policy_id = ?2 \
    AND meter_id = ?3 AND decoder_stack_id = ?4 AND schema_version = ?5";

/// What:
///      `const SELECT_EXACT: &str = "...";`.
///  Read every fingerprint whose stored decision
///           is exact (kind 0 short or 2 full),
///  for the identity tuple.
///  `&str` literal.
/// Why:
///       A warming sweep skips tracks that already carry an exact gain,
///  and re-scans only
///           those with no row or a mere probe estimate (kind 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SELECT_EXACT = "SELECT fingerprint FROM decisions WHERE ... AND kind IN (0, 2)";
/// ```
const SELECT_EXACT: &str = "SELECT fingerprint FROM decisions \
    WHERE policy_id = ?1 AND meter_id = ?2 AND decoder_stack_id = ?3 AND schema_version = ?4 \
    AND kind IN (0, 2)";

/// What:
///      `const DELETE_STALE: &str = "...";`.
///  Remove every row written under an older
///           schema version.
///  `&str` literal.
/// Why:
///       A version bump (a new row layout, or a new fingerprint hash that fills the same
///           column) makes older rows unreachable,
///  because every read and write keys on the current
///           version.
///  Deleting them on open stops a retired hash's rows from accumulating forever
///           in the user's database.
///  The comparison is `<` so a newer app's rows survive a downgrade
///           and are simply never matched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DELETE_STALE = "DELETE FROM decisions WHERE schema_version < ?1";
/// ```
const DELETE_STALE: &str = "DELETE FROM decisions WHERE schema_version < ?1";

/// What:
///      `pub const WRITE_BATCH_LIMIT: usize = 256;`.
///  The most rows a caller should hand to one `put_batch`.
///  `usize` is the platform's index-sized integer.
/// Why:
///       Bounds how long one commit holds the single connection,
///  so a read queued behind a write waits for at most this many upserts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const WRITE_BATCH_LIMIT = 256;
/// ```
pub const WRITE_BATCH_LIMIT: usize = 256;

/// What:
///      `pub struct CacheError { pub message: String }`.
///  A turso-free error wrapping a
///           message.
///  `String` (sibling `&str`) owns the text past the turso error's life.
/// Why:
///       The public cache API does not name a turso type,
///  so callers stay decoupled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CacheError extends Error {}
/// ```
#[derive(Debug)]
pub struct CacheError {
    /// What:
    ///      `pub message: String`.
    ///  The human-readable cause.
    /// Why:
    ///       Logged and shown;
    ///  owns its text.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// message: string;
    /// ```
    pub message: String,
}

/// What:
///      `impl fmt::Display for CacheError`.
///  Render the message.
/// Why:
///       Logs and the `Error` impl below.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// toString() { return `true-peak cache: ${this.message}`; }
/// ```
impl fmt::Display for CacheError {
    /// Write the cache error message on one line.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(formatter, "true-peak cache: {}", self.message)
    }
}

/// What:
///      `impl std::error::Error for CacheError {}`.
///  Opt into the std error trait.
/// Why:
///       Participate in `?` propagation and generic std error reporting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // CacheError extends Error
/// ```
impl std::error::Error for CacheError {}

/// What:
///      `impl From<turso::Error> for CacheError`.
///  Convert a turso error into ours.
/// Why:
///       Lets `?` on turso calls produce a `CacheError` without leaking the turso type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // catch (e) { throw new CacheError(String(e)); }
/// ```
impl From<turso::Error> for CacheError {
    /// Wrap a turso error's text.
    fn from(error: turso::Error) -> CacheError {
        return CacheError { message: error.to_string() }
    }
}

/// What:
///      `fn kind_to_int(kind: DecisionKind) -> i64`.
///  Encode the tag as a small integer.
/// Why:
///       The precedence `WHERE` compares kinds as integers (0 and 2 exact,
///  1 probe).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function kindToInt(kind) { return { shortFullScan: 0, probeEstimate: 1, fullScanExact: 2 }[kind]; }
/// ```
fn kind_to_int(kind: DecisionKind) -> i64 {
    // What:     `match kind { ... }`. One arm per variant; no arm discarded.
    // Why:      Fix the integer encoding the SQL precedence relies on.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "shortFullScan": return 0; ... }
    // ```
    match kind {
        DecisionKind::ShortFullScan => return 0,
        DecisionKind::ProbeEstimate => return 1,
        DecisionKind::FullScanExact => return 2,
    }
}

/// What:
///      `fn kind_from_int(value: i64) -> Result<DecisionKind, CacheError>`.
///  Decode a
///           stored integer back to a tag,
///  rejecting anything unexpected.
/// Why:
///       A corrupt or future row must not silently become the wrong kind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function kindFromInt(value) { ... }
/// ```
fn kind_from_int(value: i64) -> Result<DecisionKind, CacheError> {
    // What:     `match value { 0 => Ok(...), ..., other => Err(...) }`. Map the known
    //           encodings and reject the rest.
    // Why:      Fail loudly on an unknown kind rather than guess.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (value) { case 0: return "shortFullScan"; ... default: throw new CacheError(...); }
    // ```
    match value {
        0 => return Ok(DecisionKind::ShortFullScan),
        1 => return Ok(DecisionKind::ProbeEstimate),
        2 => return Ok(DecisionKind::FullScanExact),
        other => {
            // An unknown kind is row corruption or a future schema; fail loudly.
            tracing::error!(value = other, "unknown decision kind in cache row");
            return Err(CacheError { message: format!("unknown decision kind {other}") })
        }
    }
}

/// What:
///      `fn real_at(row: &turso::Row, index: usize) -> Result<f64, CacheError>`.
///  Read a
///           `REAL` column,
///  failing if the cell is not a real.
///  `usize` is the column index
///           type Turso's `get_value` wants.
/// Why:
///       A NULL or wrong-typed cell is corruption,
///  not a value to coerce.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function realAt(row, index) { const v = row.getValue(index); if (v.kind !== "real") throw ...; return v.value; }
/// ```
fn real_at(row: &turso::Row, index: usize) -> Result<f64, CacheError> {
    // What:     `row.get_value(index)?.as_real().copied()`. Read the cell (`?` propagates a
    //           turso error), then `.as_real()` yields `Option<&f64>`, `.copied()` an
    //           `Option<f64>`.
    // Why:      Turn the typed cell into a plain `f64` or a miss.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const real = row.getValue(index).asReal();
    // ```
    return row.get_value(index)?
        .as_real()
        .copied()
        .ok_or_else(|| return CacheError { message: format!("column {index} is not a real") })
        .inspect_err(|error| tracing::error!(index, cause = %error.message, "cache real column corrupt"))
}

/// What:
///      `pub struct DecisionCache { conn: Connection }`.
///  An open connection to the
///           decision database.
///  `Connection` is Turso's per-database handle.
/// Why:
///       All reads and writes go through one connection the platform owns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class DecisionCache { private conn: Connection; }
/// ```
pub struct DecisionCache {
    /// What:
    ///      `conn: Connection`.
    ///  The open Turso connection.
    /// Why:
    ///       Runs the select and upsert statements.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// conn: Connection;
    /// ```
    conn: Connection,
}

/// What:
///      `impl DecisionCache { ... }`.
///  Open,
///  read,
///  and write behavior.
/// Why:
///       The three operations the service needs from persistence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class DecisionCache { async open() {} async get() {} async put() {} }
/// ```
impl DecisionCache {
    /// What:
    ///      `pub async fn open(path: &str) -> Result<DecisionCache, CacheError>`.
    ///  Open
    ///           (or create) the database file at `path` and ensure the table exists.
    ///           `async` returns a future the platform's runtime drives.
    /// Why:
    ///       A ready cache the service can query.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static async open(path: string): Promise<DecisionCache> { ... }
    /// ```
    pub async fn open(path: &str) -> Result<DecisionCache, CacheError> {
        // What:     `let database = Builder::new_local(path).build().await?;`. Open the local
        //           file; `.await` runs the future, `?` converts a turso error.
        // Why:      We need the database before a connection.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const database = await Builder.newLocal(path).build();
        // ```
        let database = Builder::new_local(path).build().await?;
        // What:     `let conn = database.connect()?;`. Open a connection; `?` propagates.
        // Why:      Statements run on a connection.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const conn = database.connect();
        // ```
        let conn = database.connect()?;
        // What:     `conn.execute(CREATE_TABLE, ()).await?;`. Create the table if absent;
        //           `()` is the empty parameter list.
        // Why:      Idempotent schema setup on open.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // await conn.execute(CREATE_TABLE, []);
        // ```
        conn.execute(CREATE_TABLE, ()).await?;
        // What:     `let removed = conn.execute(DELETE_STALE, (i64::from(SCHEMA_VERSION),)).await?;`.
        //           Delete every row written under an older schema version; the one-element
        //           tuple is the single bound parameter and `removed` is the row count.
        // Why:      Rows from a retired fingerprint hash or row layout can never match again,
        //           so they are removed instead of accumulating. A first open of a fresh
        //           database removes nothing.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const removed = await conn.execute(DELETE_STALE, [SCHEMA_VERSION]);
        // ```
        let removed = conn.execute(DELETE_STALE, (i64::from(SCHEMA_VERSION),)).await?;
        if removed > 0 {
            tracing::info!(removed, schema_version = SCHEMA_VERSION, "stale decision rows removed on open");
        } else {
            tracing::debug!(schema_version = SCHEMA_VERSION, "no stale decision rows on open");
        }
        // The cache is open and the table ensured; record the file it is backing.
        tracing::debug!(path, "decision cache opened");
        // What:     `Ok(DecisionCache { conn })`. The ready handle. Tail -> return.
        // Why:      Hand the cache back.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return new DecisionCache(conn);
        // ```
        return Ok(DecisionCache { conn })
    }

    /// What:
    ///      `pub async fn get(&self, fingerprint: u64, identity: CacheIdentity) ->
    ///           Result<Option<Decision>, CacheError>`.
    ///  Look up the decision for one exact
    ///           key.
    ///  `&self` borrows the cache read-only.
    /// Why:
    ///       A hit lets playback apply a cached gain immediately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async get(fingerprint, identity): Promise<Decision | null> { ... }
    /// ```
    pub async fn get(
        &self,
        fingerprint: u64,
        identity: CacheIdentity,
    ) -> Result<Option<Decision>, CacheError> {
        // What:     `let mut rows = self.conn.query(SELECT_DECISION, params).await?;`. Run the
        //           select. The params bind the five key columns; `fingerprint as i64` is a
        //           bijective bit-cast (a large `u64` becomes a negative `i64`, matched the
        //           same way on write).
        // Why:      Fetch at most one matching row.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const rows = await conn.query(SELECT_DECISION, [fingerprint, policyId, meterId, decoderStackId, schemaVersion]);
        // ```
        let mut rows = self
            .conn
            .query(
                SELECT_DECISION,
                (
                    fingerprint as i64,
                    identity.policy_id as i64,
                    identity.meter_id as i64,
                    identity.decoder_stack_id as i64,
                    i64::from(identity.schema_version),
                ),
            )
            .await?;
        // What:     `match rows.next().await? { Some(row) => ..., None => Ok(None) }`. Pull the
        //           first row if any; `?` propagates a turso error, then the `Option` says hit
        //           or miss.
        // Why:      Turn a row into a `Decision`, or report a miss.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const row = await rows.next(); if (!row) return null;
        // ```
        match rows.next().await? {
            Some(row) => {
                // A stored row for this exact key: a cache hit.
                tracing::debug!(fingerprint, "cache get hit");
                // What:     `let kind = kind_from_int(row.get_value(0)?.as_integer()...);`.
                //           Decode the kind column; `.as_integer()` yields `Option<&i64>`.
                // Why:      The stored tag drives whether the row is exact or improvable.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const kind = kindFromInt(row.getValue(0).asInteger());
                // ```
                let kind_value = *row
                    .get_value(0)?
                    .as_integer()
                    .ok_or_else(|| return CacheError { message: "column 0 is not an integer".to_string() })
                    .inspect_err(|error| tracing::error!(cause = %error.message, "cache kind column corrupt"))?;
                // What:     `Ok(Some(Decision { ... }))`. Build the decision from the columns;
                //           `real_at` reads each REAL, `as f32` narrows the gain and peak.
                //           Tail of this arm.
                // Why:      Hand back the cached decision.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // return { kind, gain: realAt(row,1), measuredPeak: realAt(row,2), durationSecs: realAt(row,3) };
                // ```
                return Ok(Some(Decision {
                    kind: kind_from_int(kind_value)?,
                    gain: real_at(&row, 1)? as f32,
                    measured_peak: real_at(&row, 2)? as f32,
                    duration_secs: real_at(&row, 3)?,
                }))
            }
            None => {
                // No row for this key: a cache miss.
                tracing::debug!(fingerprint, "cache get miss");
                return Ok(None)
            }
        }
    }

    /// What:
    ///      `pub async fn put(&self, fingerprint: u64, identity: CacheIdentity, decision:
    ///           &Decision) -> Result<(), CacheError>`.
    ///  Store or upgrade a decision,
    ///  never
    ///           downgrading an exact row to a probe.
    ///  `&Decision` borrows it read-only.
    /// Why:
    ///       Persist a resolved decision so later plays hit the cache.
    ///  Each call is its own transaction, so each call pays one durable commit;
    ///  a caller storing many rows at once uses `put_batch`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async put(fingerprint, identity, decision): Promise<void> { ... }
    /// ```
    pub async fn put(
        &self,
        fingerprint: u64,
        identity: CacheIdentity,
        decision: &Decision,
    ) -> Result<(), CacheError> {
        // What:     `self.upsert(...).await?;`. Run the upsert in autocommit mode.
        // Why:      One statement, one commit.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // await this.upsert(fingerprint, identity, decision);
        // ```
        self.upsert(fingerprint, identity, decision).await?;
        // What:     `Ok(())`. Nothing to return on success. Tail -> return.
        // Why:      The write succeeded.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return;
        // ```
        return Ok(())
    }

    /// What:
    ///      `pub async fn put_batch(&self, identity: CacheIdentity, entries: &[(u64, Decision)]) ->
    ///           Result<(), CacheError>`.
    ///  Store or upgrade every `(fingerprint, decision)` pair in one transaction,
    ///  with the same never-downgrade rule as `put`.
    ///  `&[(u64, Decision)]` is a borrowed view of a list of pairs.
    ///  When the transaction fails,
    ///  every row is retried on its own so one bad row cannot discard its neighbours.
    /// Why:
    ///       A commit is the expensive step (one disk flush under the default durability),
    ///  so a cold sweep that stores thousands of rows pays it once per batch instead of once per row.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async putBatch(identity, entries): Promise<void> { ... }
    /// ```
    pub async fn put_batch(
        &self,
        identity: CacheIdentity,
        entries: &[(u64, Decision)],
    ) -> Result<(), CacheError> {
        // An empty batch has nothing to store and must not open a transaction.
        if entries.is_empty() {
            return Ok(())
        }
        // What:     `match self.put_batch_in_transaction(identity, entries).await { Ok(()) => return Ok(()), Err(error) => ... }`.
        //           Try the single-transaction path first.
        // Why:      The common case is one commit for the whole batch.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { await this.putBatchInTransaction(identity, entries); return; } catch (e) { warn(e); }
        // ```
        let Err(batch_error) = self.put_batch_in_transaction(identity, entries).await else {
            tracing::debug!(rows = entries.len(), "cache put batch committed");
            return Ok(())
        };
        tracing::warn!(error = %batch_error, rows = entries.len(), "cache put batch failed; retrying row by row");
        // What:     `for (fingerprint, decision) in entries { ... }`. Store each row in its own transaction and count
        //           the ones that fail.
        // Why:      The failed batch was rolled back, so every row is still unstored.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const [fingerprint, decision] of entries) { try { await put(...) } catch { failed++ } }
        // ```
        let mut failed = 0_usize;
        for (fingerprint, decision) in entries {
            if let Err(error) = self.put(*fingerprint, identity, decision).await {
                failed += 1;
                tracing::warn!(error = %error, fingerprint, "cache row failed after batch failure");
            }
        }
        if failed > 0 {
            return Err(CacheError { message: format!("{failed} of {} rows could not be stored", entries.len()) })
        }
        return Ok(())
    }

    /// What:
    ///      `async fn put_batch_in_transaction(&self, identity: CacheIdentity, entries: &[(u64, Decision)]) ->
    ///           Result<(), CacheError>`.
    ///  Begin a transaction,
    ///  upsert every pair,
    ///  commit;
    ///  roll back when any step fails.
    /// Why:
    ///       The all-or-nothing path `put_batch` tries first.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async putBatchInTransaction(identity, entries) { await begin(); ...; await commit(); }
    /// ```
    async fn put_batch_in_transaction(
        &self,
        identity: CacheIdentity,
        entries: &[(u64, Decision)],
    ) -> Result<(), CacheError> {
        let transaction = self.conn.unchecked_transaction().await?;
        for (fingerprint, decision) in entries {
            if let Err(error) = self.upsert(*fingerprint, identity, decision).await {
                // The statement error is the cause worth returning; a failed rollback is logged.
                if let Err(rollback_error) = transaction.rollback().await {
                    tracing::warn!(error = %rollback_error, "cache batch rollback failed");
                }
                return Err(error)
            }
        }
        transaction.commit().await?;
        return Ok(())
    }

    /// What:
    ///      `async fn upsert(&self, fingerprint: u64, identity: CacheIdentity, decision: &Decision) ->
    ///           Result<(), CacheError>`.
    ///  Run the upsert statement on the connection,
    ///  inside whatever transaction is open (autocommit when none is).
    /// Why:
    ///       `put` and `put_batch` share the statement and its parameter binding.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async upsert(fingerprint, identity, decision): Promise<void> { ... }
    /// ```
    async fn upsert(
        &self,
        fingerprint: u64,
        identity: CacheIdentity,
        decision: &Decision,
    ) -> Result<(), CacheError> {
        // What:     `self.conn.execute(UPSERT_DECISION, params).await?;`. Run the upsert. The
        //           nine params bind the key, the kind integer, and the three reals; `f64`
        //           widenings match the REAL columns.
        // Why:      Insert a new row, or update an existing one unless that would downgrade an
        //           exact decision to a probe (the SQL `WHERE` enforces it).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // await conn.execute(UPSERT_DECISION, [fingerprint, ...identity, kindToInt(decision.kind), decision.gain, decision.measuredPeak, decision.durationSecs]);
        // ```
        let affected = self
            .conn
            .execute(
                UPSERT_DECISION,
                (
                    fingerprint as i64,
                    identity.policy_id as i64,
                    identity.meter_id as i64,
                    identity.decoder_stack_id as i64,
                    i64::from(identity.schema_version),
                    kind_to_int(decision.kind),
                    f64::from(decision.gain),
                    f64::from(decision.measured_peak),
                    decision.duration_secs,
                ),
            )
            .await?;
        // The upsert ran; `affected == 0` means the precedence WHERE refused a probe downgrade.
        tracing::debug!(fingerprint, affected, applied = affected > 0, "cache put");
        return Ok(())
    }

    /// What:
    ///      `pub async fn remove(&self, fingerprint: u64, identity: CacheIdentity) ->
    ///           Result<bool, CacheError>`.
    ///  Delete the stored decision for one exact key,
    ///  and report whether a row was there.
    ///  `&self` borrows the cache;
    ///  the delete writes through its connection.
    /// Why:
    ///       A re-analysis clears one track so the next resolve measures it fresh,
    ///  while every other track keeps its decision.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async remove(fingerprint, identity): Promise<boolean> { ... }
    /// ```
    pub async fn remove(&self, fingerprint: u64, identity: CacheIdentity) -> Result<bool, CacheError> {
        // What:     `let affected = self.conn.execute(DELETE_DECISION, params).await?;`. Run the
        //           delete with the same five key columns a read or write uses.
        // Why:      Only the one row for this exact key is removed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const affected = await conn.execute(DELETE_DECISION, [fingerprint, ...identity]);
        // ```
        let affected = self
            .conn
            .execute(
                DELETE_DECISION,
                (
                    fingerprint as i64,
                    identity.policy_id as i64,
                    identity.meter_id as i64,
                    identity.decoder_stack_id as i64,
                    i64::from(identity.schema_version),
                ),
            )
            .await?;
        // A removed row is logged; a miss is normal (the track was never measured).
        tracing::debug!(fingerprint, affected, "cache remove");
        // What:     `Ok(affected > 0)`. True when a row was removed. Tail -> return.
        // Why:      The caller can tell a real invalidation from a no-op.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return affected > 0;
        // ```
        return Ok(affected > 0)
    }

    /// What:
    ///      `pub async fn exact_fingerprints(&self, identity: CacheIdentity) ->
    ///           Result<HashSet<u64>, CacheError>`.
    ///  Collect every fingerprint whose stored
    ///           decision is exact (short or full scan) for this identity.
    ///  `&self` borrows the
    ///           cache read-only.
    /// Why:
    ///       A warming sweep seeds its skip-check from this set,
    ///  so it re-scans only tracks
    ///           with no decision or a mere probe estimate,
    ///  upgrading them to exact over time.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async exactFingerprints(identity): Promise<Set<bigint>> { ... }
    /// ```
    pub async fn exact_fingerprints(
        &self,
        identity: CacheIdentity,
    ) -> Result<HashSet<u64>, CacheError> {
        // What:     `let mut rows = self.conn.query(SELECT_EXACT, params).await?;`. Run the
        //           scan; the four params bind the identity columns. `?` propagates a turso
        //           error.
        // Why:      Fetch every exact row's fingerprint for this identity.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const rows = await conn.query(SELECT_EXACT, [policyId, meterId, decoderStackId, schemaVersion]);
        // ```
        let mut rows = self
            .conn
            .query(
                SELECT_EXACT,
                (
                    identity.policy_id as i64,
                    identity.meter_id as i64,
                    identity.decoder_stack_id as i64,
                    i64::from(identity.schema_version),
                ),
            )
            .await?;
        // What:     `let mut set = HashSet::new();`. The accumulator.
        // Why:      Collect the fingerprints as they stream in.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const set = new Set();
        // ```
        let mut set = HashSet::new();
        // What:     `while let Some(row) = rows.next().await? { ... }`. Drain every row; `?`
        //           propagates a turso error mid-stream rather than truncating silently.
        // Why:      Read the whole result, not a prefix.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for await (const row of rows) set.add(BigInt.asUintN(64, row.getValue(0)));
        // ```
        while let Some(row) = rows.next().await? {
            // What:     `let stored = *row.get_value(0)?.as_integer().ok_or_else(...)?;`. Read
            //           column 0 as the stored `i64`, failing on a non-integer cell.
            // Why:      Fingerprints are stored as `i64` bit-casts of the original `u64`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const stored = row.getValue(0).asInteger();
            // ```
            let stored = *row
                .get_value(0)?
                .as_integer()
                .ok_or_else(|| return CacheError { message: "column 0 is not an integer".to_string() })
                .inspect_err(|error| tracing::error!(cause = %error.message, "cache fingerprint column corrupt"))?;
            // What:     `set.insert(stored as u64);`. Reverse the write-side `u64 as i64`
            //           bit-cast; the round-trip is bijective.
            // Why:      Return the fingerprints in the same `u64` domain callers hash.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // set.add(BigInt.asUintN(64, stored));
            // ```
            set.insert(stored as u64);
        }
        // What:     `Ok(set)`. The exact-fingerprint snapshot. Tail -> return.
        // Why:      Hand the sweep an owned set it reads without locking.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return set;
        // ```
        return Ok(set)
    }
}

/// What:
///      `#[cfg(test)] #[path = "cache_tests.rs"] mod tests;`.
///  Test-only submodule in the
///           sibling file,
///  gated to test builds of the `service` feature.
/// Why:
///       Keep this file to production code;
///  sibling `*_tests.rs` is max-lines exempt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // cache.integration.test.ts
/// ```
#[cfg(test)]
#[path = "cache_tests.rs"]
mod tests;
