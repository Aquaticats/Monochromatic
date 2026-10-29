/**
 * Tests the outbox in two modes:
 *
 *   1. In-memory (`idbAvailable: false`)
 *   2. "IDB requested but absent": `idbAvailable: true` in a Bun env
 *      that has no IndexedDB. The outbox should silently fall back to
 *      in-memory rather than throw at startup.
 *
 * Both modes go through the same external API; the tests here assert
 * external behaviour (fetch calls, queue drain, ack semantics) rather
 * than implementation details.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ChunkUpload,
  createOutbox,
  type Outbox,
} from './outbox.ts';

/**
 * Sample upload helper. Defaults make tests less noisy when only the
 * draftId / seq matters.
 *
 * @param overrides - fields to set on top of defaults
 *
 * @returns a ChunkUpload with sensible defaults
 */
function sampleUpload(
  overrides: {
    readonly draftId?: string;
    readonly seq?: number;
    readonly md?: string;
    readonly html?: string;
    readonly charCount?: number;
  },
): ChunkUpload {
  return {
    draftId: 'd-1',
    seq: 0,
    md: '# hi',
    html: '<h1>hi</h1>',
    charCount: 4,
    ...overrides,
  };
}

/**
 * Disposable outbox wrapper: returns the outbox plus a Symbol.dispose
 * that calls `destroy()` so test bodies can use `using` instead of
 * try/finally.
 */
type DisposableOutbox = {
  readonly outbox: Outbox;
} & Disposable;

/**
 * Builds a disposable outbox; `using` cleanup tears it down on scope
 * exit (including throws).
 *
 * @param options - createOutbox options
 *
 * @returns disposable outbox
 */
async function makeDisposableOutbox(
  options: {
    readonly idbAvailable: boolean;
  },
): Promise<DisposableOutbox> {
  const outbox = await createOutbox(options,);
  return {
    outbox,
    [Symbol.dispose]: function dispose() {
      outbox.destroy();
    },
  };
}

/**
 * Sleeps for `ms` milliseconds without using the canonical `wait`
 * helper; importing it here would create a cycle through the same
 * module-async-time entry the outbox already uses.
 *
 * @param ms - delay in milliseconds
 *
 * @returns promise that resolves after `ms`
 */
function sleep(ms: number,): Promise<void> {
  // Bridges callback API; promise-constructor is the only path here.
  return new Promise<void>(function executor(resolve,) {
    setTimeout(
      resolve,
      ms,
    );
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: createOutbox.name,
      concurrency: 1,
      children: [
        it({
          name: 'in-memory mode: enqueue + flushed sends one PUT and drains the queue',
          fn: async ({ sinon, expect: scoped, },) => {
            const fetchStub = sinon
              .stub(globalThis, 'fetch',)
              .resolves(Response.json({ ack: 0, },),);
            using d = await makeDisposableOutbox({ idbAvailable: false, },);
            await d.outbox.enqueue(sampleUpload({},),);
            await d.outbox.flushed();
            scoped(d.outbox.pendingCount(),).toBe(0,);
            scoped(fetchStub,).toHaveBeenCalledTimes(1,);
            const [callUrl, callInit,] = fetchStub.firstCall.args;
            scoped(callUrl,).toBe('/api/drafts/d-1/chunks/0',);
            if (callInit === undefined)
              throw new Error('fetch init missing',);
            scoped(callInit.method,).toBe('PUT',);
          },
        },),

        it({
          name: 'ack drops every queue entry whose seq <= ack for that draft',
          fn: async ({ sinon, expect: scoped, },) => {
            let ack = -1;
            const fetchStub = sinon.stub(globalThis, 'fetch',).callsFake(
              function fakeFetch(url,) {
                if ((typeof url) !== 'string')
                  throw new Error('expected string url',);
                // oxlint-disable-next-line no-restricted-syntax/no-regex -- extracts the chunk seq from a test fixture URL of the shape `/chunks/<int>`; single bounded capture, no nested quantifiers
                const match = /\/chunks\/(\d+)/u.exec(url,);
                if (match === null)
                  throw new Error('no seq',);
                const seq = Number.parseInt(
                  match[1] ?? '0',
                  10,
                );
                ack = Math.max(
                  ack,
                  seq,
                );
                return Promise.resolve(Response.json({ ack, },),);
              },
            );
            using d = await makeDisposableOutbox({ idbAvailable: false, },);
            for (const seq of [0, 1, 2, 3,]) {
              // oxlint-disable-next-line no-await-in-loop
              await d.outbox.enqueue(sampleUpload({ seq, },),);
            }
            await d.outbox.flushed();
            scoped(d.outbox.pendingCount(),).toBe(0,);
            scoped(fetchStub,).toHaveBeenCalledTimes(4,);
          },
        },),

        it({
          name: 'flushed() with empty queue resolves immediately',
          fn: async () => {
            using d = await makeDisposableOutbox({ idbAvailable: false, },);
            let resolved = false;
            await Promise.race([
              d.outbox.flushed().then(function setResolved() {
                resolved = true;
                return undefined;
              },),
              sleep(50,),
            ],);
            expect(resolved,).toBe(true,);
          },
        },),

        it({
          name:
            'pause on terminal PUT failure: queue keeps the entry, flushed stays pending',
          fn: async ({ sinon, expect: scoped, },) => {
            sinon
              .stub(globalThis, 'fetch',)
              .rejects(new Error('network down',),);
            using d = await makeDisposableOutbox({ idbAvailable: false, },);
            await d.outbox.enqueue(sampleUpload({},),);
            // 250 + 500 + 1000 ms across 3 retries; 2.5s gives slack.
            await sleep(2_500,);
            scoped(d.outbox.pendingCount(),).toBe(1,);
            let resolved = false;
            await Promise.race([
              d.outbox.flushed().then(function setResolved() {
                resolved = true;
                return undefined;
              },),
              sleep(100,),
            ],);
            scoped(resolved,).toBe(false,);
          },
          timeout: 5_000,
        },),

        it({
          name: 'enqueue after destroy throws',
          fn: async () => {
            const outbox = await createOutbox({ idbAvailable: false, },);
            outbox.destroy();
            let caught: unknown = null;
            try {
              await outbox.enqueue(sampleUpload({},),);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,).toContain('destroyed',);
          },
        },),

        it({
          name: 'idbAvailable=true with no IDB present falls back to in-memory silently',
          fn: async ({ sinon, },) => {
            sinon
              .stub(globalThis, 'fetch',)
              .resolves(Response.json({ ack: 0, },),);
            // Bun has no globalThis.indexedDB; createOutbox catches
            // the open failure and proceeds in-memory.
            using d = await makeDisposableOutbox({ idbAvailable: true, },);
            await d.outbox.enqueue(sampleUpload({},),);
            await d.outbox.flushed();
            expect(d.outbox.pendingCount(),).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
