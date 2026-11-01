import { reportLoggerInternalError, } from '../error-format.ts';
import { detectSessionStorageQuotaChars, } from './session-storage-quota.ts';
import { isQuotaExceededError, } from './web-storage-quota-error.ts';

/**
 Prefix for sessionStorage keys to namespace log entries.
 */
const STORAGE_KEY_PREFIX = 'monochromatic.log';

/**
 Builds the namespaced sessionStorage key for a log entry at `index`.
 
 @param index - Zero-based slot number of an entry.
 
 @returns Prefixed key such as `monochromatic.log.3`.
 
 @example
 ```ts
 storageKey(3); // 'monochromatic.log.3'
 ```
 */
function storageKey(index: number,): string {
  return `${STORAGE_KEY_PREFIX}.${index}`;
}

/**
 Builds the persistence engine behind the sessionStorage sink: each `persist`
 lands one already-serialized batch under a counter-incremented key, with
 proactive and reactive quota eviction. The counter lives in this instance's
 closure (no module-global state), so independent sinks and tests never share
 keys or need a reset hook.
 
 The engine caps its own footprint at half the runtime's sessionStorage
 quota, proactively dropping its oldest entries, and reactively drops them
 again if the real store overflows; see {@link createSessionStorageStore.persist}.
 
 @returns Engine exposing `persist` for one batch value per call.
 
 @example
 ```ts
 const store = createSessionStorageStore();
 store.persist('{"level":"info","message":"hi","timestamp":0}');
 ```
 */
export function createSessionStorageStore(): { readonly persist: (batch: string,) => void; } {
  /**
   Instance-local write cursor, eviction watermark, and footprint tally.
   `lineCounter` is the next slot to write and advances only when a `setItem`
   actually lands, so this engine's present entries occupy the contiguous
   range `[oldestIndex, lineCounter)`. `oldestIndex` is the lowest slot the
   engine still owns; eviction removes that entry and climbs `oldestIndex`
   toward `lineCounter`, so `oldestIndex < lineCounter` doubles as the "a
   prior write succeeded and an owned entry remains" guard that keeps
   eviction from ever touching another origin consumer's keys. `usedChars`
   tracks the code units this engine currently occupies so the half-quota cap
   needs no re-summing. `reportedFailure` gates the give-up diagnostic to
   once per failure episode: a persistently full store (another writer owning
   the space) would otherwise emit one `console.warn` per batch, so the flag
   stays set until a write next lands, which re-arms a single report for the
   next episode.
   */
  const state: {
    lineCounter: number;
    oldestIndex: number;
    usedChars: number;
    reportedFailure: boolean;
  } = {
    lineCounter: 0,
    oldestIndex: 0,
    usedChars: 0,
    reportedFailure: false,
  };

  /**
   Half the detected runtime sessionStorage quota, in UTF-16 code units, or
   `Number.POSITIVE_INFINITY` on an unrecognized runtime. The engine keeps
   its own footprint at or below this so the logger never claims more than
   half the store, leaving the rest for the host application. An infinite cap
   disables the proactive check, leaving only reactive quota-error eviction.
   */
  const capChars = detectSessionStorageQuotaChars() / 2;

  /**
   Removes this engine's oldest still-present entry, advancing the watermark
   and subtracting the reclaimed entry's code units from the running
   footprint. Reading the value back before removal keeps `usedChars` honest
   even if the entry drifted from what was written.
   */
  function evictOldest(): void {
    /**
     Key of the oldest owned entry, removed to reclaim its slot and its space.
     */
    const key = storageKey(state.oldestIndex,);
    /**
     Value being evicted, read back so its length can leave the footprint tally.
     */
    const evicted = globalThis.sessionStorage
      .getItem(key,);
    globalThis.sessionStorage
      .removeItem(key,);
    state.oldestIndex++;
    if (evicted !== null)
      state.usedChars = Math.max(
        0,
        state.usedChars - evicted.length,
      );
  }

  /**
   Persists one serialized batch to sessionStorage under a
   counter-incremented key.
   
   First it proactively drops its own oldest entries so its footprint stays
   at or below half the runtime's sessionStorage quota, leaving the rest for
   the host application. It then writes, and on a quota overflow (the store
   being fuller than the cap accounts for), and only while an owned entry
   remains (so the reclaimed keys are its own, never another origin
   consumer's), it drops its oldest still-present entry and retries until the
   batch fits or nothing of its own remains to drop. A batch larger than the
   whole quota therefore evicts every owned entry, then reports and gives up
   rather than looping forever. A non-quota failure is reported without any
   eviction. The sink only persists after verification, so no availability
   guard is needed here.
   
   @param batch - Serialized JSONL batch to persist.
   */
  function persist(batch: string,): void {
    /**
     Code units this batch adds; the key's length is left out as a negligible near-constant.
     */
    const batchChars = batch.length;

    // Proactively reclaim space so the engine's own footprint stays under the
    // half-quota cap, dropping oldest-first while an owned entry remains. An
    // infinite cap (unrecognized runtime) makes the guard always false.
    while ((state.oldestIndex < state.lineCounter) && ((state.usedChars + batchChars) > capChars)) {
      evictOldest();
    }

    /**
     Write-attempt bound: one try for each entry still available to evict,
     followed by one final try after every owned entry has been removed.
     */
    const maxWriteAttempts = (state.lineCounter - state.oldestIndex) + 1;
    for (let writeAttempt = 0; writeAttempt < maxWriteAttempts; writeAttempt++) {
      try {
        globalThis.sessionStorage
          .setItem(
          storageKey(state.lineCounter,),
          batch,
          );
        state.lineCounter++;
        state.usedChars += batchChars;
        // A landed write re-arms a single give-up report for the next episode.
        state.reportedFailure = false;
        return;
      }
      catch (error: unknown) {
        if (isQuotaExceededError(error,) && (state.oldestIndex < state.lineCounter)) {
          evictOldest();
          continue;
        }
        // Report once per failure episode, not once per unwritable batch, so a
        // persistently full store does not flood the console every flush.
        if (!state.reportedFailure) {
          reportLoggerInternalError({
            context: 'sessionStorage sink record write failed (repeats suppressed until a write next succeeds)',
            error,
          },);
          state.reportedFailure = true;
        }
        return;
      }
    }
  }

  return { persist, };
}
