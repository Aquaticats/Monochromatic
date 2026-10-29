/**
 Tests for the worker-source builders: template fidelity and per-kind worker
 bodies.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  errorReporterSource,
  makeCallWorkerBody,
  makeIterableWorkerBody,
  nodeWorkerPreambleSource,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: errorReporterSource,
      children: [
        it({
          name: 'carries the error serializer and its AggregateError branch',
          fn: async () => {
            expect(errorReporterSource,).toContain('getErrorProperties',);
            expect(errorReporterSource,).toContain('reportError',);
            expect(errorReporterSource,).toContain('error.errors',);
          },
        },),
      ],
    },),

    describe({
      name: nodeWorkerPreambleSource,
      children: [
        it({
          name: 'shims worker globals and wires the baseUrl module hook',
          fn: async () => {
            expect(nodeWorkerPreambleSource,).toContain('node:worker_threads',);
            expect(nodeWorkerPreambleSource,).toContain('registerHooks',);
            expect(nodeWorkerPreambleSource,).toContain('workerData.baseUrl',);
            expect(nodeWorkerPreambleSource,).toContain('globalThis.postMessage',);
          },
        },),
      ],
    },),

    describe({
      name: makeCallWorkerBody.name,
      children: [
        it({
          name: 'invokes the serialized function once per message',
          fn: async () => {
            /**
             Worker body for a doubling function.
             */
            const body = makeCallWorkerBody('function (x) { return x; }',);
            expect(body,).toContain('globalThis.onmessage',);
            expect(body,).toContain('function (x) { return x; }',);
            expect(body,).toContain('{id, output}',);
            expect(body,).toContain('reportError(error, id)',);
          },
        },),
      ],
    },),

    describe({
      name: makeIterableWorkerBody.name,
      children: [
        it({
          name: 'normalizes the first return into an iterator and guards next results',
          fn: async () => {
            /**
             Worker body for a generator function.
             */
            const body = makeIterableWorkerBody('function * (x) { yield x; }',);
            expect(body,).toContain('Symbol.asyncIterator',);
            expect(body,).toContain('Symbol.iterator',);
            expect(body,).toContain('iterator.next()',);
            expect(body,).toContain('Iterator result is not an object',);
          },
        },),
      ],
    },),
  ],
},);
