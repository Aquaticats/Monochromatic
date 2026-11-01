# Web storage `setItem` is synchronous and Window-only (WHATWG HTML, all engines), so every session-storage sink record costs main-thread time fire-and-forget cannot hide; ~5 µs per record in Chromium 149, ~10x cheaper batched

## Symptom

Every record logged through `@monochromatic-dev/logger-core` pays the
session-storage sink's full write cost on the calling thread,
even though the sink returns `Promise<void>` and the logger dispatches
writes fire-and-forget.

Discovered under Node:
a CPU profile of an oxlint semantic-plugin rebuild attributed 522 ms of
13.7 s to the session-storage sink
(383 ms self time in `write`,
 137 ms in `evictOldest`),
because oxlint runs JS-plugin visitors synchronously on the main JS
thread and the plugin logs extensively.
Self time in a sampling profiler means the main thread was executing
those frames;
a genuinely non-blocking sink would not appear there.
Commit `583f1c25b` stopped electing the sink under Node.

Browsers still elect the sink,
so the question this doc answers is what each record costs there and
whether web storage can be written more cheaply.

## Root cause

The deciding source is the WHATWG HTML spec's web storage section
(<https://html.spec.whatwg.org/multipage/webstorage.html>),
not any one engine:
the synchronous surface is the specified contract.

The `Storage` interface is a synchronous setter exposed only on
`Window`:

```webidl
[Exposed=Window]
interface Storage {
  readonly attribute unsigned long length;
  DOMString? key(unsigned long index);
  getter DOMString? getItem(DOMString key);
  setter undefined setItem(DOMString key, DOMString value);
  deleter undefined removeItem(DOMString key);
  undefined clear();
};
```

The `setItem(key, value)` method steps are a synchronous algorithm,
including the quota failure:
"If value cannot be stored,
 then throw a `QuotaExceededError`.
Set this's map[key] to value.
...
Broadcast this with key,
 oldValue,
 and value."
There is no promise-returning variant,
and `[Exposed=Window]` means workers never see `sessionStorage` or
`localStorage`,
so the write cannot be moved off the main thread at all.
The spec also declines cross-process coordination:
"This specification does not define the interaction with other agent
clusters in a multiprocess user agent,
and authors are encouraged to assume that there is no locking
mechanism."

On our side,
the sink's `write` therefore runs entirely before its promise exists
(`package/logger/core/src/sink/session-storage.ts:245`):

```ts
        globalThis.sessionStorage
          .setItem(
          storageKey(state.lineCounter,),
          serialized,
        );
        state.lineCounter++;
        state.usedChars += recordChars;
        // A landed write re-arms a single give-up report for the next episode.
        state.reportedFailure = false;
        return Promise.resolve();
```

and the logger's fire-and-forget dispatch can only skip waiting,
never offload
(`package/logger/core/src/create-logger.ts:193`):

```ts
      const trackedWrite = trackWrite({
        writePromise: entry.sink
          .write(record,),
      },);
```

By the time `trackedWrite` could be ignored,
the serialization,
 the `setItem`,
 the footprint accounting,
and any `evictOldest` scans have already executed on the caller's
stack.

## Verification

Versions under test:

- HeadlessChrome/149.0.0.0 via `agent-browser`,
   Linux x86_64.
- `@monochromatic-dev/logger-core` at workspace commit `583f1c25b`.

Harness:
save the script below,
open any page with `agent-browser open <url>`,
and run `agent-browser eval "$(cat storage-bench.js)"`.
Web storage works on a `file://` page in Chromium 149,
but OPFS `getFileHandle` there throws
`SecurityError: It was determined that certain files are unsafe for
access within a Web application`,
so serve the page over `http://127.0.0.1` (e.g. `python3 -m
http.server`) to measure the OPFS comparison.

```js
// storage-bench.js
(async () => {
  const results = {};
  const mkSerialized = (len) =>
    JSON.stringify({ level: 'debug', message: 'x'.repeat(len), timestamp: Date.now() });

  for (const storeName of ['sessionStorage', 'localStorage']) {
    const store = window[storeName];
    for (const msgLen of [100, 1000]) {
      const serialized = mkSerialized(msgLen);
      const n = msgLen === 100 ? 10000 : 1500;

      // per-record setItem under fresh keys (current sink shape)
      store.clear();
      let t0 = performance.now();
      for (let i = 0; i < n; i++) store.setItem('bench.' + i, serialized);
      let t1 = performance.now();
      const perWriteUs = ((t1 - t0) / n) * 1000;

      // steady-state at cap: each record evicts oldest (getItem+removeItem) then setItem
      store.clear();
      for (let i = 0; i < n; i++) store.setItem('bench.' + i, serialized);
      t0 = performance.now();
      for (let i = 0; i < n; i++) {
        const old = store.getItem('bench.' + i);
        store.removeItem('bench.' + i);
        if (old === null) throw new Error('evict miss');
        store.setItem('bench.' + (n + i), serialized);
      }
      t1 = performance.now();
      const perWriteEvictUs = ((t1 - t0) / n) * 1000;

      // batched: buffer 100 records in memory, one setItem per batch
      store.clear();
      const batchSize = 100;
      t0 = performance.now();
      let buf = [];
      let batchKey = 0;
      for (let i = 0; i < n; i++) {
        buf.push(serialized);
        if (buf.length === batchSize) {
          store.setItem('batch.' + batchKey++, buf.join('\n'));
          buf = [];
        }
      }
      if (buf.length > 0) store.setItem('batch.' + batchKey, buf.join('\n'));
      t1 = performance.now();
      const perWriteBatchedUs = ((t1 - t0) / n) * 1000;
      store.clear();

      results[storeName + '.msg' + msgLen] = {
        n,
        perWriteUs: +perWriteUs.toFixed(2),
        perWriteEvictUs: +perWriteEvictUs.toFixed(2),
        perWriteBatchedUs: +perWriteBatchedUs.toFixed(2),
      };
    }
  }

  // JSON.stringify alone, per record
  {
    const record = { level: 'debug', message: 'x'.repeat(100), timestamp: Date.now() };
    const n = 100000;
    let sum = 0;
    const t0 = performance.now();
    for (let i = 0; i < n; i++) sum += JSON.stringify(record).length;
    const t1 = performance.now();
    results.stringifyMsg100Us = +(((t1 - t0) / n) * 1000).toFixed(3);
    results.stringifySum = sum > 0;
  }

  // OPFS: main-thread enqueue cost (issue without awaiting) and awaited total
  try {
    const root = await navigator.storage.getDirectory();
    const fh = await root.getFileHandle('bench.jsonl', { create: true });
    const writable = await fh.createWritable();
    const line = mkSerialized(100) + '\n';
    const n = 2000;
    const promises = [];
    const t0 = performance.now();
    for (let i = 0; i < n; i++) promises.push(writable.write(line));
    const t1 = performance.now();
    await Promise.all(promises);
    const t2 = performance.now();
    await writable.close();
    await root.removeEntry('bench.jsonl');
    results.opfs = {
      n,
      enqueuePerWriteUs: +(((t1 - t0) / n) * 1000).toFixed(2),
      settledPerWriteUs: +(((t2 - t0) / n) * 1000).toFixed(2),
    };
  } catch (error) {
    results.opfs = 'unavailable: ' + String(error);
  }

  return JSON.stringify(results, null, 2);
})()
```

Measured per-record main-thread cost
(medians of two runs,
 `file://` and `http://127.0.0.1`,
 which agreed
within run-to-run noise):

- `sessionStorage`,
   130-char record:
  4.9 µs per `setItem`;
  10.2 µs at eviction steady state
  (`getItem` + `removeItem` + `setItem` per record,
   the sink's shape
  once its half-quota cap is reached);
  0.34 µs per record when 100 records share one `setItem`.
- `sessionStorage`,
   1030-char record:
  5.7 µs;
   10.5 µs evicting;
   1.6 µs batched.
- `localStorage`,
   130-char record:
  4.9 µs;
   9.7 µs evicting;
   0.26 µs batched.
- `localStorage`,
   1030-char record:
  5.1 µs;
   10.2 µs evicting;
   1.3 µs batched.
- `JSON.stringify` of the 130-char record alone:
   0.18 µs,
  so the storage call,
   not serialization,
   dominates the sink.
- OPFS `FileSystemWritableFileStream.write` issued without awaiting:
  15.9 µs per record of main-thread enqueue cost;
  382 µs per record wall time for 2000 writes issued then all awaited.

### Batch-size sweep

Rerunning the harness's batched loop with batch sizes swept across
1,
 5,
 10,
 25,
 50,
 100,
 250,
 500,
 and 1000 records per key
(same engine,
 `http://127.0.0.1` origin;
the sweep is the embedded harness with `batchSize` iterated instead of
fixed) shows the batch size is not magic;
the curve is per-call overhead amortized as `overhead / batchSize`
plus a per-char cost that grows with batch bytes:

- `sessionStorage`,
   130-char records,
   µs per record:
  4.84 at 1;
   1.37 at 5;
   0.74 at 10;
   0.47 at 25;
   0.44 at 50;
  0.35 at 100;
   0.26 at 250;
   0.26 at 500;
   0.40 at 1000.
- `sessionStorage`,
   1030-char records,
   µs per record:
  5.87 at 1;
   4.87 at 5;
   2.40 at 10;
   2.00 at 25;
   1.67 at 50;
  1.73 at 100;
   3.87 at 250;
   7.87 at 500;
   8.93 at 1000,
  a genuine U-shape:
   at 500 records the flush writes a ~515 KB value
  and per-record cost exceeds unbatched `setItem`.
- `localStorage`,
   130-char records,
   µs per record:
  5.56 at 1;
   1.11 at 5;
   0.62 at 10;
   0.37 at 25;
   0.27 at 50;
  0.24 at 100;
   0.32 at 250;
   0.22 at 500;
   0.19 at 1000.
- `localStorage`,
   1030-char records,
   µs per record:
  7.67 at 1;
   2.13 at 5;
   1.87 at 10;
   1.60 at 25;
   1.33 at 50;
  1.27 at 100;
   1.33 at 250;
   1.53 at 500;
   3.33 at 1000.
- OPFS,
   130-char records,
   main-thread enqueue µs per record:
  14.3 at 1;
   1.45 at 10;
   0.35 at 100;
   0.20 at 1000;
  awaited-settle µs per record drops 368 to 0.65 across the same
  sweep (fewer IPC round trips),
   with no U-turn up to the ~130 KB
  write.

The knee sits at roughly 10 to 25 records:
most of the win is banked there,
and the region from 25 to a few hundred records is flat for small
records.
The U-turn for 1030-char records puts the minimum near 50 to 100
records,
 about 50 KB to 100 KB per flush,
so batch bytes,
 not batch count,
 is the variable to cap.
Flush-call latency scales the same way:
50 large records flush in about 84 µs,
while 500 flush in about 3.9 ms,
which is jank territory if it lands mid-interaction.

Two conclusions the numbers force:

- The synchronous sink is cheap per record (about 5 µs) but linear in
  record count on the main thread;
  the Node incident was volume (thousands of records inside one
  synchronous rebuild),
   not a slow single write.
- The "async" OPFS sink costs about 3x more main-thread time per
  record (15.9 µs enqueue) than the synchronous `setItem` it is
  supposed to improve on;
  fire-and-forget hides its disk latency but not its per-call enqueue
  and IPC bookkeeping.

## Verified workarounds

### Reject election under Node (shipped in `583f1c25b`, reverted in `c040389f8`)

Commit `583f1c25b` made `verify` reject Node-branded runtimes despite a
working backend,
 on the judgment that the process-local store served no
diagnostic purpose there.
That judgment was reversed by decision:
the sink stays elected wherever `sessionStorage` round-trips a probe,
because availability plus one uniform cross-runtime behavior outranks
the duplication,
and the buffered write path below makes the cost acceptable.
Kept here as history so the rejection is not re-shipped;
the current `verify` is probe-only
(`package/logger/core/src/sink/session-storage.ts`).

### One uniform buffered write path (shipped in `c040389f8`)

The sink buffers serialized records and persists them as
newline-joined JSONL batches,
 one batch per counter-incremented key,
identically on every runtime;
 no per-runtime mode exists
(`package/logger/core/src/sink/session-storage.ts`,
persistence engine split to
`package/logger/core/src/sink/session-storage-store.ts`).
A batch flushes:

- synchronously from inside `write` at a 32 KiB joined-length cap
  (`FLUSH_BUFFER_CAP_CHARS`),
   the flat bottom of the measured
  batch-size curve on both Chromium 149 and Node 26,
   clear of the
  measured U-turn where ~100 KiB+ flushes cost more per record than
  not batching;
- synchronously from inside `write` on `warn`-or-worse severity,
  so every record up to and including a failure is persisted before
  control returns;
- by a 250 ms quiet-period deadline timer,
   `unref`ed where the handle
  supports it so a pending flush never holds a process open;
- on `pagehide` and on the document becoming hidden,
   where those
  events exist (no-ops elsewhere);
- via the sink `flush` hook,
   which logger-level `flush()` drains.

When appending a record would breach the cap,
 the existing entries
flush first,
 so an oversized record's quota give-up can only drop that
record,
 never its batch-mates.

Measured outcome at the consumer boundary (built `dist`,
 Node 26.5):
a 10,000-record `debug` storm through a logger with only this sink
costs 2.78 µs per record,
 of which about 2 µs is logger dispatch
overhead (a suppressed-console-only logger measures 2.05 µs),
versus ~14.7 µs per unbatched `setItem` before;
a `warn` record lands itself and the buffered records ahead of it as
one JSONL batch;
and a process holding buffered records and an armed deadline timer
exits promptly (~0.5 s total script time,
 no timer hold-open).

Tradeoffs:

- Records buffered since the last flush are lost when the tab crashes
  hard (the process dies without `pagehide` firing).
  Crash forensics is a core reason a session-storage sink exists,
  so the flush interval bounds the loss window and must stay small;
  `pagehide`/`visibilitychange` cover navigation,
   reload,
   and close.
  The crash-durability subsection below sizes this loss honestly:
  per-record `setItem` never guaranteed hard-crash durability either.
- Eviction granularity coarsens to whole batches,
  so the half-quota cap overshoots by up to one batch.
- Readers must split stored values on the record delimiter;
  the delimiter must be one JSON strings cannot contain unescaped
  (newline works,
   since `JSON.stringify` escapes it inside strings).

### OPFS sink batched through the same shared buffer (shipped in `13d993c14`)

The buffering stage is extracted to
`package/logger/core/src/sink/record-buffer.ts` (same triggers,
 same
constants) and both sinks compose it;
the OPFS sink issues one queued stream write per joined batch,
 tracks
in-flight batch writes,
 and its `flush` hook resolves only once every
issued batch has settled.
Measured at the consumer boundary (built `dist`,
 HeadlessChrome 149):
a 10,000-record `debug` storm costs 0.84 µs of main-thread enqueue per
record versus 15.9 µs unbatched,
 1.76 µs per record with all batch
writes awaited,
and an `info` plus `warn` pair through the browser session-storage
sink lands as one two-line JSONL batch,
 confirming the shared buffer
end to end.

Durability caveat,
 pre-existing and unchanged by batching:
`FileSystemWritableFileStream` stages writes and the OPFS file's
readable content updates only when the writable closes (the sink's own
`verify` works around exactly this for its probe),
and the sink keeps its stream open for the whole session,
so OPFS log content is not observable mid-session under either the
per-record or the batched design.

A localStorage sink was absent from the package until 2026-07-22:
the original sink set (commit `7fe3a2044`) shipped console,
 file,
OPFS,
 sessionStorage,
 and noop with no recorded reason for the gap,
an omission rather than a decision.
`package/logger/core/src/sink/local-storage.ts` now composes
`record-buffer.ts` exactly the way this document prescribed,
so the synchronous-cost finding applies to it the same way,
and the localStorage rows in the sweep above supplied its numbers.
Its quota table was fill-probed separately
(`local-storage-quota.ts`):
5,242,880 chars total on Node 26 with `--localstorage-file`,
5,242,880 on headless Chromium 149 over an `http://127.0.0.1` origin,
and 10,477,569 on Deno 2.9,
which lands 8 KiB short of the 10 MiB its sessionStorage measures.
Design rationale for its run-scoped keys and cross-run eviction:
`package/logger/core/DECISIONS.md`.

An IndexedDB sink followed the same day and took the default
persistent-browser slot from OPFS,
whose staged-until-close stream made its records unrecoverable in
exactly the crash scenarios a persistent sink exists for.
Headless Chromium 149 measurements that drove the shape
(130-char-equivalent records):
18.6 µs per record for one transaction per record
(worse than unbatched `setItem`,
 a trap for naive ports);
8.0 µs for one transaction per batch with one `add` per record;
0.76 µs for one `add` per batch storing an array of record objects;
0.15 µs for one `add` per batch storing the exact newline-joined JSONL
string `record-buffer.ts` emits,
which is the shape shipped,
settling in 1.3 ms per 10,000 records' worth of batches.
The rejected keep-OPFS alternative,
closing and reopening its writable around every batch for durability,
measured 1.56 ms per flush cycle,
6.26 µs per record amortized.
Full rationale,
including the relaxed-durability choice and retention cap:
`package/logger/core/DECISIONS.md`.

### Crash durability under batching

"Batching loses the records a crash was supposed to explain" holds
only for one crash class,
 and the unbatched design is weaker against
that class than it appears.

Crash classes and what each loses:

- Uncaught exception or unhandled rejection (the common "app crashed"
  for a web app):
   the tab keeps running and handlers still execute.
  A sink that flushes immediately on `warn`/`error`/`fatal` records,
  plus `window.onerror`/`unhandledrejection` listeners that flush,
  loses nothing:
  the flush drains the buffered `debug` records preceding the error.
  Severity-triggered flushes cost per-record `setItem` prices only for
  rare records,
   so amortization is unaffected.
- Navigation,
   reload,
   tab close,
   backgrounded-tab kill:
  `pagehide` and `visibilitychange` to `hidden` fire;
  flushing there loses nothing.
- Hang or runaway allocation from our own bug,
   ended by the user
  killing the unresponsive tab or the OS OOM-killing the renderer:
  nothing throws,
   so no severity flush ever fires,
  and once the main thread wedges,
   timer- and idle-based deadline
  flushes can never run again either.
  The batched buffer dies inside the wedged process's JS heap.
  Per-record writes were already handed to Chromium at emit time,
  and for `sessionStorage` the authoritative map lives in the browser
  process (that is what the async `Put` cited below updates),
  so records logged before and during the hang survive the kill,
  minus at most an in-flight tail.
  This is the class where batching genuinely forfeits forensics that
  per-record keeps,
  and because our code causes it,
   it is not rare.
- Hard renderer/browser/OS crash from below us:
   JS never runs again;
  the buffer since the last deadline flush is lost.
  Bounded by the flush deadline and byte cap.

The honest baseline for that last class:
per-record `setItem` is synchronous only into the renderer's local
cache.
In Chromium (`chromium/chromium@36ac8f31796a`,
`third_party/blink/renderer/modules/storage/cached_storage_area.cc:73`),
`SetItem` updates the in-process map and then forwards the value to
the browser process asynchronously:

```cpp
  if (!map_->SetItem(key, value, &old_value))
    return false;
  ...
  if (!is_session_storage_for_prerendering_) {
    remote_area_->Put(
        StringToUint8Vector(key, GetKeyFormat()),
        StringToUint8Vector(value, value_format), optional_old_value,
        mojom::blink::StorageAreaSource::New(page_url, source_id),
        base::IgnoreArgs<bool>(MakeVirtualTimePauserCallback(source)));
  }
```

so a renderer crash can drop records whose `Put` had not crossed the
process boundary,
regardless of how synchronously JS called `setItem`.
One layer further down,
 the browser process batches `localStorage`
disk commits itself
(`components/services/storage/dom_storage/local_storage_impl.cc:56`):

```cpp
  // Delay for a moment after a value is set in anticipation
  // of other values being set, so changes are batched.
  static constexpr base::TimeDelta kCommitDefaultDelaySecs = base::Seconds(5);
  ...
  static const size_t kMaxBytesPerHour = kPerStorageAreaQuota;
  static constexpr int kMaxCommitsPerHour = 60;
```

so a whole-browser or OS crash can lose up to about 5 seconds of
"synchronously written" `localStorage` regardless of sink design,
and the spec quoted in Root cause guarantees nothing here.

Two project decisions weigh against each other here.
The humility principle:
assume our code fails far more often than Chromium,
and Chromium far more often than the OS;
never design as though our recovery code is more robust than the
layers below it.
Under it,
 per-record wins the hang-and-leak class outright:
it is the only design that needs none of our code to run after the
record is emitted,
whereas every buffered design bets that our flush scheduling survives
whatever bug is currently destroying the app.
The uniformity-and-availability decision,
 made later and final:
the sink stays elected wherever the backend round-trips,
and every runtime uses one identical write path;
no per-runtime modes.

The shipped design (`c040389f8`) satisfies the second decision and
bounds what the first one loses.
The 32 KiB cap flush runs synchronously inside `write` itself,
on the caller's stack,
so even a wedged main thread that keeps logging can never hold more
than one batch of unpersisted records:
the hang-class loss is capped at 32 KiB of tail,
not unbounded buffering.
`warn`-or-worse records flush themselves and everything buffered ahead
of them synchronously,
so no record at or before a logged failure is ever in the window.
The residual exposure versus per-record writes is exactly:
`debug`/`trace`/`info` records younger than the last flush trigger,
at most one batch,
lost only when the process dies with no further JS and no lifecycle
event.
Chromium and OS crashes,
 where the window also bites,
are the rarest class and already lossy at layers below us
(async mojo `Put`,
 5 s disk-commit batching).

## What does not work

- Offloading web storage to a worker:
  impossible by spec,
   `Storage` is `[Exposed=Window]`;
  workers get IndexedDB and OPFS instead,
  and a worker cannot touch the tab's `sessionStorage` at all,
  so posting records to a worker just moves the write back via
  `postMessage`.
- Making the sink "more async" (returning promises,
   `queueMicrotask`,
  awaiting before `setItem`):
  the storage call still runs on the main thread in the same turn or a
  later one;
  scheduling shuffles when,
   never where.
  The current `write` returns an already-settled promise,
  so callers were never waiting on it to begin with.
- Replacing the session-storage sink with the OPFS sink to save main
  thread:
  measured OPFS enqueue is about 15.9 µs per record versus 4.9 µs for
  `setItem`,
  so per-record it spends more main-thread time,
   not less.
- Rejecting election under Node instead of batching (the `583f1c25b`
  approach,
   since reverted):
  it judged the process-local store worthless and so priced uniform
  cross-runtime behavior at zero;
  the standing decision values availability plus one identical write
  path everywhere,
   and the buffered path makes that affordable,
  so runtime-brand rejection solved the cost by discarding the
  capability instead of fixing the cost.
- Per-record `requestIdleCallback` scheduling:
  pays scheduling overhead per record without amortizing the storage
  call,
   and reorders records;
  deferral only helps combined with batching,
   where it is the flush
  trigger.

## Upstream filing decision

`.out-of-scope/` was checked
(`bun-install`,
 `cargo-workspace`,
 `claude-code-upstream-bugs`,
`codex-harness`,
 `jsr`,
 `lightningcss`,
`low-impact-typescript-formatting`,
 `module-es-monolith`,
`pi-gpt55-long-context`,
 `terminal-title-fork-parity-tests`,
`typescript-project-references`);
no exemption covers web storage or the WHATWG HTML spec.

Walking the six constraints against upstream (the WHATWG HTML spec and
the engines implementing it):

1. **Is it really upstream's fault?**
   No.
   Synchronous,
    Window-only web storage is the specified design,
   and the spec explicitly declines coordination machinery
   ("authors are encouraged to assume that there is no locking
   mechanism").
   The platform's answer to "I need async storage" is IndexedDB and
   OPFS,
    which exist and which we already use.
2. **Can upstream fix it?**
   No.
   `setItem` is an `undefined`-returning setter;
   making it asynchronous or promise-returning breaks every existing
   caller on the web.
   This is the architectural-impossibility case the constraint names.
3. **Are they supporting this use case?**
   Synchronous small-value storage,
    yes;
   high-frequency hot-path writes,
    no,
   and the async storage APIs are the documented alternative.
4. **Would the repo welcome our contribution?**
   Not evaluated further;
    constraints 1 and 2 already fail.
5. **Will they likely fix it?**
   Not evaluated further;
    constraints 1 and 2 already fail.
6. **Have we prototyped a minimal fix compatible with their
   architecture?**
   Not applicable;
    there is nothing upstream to fix.
   The consumer-side batching workaround in this doc is the fix at our
   boundary.

Decision:
nothing to file and nothing to comment;
the behavior is correct by design and the remediation is entirely
consumer-side.
No draft issue is kept.

## Key takeaway

Fire-and-forget hides latency,
 not CPU:
any sink work before the first genuine await runs on the caller's
thread,
and for web storage that is all of it,
 about 5 µs per record in
Chromium 149 and about 15 µs on Node 26.
The shipped answer (`c040389f8`) is election by probe alone plus one
uniform buffered write path on every runtime:
one `setItem` per byte-capped batch
(32 KiB lands in the flat bottom of the curve,
 3x to 60x cheaper per
record;
 a fixed record count is the wrong knob,
 since ~100 KiB+
flushes cost more per record than not batching),
flushed synchronously in-write at the cap and on `warn`-or-worse,
by unref'd 250 ms deadline,
 on `pagehide`/hidden,
 and on `flush()`.
The in-write cap flush is what keeps the humility principle satisfied:
a wedged main thread can never hold more than one batch of tail.
The measured surprise held and drove the follow-up:
the OPFS sink's enqueue cost about 3x more main-thread time per record
than `setItem`,
so the same buffer now batches it too (`13d993c14`),
measured at 0.84 µs of enqueue per record through the built artifact.
