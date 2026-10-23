/**
 Tests for the arithmetic on stretches of a text that the readers of a parse share.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { parseMarkdownMathBody, } from '@monochromatic-dev/translation-repair-document-structure';

import {
  closingBracket,
  indicesOf,
  originalStretches,
  rangeOf,
  secondLabel,
  startsAny,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'source ships ranges',
  children: [
    describe({
      name: rangeOf.name,
      children: [
        it({
          name: 'GIVES THE OFFSETS OF A NODE and refuses a node with no position',
          fn: async () => {
            /**
             Text with a paragraph after a heading.
             */
            const source = '# 猫\n\n睡觉\n';
            const [, paragraph,] = parseMarkdownMathBody({ body: source, },).children;
            if (paragraph === undefined)
              throw new Error('the text holds no second block',);
            expect(rangeOf({ node: paragraph, },),).toEqual({
              start: 5,
              end: 7,
            },);
            expect(caught(function readUnpositioned(): void {
              rangeOf({
                node: {
                  type: 'text',
                  value: 'x',
                },
              },);
            },),).toBeInstanceOf(Error,);
          },
        },),
      ],
    },),

    describe({
      name: startsAny.name,
      children: [
        it({
          name: 'TELLS WHETHER ONE OF THE STRINGS STARTS AT THE INDEX',
          fn: async () => {
            expect(startsAny({
              text: '猫 ```ts',
              at: 2,
              candidates: ['```', '~~~',],
            },),).toBe(true,);
            expect(startsAny({
              text: '猫 ```ts',
              at: 1,
              candidates: ['```', '~~~',],
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: closingBracket.name,
      children: [
        it({
          name: 'FINDS THE BRACKET THAT CLOSES A LABEL, stepping over an escaped bracket and a nested pair, and '
            + 'gives the limit when none closes',
          fn: async () => {
            expect(closingBracket({
              source: '![a [b]](u)',
              from: 2,
              limit: 11,
            },),).toBe(7,);
            expect(closingBracket({
              source: String.raw`[a \] b](u)`,
              from: 1,
              limit: 11,
            },),).toBe(7,);
            expect(closingBracket({
              source: '[a b',
              from: 1,
              limit: 4,
            },),).toBe(4,);
          },
        },),
      ],
    },),

    describe({
      name: secondLabel.name,
      children: [
        it({
          name: 'GIVES THE LABEL BETWEEN THE SECOND BRACKETS OF A FULL REFERENCE',
          fn: async () => {
            expect(secondLabel({
              source: '[x][a {b]',
              range: {
                start: 0,
                end: 9,
              },
              textFrom: 1,
            },),).toEqual({
              start: 4,
              end: 8,
            },);
          },
        },),
      ],
    },),

    describe({
      name: indicesOf.name,
      children: [
        it({
          name: 'LISTS EVERY INDEX OF A STRETCH, none for an empty one',
          fn: async () => {
            expect(indicesOf({
              range: {
                start: 3,
                end: 6,
              },
            },),).toEqual([3, 4, 5,],);
            expect(indicesOf({
              range: {
                start: 3,
                end: 3,
              },
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: originalStretches.name,
      children: [
        it({
          name: 'GIVES ONE STRETCH FOR EACH UNBROKEN RUN OF THE ORIGINAL\'S CHARACTERS and refuses an index with '
            + 'no origin',
          fn: async () => {
            expect(originalStretches({
              range: {
                start: 0,
                end: 4,
              },
              indices: [4, 5, 9, 10,],
            },),).toEqual([
              {
                start: 4,
                end: 6,
              },
              {
                start: 9,
                end: 11,
              },
            ],);
            expect(caught(function readPastTheEnd(): void {
              originalStretches({
                range: {
                  start: 0,
                  end: 2,
                },
                indices: [4,],
              },);
            },),).toBeInstanceOf(Error,);
          },
        },),
      ],
    },),
  ],
},);
