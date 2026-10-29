/**
 Tests for the worker lifecycle: host detection, specifier assembly, message
 extraction, constructor lookup, blob URLs, and worker creation.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  blobUrl,
  createWorker,
  errorReporterSource,
  getMessageData,
  isMessageEvent,
  isNodeRuntime,
  isWorkerReply,
  nodeWorkerPreambleSource,
  workerConstructorFrom,
  workerThreadsSpecifier,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: isNodeRuntime.name,
      children: [
        it({
          name: 'reports the Node.js runtime in tests',
          fn: async () => {
            expect(isNodeRuntime(),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: workerThreadsSpecifier.name,
      children: [
        it({
          name: 'assembles the worker_threads specifier without a static literal',
          fn: async () => {
            expect(workerThreadsSpecifier(),).toBe('node:worker_threads',);
          },
        },),
      ],
    },),

    describe({
      name: getMessageData.name,
      children: [
        it({
          name: 'reads the raw value on Node.js hosts',
          fn: async () => {
            /**
             Reply envelope delivered raw on Node.js.
             */
            const reply = {
              id: 3,
              output: 42,
            };
            expect(getMessageData({ value: reply, },),).toEqual(reply,);
          },
        },),

        it({
          name: 'rejects a message that is not a reply envelope',
          fn: async () => {
            let caught: unknown;
            try {
              getMessageData({ value: { id: 3, }, },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(TypeError,);
            expect((caught as TypeError).message,).toBe('Worker message is not a reply envelope',);
          },
        },),
      ],
    },),

    describe({
      name: isWorkerReply.name,
      children: [
        it({
          name: 'accepts output and error envelopes and rejects anything else',
          fn: async () => {
            expect(isWorkerReply({
              id: 1,
              output: 42,
            },),).toBe(true,);
            expect(isWorkerReply({
              error: 0,
              id: 2,
            },),).toBe(true,);
            expect(isWorkerReply(null,),).toBe(false,);
            expect(isWorkerReply({ id: 3, },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: isMessageEvent.name,
      children: [
        it({
          name: 'accepts data-wrapped replies and rejects anything else',
          fn: async () => {
            expect(isMessageEvent({
              data: {
                id: 1,
                output: 42,
              },
            },),).toBe(true,);
            expect(isMessageEvent(null,),).toBe(false,);
            expect(isMessageEvent({ id: 1, },),).toBe(false,);
            expect(isMessageEvent({
              data: { id: 2, },
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: workerConstructorFrom.name,
      children: [
        it({
          name: 'reads the Worker constructor off the namespace',
          fn: async () => {
            /**
             Constructor read off the namespace.
             */
            const ctor = workerConstructorFrom({ Worker: globalThis.Worker, },);
            expect(ctor,).toBe(globalThis.Worker,);
          },
        },),
      ],
    },),

    describe({
      name: blobUrl.name,
      children: [
        it({
          name: 'creates a revocable blob URL for worker source',
          fn: async () => {
            /**
             Blob URL revoked at the end of the test.
             */
            const url = blobUrl('globalThis.postMessage({});',);
            expect(url.startsWith('blob:',),).toBe(true,);
            URL.revokeObjectURL(url,);
          },
        },),
      ],
    },),

    describe({
      name: createWorker.name,
      children: [
        it({
          name: 'rejects before creating a worker when the signal is pre-aborted',
          fn: async () => {
            /**
             Abort reason the creation must reject with.
             */
            const failure = new Error('Aborted',);
            /**
             Controller aborted before creation starts.
             */
            const controller = new AbortController();
            controller.abort(failure,);
            let caught: unknown;
            try {
              await createWorker({
                content: `${errorReporterSource}\n`,
                nodePreamble: nodeWorkerPreambleSource,
                signal: controller.signal,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBe(failure,);
          },
        },),

        it({
          name: 'rejects a second request with the remembered worker failure',
          fn: async () => {
            /**
             Live worker failing its only request.
             */
            const worker = await createWorker({
              content: `${errorReporterSource}\n\tglobalThis.onmessage = async () => {\n\t\tthrow new Error('worker failed');\n\t};\n`,
              nodePreamble: nodeWorkerPreambleSource,
            },);
            /**
             First request observing the worker failure.
             */
            const first = worker.request([1],);
            /**
             Whether the first request settled before the second runs.
             */
            const firstSettled = await Promise.allSettled([first],);
            expect(firstSettled[0]?.status,).toBe('rejected',);
            let caught: unknown;
            try {
              await worker.request([2],);
            }
            catch (error) {
              caught = error;
            }
            expect((caught as Error).message,).toBe('worker failed',);
            worker.cleanup();
          },
          timeout: 30_000,
        },),

        it({
          name: 'serves one request and terminates on cleanup',
          fn: async () => {
            /**
             Live worker answering one doubling call.
             */
            const worker = await createWorker({
              content: `${errorReporterSource}\n\tglobalThis.onmessage = async ({data: {id, arguments_}}) => {\n\t\tglobalThis.postMessage({id, output: arguments_[0] * 2});\n\t};\n`,
              nodePreamble: nodeWorkerPreambleSource,
            },);
            expect(await worker.request([21],),).toEqual({
              id: 1,
              output: 42,
            },);
            worker.cleanup();
          },
          timeout: 30_000,
        },),

        it({
          name: 'throws synchronously when the arguments cannot be cloned',
          fn: async () => {
            /**
             Live worker posted an unclonable argument.
             */
            const worker = await createWorker({
              content: `${errorReporterSource}\n\tglobalThis.onmessage = async ({data: {id, arguments_}}) => {\n\t\tglobalThis.postMessage({id, output: 1});\n\t};\n`,
              nodePreamble: nodeWorkerPreambleSource,
            },);
            let caught: unknown;
            try {
              void worker.request([function unclonable(): void {},],);
            }
            catch (error) {
              caught = error;
            }
            expect((caught as Error).message,).toContain('could not be cloned',);
            worker.cleanup();
          },
          timeout: 30_000,
        },),
      ],
    },),
  ],
},);
