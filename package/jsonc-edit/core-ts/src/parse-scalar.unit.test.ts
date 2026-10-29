/**
 Unit tests for scalar parsing: keyword, string, and number productions, and
 the error raised when no scalar value starts at the offset (including the
 end-of-input marker).
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseScalar, } from '../dist/final/neutral/index.mjs';

await describe({
  name: parseScalar.name,
  children: [
    it({
      name: 'parses the true, false, and null keywords with their end offsets',
      fn: async () => {
        expect(parseScalar({ source: 'true', index: 0, },),).toEqual({ node: { kind: 'boolean', value: true, }, end: 4, },);
        expect(parseScalar({ source: 'false]', index: 0, },),).toEqual({ node: { kind: 'boolean', value: false, }, end: 5, },);
        expect(parseScalar({ source: 'null,', index: 0, },),).toEqual({ node: { kind: 'null', }, end: 4, },);
      },
    },),
    it({
      name: 'parses a string scalar, keeping the raw slice',
      fn: async () => {
        expect(parseScalar({ source: '"a"', index: 0, },),).toEqual({
          node: { kind: 'string', value: 'a', raw: '"a"', },
          end: 3,
        },);
      },
    },),
    it({
      name: 'parses a negative number and a bare digit',
      fn: async () => {
        expect(parseScalar({ source: '-5,', index: 0, },),).toEqual({
          node: { kind: 'number', value: -5, raw: '-5', },
          end: 2,
        },);
        expect(parseScalar({ source: '7]', index: 0, },),).toEqual({
          node: { kind: 'number', value: 7, raw: '7', },
          end: 1,
        },);
      },
    },),
    it({
      name: 'throws on a character that opens no scalar',
      fn: async () => {
        expect(() => {
          parseScalar({ source: '}', index: 0, },);
        },).toThrow('unexpected character',);
      },
    },),
    it({
      name: 'names the end-of-input marker when the offset is past the source',
      fn: async () => {
        expect(() => {
          parseScalar({ source: '', index: 0, },);
        },).toThrow('<eof>',);
      },
    },),
  ],
},);
