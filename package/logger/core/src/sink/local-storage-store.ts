import { reportLoggerInternalError, } from '../error-format.ts';
import {
  buildLogKey,
  compareLogKeys,
  parseLogKey,
  type ParsedLogKey,
} from './local-storage-key.ts';
import { detectLocalStorageQuotaChars, } from './local-storage-quota.ts';
import { isQuotaExceededError, } from './web-storage-quota-error.ts';

/**
 Radix for the run nonce so `Number.prototype.toString` yields compact
 alphanumerics.
 */
const NONCE_RADIX = 36;

/**
 Length of the run nonce; four base-36 characters make a same-millisecond
 collision between two tabs vanishingly unlikely while keeping keys short.
 */
const NONCE_LENGTH = 4;

/**
 One adopted prior-run entry: its parsed identity plus the value length it
 occupies, captured once at adoption so eviction needs no re-read.
 */
type PriorEntry = ParsedLogKey & { readonly chars: number; };

/**
 Builds the persistence engine behind the localStorage sink: each `persist`
 lands one already-serialized batch under a run-scoped counter-incremented
 key, with proactive and reactive quota eviction. Run identity and counters
 live in this instance's closure (no module-global state), so independent
 sinks and tests never share keys or need a reset hook.
 
 Unlike sessionStorage, localStorage is shared by every tab of the origin and
 survives restarts, so this engine differs from the sessionStorage engine in
 two ways. Keys carry a run identity (see `local-storage-key.ts`), so
 concurrent tabs never collide on a counter. And on its first persist the
 engine adopts every strictly-parsed entry left by other runs into its
 footprint tally, evicting those oldest-first before its own entries;
 without that, leftovers from dead sessions would fill the store until no
 run could ever write again. Adoption is deferred to first persist rather
 than construction so building the default sink set never touches
 `globalThis.localStorage` on runtimes where the sink never verifies (plain
 Node warns on mere access). Keys that fail the strict parse, including the
 host application's, are never counted and never evicted.
 
 The engine caps its own footprint (adopted entries included) at half the
 runtime's localStorage quota, proactively dropping oldest-first, and
 reactively drops again if the real store still overflows; see
 {@link createLocalStorageStore.persist}.
 
 @returns Engine exposing `persist` for one batch value per call.
 
 @example
 ```ts
 const store = createLocalStorageStore();
 store.persist('{"level":"info","message":"hi","timestamp":0}');
 ```
 */
