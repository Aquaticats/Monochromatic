//! The synchronous handle the rest of the player uses to reach the peak cache.
//!
//! `CacheHandle` holds the actor's two channel senders and nothing else,
//!  so it is
//! cheap to `Clone` and share across the engine thread,
//!  the current-track
//! measurement worker,
//!  and the background sweep.
//!  Reads block briefly on a
//! `oneshot` reply (a single indexed lookup,
//!  at human-paced track-load frequency);
//! writes are non-blocking sends.
//!  No `Mutex`:
//!  the actor owns the only mutable state.

/// What:
///      `use std::collections::HashSet;`.
///  A set of `u64` fingerprints.
/// Why:
///       `known_fingerprints` returns the actor's exact-decision fingerprint snapshot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HashSet = Set<bigint>;
/// ```
use std::collections::HashSet;

/// What:
///      `use truepeak_core::Decision;`.
///  The gain decision the cache stores and returns.
/// Why:
///       `get` returns an `Option<Decision>` and `upsert` takes one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Decision } from "truepeak-core";
/// ```
use truepeak_core::Decision;

/// What:
///      `#[cfg(test)] use std::path::PathBuf;`.
///  Owned filesystem path buffer,
///           imported only in test builds.
/// Why:
///       The test-only `open_at` constructor points the actor at a throwaway
///           database file;
///  production callers go through `open` (no `PathBuf` here).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PathBuf = string;
/// ```
#[cfg(test)]
use std::path::PathBuf;

/// What:
///      `use tokio::sync::mpsc::UnboundedSender;`.
///  The cloneable write end of
///           an unbounded channel;
///  `send` is synchronous.
/// Why:
///       The handle enqueues requests without entering the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a sync-push queue sender
/// ```
use tokio::sync::mpsc::UnboundedSender;

/// What:
///      `use tokio::sync::oneshot;`.
///  A single-value reply channel.
/// Why:
///       Read calls create a reply pair and block on the receiver.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a one-shot resolve/await pair
/// ```
use tokio::sync::oneshot;

/// What:
///      `use super::service::{self, Read, Upsert};`.
///  The actor module plus its
///           two request types.
/// Why:
///       The handle spawns the actor and builds the requests it serves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as service from "./peakcache_service";
/// ```
use super::service::{self, Read, Upsert, Write};

/// What:
///      `#[derive(Clone)] pub(crate) struct CacheHandle { .. }`.
///  A shareable
///           handle to the peak-cache actor,
///  holding only the two channel senders.
///           `pub(crate)` so the controller,
///  `peak_swap`,
///  and `measure` share it.
/// Why:
///       Replaces the old `Arc<Mutex<PeakCache>>`:
///  no lock,
///  just message passing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CacheHandle { readTx: Sender<Read>; writeTx: Sender<Upsert>; }
/// ```
#[derive(Clone)]
pub(crate) struct CacheHandle {
    /// What:
    ///      `read_tx: UnboundedSender<Read>`.
    ///  The read-request sender.
    /// Why:
    ///       Carries `Get`/`Known` to the actor's read-biased arm.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// readTx: Sender<Read>;
    /// ```
    read_tx: UnboundedSender<Read>,
    /// What:
    ///      `write_tx: UnboundedSender<Upsert>`.
    ///  The write-request sender.
    /// Why:
    ///       Carries fire-and-forget upserts to the actor's write arm.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// writeTx: Sender<Upsert>;
    /// ```
    write_tx: UnboundedSender<Write>,
}

/// What:
///      `impl CacheHandle { .. }`.
///  The handle's constructors and operations.
/// Why:
///       Group the cache surface the rest of the crate calls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CacheHandle { /* open, get, upsert, knownFingerprints */ }
/// ```
impl CacheHandle {
    /// What:
    ///      `pub(crate) fn open() -> CacheHandle`.
    ///  Start the actor on the
    ///           standard `peaks.db` location and return a handle.
    /// Why:
    ///       Called once at controller construction.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static open(): CacheHandle { return CacheHandle.spawn(dbPath()); }
    /// ```
    pub(crate) fn open() -> CacheHandle {
        // What:     Spawn the actor at the shared config-dir path (`None` => degraded).
        // Why:      One place decides where the cache lives (`super::db_path`).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const [readTx, writeTx] = service.spawn(dbPath());
        // ```
        let (read_tx, write_tx) = service::spawn(super::db_path());
        // What:     Wrap the senders. Tail -> return.
        // Why:      Hand back the shareable handle.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { readTx, writeTx };
        // ```
        return CacheHandle { read_tx, write_tx }
    }

