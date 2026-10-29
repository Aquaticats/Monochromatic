import {
  MS_PER_SECOND,
  SECONDS_PER_MINUTE,
} from '@monochromatic-dev/const-core/ts';

/**
 How long capture sets are retained in the buffer before pruning.
 */
const RETENTION_MS = 10 * SECONDS_PER_MINUTE
  * MS_PER_SECOND;

/**
 Maximum number of capture sets kept in the rolling buffer.
 */
const MAX_ENTRIES = 3;

/**
 Screenshot and webcam frame pair captured at a single point in time.
 
 @example
 ```ts
 const set: CaptureSet = { timestamp: Date.now(), screenshot: buf1, webcam: buf2 };
 ```
 */
export type CaptureSet = {
  /**
   Unix epoch milliseconds when the capture was taken.
   */
  readonly timestamp: number;
  /**
   JPEG-encoded desktop screenshot.
   */
  readonly screenshot: Buffer;
  /**
   JPEG-encoded webcam frame.
   */
  readonly webcam: Buffer;
};

/**
 Rolling buffer of recent capture sets, oldest first; runtime-trimmed to at
 most {@link MAX_ENTRIES} entries by {@link store}.
 */
type CaptureBuffer = readonly CaptureSet[];

/**
 Module-singleton mutable state for the rolling capture buffer; wrapped so it satisfies no-module-root-let.
 */
const state: { buffer: CaptureBuffer; } = { buffer: [], };

/**
 Appends a capture set to the rolling buffer, evicting the oldest entry
 if the buffer is at capacity, then runs {@link prune} to drop expired sets.
 
 @param set - capture set to store
 
 @example
 ```ts
 store({ timestamp: Date.now(), screenshot, webcam });
 ```
 */
export function store(set: CaptureSet,): void {
  /**
   Updated buffer with the new set appended; trimmed to the most recent {@link MAX_ENTRIES} entries.
   */
  const next = [
    ...state.buffer,
    set,
  ]
    .slice(-MAX_ENTRIES,);
  state.buffer = next;
  prune();
}

/**
 Runs {@link prune} then returns a shallow copy of all non-expired capture
 sets in the buffer.
 
 @returns array of recent capture sets, oldest first
 
 @example
 ```ts
 const sets = getRecent();
 log.debug(`${sets.length} capture set(s) available`);
 ```
 */
export function getRecent(): CaptureSet[] {
  prune();
  return [...state.buffer,];
}

/**
 Checks whether a capture set is newer than the retention cutoff.
 
 @param cutoff - minimum timestamp to retain
 
 @param set - capture set to check
 
 @returns true when the set should be kept
 
 @example
 ```ts
 isAfterCutoff({ cutoff: Date.now() - 600_000, set: captureSet }); // true if captured in last 10 min
 ```
 */
function isAfterCutoff(
  {
    cutoff,
    set,
  }: {
    readonly cutoff: number;
    readonly set: CaptureSet;
  },
): boolean {
  return set.timestamp
    >= cutoff;
}

/**
 Removes capture sets older than {@link RETENTION_MS} from the buffer.
 */
function prune(): void {
  /**
   Oldest timestamp to keep; sets older than this are filtered out.
   */
  const cutoff = Date.now()
    - RETENTION_MS;
  state.buffer = state.buffer
    .filter(function checkRetention(set,) {
      return isAfterCutoff({
        cutoff,
        set,
      },);
    },);
}
