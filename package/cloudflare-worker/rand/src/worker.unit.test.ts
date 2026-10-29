/**
 Tests for the wrangler entry: routing through `fetch`, log flushing through
 `waitUntil`, and propagation of a failure that is not the caller's fault.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  ExecutionContextLike,
  WorkerEnv,
} from '@monochromatic-dev/cloudflare-worker-rand';
import worker from '@monochromatic-dev/cloudflare-worker-rand/worker';

/**
 Origin every request targets.
 */
const ORIGIN = 'https://rand.test';

/**
 Env handed to the entry. Empty because `wrangler.toml` declares no binding
 and no var.
 */
const EMPTY_ENV: WorkerEnv = {};

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

await describe({
  name: 'worker.fetch',
  children: [
    it({
      name: 'routes like handleRequest and serves the value with the shared headers',
      fn: async () => {
        /**
         Response through the wrangler entry.
         */
        const response = worker.fetch(new Request(`${ORIGIN}/8`,), EMPTY_ENV, recordingContext(),);
        expect(response.status,).toBe(200,);
        expect(response.headers.get('Content-Type',),).toBe('text/plain; charset=utf-8',);
        expect(response.headers.get('Cache-Control',),).toBe('no-store, no-cache, must-revalidate',);
        expect(await response.text(),).toHaveLength(8,);
      },
    },),
    it({
      name: 'answers a refusal without throwing, so the platform does not report a 500',
      fn: async () => {
        /**
         Response for a length outside the served range.
         */
        const response = worker.fetch(new Request(`${ORIGIN}/65`,), EMPTY_ENV, recordingContext(),);
        expect(response.status,).toBe(400,);
      },
    },),
    it({
      name: 'hands the log flush to waitUntil and the flush settles',
      fn: async () => {
        /**
         Context recording the flush.
         */
        const ctx = recordingContext();
        worker.fetch(new Request(`${ORIGIN}/uuidv4`,), EMPTY_ENV, ctx,);
        expect(ctx.pending,).toHaveLength(1,);
        await Promise.all(ctx.pending,);
      },
    },),
    it({
      name: 'rethrows a failure that is not the caller fault and still flushes',
      fn: async () => {
        /**
         Context recording the flush.
         */
        const ctx = recordingContext();
        /**
         How many times the entry has read the request URL, so the injected
         failure can fire on the routing read while the catch block's log line
         still resolves a URL.
         */
        let urlReads = 0;
        /**
         Request whose URL access fails once, standing in for a platform fault
         that is neither a bad request nor an unknown route.
         */
        const broken = {
          method: 'GET',
          get url(): string {
            urlReads += 1;
            if (urlReads === 1) {
              throw new Error('url unavailable',);
            }
            return `${ORIGIN}/1`;
          },
        } as unknown as Request;
        /**
         Failure surfaced by the entry.
         */
        let caught: unknown;
        try {
          worker.fetch(broken, EMPTY_ENV, ctx,);
        }
        catch (error) {
          caught = error;
        }
        expect((caught as Error).message,).toBe('url unavailable',);
        expect(ctx.pending,).toHaveLength(1,);
        await Promise.all(ctx.pending,);
      },
    },),
  ],
},);
