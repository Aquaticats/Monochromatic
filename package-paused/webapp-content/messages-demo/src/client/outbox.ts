/**
 * Persistent chunk-PUT outbox.
 *
 * The composer enqueues each chunk upload here instead of calling
 * `fetch` directly. The outbox:
 *
 * 1. Persists the upload to IndexedDB when the storage probe says yes,
 *    so a refresh in the middle of a tier-3 save resumes from where it
 *    left off.
 * 2. PUTs the chunk via the existing `/api/drafts/:id/chunks/:seq`
 *    endpoint with bounded exponential-backoff retries.
 * 3. On acknowledgment from the server (`{ ack }`: highest contiguous
 *    seq), drops every persisted entry whose `seq <= ack` for the same
 *    `draftId`, since those are now durable on the server.
 * 4. Re-attempts terminally failed uploads when the browser fires
 *    `online` or `visibilitychange`: the most common cause of
 *    terminal failure is a closed laptop lid, not a permanent error.
 *
 * The server's `chunks` table is the source of truth. The outbox is a
 * client-side resume buffer; losing it (e.g. private browsing closes
 * IDB) just means the worker has to re-send any chunks the server has
 * not acked yet.
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { readJson, } from './json-fetch.ts';
import {
  deleteAcked,
  openOutboxDb,
  persistOne,
  readPersistedQueue,
} from './outbox-idb.ts';

/**
 * Maximum number of PUT attempts per chunk before pausing.
 */
const PUT_MAX_ATTEMPTS = 3;

/**
 * Initial backoff delay between PUT retries, in milliseconds.
 */
const PUT_BACKOFF_BASE_MS = 250;

/**
 * Sentinel for "no IndexedDB handle": the probe lied or the open failed,
 * so the outbox runs in-memory. A unique `Symbol` rather than `null`: an
 * open handle is an `IDBDatabase`, so callers gate with `=== NO_IDB`.
 */
const NO_IDB: unique symbol = Symbol('messages-demo:no-idb',);

/**
 * Sentinel returned by `tryPutWithBackoff` when the retry budget is
 * exhausted. A unique `Symbol` rather than `null`: a success carries the
 * numeric ack, so the drain loop gates with `=== PUT_FAILED`.
 */
const PUT_FAILED: unique symbol = Symbol('messages-demo:put-failed',);

/**
 * One queued chunk upload. The shape mirrors the request body of
 * `PUT /api/drafts/:id/chunks/:seq` plus the route parameters, plus a
 * timestamp used only for IDB sort order on resume.
 */
export type ChunkUpload = {
  readonly draftId: string;
  readonly seq: number;
  readonly md: string;
  readonly html: string;
  readonly charCount: number;
};

/**
 * Outbox configuration.
 */
export type OutboxOptions = {
  /**
   * Set from `StorageCaps.idb`. When false, the outbox is in-memory.
   */
  readonly idbAvailable: boolean;
};

/**
 * Outbox public surface. Both in-memory and IDB modes return one.
 */
export type Outbox = {
  /**
   * Persists the upload (when IDB is available) and schedules its PUT
   * in the background. Resolves immediately; use `flushed()` to wait
   * for completion.
   */
  enqueue: (upload: ChunkUpload,) => Promise<void>;
  /**
   * Resolves once every enqueued upload has been acked by the server.
   * If the drain loop pauses on a terminal PUT failure (the queue is
   * non-empty when retries exhaust), this promise stays pending until
   * the next online / visibilitychange kick succeeds, since send must
   * not proceed to finalize while uploads are still in flight.
   */
  flushed: () => Promise<void>;
  /**
   * Number of uploads still waiting to ack.
   */
  pendingCount: () => number;
  /**
   * Detach event listeners and close IDB.
   */
  destroy: () => void;
};

/**
 * Builds an outbox. The returned object is safe to share across the
 * composer's lifetime; outbox state is process-local but IDB-backed
 * uploads survive reload.
 *
 * @param options - capability flags from `probeStorage`
 *
 * @returns ready-to-use outbox; rehydrated from IDB when available
 *
 * @example
 * ```ts
 * const outbox = await createOutbox({ idbAvailable: caps.idb });
 * await outbox.enqueue({ draftId, seq: 0, md, html, charCount });
 * await outbox.flushed();
 * ```
 */
