/**
 Tests for the wrangler entry: routing through `fetch`, log flushing through
 `waitUntil`, and failure propagation.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createMemoryObjectStore,
  type ExecutionContextLike,
  type WorkerEnv,
} from '@monochromatic-dev/config-lfs-r2-worker';
import worker from '@monochromatic-dev/config-lfs-r2-worker/worker';

/**
 oid of the object seeded into the store.
 */
const PRESENT_OID = 'a'.repeat(64,);

/**
 Bytes seeded under {@link PRESENT_OID}.
 */
const PRESENT_BYTES = new TextEncoder().encode('present object',);

/**
 Origin every request targets.
 */
const ORIGIN = 'https://lfs.test';

/**
 Execution context that records every promise handed to `waitUntil`.
 */
type RecordingContext = ExecutionContextLike & {
  /**
   Promises handed to `waitUntil`, in call order.
   */
  readonly pending: Promise<unknown>[];
};

/**
 Build a recording execution context.

 @returns context whose `pending` list grows with each `waitUntil` call
 */
function recordingContext(): RecordingContext {
  /**
   Promises handed to `waitUntil`.
   */
  const pending: Promise<unknown>[] = [];
  return {
    pending,
    waitUntil(promise: Promise<unknown>,): void {
      pending.push(promise,);
    },
  };
}

/**
 Fresh env holding only the present object.

 @returns env for one case
 */
function freshEnv(): WorkerEnv {
  return {
    BUCKET: createMemoryObjectStore({ [PRESENT_OID]: PRESENT_BYTES, },),
    LFS_WRITE_TOKEN: 'secret',
  };
}

await describe({
  name: 'worker.fetch',
  children: [
    it({
      name: 'routes like handleRequest and serves the image media type',
      fn: async () => {
        const ctx = recordingContext();
        /**
         Response through the wrangler entry.
         */
        const response = await worker.fetch(
          new Request(`${ORIGIN}/${PRESENT_OID}/package/music-player/asset/readme/shot.png`,),
          freshEnv(),
          ctx,
        );
        expect(response.status,).toBe(200,);
        expect(response.headers.get('Content-Type',),).toBe('image/png',);
        expect(
          new Uint8Array(await response.arrayBuffer(),),
        ).toEqual(PRESENT_BYTES,);
      },
    },),
    it({
      name: 'hands the log flush to waitUntil and the flush settles',
      fn: async () => {
        const ctx = recordingContext();
        await worker.fetch(new Request(`${ORIGIN}/${PRESENT_OID}`,), freshEnv(), ctx,);
        expect(ctx.pending,).toHaveLength(1,);
        await Promise.all(ctx.pending,);
      },
    },),
    it({
      name: 'rethrows an unexpected store failure and still flushes',
      fn: async () => {
        const ctx = recordingContext();
        /**
         Env whose store fails every read.
         */
        const failing: WorkerEnv = {
          BUCKET: {
            ...createMemoryObjectStore(),
            get: async function get(): Promise<never> {
              throw new Error('bucket unavailable',);
            },
          },
        };
        /**
         Failure surfaced by the entry.
         */
        let caught: unknown;
        try {
          await worker.fetch(new Request(`${ORIGIN}/${PRESENT_OID}`,), failing, ctx,);
        }
        catch (error) {
          caught = error;
        }
        expect((caught as Error).message,).toBe('bucket unavailable',);
        expect(ctx.pending,).toHaveLength(1,);
        await Promise.all(ctx.pending,);
      },
    },),
  ],
},);
