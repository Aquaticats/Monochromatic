/**
 * Unit tests for the coalescing write buffer.
 *
 * Each test isolates a single behaviour: size-based flush, time-based
 * flush, key coalescing, close idempotency, pending counter.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import type {
  Storage,
  StoragePutItem,
} from './adapter.ts';
import { createMemoryStorage, } from './adapter-memory.ts';
import { createWriteBuffer, } from './write-buffer.ts';

const encoder = new TextEncoder();

/**
 * Milliseconds allowed for a flush promise to settle before the test treats it
 * as hung, so a stranded queue fails an assertion instead of the suite.
 */
const SETTLE_WINDOW_MS = 2_000;

/**
 * Waits up to `windowMs` for a promise to settle, counting a rejection as
 * settling so the caller can assert on the outcome afterwards.
 *
 * @param promise - promise under test
 *
 * @param windowMs - milliseconds allowed before the promise counts as hung
 *
 * @returns true when the promise settled inside the window
 *
 * @example
 * ```ts
 * const settled = await settledWithin(pending, 2000);
 * ```
 */
async function settledWithin(promise: Promise<unknown>, windowMs: number,): Promise<boolean> {
  /**
   * Sentinel the race resolves to when the window elapses before the promise
   * settles.
   */
  const WINDOW_ELAPSED = Symbol('settle window elapsed with no settlement',);
  /**
   * Handle of the timer that ends the window, held in an object so the function
   * body keeps only consts.
   */
  const timer: { handle?: ReturnType<typeof setTimeout> } = {};
  const window = new Promise<typeof WINDOW_ELAPSED>(function startTimer(resolve,) {
    timer.handle = setTimeout(
      function elapsed() {
        resolve(WINDOW_ELAPSED,);
      },
      windowMs,
    );
  },);
  const winner = await Promise.race([
    promise.then(
      function settled() {
        return undefined;
      },
      function rejected() {
        return undefined;
      },
    ),
    window,
  ],);
  clearTimeout(timer.handle,);
  return winner !== WINDOW_ELAPSED;
}

/**
 * Storage adapter that records every batch it receives, can hold the next
 * batch open until the test releases it, and fails a chosen number of leading
 * batches.
 *
 * @param row - inputs
 *
 * @returns recording adapter, the batches it saw, and the gate batches wait on
 *
 * @example
 * ```ts
 * const { storage, batches } = createRecordingStorage({ underlying, failFirstBatch: true });
 * ```
 */
