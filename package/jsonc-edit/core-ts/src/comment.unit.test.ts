/**
 Tests for the comment-as-data API: reading and setting value and key comments,
 the absence sentinel, comment preservation across edits, and mergeComments.
 
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
  jsoncSet,
  jsoncSetComment,
  jsoncSetKeyComment,
  mergeComments,
  parseJsoncEdit,
} from '../dist/final/neutral/index.mjs';

const asJsonc = (source: string,): StringJsonc => source as StringJsonc;

const fixture = (): ReturnType<typeof parseJsoncEdit> =>
  parseJsoncEdit({ source: asJsonc('{\n  // k\n  "a": 1 // v\n}',), },);

await describe({
  name: 'comment-as-data API',
  children: [
    describe({
      name: 'read',
      children: [
        it({
          name: 'reads the value comment and the key comment separately',
          fn: async () => {
            const state = fixture();
            expect(jsoncGetComment({ state, path: ['a',], },),).toEqual({ type: 'inline', text: ' v', },);
            expect(jsoncGetKeyComment({ state, path: ['a',], },),).toEqual({ type: 'inline', text: ' k', },);
          },
        },),
        it({
          name: 'returns the absence sentinel when there is no comment',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('{ "a": 1, // x\n  "b": 2 }',), },);
            expect(jsoncGetComment({ state, path: ['b',], },),).toBe(COMMENT_ABSENT,);
            expect(jsoncGetKeyComment({ state, path: ['b',], },),).toBe(COMMENT_ABSENT,);
          },
        },),
      ],
    },),
    describe({
      name: 'write',
      children: [
        it({
          name: 'sets a value comment, leaving the original untouched',
          fn: async () => {
            const state = fixture();
            const next = jsoncSetComment({ state, path: ['a',], comment: { type: 'block', text: ' new ', }, },);
            expect(jsoncGetComment({ state: next, path: ['a',], },),).toEqual({ type: 'block', text: ' new ', },);
            expect(jsoncGetComment({ state, path: ['a',], },),).toEqual({ type: 'inline', text: ' v', },);
          },
        },),
        it({
          name: 'sets a key comment',
          fn: async () => {
            const next = jsoncSetKeyComment({
              state: fixture(),
              path: ['a',],
              comment: { type: 'inline', text: ' renamed', },
            },);
            expect(jsoncGetKeyComment({ state: next, path: ['a',], },),).toEqual({ type: 'inline', text: ' renamed', },);
          },
        },),
        it({
          name: 'setting a value preserves its comment',
          fn: async () => {
            const next = jsoncSet({ state: fixture(), path: ['a',], value: 2, },);
            expect(jsoncGetComment({ state: next, path: ['a',], },),).toEqual({ type: 'inline', text: ' v', },);
          },
        },),
      ],
    },),
    describe({
      name: 'errors',
      children: [
        it({
          name: 'key comment on an empty path throws',
          fn: async () => {
            expect(() => {
              jsoncGetKeyComment({ state: fixture(), path: [], },);
            },).toThrow('empty path',);
          },
        },),
        it({
          name: 'key comment on an array-index segment throws',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('{ "list": [1, 2,] }',), },);
            expect(() => {
              jsoncGetKeyComment({ state, path: ['list', 0,], },);
            },).toThrow('not a key',);
          },
        },),
      ],
    },),
    describe({
      name: mergeComments.name,
      children: [
        it({
          name: 'merges same-type comments keeping the type',
          fn: async () => {
            expect(
              mergeComments({ first: { type: 'inline', text: 'a', }, second: { type: 'inline', text: 'b', }, },),
            ).toEqual({ type: 'inline', text: 'a\nb', },);
          },
        },),
        it({
          name: 'merges differing types into mixed',
          fn: async () => {
            expect(
              mergeComments({ first: { type: 'block', text: 'a', }, second: { type: 'inline', text: 'b', }, },).type,
            ).toBe('mixed',);
          },
        },),
      ],
    },),
  ],
},);
