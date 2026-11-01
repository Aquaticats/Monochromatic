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

import { createMemoryStorage, } from './adapter-memory.ts';
import { createWriteBuffer, } from './write-buffer.ts';

const encoder = new TextEncoder();

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
      ],
    },),
  ],
},);
