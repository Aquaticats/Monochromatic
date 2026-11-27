//! The native true-peak service handle and its JNI entry points.
//!
//! Kotlin holds one opaque `jlong` handle to a `TruePeakService`.
//!  The service owns a dedicated
//! thread running a current-thread Tokio runtime and the shared `truepeak_core::DecisionCache`
//! (a Turso-backed `decisions.db` in the app-private dir).
//!  Because a Turso connection cannot be
//! assumed `Send`,
//!  the connection never leaves that thread:
//!  the handle is only two channel
//! senders (both `Send + Sync`),
//!  so a `&TruePeakService` shared across the foreground and
//! warming JNI threads is sound.
//!  Cache reads and writes cross the channels;
//!  the BLOCKING decode
//! and shared-resolver call happen on the JNI calling thread (never the cache thread),
//!  so a
//! slow full scan never stalls a cache lookup.
//!  This module also holds the JNI functions Kotlin
//! calls:
//!  create,
//!  release,
//!  resolve (foreground),
//!  and warm (background upgrade).

/// What:
///      `use std::thread;`.
///  OS-thread spawning.
/// Why:
///       The cache actor runs on its own thread so decode never blocks it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // std::thread ~ a dedicated Worker
/// ```
use std::thread;

/// What:
///      `use jni::sys::jlong;`.
///  The JNI 64-bit integer (Kotlin `Long`),
///           the one JVM type wide enough to carry the opaque handle.
/// Why:
///       `service_ref` takes Kotlin's handle back as a `jlong` and rebuilds the
///           `&TruePeakService`;
///  the entries that call it live in `service_jni.rs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type jlong = number;
/// ```
use jni::sys::jlong;

/// What:
///      `use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};`.
///  Unbounded
///           request channels with synchronous `send`.
/// Why:
///       Sync JNI callers enqueue without an `await`;
///  the actor `recv().await`s.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // an unbounded queue with sync push and async pull
/// ```
use tokio::sync::mpsc::{
    self,
    UnboundedReceiver,
    UnboundedSender,
};

/// What:
///      `use tokio::sync::oneshot;`.
///  A single-value reply channel.
/// Why:
///       A read request carries a `oneshot::Sender` the actor answers on;
///  the caller
///           blocks on the matching receiver.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a one-shot resolve/await pair
/// ```
use tokio::sync::oneshot;

/// What:
///      `use truepeak_core::{CacheIdentity, Decision, DecisionCache, default_policy, stack_id};`.
///           The identity tuple,
///  the cached value,
///  the Turso cache,
///  the shipped policy,
///  and the
///           shared stack-id hash.
/// Why:
///       The actor opens a cache and keys every read and write on the identity.
///  The
///           ceiling gain and the decision kind that the entries fall back on live in
///           `service_jni.rs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CacheIdentity, Decision, DecisionCache, defaultPolicy, stackId } from "truepeak-core";
/// ```
use truepeak_core::{
    CacheIdentity,
    Decision,
    DecisionCache,
    default_policy,
    stack_id,
};

/// What:
///      `const DECODER_STACK_DESCRIPTION: &str = "...";`.
///  A stable text description of the
///           Android decode stack that produces the PCM the meter reads.
/// Why:
///       Its hash is the `decoder_stack_id`;
///  editing it when the decoder stack changes
///           re-keys the cache,
///  so decisions from a different decoder are never reused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DECODER_STACK_DESCRIPTION = "android:symphonia-0.6+opus-rev-5598766+f32le";
/// ```
const DECODER_STACK_DESCRIPTION: &str = "android:symphonia-0.6+opus-rev-5598766+f32le";

/// What:
///      `fn decoder_stack_id() -> u64`.
///  Hash the decoder-stack description with the
///           shared crate's `stack_id`.
/// Why:
///       The platform owns its description;
///  the shared crate owns the derivation,
///  the
///           same FNV every other identity id uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decoderStackId(): bigint { return stackId(DECODER_STACK_DESCRIPTION); }
/// ```
fn decoder_stack_id() -> u64 {
    // What:     `stack_id(DECODER_STACK_DESCRIPTION)`. Tail -> return.
    // Why:      Deterministic id that changes only when the description does.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return stackId(DECODER_STACK_DESCRIPTION);
    // ```
    return stack_id(DECODER_STACK_DESCRIPTION);
}

