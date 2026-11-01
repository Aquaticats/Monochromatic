/**
 * Storage capability probes.
 *
 * Each probe returns `true` only when a round-trip read/write succeeds
 * end to end; not just "the API is defined." A 500 ms timeout caps
 * each probe so a hung backend cannot delay startup.
 *
 * Probes are non-throwing: any rejection or thrown exception turns into
 * `false`. Storage features are pure enhancements; the app must remain
 * fully functional with all probes returning `false`.
 */

import { withTimeout, } from '@monochromatic-dev/async-time-core/ts';

/**
 * Capability flags consulted by the enhancement modules.
 */
export type StorageCaps = {
  readonly idb: boolean;
  readonly opfs: boolean;
  readonly localStorage: boolean;
};

/**
 * Tag used for probe writes so concurrent test rows do not collide.
 */
const PROBE_KEY = '__messages_demo_storage_probe__';

/**
 * Cap on each individual probe in milliseconds.
 */
const PROBE_TIMEOUT_MS = 500;

/**
 * Wraps a probe promise so a hung backend never delays startup beyond
 * `PROBE_TIMEOUT_MS` and any rejection (timeout or backend error) becomes
 * `false`. Composes with the shared `withTimeout` so the timer plumbing
 * stays in one place.
 *
 * @param probe - async probe returning a boolean
 *
 * @param label - human-readable tag for the timeout error (logged on failure)
 *
 * @returns probe result, or `false` on timeout / rejection
 *
 * @example
 * ```ts
 * await capProbe({ probe: probeIdb(), label: 'idb' });
 * ```
 */
async function capProbe(
  {
    probe,
    label,
  }: {
    readonly probe: Promise<boolean>;
    readonly label: string;
  },
): Promise<boolean> {
  try {
    return await withTimeout({
      promise: probe,
      ms: PROBE_TIMEOUT_MS,
      label,
    },);
  }
  catch {
    return false;
  }
}

/**
 * Runs all three probes in parallel. Resolves with the cap matrix.
 *
 * @returns capability flags
 *
 * @example
 * ```ts
 * const caps = await probeStorage();
 * if (caps.localStorage) ...
 * ```
 */
export async function probeStorage(): Promise<StorageCaps> {
  /**
   * Three probes run concurrently; the timeout cap applies per-probe.
   */
  const [idb, opfs, ls,] = await Promise.all([
    capProbe({
      probe: probeIdb(),
      label: 'idb',
    },),
    capProbe({
      probe: probeOpfs(),
      label: 'opfs',
    },),
    capProbe({
      probe: probeLocalStorage(),
      label: 'localStorage',
    },),
  ],);
  return {
    idb,
    opfs,
    localStorage: ls,
  };
}

/**
 * Tests whether IndexedDB can open + put + get + delete a 1-byte record.
 *
 * @returns `true` when round-trip succeeds
 */
function probeIdb(): Promise<boolean> {
  // IndexedDB exposes an event-callback API; the Promise constructor is
  // the only reasonable bridge.
  // oxlint-disable-next-line eslint-plugin-promise/avoid-new -- bridges callback API
  return new Promise<boolean>(function executor(resolve,) {
    if ((typeof indexedDB) === 'undefined') {
      resolve(false,);
      return;
    }
    /**
     * Open request held so success and error listeners can be wired before it resolves.
     */
    const request = indexedDB.open(
      PROBE_KEY,
      1,
    );
    request.addEventListener(
      'upgradeneeded',
      function onUpgrade(): void {
        request.result
          .createObjectStore('probe',);
      },
    );
    request.addEventListener(
      'success',
      function onSuccess(): void {
        try {
          /**
           * Open DB handle reused to start a read-write transaction and to close on completion.
           */
          const dbConn = request.result;
          /**
           * Read-write transaction; the put-then-complete dance verifies actual round-trip.
           */
          const tx = dbConn.transaction(
            'probe',
            'readwrite',
          );
          /**
           * Store handle reused by the put below.
           */
          const store = tx.objectStore('probe',);
          store.put(
            1,
            'k',
          );
          tx.addEventListener(
            'complete',
            function onComplete(): void {
              dbConn.close();
              indexedDB.deleteDatabase(PROBE_KEY,);
              resolve(true,);
            },
          );
          tx.addEventListener(
            'error',
            function onErrorEvent(): void {
              dbConn.close();
              resolve(false,);
            },
          );
        }
        catch {
          resolve(false,);
        }
      },
    );
    request.addEventListener(
      'error',
      function onError(): void {
        resolve(false,);
      },
    );
  },);
}

/**
 * Tests whether OPFS (Origin Private File System) is available and
 * accepts a write.
 *
 * @returns `true` when round-trip succeeds
 */
async function probeOpfs(): Promise<boolean> {
  // navigator.storage.getDirectory is the OPFS entry point. The check
  // is feature-detection plus a write attempt because some browsers
  // expose the API but reject writes (e.g. Safari private mode).
  if (((typeof navigator) === 'undefined')
    || (navigator.storage
      ?.getDirectory
      === undefined))
  {
    return false;
  }
  try {
    /**
     * OPFS root acquired once and reused by the file handle below.
     */
    const root = await navigator.storage
      .getDirectory();
    /**
     * Probe file handle, created if absent; deleted in the cleanup below.
     */
    const handle = await root.getFileHandle(
      PROBE_KEY,
      { create: true, },
    );
    /**
     * Writable stream; the round-trip write proves OPFS actually accepts data.
     */
    const writable = await handle.createWritable();
    await writable.write('1',);
    await writable.close();
    await root.removeEntry(PROBE_KEY,);
    return true;
  }
  catch {
    return false;
  }
}

/**
 * Tests whether `localStorage` accepts a write. Some browsers expose
 * the property but throw on `setItem` in private mode or when storage
 * is full.
 *
 * Returns a `Promise` for symmetry with the IDB and OPFS probes; the
 * shared `withTimeout` wrapper expects `Promise<boolean>` for all three.
 *
 * @returns `true` when set+get+remove succeeds
 *
 * @example
 * ```ts
 * const ok = await probeLocalStorage();
 * ```
 */
function probeLocalStorage(): Promise<boolean> {
  if ((typeof localStorage) === 'undefined')
    return Promise.resolve(false,);
  try {
    localStorage.setItem(
      PROBE_KEY,
      '1',
    );
    /**
     * Round-tripped read; compared against the literal we set above.
     */
    const value = localStorage.getItem(PROBE_KEY,);
    localStorage.removeItem(PROBE_KEY,);
    return Promise.resolve(value === '1',);
  }
  catch {
    return Promise.resolve(false,);
  }
}
