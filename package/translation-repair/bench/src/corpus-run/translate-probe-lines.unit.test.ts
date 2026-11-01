/**
 Tests for what the translate probe prints: the line naming the section it
 chose, the line counting slices, each slice's heading and what the
 translators answered, with every count noun shown in the singular and the
 plural.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  heardLines,
  PROBE_SLICES,
  sectionLine,
  sliceHeading,
  slicesLine,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'translate-probe-lines',
  children: [
    describe({
      name: sectionLine.name,
      children: [
        it({
          name: 'SAYS the plural for several blocks and characters on both sides, and marks a coverage below a quarter as barely translated',
          fn: async () => {
            expect(sectionLine({
              entryId: 'XingZ60',
              sourceBlocks: 76,
              sourceChars: 4_641,
              targetBlocks: 5,
              targetChars: 300,
              ratio: 5 / 76,
            },),).toBe(
              'TRANSLATE XingZ60: source 76 blocks / 4641 chars, target 5 blocks / 300 chars, '
                + 'coverage 0.066 (barely translated)',
            );
          },
        },),
        it({
          name: 'SAYS the singular for one block and one character on both sides, and leaves a coverage of a quarter or more unmarked',
          fn: async () => {
            expect(sectionLine({
              entryId: 'XingZ60',
              sourceBlocks: 1,
              sourceChars: 1,
              targetBlocks: 1,
              targetChars: 1,
              ratio: 1,
            },),).toBe('TRANSLATE XingZ60: source 1 block / 1 char, target 1 block / 1 char, coverage 1.000',);
          },
        },),
        it({
          name: 'MARKS nothing at a coverage of exactly a quarter',
          fn: async () => {
            expect(sectionLine({
              entryId: 'XingZ60',
              sourceBlocks: 4,
              sourceChars: 8,
              targetBlocks: 1,
              targetChars: 2,
              ratio: 0.25,
            },),).toBe('TRANSLATE XingZ60: source 4 blocks / 8 chars, target 1 block / 2 chars, coverage 0.250',);
          },
        },),
      ],
    },),
    describe({
      name: slicesLine.name,
      children: [
        it({
          name: 'SAYS the plural for several slices and names the number the probe asks about',
          fn: async () => {
            expect(slicesLine({ sliceCount: 7, },),).toBe(
              `TRANSLATE section subdivides into 7 slices; probing the first ${String(PROBE_SLICES,)}`,
            );
          },
        },),
        it({
          name: 'SAYS the singular for one slice and probes that one, never the three it could have asked about',
          fn: async () => {
            expect(slicesLine({ sliceCount: 1, },),).toBe(
              'TRANSLATE section subdivides into 1 slice; probing the first 1',
            );
          },
        },),
        it({
          name: 'NAMES the slices it has where the section has fewer than it asks about',
          fn: async () => {
            expect(slicesLine({ sliceCount: 2, },),).toBe(
              'TRANSLATE section subdivides into 2 slices; probing the first 2',
            );
          },
        },),
      ],
    },),
    describe({
      name: sliceHeading.name,
      children: [
        it({
          name: 'SAYS the plural for several characters on both sides, led by a blank line',
          fn: async () => {
            expect(sliceHeading({
              sourceChars: 15,
              targetChars: 18,
            },),).toBe('\n--- slice: 15 source chars, 18 target chars ---',);
          },
        },),
        it({
          name: 'SAYS the singular for one character on each side',
          fn: async () => {
            expect(sliceHeading({
              sourceChars: 1,
              targetChars: 1,
            },),).toBe('\n--- slice: 1 source char, 1 target char ---',);
          },
        },),
      ],
    },),
    describe({
      name: heardLines.name,
      children: [
        it({
          name: 'PRINTS the count alone when no voice and no finding came back',
          fn: async () => {
            expect(heardLines({
              heard: 0,
              asked: 2,
              translations: [],
              findings: [],
            },),).toEqual(['HEARD 0/2',],);
          },
        },),
        it({
          name: 'PRINTS each voice with its model, then each finding, after the count',
          fn: async () => {
            expect(heardLines({
              heard: 2,
              asked: 3,
              translations: [
                {
                  modelId: 'hf:cat/Cat-A',
                  translation: 'The cat naps.',
                },
                {
                  modelId: 'hf:cat/Cat-B',
                  translation: 'The cat sleeps.',
                },
              ],
              findings: ['one seat stayed silent',],
            },),).toEqual([
              'HEARD 2/3',
              '  hf:cat/Cat-A: The cat naps.',
              '  hf:cat/Cat-B: The cat sleeps.',
              '  finding: one seat stayed silent',
            ],);
          },
        },),
      ],
    },),
  ],
},);
