import type { Level, } from '../types.ts';

/**
 Buffered code units that force a synchronous flush from inside `add`
 itself. 32 KiB sits in the measured flat bottom of the batch-size curve on
 Chromium 149 and Node 26 (0.15 µs to 1.7 µs per record versus 5 µs to
 15 µs unbatched) while staying clear of the measured U-turn where flushes
 past ~100 KiB cost more per record than not batching; see
 `doc/troubleshooting/web-storage-sink-main-thread-cost.md`. Because this
 flush runs synchronously inside `add`, a wedged main thread that keeps
 logging can never hold more than one cap's worth of unpersisted records.
 */
const FLUSH_BUFFER_CAP_CHARS = 32_768;

/**
 Quiet-period deadline before a buffered record is flushed by timer, so
 low-volume sessions still reach the backend without waiting for the byte
 cap. Each deadline flush costs one backend call, so this cadence is
 negligible while keeping the loss window for idle periods under a quarter
 second.
 */
const FLUSH_DEADLINE_MS = 250;

/**
 Severities that flush the buffer synchronously from inside `add`, so every
 record up to and including a warning or worse reaches the backend before
 control returns to the caller. Failure forensics is why persistent sinks
 exist; these records are rare, so paying the per-batch cost immediately
 for them does not dent the amortization of the bulk `debug`/`trace`/`info`
 volume.
 */
const FLUSH_IMMEDIATELY_BY_LEVEL: Record<Level, boolean> = {
  debug: false,
  error: true,
  fatal: true,
  info: false,
  trace: false,
  warn: true,
};

/**
 Timer handle exposing Node's keep-alive release. Browsers return a bare
 number from `setTimeout` and need no release; Node returns an object whose
 `unref` lets the process exit while the timer is pending.
 */
type UnrefableTimer = { readonly unref: () => void; };

/**
 Narrows a `setTimeout` return value to a handle exposing `unref`, so a
 pending deadline flush never pins a server process open past its work.
 
 @param timer - Return value of `globalThis.setTimeout`.
 
 @returns Whether `timer` exposes a callable `unref`.
 */
function isUnrefableTimer(timer: unknown,): timer is UnrefableTimer {
  if (((typeof timer) !== 'object') || (timer === null))
    return false;
  if (!('unref' in timer))
    return false;
  return (typeof timer.unref) === 'function';
}

/**
 Builds the buffering stage shared by batch-persisting sinks: serialized
 records accumulate and leave as one newline-joined JSONL batch through
 `onFlush`. One uniform policy runs on every runtime; no per-runtime mode
 exists.
 
 A batch flushes synchronously from inside `add` when it reaches
 {@link FLUSH_BUFFER_CAP_CHARS} or when the record's severity is `warn` or
 worse, by timer after {@link FLUSH_DEADLINE_MS} of quiet, on `pagehide`
 and on the document becoming hidden (where those events exist), and on
 `drain`. The byte-cap and severity flushes run on the caller's stack, so
 neither a synchronous workload nor a wedged main thread can accumulate
 more than one cap of unhanded records. When an addition would breach the
 cap, the existing entries flush first so an oversized record's downstream
 failure can only ever drop that record, never its batch-mates.
 
 @param onFlush - Backend handoff receiving each newline-joined batch;
 called synchronously from whichever trigger fires, in record order.
 
 @returns Buffer exposing `add` for records and `drain` for forced flushes.
 
 @example
 ```ts
 const buffer = createRecordBuffer({ onFlush: (batch) => store.persist(batch) });
 buffer.add({ level: 'info', serialized: JSON.stringify(record) });
 buffer.drain();
 ```
 */