export async function createOutbox(options: OutboxOptions,): Promise<Outbox> {
  /**
   * Opens the IDB handle if the caller probed it as available, swallowing
   * any post-probe failure (private mode quirks, quota races) so the
   * composer falls back to in-memory rather than blocking startup.
   *
   * @returns the opened handle, or `NO_IDB` when probe lied or open failed
   */
  async function maybeOpenIdb(): Promise<IDBDatabase | typeof NO_IDB> {
    if (!options.idbAvailable)
      return NO_IDB;
    try {
      return await openOutboxDb();
    }
    catch {
      return NO_IDB;
    }
  }
  /**
   * Lazily assigned IDB handle; left `NO_IDB` when the probe lied or the open call fails post-probe.
   */
  const idb = await maybeOpenIdb();

  /**
   * Reads the persisted queue from IDB, falling back to an empty queue if
   * the read raises (corrupted store, schema mismatch). Returns immediately
   * with `[]` when no handle is available.
   *
   * @returns persisted queue snapshot, or `[]` on missing handle or read failure
   */
  async function readInitialQueue(): Promise<ChunkUpload[]> {
    if (idb === NO_IDB)
      return [];
    try {
      return await readPersistedQueue(idb,);
    }
    catch {
      return [];
    }
  }
  /**
   * Initial queue populated from IDB when available; pushes from `enqueue` extend it later.
   */
  const queue = await readInitialQueue();
  /**
   * Closure-scoped state shared by every helper in this factory; queue and flags live here.
   */
  const state: {
    queue: ChunkUpload[];
    idb: IDBDatabase | typeof NO_IDB;
    draining: boolean;
    destroyed: boolean;
    waiters: (() => void)[];
  } = {
    queue,
    idb,
    draining: false,
    destroyed: false,
    waiters: [],
  };

  /**
   * Resolves every pending `flushed()` waiter and clears the list.
   */
  function notifyFlushed(): void {
    /**
     * Snapshot of `state.waiters` so the list can be reset before each resolver runs.
     */
    const pending = state.waiters;
    state.waiters = [];
    for (const resolve of pending)
      resolve();
  }

  /**
   * Background drain loop. Processes the queue head-first, sequential,
   * stopping when the queue empties or a PUT terminally fails (in
   * which case the next online/visibilitychange resumes). Uses
   * `using` for the draining-flag cleanup so a thrown IDB error in
   * `dropAcked` still resets the flag and notifies waiters.
   */
  async function drain(): Promise<void> {
    if (state.draining
      || state
      .destroyed)
      return;
    state.draining = true;
    /**
     * `using` disposable so the draining flag clears even when the body throws.
     */
    using _drainCleanup = {
      /**
       * Cleared at scope exit (including throws).
       */
      [Symbol.dispose]: function dispose(): void {
        state.draining = false;
        if (state.queue
          .length
          === 0)
          notifyFlushed();
      },
    };
    /* oxlint-disable eslint/no-await-in-loop -- sequential drain: each PUT must ack before the next can issue, so the server can return ack-up-to-N */
    while ((state.queue
      .length
      > 0) && (!state.destroyed)) {
      /**
       * Destructured head so the next iteration sees the new front element.
       */
      const [head,] = state.queue;
      if (head === undefined)
        break;
      /**
       * Server ack from the PUT; `PUT_FAILED` signals retries exhausted and the loop pauses.
       */
      const ack = await tryPutWithBackoff(head,);
      if (ack === PUT_FAILED)
        break;
      await dropAcked({
        idb: state.idb,
        queue: state.queue,
        draftId: head.draftId,
        ack,
      },);
    }
    /* oxlint-enable eslint/no-await-in-loop */
  }

  /**
   * Idempotent drain trigger. Multiple concurrent calls collapse.
   */
  function kick(): void {
    void drain();
  }

  /**
   * Online/visibility listener: kicks the drain when conditions improve.
   */
  function onOnlineOrVisible(): void {
    if (state.destroyed)
      return;
    if (((typeof document) !== 'undefined') && (document.visibilityState
      !== 'visible'))
      return;
    kick();
  }

  if ((typeof globalThis.addEventListener) === 'function') {
    globalThis.addEventListener(
      'online',
      onOnlineOrVisible,
    );
  }
  if (((typeof document) !== 'undefined')
    && ((typeof document.addEventListener) === 'function'))
  {
    document.addEventListener(
      'visibilitychange',
      onOnlineOrVisible,
    );
  }

  // Drain anything we rehydrated from IDB on construction.
  if (queue.length
    > 0)
    kick();

  return {
    async enqueue(upload,) {
      if (state.destroyed)
        throw new Error('outbox destroyed',);
      if (state.idb
        !== NO_IDB) {
        await persistOne({
          idb: state.idb,
          upload,
        },);
      }
      state.queue
        .push(upload,);
      kick();
    },
    flushed() {
      if ((state.queue
        .length
        === 0) && (!state.draining))
        return Promise.resolve();
      // The Promise constructor is the only way to capture a resolver
      // we hand off to the drain loop for later notification.
      // oxlint-disable-next-line eslint-plugin-promise/avoid-new -- one-shot resolver
      return new Promise<void>(function executor(resolve,) {
        state.waiters
          .push(resolve,);
      },);
    },
    pendingCount() {
      return state.queue
        .length;
    },
    destroy() {
      state.destroyed = true;
      if ((typeof globalThis.removeEventListener) === 'function') {
        globalThis.removeEventListener(
          'online',
          onOnlineOrVisible,
        );
      }
      if (((typeof document) !== 'undefined')
        && ((typeof document.removeEventListener) === 'function'))
      {
        document.removeEventListener(
          'visibilitychange',
          onOnlineOrVisible,
        );
      }
      if (state.idb
        !== NO_IDB)
        state.idb
          .close();
      notifyFlushed();
    },
  };
}

