/**
 Unit tests for the emit helpers: scalar text (raw reuse versus re-encoding),
 re-indented fast-path JSON, single-line detection, trailing comment text, and
 leading comment lines (block kept as a block, or folded to safe `//` lines).
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  emitPlainJson,
  emitScalar,
  isSingleLineComment,
  leadingComment,
  trailingComment,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: 'emit',
  children: [
    describe({
      name: emitScalar.name,
      children: [
        it({
          name: 'reuses the raw token when present and re-encodes when absent',
          fn: async () => {
            expect(emitScalar({ node: { kind: 'number', value: 1, raw: '1.0', }, },),).toBe('1.0',);
            expect(emitScalar({ node: { kind: 'number', value: 2, }, },),).toBe('2',);
            expect(emitScalar({ node: { kind: 'string', value: 'a', raw: '"a"', }, },),).toBe('"a"',);
            expect(emitScalar({ node: { kind: 'string', value: 'a"b', }, },),).toBe(String.raw`"a\"b"`,);
          },
        },),
        it({
          name: 'emits canonical spellings for booleans and null',
          fn: async () => {
            expect(emitScalar({ node: { kind: 'boolean', value: true, }, },),).toBe('true',);
            expect(emitScalar({ node: { kind: 'boolean', value: false, }, },),).toBe('false',);
            expect(emitScalar({ node: { kind: 'null', }, },),).toBe('null',);
          },
        },),
      ],
    },),
    describe({
      name: emitPlainJson.name,
      children: [
        it({
          name: 'emits canonical 2-space JSON at the top level',
          fn: async () => {
            expect(emitPlainJson({ json: { a: 1, }, indent: 0, },),).toBe('{\n  "a": 1\n}',);
          },
        },),
        it({
          name: 'reindents continuation lines for a nested leaf',
          fn: async () => {
            expect(emitPlainJson({ json: { a: 1, }, indent: 1, },),).toBe('{\n    "a": 1\n  }',);
          },
        },),
      ],
    },),
    describe({
      name: isSingleLineComment.name,
      children: [
        it({
          name: 'is true without a newline and false with one',
          fn: async () => {
            expect(isSingleLineComment({ type: 'inline', text: ' a', },),).toBe(true,);
            expect(isSingleLineComment({ type: 'block', text: 'a\nb', },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: trailingComment.name,
      children: [
        it({
          name: 'emits a single-line comment as a line comment',
          fn: async () => {
            expect(trailingComment({ comment: { type: 'inline', text: ' note', }, },),).toBe('// note',);
          },
        },),
      ],
    },),
    describe({
      name: leadingComment.name,
      children: [
        it({
          name: 'keeps a block comment as a block when its body has no close delimiter',
          fn: async () => {
            expect(leadingComment({ comment: { type: 'block', text: ' k ', }, indent: 1, },),).toBe('  /* k */\n',);
          },
        },),
        it({
          name: 'folds a block comment whose body contains a close delimiter to line comments',
          fn: async () => {
            expect(leadingComment({ comment: { type: 'block', text: ' a */ b ', }, indent: 0, },),).toBe(
              '// a */ b \n',
            );
          },
        },),
        it({
          name: 'emits a multi-line comment as one line comment per body line, indented',
          fn: async () => {
            expect(leadingComment({ comment: { type: 'inline', text: ' x\n y', }, indent: 2, },),).toBe(
              '    // x\n    // y\n',
            );
          },
        },),
      ],
    },),
  ],
},);
