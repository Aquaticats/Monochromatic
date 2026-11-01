/**
 Unit tests for the structural edit branches: setting an existing array element
 versus appending, missing-segment and out-of-range errors, duplicate-key
 last-wins, nested descent, comment preservation, deletion, and the read-side
 navigation guards.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type StringJsonc,
  COMMENT_ABSENT,
  jsoncDelete,
  jsoncGetComment,
  jsoncGetValue,
  jsoncHas,
  jsoncKeys,
  jsoncSet,
  jsoncStringify,
  parseJsoncEdit,
} from '../dist/final/neutral/index.mjs';

const asJsonc = (source: string,): StringJsonc => source as StringJsonc;

const base = (): ReturnType<typeof parseJsoncEdit> =>
  parseJsoncEdit({ source: asJsonc('{ "list": [10, 20], "nested": { "b": 2 } } // c',), },);

const matrix = (): ReturnType<typeof parseJsoncEdit> =>
  parseJsoncEdit({ source: asJsonc('{ "m": [[1, 2], [3, 4]] } // c',), },);

const dup = (): ReturnType<typeof parseJsoncEdit> =>
  parseJsoncEdit({ source: asJsonc('{ "a": 1, "a": 2 } // c',), },);

/**
 Fractional segment used to verify array indexes must be integers.
 */
const HALF_INDEX = 1 / 2;