export function createRecordBuffer(
  { onFlush, }: { readonly onFlush: (batch: string,) => void; },
): {
  readonly add: (entry: {
    readonly level: Level;
    readonly serialized: string;
  },) => void;
  readonly drain: () => void;
} {
  /**
   Serialized records awaiting one joined handoff; drained in add order by
   every flush trigger.
   */
  const entries: string[] = [];

  /**
   Instance-local buffer bookkeeping. `chars` mirrors the joined length of
   {@link entries} (records plus one separator between neighbors) so cap
   checks need no re-summing; `timer`, present only while armed, holds the
   quiet-period deadline flush so idle sessions still hand off.
   */
  const bufferState: {
    chars: number;
    timer?: ReturnType<typeof globalThis.setTimeout>;
  } = { chars: 0, };

  /**
   Joined length the buffer would have after appending `serialized`,
   counting the newline separator a non-empty buffer needs before it.
   
   @param serialized - Record about to be appended.
   
   @returns Prospective joined batch length in code units.
   */
  function charsWith(serialized: string,): number {
    /**
     Newline separator the join adds before this record when the buffer already holds one.
     */
    const separatorChars = (entries.length > 0) ? 1 : 0;
    /**
     Length of the buffer as currently joined, before this record.
     */
    const joinedChars = bufferState.chars + separatorChars;
    return joinedChars + serialized.length;
  }

  /**
   Hands the buffered records to `onFlush` as one newline-joined batch and
   disarms the deadline timer. Runs synchronously so byte-cap and severity
   flushes complete on the caller's stack. Safe to call with an empty
   buffer.
   */
  function drain(): void {
    if (bufferState.timer !== undefined) {
      globalThis.clearTimeout(bufferState.timer,);
      delete bufferState.timer;
    }
    if (entries.length === 0)
      return;
    /**
     Newline-joined JSONL batch; `JSON.stringify` escapes newlines inside
     records, so the separator is unambiguous for readers splitting lines.
     */
    const batch = entries.join('\n',);
    entries.length = 0;
    bufferState.chars = 0;
    onFlush(batch,);
  }

  /**
   Arms the quiet-period deadline flush if none is pending, releasing the
   runtime's keep-alive where the handle supports it so a pending flush
   never holds a process open.
   */
  function scheduleDeadlineFlush(): void {
    if (bufferState.timer !== undefined)
      return;
    /**
     Freshly armed deadline handle; kept on {@link bufferState} so a cap or severity flush can disarm it.
     */
    const timer = globalThis.setTimeout(
      drain,
      FLUSH_DEADLINE_MS,
    );
    if (isUnrefableTimer(timer,))
      timer.unref();
    bufferState.timer = timer;
  }

  /**
   Buffers one serialized record, flushing synchronously when the joined
   batch reaches the byte cap or the record's severity is `warn` or worse.
   
   @param entry - Serialized record plus the severity that decides an
   immediate flush.
   */
  function add(entry: {
    readonly level: Level;
    readonly serialized: string;
  },): void {
    // Protect batch-mates: an addition that would breach the cap flushes the
    // current entries first, isolating any oversized record in its own batch.
    if ((entries.length > 0) && (charsWith(entry.serialized,) > FLUSH_BUFFER_CAP_CHARS))
      drain();

    bufferState.chars = charsWith(entry.serialized,);
    entries.push(entry.serialized,);

    if (FLUSH_IMMEDIATELY_BY_LEVEL[entry.level] || (bufferState.chars >= FLUSH_BUFFER_CAP_CHARS))
      drain();
    else
      scheduleDeadlineFlush();
  }

  // A leaving or hidden page is the last chance to hand off; both hooks are
  // harmless no-ops on runtimes where the events never fire.
  globalThis.addEventListener?.(
    'pagehide',
    drain,
  );
  globalThis.document
    ?.addEventListener(
    'visibilitychange',
    function flushWhenHidden(): void {
        /**
         Current page visibility; the listener only fires where a document exists.
         */
      const visibility = globalThis.document
        ?.visibilityState;
      if (visibility === 'hidden')
        drain();
    },
    );

  return {
    add,
    drain,
  };
}
