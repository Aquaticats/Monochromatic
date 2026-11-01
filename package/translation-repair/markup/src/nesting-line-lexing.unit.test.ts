/**
 Tests for the line readers the nesting bound counts with: the blanks opening
 a line, the container markers after them, and whether a line is a
 thematic break.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  blanksEnd,
  containerPrefixOf,
  indentationOf,
  isRuleLine,
} from '../dist/final/node/index.mjs';

//region Nesting line lexing tests

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: blanksEnd.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FINDS the first character that is no space or tab, from a position, and the end of a line of blanks',
          fn: async () => {
            expect(blanksEnd({ line: ' \t cat', from: 0, },),).toBe(3,);
            expect(blanksEnd({ line: ' \t cat', from: 3, },),).toBe(3,);
            expect(blanksEnd({ line: ' \t ', from: 0, },),).toBe(3,);
            expect(blanksEnd({ line: '', from: 0, },),).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: indentationOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS a space as one column and a tab as four, and says where the indentation ends',
          fn: async () => {
            expect(indentationOf({ line: '    cat', },),).toEqual({ columns: 4, end: 4, },);
            expect(indentationOf({ line: '\t cat', },),).toEqual({ columns: 5, end: 2, },);
            expect(indentationOf({ line: 'cat', },),).toEqual({ columns: 0, end: 0, },);
          },
        },),
      ],
    },),

    describe({
      name: containerPrefixOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS quotation marks, bullet markers, ordered markers and footnote labels as markers, one each',
          fn: async () => {
            expect(containerPrefixOf({ line: '> > cat', start: 0, },),).toEqual({
              end: 3,
              count: 2,
              beyondColumn: 0,
            },);
            expect(containerPrefixOf({ line: '1. - cat', start: 0, },),).toEqual({
              end: 4,
              count: 2,
              beyondColumn: 0,
            },);
            expect(containerPrefixOf({ line: '[^1]: cat', start: 0, },),).toEqual({
              end: 5,
              count: 1,
              beyondColumn: 0,
            },);
            expect(containerPrefixOf({ line: '+ cat', start: 0, },),).toEqual({
              end: 1,
              count: 1,
              beyondColumn: 0,
            },);
          },
        },),
        it({
          name: 'READS a bullet that ends its line as a marker and a mark with no blank after it as none',
          fn: async () => {
            expect(containerPrefixOf({ line: '-', start: 0, },),).toEqual({
              end: 1,
              count: 1,
              beyondColumn: 0,
            },);
            expect(containerPrefixOf({ line: '-not a list', start: 0, },),).toEqual({
              end: 0,
              count: 0,
              beyondColumn: 0,
            },);
            expect(containerPrefixOf({ line: '2024.cat', start: 0, },),).toEqual({
              end: 0,
              count: 0,
              beyondColumn: 0,
            },);
            expect(containerPrefixOf({ line: '[^1 cat]: no', start: 0, },),).toEqual({
              end: 0,
              count: 0,
              beyondColumn: 0,
            },);
          },
        },),
        it({
          name: 'READS a footnote label the line ends inside as no marker, since no bracket closes it, and keeps the '
            + 'markers before it',
          fn: async () => {
            expect(containerPrefixOf({ line: '[^cat', start: 0, },),).toEqual({
              end: 0,
              count: 0,
              beyondColumn: 0,
            },);
            expect(containerPrefixOf({ line: '> [^cat', start: 0, },),).toEqual({
              end: 1,
              count: 1,
              beyondColumn: 0,
            },);
          },
        },),
        it({
          name: 'NAMES the one-based column of the 257th marker as the first past the bound, and counts them all',
          fn: async () => {
            expect(containerPrefixOf({ line: `${'>'.repeat(300,)} cat`, start: 0, },),).toEqual({
              end: 300,
              count: 300,
              beyondColumn: 257,
            },);
          },
        },),
        it({
          name: 'STARTS reading at the position it is given, after the indentation',
          fn: async () => {
            expect(containerPrefixOf({ line: '  > cat', start: 2, },),).toEqual({
              end: 3,
              count: 1,
              beyondColumn: 0,
            },);
          },
        },),
      ],
    },),

    describe({
      name: isRuleLine.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS three or more of one of stars, underscores and hyphens, with only spaces and tabs between, as a rule',
          fn: async () => {
            expect(isRuleLine({ line: '* * *', },),).toBe(true,);
            expect(isRuleLine({ line: '___', },),).toBe(true,);
            expect(isRuleLine({ line: '- - -\t', },),).toBe(true,);
            expect(isRuleLine({ line: '*'.repeat(2_000,), },),).toBe(true,);
          },
        },),
        it({
          name: 'READS a mix of those characters, fewer than three of them, and any other character as no rule',
          fn: async () => {
            expect(isRuleLine({ line: '*-*-*-', },),).toBe(false,);
            expect(isRuleLine({ line: '*_*_*_', },),).toBe(false,);
            expect(isRuleLine({ line: '**', },),).toBe(false,);
            expect(isRuleLine({ line: '', },),).toBe(false,);
            expect(isRuleLine({ line: 'cat', },),).toBe(false,);
            expect(isRuleLine({ line: '**bold**', },),).toBe(false,);
            expect(isRuleLine({ line: '***~', },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);

//endregion Nesting line lexing tests