await describe({
  name: 'edit-set branches',
  children: [
    describe({
      name: 'set into arrays',
      children: [
        it({
          name: 'replaces an existing element in range, leaving siblings intact',
          fn: async () => {
            const next = jsoncSet({ state: base(), path: ['list', 1,], value: 99, },);
            expect(jsoncGetValue({ state: next, path: ['list',], },),).toEqual([10, 99,],);
          },
        },),
        it({
          name: 'refuses to replace the document root with a scalar',
          fn: async () => {
            // The root must stay a container: a scalar root emits text this package's own parser
            // rejects. Fuzzing the Rust port found it as an emission of `null`.
            expect(() => {
              jsoncSet({ state: base(), path: [], value: null, },);
            },).toThrow('root must stay an object or array',);
            expect(() => {
              jsoncSet({ state: base(), path: [], value: 7, },);
            },).toThrow('root must stay an object or array',);
            const next = jsoncSet({ state: base(), path: [], value: [], },);
            expect(jsoncGetValue({ state: next, path: [], },),).toEqual([],);
          },
        },),
        it({
          name: 'appends at the length index',
          fn: async () => {
            const next = jsoncSet({ state: base(), path: ['list', 2,], value: 30, },);
            expect(jsoncGetValue({ state: next, path: ['list',], },),).toEqual([10, 20, 30,],);
          },
        },),
        it({
          name: 'throws on a negative or fractional index',
          fn: async () => {
            expect(() => {
              jsoncSet({ state: base(), path: ['list', -1,], value: 1, },);
            },).toThrow('no JSONC node at path',);
            expect(() => {
              jsoncSet({ state: base(), path: ['list', HALF_INDEX,], value: 1, },);
            },).toThrow('no JSONC node at path',);
          },
        },),
        it({
          name: 'throws on an index past the length',
          fn: async () => {
            expect(() => {
              jsoncSet({ state: base(), path: ['list', 5,], value: 1, },);
            },).toThrow('no JSONC node at path',);
          },
        },),
        it({
          name: 'throws when the length index is not the final segment',
          fn: async () => {
            expect(() => {
              jsoncSet({ state: base(), path: ['list', 2, 'x',], value: 1, },);
            },).toThrow('no JSONC node at path',);
          },
        },),
        it({
          name: 'descends into a nested array element',
          fn: async () => {
            const next = jsoncSet({ state: matrix(), path: ['m', 1, 0,], value: 9, },);
            expect(jsoncGetValue({ state: next, path: ['m',], },),).toEqual([[1, 2,], [9, 4,],],);
          },
        },),
      ],
    },),
    describe({
      name: 'set into records',
      children: [
        it({
          name: 'creates a missing trailing key',
          fn: async () => {
            const next = jsoncSet({ state: base(), path: ['fresh',], value: true, },);
            expect(jsoncGetValue({ state: next, path: ['fresh',], },),).toBe(true,);
          },
        },),
        it({
          name: 'throws when an intermediate key is missing',
          fn: async () => {
            expect(() => {
              jsoncSet({ state: base(), path: ['missing', 'x',], value: 1, },);
            },).toThrow('no JSONC node at path',);
          },
        },),
        it({
          name: 'updates the last of duplicate keys',
          fn: async () => {
            const next = jsoncSet({ state: dup(), path: ['a',], value: 9, },);
            expect(jsoncGetValue({ state: next, path: ['a',], },),).toBe(9,);
          },
        },),
        it({
          name: 'throws when indexing a scalar',
          fn: async () => {
            expect(() => {
              jsoncSet({ state: dup(), path: ['a', 'b',], value: 1, },);
            },).toThrow('cannot index',);
          },
        },),
      ],
    },),
    describe({
      name: 'set and comments and immutability',
      children: [
        it({
          name: 'preserves the target value comment',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('{ "a": 1 // keep\n}',), },);
            const next = jsoncSet({ state, path: ['a',], value: 2, },);
            expect(jsoncGetComment({ state: next, path: ['a',], },),).toEqual({ type: 'inline', text: ' keep', },);
          },
        },),
        it({
          name: 'leaves a replaced value without a comment when it had none',
          fn: async () => {
            const next = jsoncSet({ state: base(), path: ['list',], value: [1,], },);
            expect(jsoncGetComment({ state: next, path: ['list',], },),).toBe(COMMENT_ABSENT,);
          },
        },),
        it({
          name: 'does not mutate the source state',
          fn: async () => {
            const state = base();
            jsoncSet({ state, path: ['list', 0,], value: 99, },);
            expect(jsoncGetValue({ state, path: ['list', 0,], },),).toBe(10,);
          },
        },),
        it({
          name: 'round-trips through the serializer after a set',
          fn: async () => {
            const next = jsoncSet({ state: base(), path: ['list', 1,], value: 99, },);
            const reparsed = parseJsoncEdit({ source: jsoncStringify({ state: next, },) as StringJsonc, },);
            expect(jsoncGetValue({ state: reparsed, path: ['list',], },),).toEqual([10, 99,],);
          },
        },),
      ],
    },),
    describe({
      name: 'delete',
      children: [
        it({
          name: 'deletes a record key',
          fn: async () => {
            const next = jsoncDelete({ state: base(), path: ['nested',], },);
            expect(jsoncHas({ state: next, path: ['nested',], },),).toBe(false,);
            expect(jsoncKeys({ state: next, path: [], },),).toEqual(['list',],);
          },
        },),
        it({
          name: 'deletes an array element and shifts the rest',
          fn: async () => {
            const next = jsoncDelete({ state: base(), path: ['list', 0,], },);
            expect(jsoncGetValue({ state: next, path: ['list',], },),).toEqual([20,],);
          },
        },),
        it({
          name: 'descends into a nested array element to delete it, leaving siblings intact',
          fn: async () => {
            const next = jsoncDelete({ state: matrix(), path: ['m', 0, 0,], },);
            expect(jsoncGetValue({ state: next, path: ['m', 0,], },),).toEqual([2,],);
            expect(jsoncGetValue({ state: next, path: ['m', 1,], },),).toEqual([3, 4,],);
          },
        },),
        it({
          name: 'throws on a numeric segment into a record',
          fn: async () => {
            expect(() => {
              jsoncDelete({ state: base(), path: [0,], },);
            },).toThrow('cannot index',);
          },
        },),
        it({
          name: 'throws on a boundary intermediate array index equal to the length',
          fn: async () => {
            expect(() => {
              jsoncDelete({ state: matrix(), path: ['m', 2, 0,], },);
            },).toThrow('no JSONC node at path',);
          },
        },),
        it({
          name: 'removes all duplicate keys',
          fn: async () => {
            const next = jsoncDelete({ state: dup(), path: ['a',], },);
            expect(jsoncHas({ state: next, path: ['a',], },),).toBe(false,);
          },
        },),
        it({
          name: 'throws on an out-of-range intermediate array index',
          fn: async () => {
            expect(() => {
              jsoncDelete({ state: matrix(), path: ['m', 5, 0,], },);
            },).toThrow('no JSONC node at path',);
          },
        },),
        it({
          name: 'throws on a negative intermediate array index',
          fn: async () => {
            expect(() => {
              jsoncDelete({ state: matrix(), path: ['m', -1, 0,], },);
            },).toThrow('no JSONC node at path',);
          },
        },),
        it({
          name: 'throws on a fractional array index',
          fn: async () => {
            expect(() => {
              jsoncDelete({ state: base(), path: ['list', HALF_INDEX,], },);
            },).toThrow('no JSONC node at path',);
          },
        },),
        it({
          name: 'throws when an intermediate key is missing',
          fn: async () => {
            expect(() => {
              jsoncDelete({ state: base(), path: ['missing', 'x',], },);
            },).toThrow('no JSONC node at path',);
          },
        },),
        it({
          name: 'throws on the empty document-root path',
          fn: async () => {
            expect(() => {
              jsoncDelete({ state: base(), path: [], },);
            },).toThrow('document root',);
          },
        },),
        it({
          name: 'throws when indexing a scalar',
          fn: async () => {
            expect(() => {
              jsoncDelete({ state: dup(), path: ['a', 'b',], },);
            },).toThrow('cannot index',);
          },
        },),
      ],
    },),
    describe({
      name: 'navigation guards',
      children: [
        it({
          name: 'a string segment into an array does not resolve',
          fn: async () => {
            expect(jsoncHas({ state: base(), path: ['list', 'x',], },),).toBe(false,);
          },
        },),
        it({
          name: 'keys on a non-record target throws',
          fn: async () => {
            expect(() => {
              jsoncKeys({ state: base(), path: ['list',], },);
            },).toThrow('not an object',);
          },
        },),
      ],
    },),
  ],
},);
