/**
 Retention and eviction of the rolling capture buffer.

 The buffer decides what the vision model is shown: it keeps the newest
 {@link MAX_ENTRIES} capture sets and drops any set older than
 {@link RETENTION_MS}. Both boundaries are off-by-one risks the package test
 policy (`PKG`, `TCV`) requires covered: one entry too few loses context, one
 too many feeds the model a frame outside the retention window.

 Retention is asserted on {@link retainedCaptureBuffer}, the pure function
 that takes the clock reading, because a set stored through the module
 singleton cannot be made to expire without waiting the full ten minutes. The
 singleton cases cover the append, eviction and copy behaviour that does not
 depend on time passing.

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
} from './memory.ts';

import type { CaptureBuffer, CaptureSet, } from './memory.ts';

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
  return Array.from(
    { length, },
    function makeSet(_unused, index,) {
      return captureSet(NOW - length + index,);
    },
  );
}

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
            /** The buffer after the append, read by timestamp. */
            const appended = appendedCaptureBuffer({
              buffer: [captureSet(NOW - 1,)],
              set: captureSet(NOW,),
            },);

            expect(timestampsOf(appended,),).toEqual([
              NOW - 1,
              NOW,
            ],);
          },
        },),

        ...Array.from(
          { length: MAX_ENTRIES, },
          function makeUnderCapacityCase(_unused, index,) {
            /** Entry count this case starts from; the array covers one entry up to the capacity. */
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

                expect(appended,).toHaveLength(length + 1,);
                expect(timestampsOf(appended,),).toEqual([
                  ...timestampsOf(buffer,),
                  NOW,
                ],);
              },
            },);
          },
        ),

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

            expect(appended,).toHaveLength(MAX_ENTRIES,);
            expect(timestampsOf(appended,),).toEqual([
              ...timestampsOf(buffer,)
                .slice(1,),
              NOW,
            ],);
          },
        },),

        it({
          name: 'never grows past the capacity, however many sets arrive',
          fn: async () => {
            /** Buffer threaded through two appends at capacity. */
            let buffer: CaptureBuffer = bufferOfLength(MAX_ENTRIES,);
            for (const set of [captureSet(NOW,), captureSet(NOW + 1,)]) {
              buffer = appendedCaptureBuffer({
                buffer,
                set,
              },);
            }

            expect(buffer,).toHaveLength(MAX_ENTRIES,);
            expect(timestampsOf(buffer,)
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
            expect(retainedCaptureBuffer({
              buffer: [captureSet(NOW,)],
              now: NOW,
            },),).toHaveLength(1,);
          },
        },),

        it({
          name: 'keeps a set captured exactly one retention window ago',
          fn: async () => {
            /** The oldest timestamp the window still keeps: the cutoff itself. */
            const cutoff = NOW - RETENTION_MS;

            expect(timestampsOf(retainedCaptureBuffer({
              buffer: [captureSet(cutoff,)],
              now: NOW,
            },),),).toEqual([cutoff,],);
          },
        },),

        it({
          name: 'drops a set one millisecond older than the retention window',
          fn: async () => {
            expect(retainedCaptureBuffer({
              buffer: [captureSet(NOW - RETENTION_MS - 1,)],
              now: NOW,
            },),).toHaveLength(0,);
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

            expect(timestampsOf(retainedCaptureBuffer({
              buffer,
              now: NOW,
            },),),).toEqual([
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

            expect(getRecent()
              .at(-1,),).toEqual(set,);
          },
        },),

        it({
          name: 'never keeps more than the capacity, dropping the oldest sets',
          fn: async () => {
            /** Fresh timestamps, one per set, none of them older than the retention window. */
            const timestamps = Array.from(
              { length: MAX_ENTRIES + 1, },
              function makeTimestamp(_unused, index,) {
                return Date.now() - MAX_ENTRIES + index;
              },
            );
            for (const timestamp of timestamps) {
              store(captureSet(timestamp,),);
            }

            expect(timestampsOf(getRecent(),),).toEqual(timestamps.slice(1,),);
          },
        },),

        it({
          name: 'drops a set that is already outside the retention window',
          fn: async () => {
            /** A set captured long before the window. */
            const expired = captureSet(Date.now() - RETENTION_MS - 1,);

            store(expired,);

            expect(timestampsOf(getRecent(),),).not.toContain(expired.timestamp,);
          },
        },),

        it({
          name: 'returns a copy, so a caller cannot change the buffer',
          fn: async () => {
            /** Snapshot the case starts from. */
            const before = timestampsOf(getRecent(),);
            /** The snapshot with one set a caller added for itself. */
            const callerOwned = getRecent();
            callerOwned.push(captureSet(Date.now(),),);

            expect(timestampsOf(getRecent(),),).toEqual(before,);
          },
        },),
      ],
    },),
  ],
},);