/// What:
///      `struct Read { fingerprint: u64, reply: oneshot::Sender<Option<Decision>> }`.
///  One
///           point-read request carrying its reply channel.
/// Why:
///       A read blocks on the reply;
///  writes are a separate fire-and-forget channel.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Read = { fingerprint: bigint; reply: Resolve<Decision | null> };
/// ```
struct Read {
    /// What:
    ///      `fingerprint: u64`.
    ///  The cache key to look up.
    /// Why:
    ///       Bound into the shared `get`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// fingerprint: bigint;
    /// ```
    fingerprint: u64,
    /// What:
    ///      `reply: oneshot::Sender<Option<Decision>>`.
    ///  Where the decision (or a miss) goes.
    /// Why:
    ///       The blocking caller awaits exactly one answer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reply: Resolve<Decision | null>;
    /// ```
    reply: oneshot::Sender<Option<Decision>>,
}

/// What:
///      `struct Write { fingerprint: u64, decision: Decision }`.
///  One fire-and-forget upsert.
/// Why:
///       Writes never block the JNI thread;
///  the cache's precedence keeps an exact row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Write = { fingerprint: bigint; decision: Decision };
/// ```
struct Write {
    /// What:
    ///      `fingerprint: u64`.
    ///  The cache key.
    /// Why:
    ///       Part of the row's primary key.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// fingerprint: bigint;
    /// ```
    fingerprint: u64,
    /// What:
    ///      `decision: Decision`.
    ///  The decision to store (`Copy`,
    ///  cheap over the channel).
    /// Why:
    ///       The cached value.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// decision: Decision;
    /// ```
    decision: Decision,
    /// What:
    ///      `applied: Option<oneshot::Sender<()>>`.
    ///  Sent once the upsert has run,
    /// when the caller waits for it.
    /// Why:
    ///       A warm must not return before its decision is readable by the peak read.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// applied: (() => void) | null;
    /// ```
    applied: Option<oneshot::Sender<()>>,
}

/// What:
///      `struct Forget { fingerprint: u64, reply: oneshot::Sender<bool> }`.
///  One removal of a track's stored decision,
///  answered with whether a row was removed.
/// Why:
///       A re-analysis must drop the old decision before measuring again,
///  and the caller waits so the next read cannot see the stale row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Forget = { fingerprint: bigint; reply: (removed: boolean) => void };
/// ```
struct Forget {
    /// What:
    ///      `fingerprint: u64`.
    ///  The cache key of the decision to remove.
    /// Why:
    ///       Names the one row to delete.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// fingerprint: bigint;
    /// ```
    fingerprint: u64,
    /// What:
    ///      `reply: oneshot::Sender<bool>`.
    ///  Carries whether a row was removed back to the caller.
    /// Why:
    ///       The JNI caller blocks on it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reply: (removed: boolean) => void;
    /// ```
    reply: oneshot::Sender<bool>,
}

/// What:
///      `enum Mutation { Write(Write), Forget(Forget) }`.
///  Everything that changes the cache,
/// in the order it was sent.
/// Why:
///       One FIFO channel keeps a removal behind any earlier upsert for the same track.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Mutation = { kind: "write"; write: Write } | { kind: "forget"; forget: Forget };
/// ```
enum Mutation {
    /// What:
    ///      `Write(Write)`.
    ///  Store or upgrade one decision.
    /// Why:
    ///       The cache's precedence applies.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "write", write }
    /// ```
    Write(
        /// The decision to store.
        Write,
    ),
    /// What:
    ///      `Forget(Forget)`.
    ///  Remove one decision.
    /// Why:
    ///       Clears a track before it is measured again.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "forget", forget }
    /// ```
    Forget(
        /// The track whose decision to remove.
        Forget,
    ),
}

/// What:
///      `pub struct TruePeakService { read_tx, write_tx }`.
///  The handle Kotlin holds as a
///           `jlong`:
///  only the two channel senders (both `Send + Sync`),
///  so a shared
///           `&TruePeakService` is sound across JNI threads.
/// Why:
///       The Turso connection stays on the actor thread;
///  callers reach it by message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TruePeakService { readTx: Sender<Read>; writeTx: Sender<Write>; }
/// ```
pub struct TruePeakService {
    /// What:
    ///      `read_tx: UnboundedSender<Read>`.
    ///  The read-request sender.
    /// Why:
    ///       Carries `Get`s to the actor.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// readTx: Sender<Read>;
    /// ```
    read_tx: UnboundedSender<Read>,
    /// What:
    ///      `write_tx: UnboundedSender<Write>`.
    ///  The write-request sender.
    /// Why:
    ///       Carries fire-and-forget upserts to the actor.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// writeTx: Sender<Write>;
    /// ```
    write_tx: UnboundedSender<Mutation>,
}

