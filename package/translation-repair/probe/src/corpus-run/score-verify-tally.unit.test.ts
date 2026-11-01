/**
 Tests for the join of graded items to manifest rows that `score-verify`
 counts and prints.

 A wrong join does not fail, it mislabels, so a case holds each verdict's
 count, the refusal of two files that differ in length, and the line each set
 is reported on.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import type { GradedItem, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  tallyByKind,
  verifyKindLines,
} from '../../dist/final/node/index.mjs';

/**
 Graded item of a position.

 @param index - one-based position

 @param verdict - what the grader decided

 @returns Item as the sheet reader yields it

 @example
 ```ts
 const item = gradedAt({ index: 1, verdict: 'real-defect', },);
 ```
 */
function gradedAt(
  {
    index,
    verdict,
  }: {
    readonly index: number;
    readonly verdict: GradedItem['verdict'];
  },
): GradedItem {
  return {
    index,
    verdict,
    note: '',
  };
}

/**
 Manifest row of a position.

 @param position - one-based position

 @param kind - set the position came from

 @returns Row as the manifest reader yields it

 @example
 ```ts
 const row = rowAt({ position: 1, kind: 'control', },);
 ```
 */
function rowAt(
  {
    position,
    kind,
  }: {
    readonly position: number;
    readonly kind: string;
  },
): {
  readonly position: number;
  readonly entryId: string;
  readonly kind: string;
} {
  return {
    position,
    entryId: `cat${String(position,)}`,
    kind,
  };
}

await describe({
  name: 'score-verify-tally',
  children: [
    describe({
      name: tallyByKind.name,
      children: [
        it({
          name: 'COUNTS each verdict under the set its position names, a duplicate with the ungraded',
          fn: async () => {
            expect(tallyByKind({
              graded: [
                gradedAt({ index: 1, verdict: 'real-defect', },),
                gradedAt({ index: 2, verdict: 'false-positive', },),
                gradedAt({ index: 3, verdict: 'unscored', },),
                gradedAt({ index: 4, verdict: 'duplicate', },),
                gradedAt({ index: 5, verdict: 'real-defect', },),
              ],
              manifest: [
                rowAt({ position: 1, kind: 'control', },),
                rowAt({ position: 2, kind: 'control', },),
                rowAt({ position: 3, kind: 'control', },),
                rowAt({ position: 4, kind: 'flagged', },),
                rowAt({ position: 5, kind: 'flagged', },),
              ],
            },),).toStrictEqual(new Map([
              [
                'control',
                {
                  damage: 1,
                  invented: 1,
                  unscored: 1,
                },
              ],
              [
                'flagged',
                {
                  damage: 1,
                  invented: 0,
                  unscored: 1,
                },
              ],
            ],),);
          },
        },),

        it({
          name: 'TALLIES nothing for a sheet and a manifest that both hold no item',
          fn: async () => {
            expect(tallyByKind({ graded: [], manifest: [], },),).toStrictEqual(new Map(),);
          },
        },),

        it({
          name: 'REFUSES in its own words a sheet longer than its manifest, naming both counts, one manifest row in the singular',
          fn: async () => {
            /**
             What joining two items to one row raised.
             */
            const refusal = caught(function joinsTwoToOne(): unknown {
              return tallyByKind({
                graded: [
                  gradedAt({ index: 1, verdict: 'real-defect', },),
                  gradedAt({ index: 2, verdict: 'real-defect', },),
                ],
                manifest: [rowAt({ position: 1, kind: 'control', },),],
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              'StatedRefusalError: sheet carries 2 items and manifest carries 1; a positional join between '
                + 'them would mislabel verdicts, so neither file describes the other',
            );
          },
        },),

        it({
          name: 'REFUSES in its own words a sheet of one item against a longer manifest, with the item in the singular',
          fn: async () => {
            /**
             What joining one item to two rows raised.
             */
            const refusal = caught(function joinsOneToTwo(): unknown {
              return tallyByKind({
                graded: [gradedAt({ index: 1, verdict: 'real-defect', },),],
                manifest: [
                  rowAt({ position: 1, kind: 'control', },),
                  rowAt({ position: 2, kind: 'control', },),
                ],
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              'StatedRefusalError: sheet carries 1 item and manifest carries 2; a positional join between '
                + 'them would mislabel verdicts, so neither file describes the other',
            );
          },
        },),
      ],
    },),

    describe({
      name: verifyKindLines.name,
      children: [
        it({
          name: 'PRINTS a line per set in code-point order of the label, with its precision to three places',
          fn: async () => {
            expect(verifyKindLines({
              tallies: new Map([
                [
                  'flagged',
                  {
                    damage: 2,
                    invented: 1,
                    unscored: 0,
                  },
                ],
                [
                  'Control',
                  {
                    damage: 1,
                    invented: 1,
                    unscored: 1,
                  },
                ],
              ],),
            },),).toStrictEqual([
              'Control  flags=3 realDamage=1 invented=1 unscored=1 precision=0.500',
              'flagged  flags=3 realDamage=2 invented=1 unscored=0 precision=0.667',
            ],);
          },
        },),

        it({
          name: 'NAMES the precision n/a for a set no flag of which carries a verdict, and keeps a long label whole',
          fn: async () => {
            expect(verifyKindLines({
              tallies: new Map([
                [
                  'probe-silent',
                  {
                    damage: 0,
                    invented: 0,
                    unscored: 2,
                  },
                ],
              ],),
            },),).toStrictEqual([
              'probe-silent flags=2 realDamage=0 invented=0 unscored=2 precision=n/a',
            ],);
          },
        },),

        it({
          name: 'PRINTS no line for no set',
          fn: async () => {
            expect(verifyKindLines({ tallies: new Map(), },),).toStrictEqual([],);
          },
        },),
      ],
    },),
  ],
},);