function createRecordingStorage(row: {
  readonly underlying: Storage;
  readonly failFirstBatch: boolean;
}): {
  readonly storage: Storage;
  readonly batches: readonly (readonly StoragePutItem[])[];
} {
  const batches: (readonly StoragePutItem[])[] = [];
  const storage: Storage = {
    ...row.underlying,
    async putBatch(items: readonly StoragePutItem[],): Promise<void> {
      batches.push([...items,],);
      if (row.failFirstBatch && (batches.length === 1))
        throw new Error('storage backend unavailable',);
      await row.underlying.putBatch(items,);
    },
  };
  return {
    storage,
    batches,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: createWriteBuffer.name,
      children: [
        it({
          name: 'flushes after enqueue when item count reaches flushAtItems',
          async fn() {
            const storage = createMemoryStorage();
            const buffer = createWriteBuffer({
              storage,
              flushAtItems: 2,
              flushAtMs: 10_000,
            },);
            buffer.enqueue({
              key: 'a',
              body: encoder.encode('1',),
            },);
            buffer.enqueue({
              key: 'b',
              body: encoder.encode('2',),
            },);
            await buffer.flush();
            expect(await storage.get('a',),).toBeDefined();
            expect(await storage.get('b',),).toBeDefined();
            await buffer.close();
          },
        },),
        it({
          name: 'flushes after flushAtMs even when below the size threshold',
          async fn() {
            const storage = createMemoryStorage();
            const buffer = createWriteBuffer({
              storage,
              flushAtItems: 100,
              flushAtMs: 5,
            },);
            buffer.enqueue({
              key: 'a',
              body: encoder.encode('1',),
            },);
            await new Promise(function delay(resolve,) {
              setTimeout(
                resolve,
                30,
              );
            },);
            expect(await storage.get('a',),).toBeDefined();
            await buffer.close();
          },
        },),
        it({
          name: 'coalesces repeated keys to the latest body',
          async fn() {
            const storage = createMemoryStorage();
            const buffer = createWriteBuffer({
              storage,
              flushAtItems: 100,
              flushAtMs: 100,
            },);
            buffer.enqueue({
              key: 'a',
              body: encoder.encode('first',),
            },);
            buffer.enqueue({
              key: 'a',
              body: encoder.encode('second',),
            },);
            await buffer.flush();
            const value = await storage.get('a',);
            expect(new TextDecoder().decode(value,),).toBe('second',);
            await buffer.close();
          },
        },),
        it({
          name: 'close prevents further enqueue',
          async fn() {
            const storage = createMemoryStorage();
            const buffer = createWriteBuffer({ storage, },);
            await buffer.close();
            expect(function reEnqueueAfterClose() {
              buffer.enqueue({
                key: 'a',
                body: encoder.encode('1',),
              },);
            },)
              // oxlint-disable-next-line no-restricted-syntax/no-regex -- Test assertion matches a short literal substring in a thrown error message; expect API requires a RegExp.
              .toThrow(/closed/u,);
          },
        },),
        it({
          name: 'pending counter reflects the queue size before flush',
          async fn() {
            const storage = createMemoryStorage();
            const buffer = createWriteBuffer({
              storage,
              flushAtItems: 100,
              flushAtMs: 1_000,
            },);
            buffer.enqueue({
              key: 'a',
              body: encoder.encode('1',),
            },);
            buffer.enqueue({
              key: 'b',
              body: encoder.encode('2',),
            },);
            expect(buffer.pending,).toBe(2,);
            await buffer.flush();
            expect(buffer.pending,).toBe(0,);
            await buffer.close();
          },
        },),
        it({
          name: 'close() flushes pending items before returning',
          async fn() {
            const storage = createMemoryStorage();
            const buffer = createWriteBuffer({
              storage,
              flushAtItems: 100,
              flushAtMs: 10_000,
            },);
            buffer.enqueue({
              key: 'a',
              body: encoder.encode('written-by-close',),
            },);
            await buffer.close();
            const value = await storage.get('a',);
            expect(new TextDecoder().decode(value,),).toBe(
              'written-by-close',
            );
          },
        },),
        it({
          name: 're-queues a batch whose putBatch rejected and delivers it on the next flush',
          async fn() {
            const memory = createMemoryStorage();
            const { storage, batches } = createRecordingStorage({
              underlying: memory,
              failFirstBatch: true,
            },);
            const buffer = createWriteBuffer({
              storage,
              flushAtItems: 100,
              flushAtMs: 100,
            },);
            buffer.enqueue({
              key: 'a',
              body: encoder.encode('1',),
            },);
            buffer.enqueue({
              key: 'b',
              body: encoder.encode('2',),
            },);
            /**
             * First flush promise; expected to reject with the backend failure.
             */
            const firstFlush = buffer.flush();
            await expect(firstFlush,).rejects.toThrow('unavailable',);
            expect(batches.length,).toBe(1,);
            expect(await memory.get('a',),).toBeUndefined();
            await buffer.flush();
            expect(batches.length,).toBe(2,);
            expect(batches[1]?.map(function keyOf(item,) {
              return item.key;
            },),).toEqual([
              'a',
              'b',
            ],);
            /**
             * Bodies read back after the retried batch.
             */
            const bodyA = new TextDecoder().decode(await memory.get('a',),);
            const bodyB = new TextDecoder().decode(await memory.get('b',),);
            expect(bodyA,).toBe('1',);
            expect(bodyB,).toBe('2',);
            await buffer.close();
          },
        },),
        it({
          name: 'lets a body enqueued after a failed batch win over the re-queued one',
          async fn() {
            const memory = createMemoryStorage();
            const { storage, batches } = createRecordingStorage({
              underlying: memory,
              failFirstBatch: true,
            },);
            const buffer = createWriteBuffer({
              storage,
              flushAtItems: 100,
              flushAtMs: 100,
            },);
            buffer.enqueue({
              key: 'a',
              body: encoder.encode('superseded',),
            },);
            await expect(buffer.flush(),).rejects.toThrow('unavailable',);
            buffer.enqueue({
              key: 'a',
              body: encoder.encode('latest',),
            },);
            await buffer.flush();
            expect(batches.length,).toBe(2,);
            /**
             * Body the retried batch delivered for the superseded key.
             */
            const latestBody = new TextDecoder().decode(await memory.get('a',),);
            expect(latestBody,).toBe('latest',);
            await buffer.close();
          },
        },),
        it({
          name: 'does not restore a body superseded while its batch was in flight',
          async fn() {
            const memory = createMemoryStorage();
            /**
             * One entry per putBatch call, each settled by the test.
             */
            const attempts: {
              readonly items: readonly StoragePutItem[];
              readonly deliver: () => void;
              readonly fail: () => void;
            }[] = [];
            const storage: Storage = {
              ...memory,
              putBatch(items: readonly StoragePutItem[],) {
                return new Promise<void>(function hold(resolve, reject,) {
                  attempts.push({
                    items: [...items,],
                    deliver: function deliverBatch() {
                      resolve(memory.putBatch(items,),);
                    },
                    fail: function failBatch() {
                      reject(new Error('backend unavailable',),);
                    },
                  },);
                },);
              },
            };
            const buffer = createWriteBuffer({
              storage,
              flushAtItems: 100,
              flushAtMs: 10_000,
            },);
            buffer.enqueue({
              key: 'a',
              body: encoder.encode('superseded',),
            },);
            /**
             * First flush holds its batch open, so the next enqueue and flush
             * both land while it is still in flight.
             */
            const firstFlush = buffer.flush();
            buffer.enqueue({
              key: 'a',
              body: encoder.encode('latest',),
            },);
            const secondFlush = buffer.flush();
            expect(attempts.length,).toBe(2,);
            attempts[0]
              ?.fail();
            await expect(firstFlush,).rejects.toThrow('unavailable',);
            attempts[1]
              ?.deliver();
            const secondSettled = await settledWithin(
              secondFlush,
              SETTLE_WINDOW_MS,
            );
            expect(secondSettled,).toBe(true,);
            /**
             * The superseded body must not be waiting for a further flush.
             */
            expect(buffer.pending,).toBe(0,);
            expect(attempts.length,).toBe(2,);
            /**
             * Body the store ends up with: the later write, never the failed
             * older one coming back from the requeue.
             */
            const stored = await memory.get('a',);
            expect(new TextDecoder().decode(stored,),).toBe('latest',);
            await buffer.close();
          },
        },),
        it({
          name: 'close rejects naming the keys it could not write',
          async fn() {
            const memory = createMemoryStorage();
            const { storage } = createRecordingStorage({
              underlying: memory,
              failFirstBatch: true,
            },);
            const buffer = createWriteBuffer({
              storage,
              flushAtItems: 100,
              flushAtMs: 10_000,
            },);
            buffer.enqueue({
              key: 'fragments/lost',
              body: encoder.encode('1',),
            },);
            await expect(buffer.close(),).rejects.toThrow('fragments/lost',);
            await expect(buffer.flush(),).resolves.toBeUndefined();
            expect(await memory.get('fragments/lost',),).toBeDefined();
          },
        },),
      ],
    },),
  ],
},);
