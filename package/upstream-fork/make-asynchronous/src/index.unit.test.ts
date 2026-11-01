/**
 Tests for the package entry point's export surface.
 
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
  getResult,
  isNodeRuntime,
  makeAsynchronous,
  makeAsynchronousIterable,
  makeCallWorkerBody,
  makeIterableWorkerBody,
  nodeWorkerPreambleSource,
  workerConstructorFrom,
  workerThreadsSpecifier,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: '',
  children: [
    it({
      name: 'exports both factories and every lifecycle helper',
      fn: async () => {
        expect(typeof makeAsynchronous,).toBe('function',);
        expect(typeof makeAsynchronousIterable,).toBe('function',);
        expect(typeof isNodeRuntime,).toBe('function',);
        expect(typeof workerThreadsSpecifier,).toBe('function',);
        expect(typeof workerConstructorFrom,).toBe('function',);
        expect(typeof blobUrl,).toBe('function',);
        expect(typeof getMessageData,).toBe('function',);
        expect(typeof createWorker,).toBe('function',);
        expect(typeof getResult,).toBe('function',);
        expect(typeof makeCallWorkerBody,).toBe('function',);
        expect(typeof makeIterableWorkerBody,).toBe('function',);
        expect(typeof errorReporterSource,).toBe('string',);
        expect(typeof nodeWorkerPreambleSource,).toBe('string',);
      },
    },),

    it({
      name: 'runs a wrapped function through the package entry point',
      fn: async () => {
        const fn = makeAsynchronous({
          fn: function greet(name: string,): string {
            return `hello ${name}`;
          },
        },);
        expect(await fn({ args: ['entry'], }),).toBe('hello entry',);
      },
    },),

    it({
      name: 'iterates a wrapped generator through the package entry point',
      fn: async () => {
        const fn = makeAsynchronousIterable({
          // oxlint-disable-next-line eslint/object-shorthand -- serialized generator functions must keep `function *` expression syntax; method shorthand serializes as `*fn` which is not a valid worker expression
          fn: function * (count: number,): Generator<number, void, unknown> {
            yield count;
          },
        },);
        /**
         Values drained through the entry point iterable.
         */
        const values: unknown[] = [];
        for await (const value of fn({ args: [7], }))
          values.push(value,);
        expect(values,).toEqual([
          7,
        ],);
      },
    },),
  ],
},);
