import {
  MS_PER_SECOND,
  SECONDS_PER_MINUTE,
} from '@monochromatic-dev/const-core/ts';

//region Buffer shape

/**
 How long capture sets are retained in the buffer before pruning.

 Exported so a test states the cutoff it asserts against instead of repeating
 the window.
 */
export const RETENTION_MS: number = 10 * SECONDS_PER_MINUTE
  * MS_PER_SECOND;

/**
 Maximum number of capture sets kept in the rolling buffer.

 Exported so a test states the capacity it asserts against instead of
 repeating the number.
 */
export const MAX_ENTRIES: number = 3;

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
export type CaptureBuffer = readonly CaptureSet[];

//endregion Buffer shape

//region Live buffer

/**
 Module-singleton mutable state for the rolling capture buffer; wrapped so it satisfies no-module-root-let.
 */
const state: { buffer: CaptureBuffer; } = { buffer: [], };

/**
 Appends a capture set to the rolling buffer, evicting the oldest entry when
 the buffer is at capacity.
 
 Retention is {@link getRecent}'s job: it is the one place a reader asks what
 is still fresh, so this function needs no clock reading and an expired set
 waits in the buffer at most until the next read (bounded by
 {@link MAX_ENTRIES} sets).
 
 @param set - capture set to store
 
 @example
 ```ts
 store({ timestamp: Date.now(), screenshot, webcam });
 ```
 */
export function store(set: CaptureSet,): void {
  state.buffer = appendedCaptureBuffer({
    buffer: state.buffer,
    set,
  },);
}

/**
 Drops the expired sets, then returns a shallow copy of what remains.
 
 A set captured exactly {@link RETENTION_MS} ago is still returned; one
 captured a millisecond earlier is not.
 
 @returns array of recent capture sets, oldest first
 
 @example
 ```ts
 const sets = getRecent();
 log.debug(`${sets.length} capture set(s) available`);
 ```
 */
export function getRecent(): CaptureSet[] {
  state.buffer = retainedCaptureBuffer({
    buffer: state.buffer,
    now: Date.now(),
  },);
  return [...state.buffer,];
}

//endregion Live buffer

//region Buffer arithmetic

/**
 Appends a capture set to a buffer, keeping only the newest {@link MAX_ENTRIES}
 entries.
 
 @param buffer - buffer to append to
 
 @param set - capture set to append
 
 @returns the appended buffer, oldest first
 
 @example
 ```ts
 appendedCaptureBuffer({ buffer: [], set: captureSet }); // [captureSet]
 ```
 */
export function appendedCaptureBuffer(
  {
    buffer,
    set,
  }: {
    readonly buffer: CaptureBuffer;
    readonly set: CaptureSet;
  },
): CaptureBuffer {
  return [
    ...buffer,
    set,
  ]
    .slice(-MAX_ENTRIES,);
}

/**
 Keeps only the capture sets newer than the retention cutoff, which is
 {@link RETENTION_MS} before the given clock reading. A set captured exactly
 at the cutoff is kept, so the window is inclusive at its oldest end.
 
 @param buffer - buffer to trim
 
 @param now - Unix epoch milliseconds to measure the cutoff from
 
 @returns the retained sets, oldest first
 
 @example
 ```ts
 retainedCaptureBuffer({ buffer, now: Date.now() }).length; // sets captured in the last 10 min
 ```
 */
export function retainedCaptureBuffer(
  {
    buffer,
    now,
  }: {
    readonly buffer: CaptureBuffer;
    readonly now: number;
  },
): CaptureBuffer {
  /**
   Oldest timestamp to keep; sets older than this are dropped.
   */
  const cutoff = now
    - RETENTION_MS;
  return buffer
    .filter(function checkRetention(set,) {
      return isAfterCutoff({
        cutoff,
        set,
      },);
    },);
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

//endregion Buffer arithmetic
