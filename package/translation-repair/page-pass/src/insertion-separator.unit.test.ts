/**
 Tests for the whitespace assembly owns around an insertion.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  composedInsertion,
  documentLineEnding,
  fragmentBody,
  fragmentCut,
} from '@monochromatic-dev/translation-repair-corpus-source';
import { spliceSlices, } from '../dist/final/node/index.mjs';

/**
 The text `composedInsertion` writes at a boundary, without where the words stand in it.

 @param asked - what the boundary holds and the fragments anchored at it

 @returns Text to write

 @example
 ```ts
 const text = composeInsertion({ fragments: ['The cat naps.',], before: '', after: '', eol: '\n', },);
 ```
 */
function composeInsertion(
  asked: Parameters<typeof composedInsertion>[0],
): string {
  return composedInsertion(asked,).text;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: fragmentBody.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'strips the blank lines around a fragment and keeps its INDENTATION, because a rendering '
            + 'that begins with spaces is inside a list or a quote and cutting them moves it out of the '
            + 'structure it belongs to',
          fn: async () => {
            expect(fragmentBody({ fragment: '\n\n  The cat naps.\n\n', },),).toBe('  The cat naps.',);
            expect(fragmentBody({ fragment: 'The cat naps.', },),).toBe('The cat naps.',);
            expect(fragmentBody({ fragment: '   \n\nThe cat naps.', },),).toBe('The cat naps.',);
            expect(fragmentBody({ fragment: '\n\n\n', },),).toBe('',);
            // TRAILING SPACES GO, including the two that make a Markdown hard
            // break. Safe only because composition joins fragments with a blank
            // line, where a hard break breaks nothing; a join that ever put two
            // fragments on consecutive lines would make this case wrong.
            expect(fragmentBody({ fragment: 'The cat naps.  \n', },),).toBe('The cat naps.',);
            // Inside the fragment the same two spaces survive, since only the ends
            // are touched.
            expect(fragmentBody({ fragment: 'The cat naps.  \nShe purrs.\n', },),)
              .toBe('The cat naps.  \nShe purrs.',);
          },
        },),
        it({
          name: 'keeps the blank line INSIDE a fragment, since a rendering of two paragraphs is two blocks '
            + 'and joining them would make one',
          fn: async () => {
            expect(fragmentBody({ fragment: '\nThe cat naps.\n\nShe purrs.\n', },),)
              .toBe('The cat naps.\n\nShe purrs.',);
          },
        },),
      ],
    },),

    describe({
      name: fragmentCut.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GIVES THE BODY fragmentBody gives and the offset in the fragment where it starts, before the indentation the '
            + 'body keeps, so a reader can find the fragment`s words in the page it lands in',
          fn: async () => {
            expect(fragmentCut({ fragment: '\n\n  The cat naps.\n\n', },),).toEqual({ body: '  The cat naps.', start: 2, },);
            expect(fragmentCut({ fragment: 'The cat naps.', },),).toEqual({ body: 'The cat naps.', start: 0, },);
            expect(fragmentCut({ fragment: '   \n\nThe cat naps.', },),).toEqual({ body: 'The cat naps.', start: 5, },);
            expect(fragmentCut({ fragment: '\n\n\n', },),).toEqual({ body: '', start: 3, },);
          },
        },),
      ],
    },),

    describe({
      name: composedInsertion.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WRITES THE TEXT of the boundary and says where each fragment`s body stands in it, with the first '
            + 'character of the fragment before the body when blank lines were cut from its start',
          fn: async () => {
            /**
             Two fragments between a block and a block, the first with a blank line cut from its start.
             */
            const asked = {
              fragments: ['\n\nThe cat naps.\n', 'She purrs.',],
              before: 'Intro.\n\n',
              after: 'Outro.',
              eol: '\n',
            };
            const { text, placed, } = composedInsertion(asked,);
            expect(text,).toBe('The cat naps.\n\nShe purrs.\n\n',);
            expect(placed,).toEqual([
              { fragment: 0, englishAt: -2, from: 0, to: 13, },
              { fragment: 1, englishAt: 15, from: 15, to: 25, },
            ],);
          },
        },),
        it({
          name: 'SHIFTS EVERY PLACE BY THE SEPARATOR written before the first fragment, in the document`s own line ending',
          fn: async () => {
            /**
             A boundary after a block with no line ending, in a document that uses the Windows one.
             */
            const { text, placed, } = composedInsertion({
              fragments: ['She purrs.',],
              before: 'Intro.',
              after: 'Outro.',
              eol: '\r\n',
            },);
            expect(text,).toBe('\r\n\r\nShe purrs.\r\n\r\n',);
            expect(placed,).toEqual([{ fragment: 0, englishAt: 4, from: 4, to: 14, },],);
          },
        },),
        it({
          name: 'GIVES NO PLACE TO A FRAGMENT THAT SHOWS A READER NOTHING and counts none of its width, and no place to any when '
            + 'none shows anything, since nothing is then written',
          fn: async () => {
            expect(composedInsertion({
              fragments: ['\u{200B}', 'She purrs.',],
              before: 'Intro.\n\n',
              after: 'Outro.',
              eol: '\n',
            },),).toEqual({
              text: 'She purrs.\n\n',
              placed: [{ fragment: 1, englishAt: 0, from: 0, to: 10, },],
            },);
            expect(composedInsertion({
              fragments: ['', '\u{200B}',],
              before: 'Intro.\n\n',
              after: 'Outro.',
              eol: '\n',
            },),).toEqual({
              text: '',
              placed: [],
            },);
          },
        },),
      ],
    },),

    describe({
      name: documentLineEnding.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads the ending the DOCUMENT uses rather than the platform`s, so a translation written on '
            + 'Windows does not come back with two conventions in it and a diff naming lines nobody touched',
          fn: async () => {
            expect(documentLineEnding({ targetText: 'The cat sleeps.\r\n\r\nShe purrs.\r\n', },),).toBe('\r\n',);
            expect(documentLineEnding({ targetText: 'The cat sleeps.\n', },),).toBe('\n',);
            expect(documentLineEnding({ targetText: '', },),).toBe('\n',);
          },
        },),
      ],
    },),

    describe({
      name: composeInsertion.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ADDS the separator an insertion needs and no more: a boundary that already carries a blank '
            + 'line keeps exactly that, and one that carries none is topped up. Existing whitespace is the '
            + 'archive`s, and this only ever adds',
          fn: async () => {
            expect(composeInsertion({
              fragments: ['The cat naps.',],
              before: 'The cat sleeps.\n\n',
              after: 'She purrs.\n',
              eol: '\n',
            },),).toBe('The cat naps.\n\n',);
            expect(composeInsertion({
              fragments: ['The cat naps.',],
              before: 'The cat sleeps.',
              after: 'She purrs.',
              eol: '\n',
            },),).toBe('\n\nThe cat naps.\n\n',);
          },
        },),
        it({
          name: 'READS A BOUNDARY AFTER NOTHING BUT A LEADING BYTE ORDER MARK AS THE START OF THE DOCUMENT, so '
            + 'an insertion there adds no blank line before it and the mark stays the page\'s first character '
            + 'with the insertion straight after it',
          fn: async () => {
            expect(composeInsertion({
              fragments: ['---\nname: Whiskers\n---\n',],
              before: '\uFEFF',
              after: 'The cat naps.\n',
              eol: '\n',
            },),).toBe(composeInsertion({
              fragments: ['---\nname: Whiskers\n---\n',],
              before: '',
              after: 'The cat naps.\n',
              eol: '\n',
            },),);
            expect(composeInsertion({
              fragments: ['The cat naps.',],
              before: '\uFEFFThe cat sleeps.',
              after: 'She purrs.',
              eol: '\n',
            },),).toBe('\n\nThe cat naps.\n\n',);
          },
        },),
        it({
          name: 'writes ONE blank line between fragments sharing a boundary, not one per fragment. Several '
            + 'slices of an untranslated section land here, and each carrying its own separators would put '
            + 'two blank lines between every pair',
          fn: async () => {
            expect(composeInsertion({
              fragments: [
                '\nThe cat naps.\n\n',
                '\n\nShe purrs.\n',
              ],
              before: 'The cat sleeps.\n\n',
              after: 'She wakes.\n',
              eol: '\n',
            },),).toBe('The cat naps.\n\nShe purrs.\n\n',);
          },
        },),
        it({
          name: 'terminates the FILE rather than separating two blocks when the boundary is the end of the '
            + 'document, since there is nothing after it to be separated from and a text file ends with one '
            + 'line ending',
          fn: async () => {
            expect(composeInsertion({
              fragments: ['She sleeps again.',],
              before: 'The cat sleeps.\n',
              after: '',
              eol: '\n',
            },),).toBe('\nShe sleeps again.\n',);
          },
        },),
        it({
          name: 'writes the document`s own line ending, so an insertion into a Windows translation carries '
            + 'the convention the rest of the file uses',
          fn: async () => {
            expect(composeInsertion({
              fragments: ['The cat naps.',],
              before: 'The cat sleeps.',
              after: 'She purrs.',
              eol: '\r\n',
            },),).toBe('\r\n\r\nThe cat naps.\r\n\r\n',);
          },
        },),
        it({
          name: 'writes NOTHING for fragments that say nothing, which keeps an empty group from opening a '
            + 'blank line at a boundary nobody wrote to',
          fn: async () => {
            expect(composeInsertion({
              fragments: [
                '',
                '\n\n',
              ],
              before: 'The cat sleeps.',
              after: 'She purrs.',
              eol: '\n',
            },),).toBe('',);
          },
        },),
        it({
          name: 'writes NOTHING for fragments that show a reader nothing, as for empty ones: a zero-width space, a '
            + 'Hangul filler, or both among spaces open no paragraph at the boundary, and beside a fragment of '
            + 'words only the words are written',
          fn: async () => {
            expect({
              alone: composeInsertion({
                fragments: [
                  '\u{200B}',
                  '\u{3164}',
                  ' \u{200B}\u{3164} \n',
                ],
                before: 'The cat sleeps.',
                after: 'She purrs.',
                eol: '\n',
              },),
              besideWords: composeInsertion({
                fragments: [
                  '\u{200B}',
                  'The cat naps.',
                  '\u{3164}',
                ],
                before: 'The cat sleeps.\n\n',
                after: 'She purrs.\n',
                eol: '\n',
              },),
            },).toEqual({
              alone: '',
              besideWords: 'The cat naps.\n\n',
            },);
          },
        },),
      ],
    },),

    describe({
      name: `${spliceSlices.name} separators`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'writes an insertion BEFORE a heading without running into it, which is the defect this '
            + 'exists for: a rendering written verbatim at that boundary produces a line that still parses '
            + 'as Markdown and says something else',
          fn: async () => {
            /** Archive whose second section is where the insertion goes. */
            const targetText = `## Intro

The cat sleeps.

## Habits

She purrs.
`;

            /** Boundary just before the second heading. */
            const offset = targetText.indexOf('## Habits',);
            expect(spliceSlices({
              targetText,
              slices: [
                {
                  source: {
                    sliceIndex: 0,
                    nodes: [],
                    startOffset: 0,
                    endOffset: 0,
                    text: '## 白天\n\n猫猫晒太阳。',
                  },
                  target: makeInsertionChunk({
                    sliceIndex: 0,
                    offset,
                  },),
                },
              ],
              replacements: [
                {
                  sliceIndex: 0,
                  replacementText: '## Daytime\n\nThe cat suns herself.',
                },
              ],
            },),).toBe(`## Intro

The cat sleeps.

## Daytime

The cat suns herself.

## Habits

She purrs.
`,);
          },
        },),
        it({
          name: 'separates an anchor from the span it sits at the END of, even though that span is itself '
            + 'replaced: composition reads the document as it stands when the anchor is written, and the '
            + 'replacement lands afterwards',
          fn: async () => {
            /** Archive whose first paragraph is rewritten and followed by an insertion. */
            const targetText = `## Intro

The cat sleeps.

## Habits

She purrs.
`;

            /** Where the paragraph being replaced begins. */
            const startOffset = targetText.indexOf('The cat sleeps.',);

            /** Where it ends, which is also the boundary the anchor names. */
            const endOffset = startOffset + 'The cat sleeps.'.length;
            expect(spliceSlices({
              targetText,
              slices: [
                {
                  source: {
                    sliceIndex: 0,
                    nodes: [],
                    startOffset: 0,
                    endOffset: 0,
                    text: '猫猫睡着了。',
                  },
                  target: {
                    sliceIndex: 0,
                    nodes: [],
                    startOffset,
                    endOffset,
                    text: 'The cat sleeps.',
                  },
                },
                {
                  source: {
                    sliceIndex: 1,
                    nodes: [],
                    startOffset: 0,
                    endOffset: 0,
                    text: '她伸了个懒腰。',
                  },
                  target: makeInsertionChunk({
                    sliceIndex: 1,
                    offset: endOffset,
                  },),
                },
              ],
              replacements: [
                {
                  sliceIndex: 0,
                  replacementText: 'The cat dozes.',
                },
                {
                  sliceIndex: 1,
                  replacementText: 'She stretches.',
                },
              ],
            },),).toBe(`## Intro

The cat dozes.

She stretches.

## Habits

She purrs.
`,);
          },
        },),
      ],
    },),
  ],
},);