export function createLocalStorageStore(): { readonly persist: (batch: string,) => void; } {
  /**
   Identity of this run, embedded in every key this engine writes: the stamp
   orders runs for cross-run eviction and the nonce keeps two tabs started
   in the same millisecond apart.
   */
  const runIdentity = {
    stamp: Date.now(),
    nonce: Math.random()
      .toString(NONCE_RADIX,)
      .slice(
      2,
      2 + NONCE_LENGTH,
      )
      .padEnd(
      NONCE_LENGTH,
      '0',
      ),
  };

  /**
   Instance-local write cursor, eviction watermark, and footprint tally,
   mirroring the sessionStorage engine: this run's own entries occupy the
   contiguous index range `[oldestIndex, lineCounter)` and `usedChars`
   tracks the code units the engine accounts for (adopted prior-run entries
   included) so the half-quota cap needs no re-summing. `reportedFailure`
   gates the give-up diagnostic to once per failure episode, re-armed by the
   next landed write. `adoptedPrior` defers the prior-run scan to the first
   persist, which only happens after verification.
   */
  const state: {
    lineCounter: number;
    oldestIndex: number;
    usedChars: number;
    reportedFailure: boolean;
    adoptedPrior: boolean;
  } = {
    lineCounter: 0,
    oldestIndex: 0,
    usedChars: 0,
    reportedFailure: false,
    adoptedPrior: false,
  };

  /**
   Prior-run entries adopted at first persist, sorted oldest-first, with a
   cursor marking how far eviction has consumed them; entries before the
   cursor are already removed. Prior entries always evict before this run's
   own, since they predate everything this run writes.
   */
  const prior: {
    entries: readonly PriorEntry[];
    cursor: number;
  } = {
    entries: [],
    cursor: 0,
  };

  /**
   Half the detected runtime localStorage quota, in UTF-16 code units, or
   `Number.POSITIVE_INFINITY` on an unrecognized runtime. The engine keeps
   its accounted footprint at or below this so the logger never claims more
   than half the store, leaving the rest for the host application. An
   infinite cap disables the proactive check, leaving only reactive
   quota-error eviction.
   */
  const capChars = detectLocalStorageQuotaChars() / 2;

  /**
   Builds this run's key for a batch slot.
   
   @param index - Zero-based batch slot within this run.
   
   @returns Run-scoped namespaced key.
   */
  function ownKey(index: number,): string {
    return buildLogKey({
      stamp: runIdentity.stamp,
      nonce: runIdentity.nonce,
      index,
    },);
  }

  /**
   Scans localStorage once for strictly-parsed entries left by other runs,
   sorts them oldest-first for eviction, and adds their lengths to the
   footprint tally. Entries another tab writes after this scan are invisible
   to the tally; the reactive quota loop covers that staleness.
   */
  function adoptPriorEntries(): void {
    /**
     Entry count at scan time; enumeration is by index because `Storage`
     exposes no iterator.
     */
    const total = globalThis.localStorage
      .length;
    /**
     Strictly-parsed foreign-run entries found by the scan, unsorted.
     */
    const found: PriorEntry[] = [];
    for (let slot = 0; slot < total; slot++) {
      /**
       Key at this enumeration slot; `null` past the end under concurrent removal.
       */
      const key = globalThis.localStorage
        .key(slot,);
      if (key === null)
        continue;
      /**
       Parsed run identity, absent for any key the engine must not touch.
       */
      const { parsed, } = parseLogKey(key,);
      if (parsed === undefined)
        continue;
      if ((parsed.stamp === runIdentity.stamp) && (parsed.nonce === runIdentity.nonce))
        continue;
      /**
       Stored batch, read so its length enters the footprint tally.
       */
      const value = globalThis.localStorage
        .getItem(key,);
      if (value === null)
        continue;
      found.push({
        ...parsed,
        chars: value.length,
      },);
    }
    prior.entries = found.toSorted(function byOldestFirst(
      first,
      second,
    ) {
      return compareLogKeys({
        first,
        second,
      },);
    },);
    state.usedChars += found.reduce(
      function sumChars(
        sum,
        entry,
      ) {
        return sum + entry.chars;
      },
      0,
    );
  }

  /**
   Reports whether anything remains this engine may evict: an adopted
   prior-run entry past the cursor, or one of this run's own entries.
   
   @returns Whether an eviction call would reclaim something.
   */
  function hasEvictable(): boolean {
    /**
     Count of adopted prior-run entries; those before the cursor are gone.
     */
    const priorCount = prior.entries
      .length;
    return (prior.cursor < priorCount)
      || (state.oldestIndex < state.lineCounter);
  }

  /**
   Removes the oldest not-yet-evicted adopted prior-run entry, if one
   remains. Its length leaves the tally from the adoption snapshot: prior
   keys are never rewritten (counters only advance), so a re-read could only
   observe the same value or a concurrent removal, and in both cases the
   snapshot is what the tally counted.
   
   @returns Whether a prior-run entry was evicted.
   */
  function evictOldestPrior(): boolean {
    /**
     Oldest remaining adopted entry, or `undefined` when all are consumed.
     */
    const entry = prior.entries[prior.cursor];
    if (entry === undefined)
      return false;
    prior.cursor++;
    globalThis.localStorage
      .removeItem(entry.key,);
    state.usedChars = Math.max(
      0,
      state.usedChars - entry.chars,
    );
    return true;
  }

  /**
   Removes this run's oldest still-present entry, advancing the watermark
   and subtracting the reclaimed entry's code units from the running
   footprint. Reading the value back before removal keeps `usedChars` honest
   even if the entry drifted from what was written.
   */
  function evictOldestOwn(): void {
    /**
     Key of the oldest owned entry, removed to reclaim its slot and its space.
     */
    const key = ownKey(state.oldestIndex,);
    /**
     Value being evicted, read back so its length can leave the footprint tally.
     */
    const evicted = globalThis.localStorage
      .getItem(key,);
    globalThis.localStorage
      .removeItem(key,);
    state.oldestIndex++;
    if (evicted !== null)
      state.usedChars = Math.max(
        0,
        state.usedChars - evicted.length,
      );
  }

  /**
   Evicts the single oldest thing the engine still owns: adopted prior-run
   entries first (they predate everything this run wrote), then this run's
   own oldest. Callers guard with {@link hasEvictable}.
   */
  function evictOldest(): void {
    if (evictOldestPrior())
      return;
    if (state.oldestIndex < state.lineCounter)
      evictOldestOwn();
  }

  /**
   Persists one serialized batch to localStorage under this run's next
   counter-incremented key.
   
   The first call adopts prior-run entries into the footprint tally. Each
   call then proactively evicts oldest-first (prior runs before this run's
   own) until the accounted footprint fits under half the runtime's
   localStorage quota, writes, and on a quota overflow (the store being
   fuller than the tally accounts for, such as another live tab writing
   concurrently) evicts and retries until the batch fits or nothing owned
   remains to drop. A batch larger than the whole quota therefore evicts
   everything owned, then reports and gives up rather than looping forever.
   A non-quota failure is reported without any eviction. The sink only
   persists after verification, so no availability guard is needed here.
   
   @param batch - Serialized JSONL batch to persist.
   */
  function persist(batch: string,): void {
    if (!state.adoptedPrior) {
      state.adoptedPrior = true;
      adoptPriorEntries();
    }

    /**
     Code units this batch adds; the key's length is left out as a negligible near-constant.
     */
    const batchChars = batch.length;

    // Proactively reclaim space so the accounted footprint stays under the
    // half-quota cap, dropping oldest-first while anything owned remains. An
    // infinite cap (unrecognized runtime) makes the guard always false.
    while (hasEvictable() && ((state.usedChars + batchChars) > capChars)) {
      evictOldest();
    }

    /**
     Write-attempt bound: one try for each entry still available to evict,
     followed by one final try after every owned entry has been removed.
     */
    const maxWriteAttempts = (prior
      .entries
      .length
      - prior.cursor)
      + (state.lineCounter - state.oldestIndex)
      + 1;
    for (let writeAttempt = 0; writeAttempt < maxWriteAttempts; writeAttempt++) {
      try {
        globalThis.localStorage
          .setItem(
          ownKey(state.lineCounter,),
          batch,
          );
        state.lineCounter++;
        state.usedChars += batchChars;
        // A landed write re-arms a single give-up report for the next episode.
        state.reportedFailure = false;
        return;
      }
      catch (error: unknown) {
        if (isQuotaExceededError(error,) && hasEvictable()) {
          evictOldest();
          continue;
        }
        // Report once per failure episode, not once per unwritable batch, so a
        // persistently full store does not flood the console every flush.
        if (!state.reportedFailure) {
          reportLoggerInternalError({
            context: 'localStorage sink record write failed (repeats suppressed until a write next succeeds)',
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
