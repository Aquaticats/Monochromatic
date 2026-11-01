/**
 Tests for the JSONC parser: structured parsing of clean and commented input,
 comment attachment to keys and values, merged stacked comments, trailing-comma
 tolerance, separators that follow trivia on a later line, and error cases.
 
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
  jsoncGetValue,
  jsoncStringify,
  parseJsonc,
  parseJsoncEdit,
} from '../dist/final/neutral/index.mjs';

const asJsonc = (source: string,): StringJsonc => source as StringJsonc;

const nestArrays = (depth: number,): string => `${'['.repeat(depth,)}${']'.repeat(depth,)} // x`;

const nestObjects = (depth: number,): string => `${'{"a":'.repeat(depth,)}1${'}'.repeat(depth,)} // x`;

/**
 Builds a comment-free nested array document, so the depth limit is exercised on
 input that a native-JSON shortcut would also accept.
 
 @param depth - Number of nested containers.
 
 @returns Clean JSON array source.
 */
const cleanNestArrays = (depth: number,): string => `${'['.repeat(depth,)}0${']'.repeat(depth,)}`;

await describe({
  name: parseJsonc.name,
  children: [
    describe({
      name: 'structured parsing',
      children: [
        it({
          name: 'clean object parses to a structured record',
          fn: async () => {
            const result = parseJsonc({ source: asJsonc('{"a":1,"b":[1,2,3]}',), },);
            expect(result.kind,).toBe('record',);
            if (result.kind === 'record') {
              expect(result.entries.length,).toBe(2,);
              expect(jsoncGetValue({ state: { root: result, }, path: ['b', 1,], },),).toBe(2,);
            }
          },
        },),
        it({
          name: 'clean array parses to a structured array',
          fn: async () => {
            expect(parseJsonc({ source: asJsonc('[1,2,3]',), },).kind,).toBe('array',);
          },
        },),
        it({
          name: 'an unedited number keeps its source spelling on output',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('{"n":1e0,"m":1.500,"z":-0}',), },);
            const text = jsoncStringify({ state, },);
            expect(text.includes('1e0'),).toBe(true,);
            expect(text.includes('1.500'),).toBe(true,);
            expect(text.includes('-0'),).toBe(true,);
          },
        },),
        it({
          name: 'a clean document is subject to the same nesting limit',
          fn: async () => {
            expect(parseJsonc({ source: asJsonc(cleanNestArrays(512,),), },).kind,).toBe('array',);
            expect(() => {
              parseJsonc({ source: asJsonc(cleanNestArrays(513,),), },);
            },).toThrow('nesting too deep',);
          },
        },),
      ],
    },),
    describe({
      name: 'separator after trivia',
      children: [
        it({
          name: 'accepts a comma on a later line in an array',
          fn: async () => {
            const result = parseJsonc({ source: asJsonc('[1\n,2,]',), },);
            expect(result.kind,).toBe('array',);
            if (result.kind === 'array')
              expect(result.elements.length,).toBe(2,);
          },
        },),
        it({
          name: 'accepts a comma on a later line in an object',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('{"a":1\n,"b":2}',), },);
            expect(jsoncGetValue({ state, path: ['b',], },),).toBe(2,);
          },
        },),
        it({
          name: 'gives a comment before a later-line comma to the preceding value',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('{"a":1\n/* why */,"b":2}',), },);
            expect(jsoncGetComment({ state, path: ['a',], },),).toEqual({ type: 'block', text: ' why ', },);
            expect(jsoncGetKeyComment({ state, path: ['b',], },),).toBe(COMMENT_ABSENT,);
          },
        },),
        it({
          name: 'gives a comment after a later-line comma to the following key',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('{"a":1\n, //inline\n"b":2}',), },);
            expect(jsoncGetKeyComment({ state, path: ['b',], },),).toEqual({ type: 'inline', text: 'inline', },);
            expect(jsoncGetComment({ state, path: ['a',], },),).toBe(COMMENT_ABSENT,);
          },
        },),
        it({
          name: 'still rejects a value that follows without a separator',
          fn: async () => {
            expect(() => {
              parseJsonc({ source: asJsonc('[1\n2]',), },);
            },).toThrow('expected , or ] in array',);
            expect(() => {
              parseJsonc({ source: asJsonc('{"a":1\n"b":2}',), },);
            },).toThrow('expected , or } in object',);
          },
        },),
      ],
    },),
    describe({
      name: 'line endings',
      children: [
        it({
          name: 'a carriage return ends a line comment',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('{"a":1,// x\r"b":2}',), },);
            expect(jsoncGetComment({ state, path: ['a',], },),).toEqual({ type: 'inline', text: ' x', },);
            expect(jsoncGetValue({ state, path: ['b',], },),).toBe(2,);
          },
        },),
        it({
          name: 'a CRLF pair leaves no carriage return in the comment body',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('{"a":1,// x\r\n"b":2}',), },);
            expect(jsoncGetComment({ state, path: ['a',], },),).toEqual({ type: 'inline', text: ' x', },);
            expect(jsoncGetValue({ state, path: ['b',], },),).toBe(2,);
          },
        },),
      ],
    },),
    describe({
      name: 'comments',
      children: [
        it({
          name: 'trailing inline comment attaches to the value',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('{\n  "a": 1 // note\n}',), },);
            expect(jsoncGetComment({ state, path: ['a',], },),).toEqual({ type: 'inline', text: ' note', },);
          },
        },),
        it({
          name: 'block comment attaches to the value',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('{\n  "a": 1 /* blk */\n}',), },);
            expect(jsoncGetComment({ state, path: ['a',], },),).toEqual({ type: 'block', text: ' blk ', },);
          },
        },),
        it({
          name: 'leading comment before a key attaches to the key',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('{\n  // server\n  "a": 1\n}',), },);
            expect(jsoncGetKeyComment({ state, path: ['a',], },),).toEqual({ type: 'inline', text: ' server', },);
            expect(jsoncGetComment({ state, path: ['a',], },),).toBe(COMMENT_ABSENT,);
          },
        },),
        it({
          name: 'stacked comments merge into a mixed comment',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('{\n  //region a\n  /* b */\n  "a": 1\n}',), },);
            expect(jsoncGetKeyComment({ state, path: ['a',], },),).toEqual({ type: 'mixed', text: 'region a\n b ', },);
          },
        },),
        it({
          name: 'leading document comment attaches to the root',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('// hi\n{"a":1}',), },);
            expect(jsoncGetComment({ state, path: [], },),).toEqual({ type: 'inline', text: ' hi', },);
          },
        },),
      ],
    },),
    describe({
      name: 'trailing commas',
      children: [
        it({
          name: 'object trailing comma is tolerated',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('{\n  "a": 1, // c\n}',), },);
            expect(jsoncGetValue({ state, path: ['a',], },),).toBe(1,);
          },
        },),
        it({
          name: 'array trailing comma is tolerated',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('[\n  1, // c\n  2,\n]',), },);
            expect(jsoncGetValue({ state, path: [], },),).toEqual([1, 2,],);
          },
        },),
      ],
    },),
    describe({
      name: 'errors',
      children: [
        it({
          name: 'top-level scalar throws',
          fn: async () => {
            expect(() => {
              parseJsonc({ source: asJsonc('42',), },);
            },).toThrow('object or array',);
          },
        },),
        it({
          name: 'unterminated object throws',
          fn: async () => {
            expect(() => {
              parseJsonc({ source: asJsonc('{ // x',), },);
            },).toThrow('unterminated object',);
          },
        },),
        it({
          name: 'trailing content after the top-level value throws',
          fn: async () => {
            expect(() => {
              parseJsonc({ source: asJsonc('{ "a": 1 } // c\n2',), },);
            },).toThrow('trailing content',);
          },
        },),
        it({
          name: 'missing colon throws',
          fn: async () => {
            expect(() => {
              parseJsonc({ source: asJsonc('{ "a" 1, // x\n}',), },);
            },).toThrow('expected :',);
          },
        },),
        it({
          name: 'unterminated block comment throws',
          fn: async () => {
            expect(() => {
              parseJsonc({ source: asJsonc('{ /* open\n"a": 1 }',), },);
            },).toThrow('unterminated block comment',);
          },
        },),
      ],
    },),
    describe({
      name: 'depth guard',
      children: [
        it({
          name: 'parses arrays nested to the limit without throwing',
          fn: async () => {
            expect(() => {
              parseJsonc({ source: asJsonc(nestArrays(513,),), },);
            },).not
              .toThrow();
          },
        },),
        it({
          name: 'throws on arrays nested past the limit',
          fn: async () => {
            expect(() => {
              parseJsonc({ source: asJsonc(nestArrays(514,),), },);
            },).toThrow('nesting too deep',);
          },
        },),
        it({
          name: 'throws on objects nested past the limit',
          fn: async () => {
            expect(() => {
              parseJsonc({ source: asJsonc(nestObjects(514,),), },);
            },).toThrow('nesting too deep',);
          },
        },),
      ],
    },),
    describe({
      name: 'structural errors',
      children: [
        it({
          name: 'unterminated array throws',
          fn: async () => {
            expect(() => {
              parseJsonc({ source: asJsonc('[',), },);
            },).toThrow('unterminated array',);
          },
        },),
        it({
          name: 'a missing comma between array elements throws',
          fn: async () => {
            expect(() => {
              parseJsonc({ source: asJsonc('[1 2]',), },);
            },).toThrow('expected , or ]',);
          },
        },),
        it({
          name: 'a non-string object key throws',
          fn: async () => {
            expect(() => {
              parseJsonc({ source: asJsonc('{ 1: 2 }',), },);
            },).toThrow('expected string key',);
          },
        },),
        it({
          name: 'a missing comma between object entries throws',
          fn: async () => {
            expect(() => {
              parseJsonc({ source: asJsonc('{ "a": 1 "b": 2 }',), },);
            },).toThrow('expected , or }',);
          },
        },),
      ],
    },),
    describe({
      name: 'top-level shape and fast-path',
      children: [
        it({
          name: 'rejects a top-level null, boolean, and string',
          fn: async () => {
            expect(() => {
              parseJsonc({ source: asJsonc('null',), },);
            },).toThrow('object or array',);
            expect(() => {
              parseJsonc({ source: asJsonc('true',), },);
            },).toThrow('object or array',);
            expect(() => {
              parseJsonc({ source: asJsonc('"x"',), },);
            },).toThrow('object or array',);
          },
        },),
        it({
          name: 'a clean document parses structurally',
          fn: async () => {
            expect(parseJsonc({ source: asJsonc('{"a":1}',), },).kind,).toBe('record',);
          },
        },),
        it({
          name: 'a commented document uses the structured parser',
          fn: async () => {
            expect(parseJsonc({ source: asJsonc('{ "a": 1 } // c',), },).kind,).toBe('record',);
          },
        },),
      ],
    },),
    describe({
      name: 'dangling comments before close',
      children: [
        it({
          name: 'an array dangling comment folds onto the last element',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('[\n  1,\n  2,\n  // tail\n]',), },);
            expect(jsoncGetComment({ state, path: [1,], },),).toEqual({ type: 'inline', text: ' tail', },);
            expect(jsoncGetValue({ state, path: [], },),).toEqual([1, 2,],);
          },
        },),
        it({
          name: 'a record dangling comment folds onto the last value',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('{\n  "a": 1,\n  // tail\n}',), },);
            expect(jsoncGetComment({ state, path: ['a',], },),).toEqual({ type: 'inline', text: ' tail', },);
          },
        },),
        it({
          name: 'an empty array dangling comment attaches to the array node',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('[\n  // only\n]',), },);
            expect(jsoncGetComment({ state, path: [], },),).toEqual({ type: 'inline', text: ' only', },);
            expect(jsoncGetValue({ state, path: [], },),).toEqual([],);
          },
        },),
        it({
          name: 'an empty record dangling comment attaches to the record node',
          fn: async () => {
            const state = parseJsoncEdit({ source: asJsonc('{\n  // only\n}',), },);
            expect(jsoncGetComment({ state, path: [], },),).toEqual({ type: 'inline', text: ' only', },);
            expect(jsoncGetValue({ state, path: [], },),).toEqual({},);
          },
        },),
      ],
    },),
  ],
},);
