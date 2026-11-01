/**
 Unit tests for the comment-transform branches: descending into array elements
 and the right record entry, missing-path and type errors, duplicate-key
 last-wins for both value and key comments, and targeting one key among many.
 
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
  jsoncGetComment,
  jsoncGetKeyComment,
  jsoncSetComment,
  jsoncSetKeyComment,
  parseJsoncEdit,
} from '../dist/final/neutral/index.mjs';

const asJsonc = (source: string,): StringJsonc => source as StringJsonc;

const twoKeys = (): ReturnType<typeof parseJsoncEdit> =>
  parseJsoncEdit({ source: asJsonc('{ "a": 1, "b": 2 } // c',), },);

const dup = (): ReturnType<typeof parseJsoncEdit> =>
  parseJsoncEdit({ source: asJsonc('{ "a": 1, "a": 2 } // c',), },);

const block = { type: 'block', text: ' e ', } as const;

/**
 Fractional segment used to verify array indexes must be integers.
 */
const HALF_INDEX = 1 / 2;

await describe({
  name: 'edit-comment branches',
  children: [
    describe({
      name: 'value comments',
      children: [
        it({
          name: 'sets a comment on a specific array element only',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('{ "list": [1, 2] } // c',), },);
            const next = jsoncSetComment({ state, path: ['list', 0,], comment: block, },);
            expect(jsoncGetComment({ state: next, path: ['list', 0,], },),).toEqual(block,);
            expect(jsoncGetComment({ state: next, path: ['list', 1,], },),).toBe(COMMENT_ABSENT,);
          },
        },),
        it({
          name: 'sets a comment on one key among many, leaving the others bare',
          fn: async () => {
            const next = jsoncSetComment({ state: twoKeys(), path: ['a',], comment: block, },);
            expect(jsoncGetComment({ state: next, path: ['a',], },),).toEqual(block,);
            expect(jsoncGetComment({ state: next, path: ['b',], },),).toBe(COMMENT_ABSENT,);
          },
        },),
        it({
          name: 'targets the last of duplicate keys',
          fn: async () => {
            const next = jsoncSetComment({ state: dup(), path: ['a',], comment: block, },);
            expect(jsoncGetComment({ state: next, path: ['a',], },),).toEqual(block,);
          },
        },),
        it({
          name: 'throws when the path does not resolve',
          fn: async () => {
            expect(() => {
              jsoncSetComment({ state: twoKeys(), path: ['nope',], comment: block, },);
            },).toThrow('no JSONC node at path',);
          },
        },),
        it({
          name: 'getting a comment at a missing path throws',
          fn: async () => {
            expect(() => {
              jsoncGetComment({ state: twoKeys(), path: ['nope',], },);
            },).toThrow('no JSONC node at path',);
          },
        },),
        it({
          name: 'throws when a segment cannot index the node kind',
          fn: async () => {
            expect(() => {
              jsoncSetComment({ state: twoKeys(), path: ['a', 'b',], comment: block, },);
            },).toThrow('cannot index',);
          },
        },),
        it({
          name: 'throws on a numeric segment into a record',
          fn: async () => {
            expect(() => {
              jsoncSetComment({ state: twoKeys(), path: [0,], comment: block, },);
            },).toThrow('cannot index',);
          },
        },),
        it({
          name: 'throws on a string segment into an array',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('{ "list": [1, 2] } // c',), },);
            expect(() => {
              jsoncSetComment({ state, path: ['list', 'x',], comment: block, },);
            },).toThrow('cannot index',);
          },
        },),
        it({
          name: 'throws on an out-of-range, boundary, negative, or fractional array index',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('{ "list": [1, 2] } // c',), },);
            for (const index of [5, 2, -1, HALF_INDEX,]) {
              expect(() => {
                jsoncSetComment({ state, path: ['list', index,], comment: block, },);
              },).toThrow('no JSONC node at path',);
            }
          },
        },),
      ],
    },),
    describe({
      name: 'key comments',
      children: [
        it({
          name: 'sets a key comment on one key among many, leaving the others bare',
          fn: async () => {
            const next = jsoncSetKeyComment({
              state: twoKeys(),
              path: ['a',],
              comment: { type: 'inline', text: ' ka', },
            },);
            expect(jsoncGetKeyComment({ state: next, path: ['a',], },),).toEqual({ type: 'inline', text: ' ka', },);
            expect(jsoncGetKeyComment({ state: next, path: ['b',], },),).toBe(COMMENT_ABSENT,);
          },
        },),
        it({
          name: 'reads the last of duplicate keys',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('{\n  "a": 1,\n  // klast\n  "a": 2\n}',), },);
            expect(jsoncGetKeyComment({ state, path: ['a',], },),).toEqual({ type: 'inline', text: ' klast', },);
          },
        },),
        it({
          name: 'targets the last of duplicate keys when setting',
          fn: async () => {
            const next = jsoncSetKeyComment({
              state: dup(),
              path: ['a',],
              comment: { type: 'inline', text: ' last', },
            },);
            expect(jsoncGetKeyComment({ state: next, path: ['a',], },),).toEqual({ type: 'inline', text: ' last', },);
          },
        },),
        it({
          name: 'throws when the key does not resolve',
          fn: async () => {
            expect(() => {
              jsoncSetKeyComment({ state: twoKeys(), path: ['missing',], comment: { type: 'inline', text: ' x', }, },);
            },).toThrow('no JSONC node at path',);
          },
        },),
      ],
    },),
  ],
},);