/// What:
///      `impl TruePeakService { ... }`.
///  Open the actor and the sync get/put surface.
/// Why:
///       The JNI orchestrates decode around these fast cache ops.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TruePeakService { static open() {} get() {} put() {} }
/// ```
impl TruePeakService {
    /// What:
    ///      `pub(crate) fn open(db_path: String) -> TruePeakService`.
    ///  Spawn the actor thread
    ///           for the database at `db_path` and return a handle;
    ///  `pub(crate)` means every module in
    ///           this crate may call it,
    ///  though only the create entry in `service_jni.rs` does.
    /// Why:
    ///       One actor owns the runtime,
    ///  the connection,
    ///  and the identity.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static open(dbPath: string): TruePeakService { ... }
    /// ```
    pub(crate) fn open(db_path: String) -> TruePeakService {
        // What:     Create the read and write channels.
        // Why:      Separate queues so the actor can bias reads ahead of a write backlog.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const [readTx, readRx] = channel(); const [writeTx, writeRx] = channel();
        // ```
        let (read_tx, read_rx) = mpsc::unbounded_channel::<Read>();
        let (write_tx, write_rx) = mpsc::unbounded_channel::<Mutation>();
        // What:     Spawn the detached actor thread with a current-thread runtime.
        // Why:      Keep the Turso connection on one thread; the JNI threads stay decode-only.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // startWorker(() => runtime.blockOn(run(dbPath, readRx, writeRx)));
        // ```
        thread::spawn(move || {
            let built = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            // Log a failed build before `expect` panics, as the former `inspect_err` did.
            if let Err(error) = &built {
                tracing::error!(error = %error, "could not build cache runtime");
            }
            let runtime = built.expect("music-player: build cache runtime");
            runtime.block_on(run(db_path, read_rx, write_rx));
        });
        // What:     Return the handle. Tail -> return.
        // Why:      Kotlin holds it; dropping it closes the channels and the actor exits.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { readTx, writeTx };
        // ```
        return TruePeakService {
            read_tx,
            write_tx,
        };
    }

    /// What:
    ///      `fn get(&self, fingerprint: u64) -> Option<Decision>`.
    ///  Block briefly for one
    ///           cached decision,
    ///  or `None` on miss/closed actor.
    /// Why:
    ///       Both resolve and warm check the cache before decoding.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get(fingerprint: bigint): Decision | null { ... }
    /// ```
    pub(crate) fn get(
        &self,
        fingerprint: u64,
    ) -> Option<Decision> {
        // What:     Make the reply pair; send `Read`.
        // Why:      The actor answers on `reply_tx`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const [replyTx, replyRx] = oneshot();
        // ```
        let (reply_tx, reply_rx) = oneshot::channel();
        // What:     Send the read; a closed actor => treat as a miss.
        // Why:      Never hang if the cache thread is gone.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!readTx.send({ fingerprint, reply: replyTx })) return null;
        // ```
        if self
            .read_tx
            .send(Read {
                fingerprint,
                reply: reply_tx,
            })
            .is_err()
        {
            return None;
        }
        // What:     Block for the answer; flatten send/recv failure into `None`. Tail -> return.
        // Why:      `blocking_recv` is valid: the JNI thread is not inside the runtime.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return (await replyRx) ?? null;
        // ```
        return reply_rx.blocking_recv().ok().flatten();
    }

    /// What:
    ///      `fn put(&self, fingerprint: u64, decision: Decision)`.
    ///  Fire-and-forget a
    ///           decision to the actor.
    /// Why:
    ///       Persist without blocking the JNI thread;
    ///  precedence keeps an exact row.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// put(fingerprint: bigint, decision: Decision): void { ... }
    /// ```
    pub(crate) fn put(
        &self,
        fingerprint: u64,
        decision: Decision,
    ) {
        // What:     Send the write; ignore a closed actor.
        // Why:      A dropped cache thread just means nothing is persisted.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // writeTx.send({ fingerprint, decision });
        // ```
        let _ = self.write_tx.send(Mutation::Write(Write {
            fingerprint,
            decision,
            applied: None,
        }));
    }

