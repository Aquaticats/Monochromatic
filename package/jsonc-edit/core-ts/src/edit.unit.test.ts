/**
 Tests for the edit API: reading, setting (including creating keys and
 appending), deleting, key listing, immutability, and type errors.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type StringJsonc,
  jsoncDelete,
  jsoncGetValue,
  jsoncHas,
  jsoncKeys,
  jsoncSet,
  parseJsoncEdit,
} from '../dist/final/neutral/index.mjs';

const asJsonc = (source: string,): StringJsonc => source as StringJsonc;

const fixture = (): ReturnType<typeof parseJsoncEdit> =>
  parseJsoncEdit({ source: asJsonc('{\n  "a": 1, // c\n  "nested": { "b": 2 },\n  "list": [10, 20]\n}',), },);

await describe({
  name: 'edit API',
  children: [
    describe({
      name: 'read',
      children: [
        it({
          name: 'jsoncGetValue reads a nested value',
          fn: async () => {
            expect(jsoncGetValue({ state: fixture(), path: ['nested', 'b',], },),).toBe(2,);
          },
        },),
        it({
          name: 'jsoncGetValue reads an array element',
          fn: async () => {
            expect(jsoncGetValue({ state: fixture(), path: ['list', 1,], },),).toBe(20,);
          },
        },),
        it({
          name: 'jsoncHas distinguishes present from absent',
          fn: async () => {
            const state = fixture();
            expect(jsoncHas({ state, path: ['a',], },),).toBe(true,);
            expect(jsoncHas({ state, path: ['missing',], },),).toBe(false,);
          },
        },),
        it({
          name: 'jsoncKeys lists record keys in order',
          fn: async () => {
            expect(jsoncKeys({ state: fixture(), path: [], },),).toEqual(['a', 'nested', 'list',],);
          },
        },),
        it({
          name: 'jsoncGetValue throws on a missing path',
          fn: async () => {
            expect(() => {
              jsoncGetValue({ state: fixture(), path: ['nope',], },);
            },).toThrow('no JSONC node at path',);
          },
        },),
      ],
    },),
    describe({
      name: 'set',
      children: [
        it({
          name: 'sets an existing value and leaves the original untouched',
          fn: async () => {
            const state = fixture();
            const next = jsoncSet({ state, path: ['a',], value: 99, },);
            expect(jsoncGetValue({ state: next, path: ['a',], },),).toBe(99,);
            expect(jsoncGetValue({ state, path: ['a',], },),).toBe(1,);
          },
        },),
        it({
          name: 'creates a new trailing key',
          fn: async () => {
            const next = jsoncSet({ state: fixture(), path: ['fresh',], value: { x: true, }, },);
            expect(jsoncGetValue({ state: next, path: ['fresh', 'x',], },),).toBe(true,);
          },
        },),
        it({
          name: 'appends to an array at the length index',
          fn: async () => {
            const next = jsoncSet({ state: fixture(), path: ['list', 2,], value: 30, },);
            expect(jsoncGetValue({ state: next, path: ['list',], },),).toEqual([10, 20, 30,],);
          },
        },),
        it({
          name: 'throws when indexing a scalar',
          fn: async () => {
            expect(() => {
              jsoncSet({ state: fixture(), path: ['a', 'b',], value: 1, },);
            },).toThrow('cannot index',);
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
            const next = jsoncDelete({ state: fixture(), path: ['a',], },);
            expect(jsoncHas({ state: next, path: ['a',], },),).toBe(false,);
            expect(jsoncKeys({ state: next, path: [], },),).toEqual(['nested', 'list',],);
          },
        },),
        it({
          name: 'deletes an array element and shifts the rest',
          fn: async () => {
            const next = jsoncDelete({ state: fixture(), path: ['list', 0,], },);
            expect(jsoncGetValue({ state: next, path: ['list',], },),).toEqual([20,],);
          },
        },),
        it({
          name: 'throws on deleting the document root',
          fn: async () => {
            expect(() => {
              jsoncDelete({ state: fixture(), path: [], },);
            },).toThrow('document root',);
          },
        },),
      ],
    },),
  ],
},);
