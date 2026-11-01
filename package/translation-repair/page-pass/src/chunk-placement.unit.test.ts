/**
 Tests for the two kinds a chunk can be.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type ContentChunk,
  isInsertionChunk,
  makeInsertionChunk,
  parseDocument,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { chunkByHeadings, } from '@monochromatic-dev/translation-repair-slice-reading';
import { spliceSlices, } from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: makeInsertionChunk.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'covers NOTHING at the boundary it names, which is what makes it a place rather than a span: '
            + 'one offset in, two equal offsets out, so a caller cannot hand in a start and an end that '
            + 'disagree about how much text it stands for',
          fn: async () => {
            /** Anchor somewhere inside a document. */
            const anchor = makeInsertionChunk({
              sliceIndex: 4,
              offset: 128,
            },);
            expect(anchor.startOffset,).toBe(128,);
            expect(anchor.endOffset,).toBe(anchor.startOffset,);
            expect(anchor.text,).toBe('',);
            expect(anchor.nodes,).toEqual([],);
            expect(anchor.sliceIndex,).toBe(4,);
          },
        },),
        it({
          name: 'is spliced INTO a document rather than over it, which is the property assembly rests on: an '
            + 'anchor at a paragraph boundary inserts there and leaves every character of the archive standing',
          fn: async () => {
            /** Archive the anchor points into. */
            const targetText = `## Section one

The cat sleeps.

## Section two

The cat wakes.
`;

            /** Boundary just before the second heading. */
            const offset = targetText.indexOf('## Section two',);

            /** Anchor there, spliced with fresh wording. */
            const spliced = spliceSlices({
              targetText,
              slices: [
                {
                  source: {
                    sliceIndex: 0,
                    nodes: [],
                    startOffset: 0,
                    endOffset: 0,
                    text: '插入',
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
                  replacementText: '## Section one and a half\n\nThe cat stretches.\n\n',
                },
              ],
            },);
            expect(spliced,).toContain('The cat stretches.',);
            expect(spliced,).toContain('The cat sleeps.',);
            expect(spliced,).toContain('The cat wakes.',);
            expect(spliced.indexOf('stretches',),).toBeLessThan(spliced.indexOf('wakes',),);
            expect(spliced.indexOf('sleeps',),).toBeLessThan(spliced.indexOf('stretches',),);
          },
        },),
      ],
    },),

    describe({
      name: isInsertionChunk.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads the FIELD rather than the emptiness. An empty content chunk is not something the '
            + 'constructors produce, but the type is structural so any caller can build one, and under a '
            + 'nodes-length test that fabrication would silently become an insertion',
          fn: async () => {
            /** Content chunk carrying nothing, which no constructor here emits. */
            const hollow: ContentChunk = {
              sliceIndex: 0,
              nodes: [],
              startOffset: 12,
              endOffset: 12,
              text: '',
            };
            expect(isInsertionChunk(hollow,),).toBe(false,);

            /** Anchor covering the same nothing, at the same offset. */
            const anchor = makeInsertionChunk({
              sliceIndex: 0,
              offset: 12,
            },);
            expect(isInsertionChunk(anchor,),).toBe(true,);
          },
        },),
        it({
          name: 'says NO to every chunk a real document produces, which is the case that has to keep holding: '
            + 'nothing in production makes an insertion yet, so a document full of them would mean the '
            + 'predicate answers about something other than what it was asked',
          fn: async () => {
            /** Two ordinary sections. */
            const chunks = chunkByHeadings({
              document: parseDocument({
                text: `## 简介

猫猫喜欢晒太阳。

## 习惯

它每天都在窗边睡觉。
`,
              },),
            },);
            expect(chunks.length,).toBeGreaterThan(0,);
            expect(chunks.some(function isAnchor(chunk,): boolean {
              return isInsertionChunk(chunk,);
            },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