    /// What:
    ///      `pub(crate) fn put_and_wait(&self, fingerprint: u64, decision: Decision)`.
    ///  Store a decision and block until the actor has applied it.
    /// Why:
    ///       A background warm returns only after its exact decision is readable,
    ///  so the peak read that follows sees the new value.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async putAndWait(fingerprint, decision): Promise<void> { ... }
    /// ```
    pub(crate) fn put_and_wait(
        &self,
        fingerprint: u64,
        decision: Decision,
    ) {
        let (applied_tx, applied_rx) = oneshot::channel();
        let write = Write {
            fingerprint,
            decision,
            applied: Some(applied_tx),
        };
        if self.write_tx.send(Mutation::Write(write)).is_err() {
            return;
        }
        let _ = applied_rx.blocking_recv();
    }

    /// What:
    ///      `pub(crate) fn forget(&self, fingerprint: u64) -> bool`.
    ///  Remove one track's stored decision and block until it is gone.
    /// Why:
    ///       A re-analysis must not measure while the old exact row is still present.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// forget(fingerprint): boolean { ... }
    /// ```
    pub(crate) fn forget(
        &self,
        fingerprint: u64,
    ) -> bool {
        let (reply_tx, reply_rx) = oneshot::channel();
        let request = Forget {
            fingerprint,
            reply: reply_tx,
        };
        if self.write_tx.send(Mutation::Forget(request)).is_err() {
            return false;
        }
        return reply_rx.blocking_recv().unwrap_or(false);
    }
}

/// What:
///      `async fn run(db_path, mut read_rx, mut write_rx)`.
///  The actor body:
///  open the cache,
///           compute the identity,
///  then serve until both channels close.
/// Why:
///       One place owns the cache,
///  the identity,
///  and the read-biased loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function run(dbPath, readRx, writeRx) { ... }
/// ```
async fn run(
    db_path: String,
    mut read_rx: UnboundedReceiver<Read>,
    mut write_rx: UnboundedReceiver<Mutation>,
) {
    // What:     `let cache = open_cache(&db_path).await;`. The open cache, or `None` (degraded).
    // Why:      A degraded actor still answers every request (misses and drops).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const cache = await openCache(dbPath);
    // ```
    let cache = open_cache(&db_path).await;
    // What:     `let identity = default_policy().cache_identity(decoder_stack_id());`. The
    //           four-part cache key, fixed for the run.
    // Why:      Every get/put keys on it, so a policy or decoder change starts fresh.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const identity = defaultPolicy().cacheIdentity(decoderStackId());
    // ```
    let identity = default_policy().cache_identity(decoder_stack_id());
    // The actor is ready; log whether it opened a cache or is running degraded.
    tracing::info!(degraded = cache.is_none(), "truepeak service actor started");
    // What:     The read-biased serve loop; `else => break` fires once both channels close.
    // Why:      Reads win over a write backlog; clean shutdown when the handle drops.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // while (true) { select reads-first; on both-closed break; }
    // ```
    loop {
        tokio::select! {
            biased;
            Some(request) = read_rx.recv() => {
                let _ = request.reply.send(get(cache.as_ref(), identity, request.fingerprint).await);
            }
            Some(mutation) = write_rx.recv() => {
                match mutation {
                    Mutation::Write(request) => put(cache.as_ref(), identity, request).await,
                    Mutation::Forget(request) => {
                        let removed = forget(cache.as_ref(), identity, request.fingerprint).await;
                        let _ = request.reply.send(removed);
                    }
                }
            }
            else => break,
        }
    }
    // Both channels closed (the handle was released); the actor shuts down cleanly.
    tracing::info!("truepeak service actor stopped");
}

/// What:
///      `async fn open_cache(db_path: &str) -> Option<DecisionCache>`.
///  Open the shared
///           decision cache,
///  or `None` on failure (degraded).
/// Why:
///       A bad cache file must not crash the actor;
///  it degrades to a no-op cache.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function openCache(dbPath) { try { return await DecisionCache.open(dbPath); } catch { return null; } }
/// ```
async fn open_cache(db_path: &str) -> Option<DecisionCache> {
    // What:     `match DecisionCache::open(db_path).await { ... }`. Open (creating the schema);
    //           log and degrade on error. Tail -> return.
    // Why:      Keep the actor alive even if persistence is unavailable.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { return await DecisionCache.open(dbPath); } catch (e) { warn(e); return null; }
    // ```
    match DecisionCache::open(db_path).await {
        Ok(cache) => return Some(cache),
        Err(error) => {
            tracing::warn!(error = %error, "cache open failed; running degraded");
            return None;
        }
    }
}