    /// What:
    ///      `#[cfg(test)] pub(crate) fn open_at(path: PathBuf) -> CacheHandle`.
    ///           Start the actor on an explicit database file.
    ///  Test-only.
    /// Why:
    ///       Tests must hit a throwaway file,
    ///  never the real config dir.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static openAt(path: string): CacheHandle { return CacheHandle.spawn(path); }
    /// ```
    #[cfg(test)]
    pub(crate) fn open_at(path: PathBuf) -> CacheHandle {
        // What:     Spawn the actor at the given path.
        // Why:      Disposable persistence for a single test.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const [readTx, writeTx] = service.spawn(path);
        // ```
        let (read_tx, write_tx) = service::spawn(Some(path));
        // What:     Wrap the senders. Tail -> return.
        // Why:      Hand back the test handle.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { readTx, writeTx };
        // ```
        return CacheHandle { read_tx, write_tx }
    }

    /// What:
    ///      `#[cfg(test)] pub(crate) fn open_degraded() -> CacheHandle`.
    ///  Start a cache
    ///           actor with NO database file (degraded:
    ///  reads miss,
    ///  writes drop,
    ///  key set
    ///           empty).
    ///  Test-only.
    /// Why:
    ///       Controller tests that never exercise the cache must not open or create the
    ///           real `peaks.db`;
    ///  a degraded handle touches no disk at all.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static openDegraded(): CacheHandle { return CacheHandle.spawn(null); }
    /// ```
    #[cfg(test)]
    pub(crate) fn open_degraded() -> CacheHandle {
        // What:     Spawn the actor with no path.
        // Why:      No persistence, no file, a pure no-op cache.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const [readTx, writeTx] = service.spawn(null);
        // ```
        let (read_tx, write_tx) = service::spawn(None);
        // What:     Wrap the senders. Tail -> return.
        // Why:      Hand back the degraded handle.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { readTx, writeTx };
        // ```
        return CacheHandle { read_tx, write_tx }
    }

    /// What:
    ///      `pub(crate) fn get(&self, fingerprint: u64) -> Option<Decision>`.
    ///  Block
    ///           briefly for one cached decision,
    ///  or `None` on miss/closed actor.
    /// Why:
    ///       `peak_swap` reads the current track's cached gain decision at load time.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get(fingerprint: bigint): Decision | null { ... }
    /// ```
    pub(crate) fn get(&self, fingerprint: u64) -> Option<Decision> {
        // What:     Make the reply pair; send `Get` with the key.
        // Why:      The actor answers on `reply_tx`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const [replyTx, replyRx] = oneshot();
        // ```
        let (reply_tx, reply_rx) = oneshot::channel();
        // What:     Send the request; a closed actor => treat as a miss.
        // Why:      Never hang or panic if the cache thread is gone.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!readTx.send({ kind: "get", fingerprint, reply: replyTx })) return null;
        // ```
        if self
            .read_tx
            .send(Read::Get {
                fingerprint,
                reply: reply_tx,
            })
            .is_err()
        {
            return None;
        }
        // What:     Block for the answer; flatten send/recv failure into `None`.
        // Why:      `blocking_recv` is valid here (the caller is NOT inside the runtime).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return (await replyRx) ?? null;
        // ```
        return reply_rx.blocking_recv().ok().flatten()
    }

    /// What:
    ///      `pub(crate) fn upsert(&self, fingerprint: u64, decision: Decision)`.
    ///           Fire-and-forget a resolved decision to the actor.
    /// Why:
    ///       Sweep workers and the current-track worker store results without blocking on
    ///           persistence.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// upsert(fingerprint: bigint, decision: Decision): void { ... }
    /// ```
    pub(crate) fn upsert(&self, fingerprint: u64, decision: Decision) {
        // What:     Send the write; ignore a closed actor.
        // Why:      A dropped cache thread just means nothing is persisted.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // writeTx.send({ fingerprint, decision });
        // ```
        let _ = self.write_tx.send(Write::Put(Upsert { fingerprint, decision }));
    }

    /// What:
    ///      `pub(crate) fn known_fingerprints(&self) -> HashSet<u64>`.
    ///  Block briefly for a
    ///           snapshot of every fingerprint whose decision is already exact.
    /// Why:
    ///       The sweep seeds its skip-check from one cheap snapshot per run and re-scans
    ///           only tracks with no decision or a mere probe estimate.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// knownFingerprints(): Set<bigint> { ... }
    /// ```
    pub(crate) fn known_fingerprints(&self) -> HashSet<u64> {
        // What:     Make the reply pair; send `Known`.
        // Why:      The actor replies with a clone of its in-memory key set.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const [replyTx, replyRx] = oneshot();
        // ```
        let (reply_tx, reply_rx) = oneshot::channel();
        // What:     Send; a closed actor => empty set (skip-check finds nothing cached).
        // Why:      Degrade to "re-measure everything", never hang.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!readTx.send({ kind: "known", reply: replyTx })) return new Set();
        // ```
        if self.read_tx.send(Read::Known { reply: reply_tx }).is_err() {
            return HashSet::new();
        }
        // What:     Block for the snapshot; any failure => empty set.
        // Why:      The skip-check tolerates an empty snapshot (it just re-measures).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return (await replyRx) ?? new Set();
        // ```
        return reply_rx.blocking_recv().unwrap_or_default()
    }
}
