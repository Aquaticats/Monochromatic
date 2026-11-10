/**
 Retention and eviction of the rolling capture buffer.

 The buffer decides what the vision model is shown: it keeps the newest
 {@link MAX_ENTRIES} capture sets and drops any set older than
 {@link RETENTION_MS}. Both boundaries are off-by-one risks the package test
 policy (`PKG`, `TCV`) requires covered: one entry too few loses context, one
 too many feeds the model a frame outside the retention window.

 The cutoff is asserted twice on purpose. {@link retainedCaptureBuffer} takes
 the clock reading, so every boundary around it (exactly at the cutoff, one
 millisecond past it, mixed buffers) is a pure comparison. The singleton cases
 then drive {@link store} and {@link getRecent} themselves, one of them with a
 faked `Date` so a stored set can expire while it sits in the buffer; that is
 the only way to show the read, and not the write, is what drops it.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  appendedCaptureBuffer,
  getRecent,
  MAX_ENTRIES,
  retainedCaptureBuffer,
  RETENTION_MS,
  store,
  type CaptureBuffer,
  type CaptureSet,
} from '../../dist/final/node/testing.mjs';

/**
 An arbitrary fixed clock reading.

 Only differences between timestamps matter here, and a fixed value keeps
 every retention case independent of the machine's clock.
 */
const NOW = 1_700_000_000_000;

/**
 One capture set whose timestamp is the only fact the cases read.
 
 @param timestamp - Unix epoch milliseconds the set was captured at
 
 @returns capture set carrying that timestamp
 */
function captureSet(timestamp: number,): CaptureSet {
  return {
    timestamp,
    screenshot: Buffer.from(`screenshot-${String(timestamp,)}`,),
    webcam: Buffer.from(`webcam-${String(timestamp,)}`,),
  };
}

/**
 The timestamps of a buffer, oldest first.
 
 @param buffer - capture sets to read
 
 @returns their timestamps in buffer order
 */
function timestampsOf(buffer: CaptureBuffer,): number[] {
  return buffer.map(function timestampOf(set,) {
    return set.timestamp;
  },);
}

/**
 A buffer of consecutive sets one millisecond apart, ending at {@link NOW}.
 
 @param length - how many sets to build
 
 @returns the capture sets, oldest first
 */
function bufferOfLength(length: number,): CaptureBuffer {
  /** Timestamp of the oldest set the buffer holds. */
  const oldest = NOW - length;

  return Array.from(
    { length, },
    function makeSet(_unused, index,) {
      return captureSet(oldest + index,);
    },
  );
}

/**
 Spare-room cases: a buffer holding one entry up to one short of the capacity,
 which gains the new set without losing anything.
 */
const underCapacityCases = Array.from(
  { length: MAX_ENTRIES - 1, },
  function makeUnderCapacityCase(_unused, index,) {
    /** Entry count this case starts from. */
    const length = index + 1;
    return it({
      name: `keeps all ${String(length,)} buffered entries while under capacity`,
      fn: async () => {
        /** Buffer holding fewer entries than {@link MAX_ENTRIES}. */
        const buffer = bufferOfLength(length,);
        /** The buffer after one more set arrives. */
        const appended = appendedCaptureBuffer({
          buffer,
          set: captureSet(NOW,),
        },);
        /** Timestamps of the result. */
        const timestamps = timestampsOf(appended,);

        expect(appended,).toHaveLength(length + 1,);
        expect(timestamps,)
          .toEqual([
            ...timestampsOf(buffer,),
            NOW,
          ],);
      },
    },);
  },
);