/// What:
///      `async fn get(cache: Option<&DecisionCache>, identity: CacheIdentity, fingerprint:
///           u64) -> Option<Decision>`.
///  Point-read a decision,
///  or `None` on miss/degraded/error.
/// Why:
///       A read failure is a cache miss,
///  never a crash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function get(cache, identity, fingerprint) { ... }
/// ```
async fn get(
    cache: Option<&DecisionCache>,
    identity: CacheIdentity,
    fingerprint: u64,
) -> Option<Decision> {
    // What:     `let decision_cache = cache?;`. Degraded run has no cache. The unwrapped
    //           reference gets its own name so it never shadows the `Option` parameter.
    // Why:      Answer a miss without a database.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!cache) return null;
    // ```
    let decision_cache = cache?;
    // What:     `decision_cache.get(fingerprint, identity).await.ok().flatten()`. Read; any
    //           error collapses to a miss. Tail -> return.
    // Why:      Degrade to a miss, never propagate a crash across JNI.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { return await cache.get(fingerprint, identity); } catch { return null; }
    // ```
    return decision_cache
        .get(fingerprint, identity)
        .await
        .ok()
        .flatten();
}

/// What:
///      `async fn put(cache: Option<&DecisionCache>, identity: CacheIdentity, request:
///           Write)`.
///  Store one decision;
///  no-op when degraded.
/// Why:
///       Serves a write;
///  the cache's precedence keeps an exact row from downgrading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function put(cache, identity, request) { ... }
/// ```
async fn put(
    cache: Option<&DecisionCache>,
    identity: CacheIdentity,
    request: Write,
) {
    // A degraded run has no cache to write to; the ack below still fires so a waiter is released.
    if let Some(decision_cache) = cache {
        // One bad write must not stall the actor, so the error is logged and dropped.
        if let Err(error) = decision_cache
            .put(request.fingerprint, identity, &request.decision)
            .await
        {
            tracing::warn!(error = %error, "cache put failed; write dropped");
        }
    }
    if let Some(applied) = request.applied {
        let _ = applied.send(());
    }
}

/// Remove one track's row;
/// a degraded run has nothing stored,
/// so nothing is removed.
async fn forget(
    cache: Option<&DecisionCache>,
    identity: CacheIdentity,
    fingerprint: u64,
) -> bool {
    let Some(decision_cache) = cache else {
        return false;
    };
    match decision_cache.remove(fingerprint, identity).await {
        Ok(removed) => return removed,
        Err(error) => {
            tracing::warn!(error = %error, "cache remove failed; decision kept");
            return false;
        }
    }
}

/// What:
///      `fn service_ref<'a>(handle: jlong) -> Option<&'a TruePeakService>`.
///  Rebuild a
///           shared reference from the opaque handle,
///  or `None` for the `0` sentinel.
/// Why:
///       Every JNI method turns the `jlong` back into a `&TruePeakService`;
///  sharing (not
///           `&mut`) is sound because the handle is `Send + Sync` (only channel senders).
/// Gotcha:
///    `unsafe` is a PROMISE the handle is valid and not released;
///  Kotlin owns that
///           contract (one live handle,
///  released once).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function serviceRef(handle) { return handle === 0 ? null : handleTable.get(handle); }
/// ```
pub(crate) fn service_ref<'a>(handle: jlong) -> Option<&'a TruePeakService> {
    // What:     `if handle == 0 { return None; }`. The create sentinel means "no service".
    // Why:      Never turn `0` into a pointer and dereference it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (handle === 0) return null;
    // ```
    if handle == 0 {
        return None;
    }
    // What:     `Some(unsafe { &*(handle as *const TruePeakService) })`. Cast the int to a
    //           const pointer and take a SHARED borrow. Tail -> return.
    // Why:      Shared, because concurrent JNI threads read it; the type is `Sync`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return handleTable.get(handle);
    // ```
    return Some(unsafe { &*(handle as *const TruePeakService) });
}
