/**
 Unit tests for trivia handling: skipping leading whitespace and comments,
 capturing trailing same-line comments and the separating comma, and merging
 leading and trailing comments onto a node.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type JsoncValue,
  appendComments,
  captureTrailing,
  prependComments,
  skipTrivia,
} from '../dist/final/neutral/index.mjs';

const nul = (): JsoncValue => ({ kind: 'null', });

const nulOld = (): JsoncValue => ({ kind: 'null', comment: { type: 'inline', text: 'old', }, });

await describe({
  name: 'parse-trivia',
  children: [
    describe({
      name: skipTrivia.name,
      children: [
        it({
          name: 'collects line and block comments across whitespace, in order',
          fn: async () => {
            expect(skipTrivia({ source: '  // a\n  /* b */ {', index: 0, },),).toEqual({
              comments: [
                { type: 'inline', text: ' a', },
                { type: 'block', text: ' b ', },
              ],
              end: 17,
            },);
          },
        },),
        it({
          name: 'stops at the first significant non-comment character',
          fn: async () => {
            expect(skipTrivia({ source: 'true', index: 0, },),).toEqual({ comments: [], end: 0, },);
          },
        },),
        it({
          name: 'returns end of input when only whitespace remains',
          fn: async () => {
            expect(skipTrivia({ source: '{  \n  ', index: 1, },),).toEqual({ comments: [], end: 6, },);
          },
        },),
      ],
    },),
    describe({
      name: captureTrailing.name,
      children: [
        it({
          name: 'captures a comma then a same-line line comment',
          fn: async () => {
            expect(captureTrailing({ source: ', // note\n', index: 0, },),).toEqual({
              comments: [{ type: 'inline', text: ' note', },],
              commaSeen: true,
              end: 9,
            },);
          },
        },),
        it({
          name: 'skips inline whitespace, including tab and carriage return, before the comma',
          fn: async () => {
            expect(captureTrailing({ source: ' \t\r,x', index: 0, },),).toEqual({
              comments: [],
              commaSeen: true,
              end: 4,
            },);
          },
        },),
        it({
          name: 'captures a trailing block comment before the comma',
          fn: async () => {
            expect(captureTrailing({ source: '/* t */,', index: 0, },),).toEqual({
              comments: [{ type: 'block', text: ' t ', },],
              commaSeen: true,
              end: 8,
            },);
          },
        },),
        it({
          name: 'stops at the next value without consuming it or seeing a comma',
          fn: async () => {
            expect(captureTrailing({ source: '2', index: 0, },),).toEqual({
              comments: [],
              commaSeen: false,
              end: 0,
            },);
          },
        },),
        it({
          name: 'stops at a newline, leaving the next line uncaptured',
          fn: async () => {
            expect(captureTrailing({ source: ', // a\n// b', index: 0, },),).toEqual({
              comments: [{ type: 'inline', text: ' a', },],
              commaSeen: true,
              end: 6,
            },);
          },
        },),
      ],
    },),
    describe({
      name: prependComments.name,
      children: [
        it({
          name: 'returns the node unchanged when there are no comments',
          fn: async () => {
            expect(prependComments({ node: nul(), comments: [], },),).toEqual({ kind: 'null', },);
          },
        },),
        it({
          name: 'attaches a single leading comment when the node has none',
          fn: async () => {
            expect(prependComments({
              node: nul(),
              comments: [{ type: 'inline', text: 'x', },],
            },),).toEqual({ kind: 'null', comment: { type: 'inline', text: 'x', }, },);
          },
        },),
        it({
          name: 'merges ahead of an existing comment, leading first',
          fn: async () => {
            expect(prependComments({
              node: nulOld(),
              comments: [{ type: 'inline', text: 'new', },],
            },),).toEqual({ kind: 'null', comment: { type: 'inline', text: 'new\nold', }, },);
          },
        },),
      ],
    },),
    describe({
      name: appendComments.name,
      children: [
        it({
          name: 'returns the node unchanged when there are no comments',
          fn: async () => {
            expect(appendComments({ node: nul(), comments: [], },),).toEqual({ kind: 'null', },);
          },
        },),
        it({
          name: 'attaches a single trailing comment when the node has none',
          fn: async () => {
            expect(appendComments({
              node: nul(),
              comments: [{ type: 'block', text: 'x', },],
            },),).toEqual({ kind: 'null', comment: { type: 'block', text: 'x', }, },);
          },
        },),
        it({
          name: 'merges after an existing comment, existing first',
          fn: async () => {
            expect(appendComments({
              node: nulOld(),
              comments: [{ type: 'inline', text: 'new', },],
            },),).toEqual({ kind: 'null', comment: { type: 'inline', text: 'old\nnew', }, },);
          },
        },),
      ],
    },),
    describe({
      name: 'lone slashes and stars are not comments',
      children: [
        it({
          name: 'skipTrivia stops at a slash or star that opens no comment',
          fn: async () => {
            expect(skipTrivia({ source: '/y', index: 0, },),).toEqual({ comments: [], end: 0, },);
            expect(skipTrivia({ source: 'y/', index: 0, },),).toEqual({ comments: [], end: 0, },);
            expect(skipTrivia({ source: 'y*', index: 0, },),).toEqual({ comments: [], end: 0, },);
          },
        },),
        it({
          name: 'captureTrailing stops at a slash or star that opens no comment',
          fn: async () => {
            expect(captureTrailing({ source: '/y', index: 0, },),).toEqual({
              comments: [],
              commaSeen: false,
              end: 0,
            },);
            expect(captureTrailing({ source: 'y/', index: 0, },),).toEqual({
              comments: [],
              commaSeen: false,
              end: 0,
            },);
            expect(captureTrailing({ source: 'y*', index: 0, },),).toEqual({
              comments: [],
              commaSeen: false,
              end: 0,
            },);
          },
        },),
      ],
    },),
  ],
},);