await describe({
  name: '',
  children: [
    describe({
      name: appendedCaptureBuffer.name,
      children: [
        it({
          name: 'starts an empty buffer from one set',
          fn: async () => {
            /** The only set the empty buffer receives. */
            const only = captureSet(NOW,);

            expect(appendedCaptureBuffer({
              buffer: [],
              set: only,
            },),).toEqual([only,],);
          },
        },),

        it({
          name: 'appends the new set after the ones already buffered',
          fn: async () => {
            /** The buffer after the append. */
            const appended = appendedCaptureBuffer({
              buffer: [captureSet(NOW - 1,)],
              set: captureSet(NOW,),
            },);
            /** Timestamps of the result. */
            const timestamps = timestampsOf(appended,);

            expect(timestamps,)
              .toEqual([
                NOW - 1,
                NOW,
              ],);
          },
        },),

        ...underCapacityCases,

        it({
          name: 'evicts the oldest entry once the buffer is at capacity',
          fn: async () => {
            /** Buffer already holding exactly {@link MAX_ENTRIES} sets. */
            const buffer = bufferOfLength(MAX_ENTRIES,);
            /** The buffer after one more set arrives. */
            const appended = appendedCaptureBuffer({
              buffer,
              set: captureSet(NOW,),
            },);
            /** Timestamps of the result. */
            const timestamps = timestampsOf(appended,);

            expect(appended,).toHaveLength(MAX_ENTRIES,);
            expect(timestamps,)
              .toEqual([
                ...timestampsOf(buffer,)
                  .slice(1,),
                NOW,
              ],);
          },
        },),

        it({
          name: 'never grows past the capacity, however many sets arrive',
          fn: async () => {
            /** Buffer after one set arrives at capacity. */
            const onceAppended = appendedCaptureBuffer({
              buffer: bufferOfLength(MAX_ENTRIES,),
              set: captureSet(NOW,),
            },);
            /** Buffer after a second set arrives at capacity. */
            const twiceAppended = appendedCaptureBuffer({
              buffer: onceAppended,
              set: captureSet(NOW + 1,),
            },);

            expect(twiceAppended,).toHaveLength(MAX_ENTRIES,);
            expect(timestampsOf(twiceAppended,)
              .at(-1,),).toBe(NOW + 1,);
          },
        },),
      ],
    },),

    describe({
      name: retainedCaptureBuffer.name,
      children: [
        it({
          name: 'keeps a set captured now',
          fn: async () => {
            /** Buffer trimmed at its own capture time. */
            const retained = retainedCaptureBuffer({
              buffer: [captureSet(NOW,)],
              now: NOW,
            },);

            expect(retained,).toHaveLength(1,);
          },
        },),

        it({
          name: 'keeps a set captured exactly one retention window ago',
          fn: async () => {
            /** The oldest timestamp the window still keeps: the cutoff itself. */
            const cutoff = NOW - RETENTION_MS;
            /** Buffer trimmed at {@link NOW}. */
            const retained = retainedCaptureBuffer({
              buffer: [captureSet(cutoff,)],
              now: NOW,
            },);
            /** Timestamps of the result. */
            const timestamps = timestampsOf(retained,);

            expect(timestamps,).toEqual([cutoff,],);
          },
        },),

        it({
          name: 'drops a set one millisecond older than the retention window',
          fn: async () => {
            /** Buffer trimmed at {@link NOW}. */
            const retained = retainedCaptureBuffer({
              buffer: [captureSet(NOW - RETENTION_MS - 1,)],
              now: NOW,
            },);

            expect(retained,).toHaveLength(0,);
          },
        },),

        it({
          name: 'drops only the expired sets and keeps the rest in order',
          fn: async () => {
            /** Buffer straddling the cutoff: one expired, one at the cutoff, one fresh. */
            const buffer: CaptureBuffer = [
              captureSet(NOW - RETENTION_MS - 1,),
              captureSet(NOW - RETENTION_MS,),
              captureSet(NOW,),
            ];
            /** Buffer trimmed at {@link NOW}. */
            const retained = retainedCaptureBuffer({
              buffer,
              now: NOW,
            },);
            /** Timestamps of the result. */
            const timestamps = timestampsOf(retained,);

            expect(timestamps,)
              .toEqual([
                NOW - RETENTION_MS,
                NOW,
              ],);
          },
        },),

        it({
          name: 'returns an empty buffer unchanged',
          fn: async () => {
            expect(retainedCaptureBuffer({
              buffer: [],
              now: NOW,
            },),).toEqual([],);
          },
        },),
      ],
    },),

    // The module singleton is shared state, so these cases run one at a time in array order.
    describe({
      name: 'store and getRecent',
      concurrency: 1,
      children: [
        it({
          name: 'returns the just-stored set as the newest entry',
          fn: async () => {
            /** A set captured during this case. */
            const set = captureSet(Date.now(),);
            store(set,);
            /** The newest buffered entry. */
            const newest = getRecent()
              .at(-1,);

            expect(newest,).toEqual(set,);
          },
        },),

        it({
          name: 'never keeps more than the capacity, dropping the oldest sets',
          fn: async () => {
            /** Fresh timestamps, one per set, none of them older than the retention window. */
            const timestamps = Array.from(
              { length: MAX_ENTRIES + 1, },
              function makeTimestamp(_unused, index,) {
                /** Timestamp of the oldest set the case stores. */
                const oldest = Date.now() - MAX_ENTRIES;
                return oldest + index;
              },
            );
            for (const timestamp of timestamps) {
              store(captureSet(timestamp,),);
            }
            /** Timestamps left in the buffer. */
            const buffered = timestampsOf(getRecent(),);

            expect(buffered,).toEqual(timestamps.slice(1,),);
          },
        },),

        it({
          name: 'drops a set that expires while it sits in the buffer',
          fn: async ({ sinon, },) => {
            // Only `Date` is faked, so the harness deadline clock keeps running.
            /** A clock this case controls, started at the shared fixed reading. */
            const clock = sinon.useFakeTimers({
              toFake: ['Date',],
              now: NOW,
            },);
            /** A set captured at the controlled clock's first reading. */
            const set = captureSet(Date.now(),);
            store(set,);
            /** What a read returns while the set is still fresh. */
            const fresh = timestampsOf(getRecent(),);

            // `store` does not prune, so this read is the only thing that can drop the set.
            expect(fresh.at(-1,),).toBe(set.timestamp,);

            clock.tick(RETENTION_MS + 1,);
            /** What a read returns once the set has aged past the window. */
            const expired = timestampsOf(getRecent(),);

            expect(expired,).not.toContain(set.timestamp,);
          },
        },),

        it({
          name: 'never returns a set whose timestamp is older than the window',
          fn: async () => {
            /** The oldest timestamp the window still keeps. */
            const cutoff = Date.now() - RETENTION_MS;
            /** A set captured one millisecond before the cutoff. */
            const before = captureSet(cutoff - 1,);
            store(before,);
            /** Timestamps left in the buffer. */
            const buffered = timestampsOf(getRecent(),);

            expect(buffered,).not.toContain(before.timestamp,);
          },
        },),

        it({
          name: 'returns a copy, so a caller cannot change the buffer',
          fn: async () => {
            /** Snapshot the case starts from. */
            const before = timestampsOf(getRecent(),);
            /** The snapshot the case adds one set to. */
            const callerOwned = getRecent();
            /** A set that exists only in the caller's copy. */
            const extra = captureSet(Date.now(),);
            callerOwned.push(extra,);
            /** Snapshot taken after the caller changed its own copy. */
            const after = timestampsOf(getRecent(),);

            expect(after,).toEqual(before,);
          },
        },),
      ],
    },),
  ],
},);