/**
 * Sends one upload with bounded exponential backoff. Returns the
 * server-reported ack on success or `PUT_FAILED` after the retry budget is
 * exhausted; the caller pauses the queue until the next online /
 * visibility event.
 *
 * @param upload - chunk to PUT
 *
 * @returns highest contiguous seq returned by the server, or `PUT_FAILED`
 *
 * @example
 * ```ts
 * const ack = await tryPutWithBackoff(upload);
 * ```
 */
async function tryPutWithBackoff(upload: ChunkUpload,): Promise<number | typeof PUT_FAILED> {
  /**
   * Stable URL captured once outside the retry loop so each attempt targets the same slot.
   */
  const url = `/api/drafts/${encodeURIComponent(upload.draftId,)}/chunks/${
    String(upload.seq,)
  }`;
  /**
   * JSON body serialised once outside the retry loop to avoid repeated stringify cost.
   */
  const body = JSON.stringify({
    md: upload.md,
    html: upload.html,
    char_count: upload.charCount,
  },);
  /* oxlint-disable eslint/no-await-in-loop -- retry attempts are inherently sequential: each retry must wait for the prior attempt's failure plus its backoff delay before issuing */
  for (let attempt = 0; attempt < PUT_MAX_ATTEMPTS; attempt += 1) {
    try {
      /**
       * Awaited so both the status check and the JSON read can reuse the same response.
       */
      const response = await fetch(
        url,
        {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json', },
          body,
        },
      );
      if (!response.ok)
        throw new Error(`PUT returned ${String(response.status,)}`,);
      /**
       * Server ack envelope; falls back to `upload.seq` when `ack` is missing or non-numeric.
       */
      const parsed = await readJson<{ ack?: unknown; }>(response,);
      return (typeof parsed.ack) === 'number' ? parsed.ack : upload.seq;
    }
    catch {
      /**
       * Exponential backoff per retry; doubles on each attempt (250 ms, 500 ms, 1 s).
       */
      const delay = PUT_BACKOFF_BASE_MS * (1 << attempt);
      await wait(delay,);
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return PUT_FAILED;
}

/**
 * Drops every queue entry whose `(draftId, seq)` is implied by the
 * server's ack (`seq <= ack`) and removes the same entries from IDB.
 *
 * @param input - queue, IDB handle, draft id, server ack
 *
 * @example
 * ```ts
 * await dropAcked({ idb, queue, draftId, ack });
 * ```
 */
async function dropAcked(
  input: {
    idb: IDBDatabase | typeof NO_IDB;
    queue: ChunkUpload[];
    draftId: string;
    ack: number;
  },
): Promise<void> {
  /**
   * Pre-sweep length captured so the reverse walk terminates at a known head.
   */
  const before = input.queue
    .length;
  for (let loopIndex = before - 1; loopIndex >= 0; loopIndex -= 1) {
    /**
     * Currently-visited entry; the guard below treats sparse holes as already-dropped.
     */
    const entry = input.queue[loopIndex];
    if (entry === undefined)
      continue;
    if ((entry.draftId
      === input
      .draftId) && (entry.seq
        <= input
        .ack)) {
      input.queue
        .splice(
        loopIndex,
        1,
      );
    }
  }
  if (input.idb
    === NO_IDB)
    return;
  await deleteAcked({
    idb: input.idb,
    draftId: input.draftId,
    ack: input.ack,
  },);
}
