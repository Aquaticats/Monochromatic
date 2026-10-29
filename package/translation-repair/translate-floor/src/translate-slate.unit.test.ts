/**
 Tests for the record of what the judges were actually shown.

 Ballots name a one-based position and the slate is rotated per slice, so
 without this record a stored ballot saying "candidate 2" cannot be joined to
 any text or producer afterwards. These cases pin the join.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { Candidate, } from '@monochromatic-dev/translation-repair-house-rule';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  describeSlate,
  NOT_ON_SLATE,
  positionOf,
  rotateCandidates,
  type TranslateCandidateValue,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Translation already in the archive.
 */
const INCUMBENT = 'The cat naps on the windowsill.';

/**
 One model's rendering.
 */
const FRESH_ONE = 'The cat is dozing on the windowsill.';

/**
 Another model's rendering.
 */
const FRESH_TWO = 'On the windowsill, the cat dozes.';

/**
 Slate as the lane assembles it, incumbent first.
 */
const CANDIDATES: readonly Candidate<TranslateCandidateValue>[] = [
  {
    producer: {
      kind: 'incumbent',
      matched: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
    },
    value: {
      text: INCUMBENT,
      origin: 'incumbent',
    },
    rendered: INCUMBENT,
  },
  {
    producer: {
      kind: 'model',
      modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
    },
    value: {
      text: FRESH_ONE,
      origin: 'fresh',
    },
    rendered: FRESH_ONE,
  },
  {
    producer: {
      kind: 'model',
      modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
    },
    value: {
      text: FRESH_TWO,
      origin: 'fresh',
    },
    rendered: FRESH_TWO,
  },
];

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: describeSlate.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'numbers positions from ONE, matching the ballot indexes judges '
            + 'return, and carries the producer of each. Off-by-one here would '
            + 'attribute every vote to its neighbour, and nothing downstream could '
            + 'detect it since every index would still be valid',
          fn: async () => {
            const slate = describeSlate({ candidates: CANDIDATES, },);
            expect(slate.map(function toIndex(entry,) {
              return entry.index;
            },),).toEqual([
              1,
              2,
              3,
            ],);
            expect(slate[0]?.origin,).toBe('incumbent',);
            expect(slate[0]?.producer,).toEqual({
              kind: 'incumbent',
              matched: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
            },);
            expect(slate[2]?.producer,).toEqual({
              kind: 'model',
              modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
            },);
          },
        },),

        it({
          name: 'hashes each candidate, so an artifact can be checked against a '
            + 'rebuilt slice without storing every candidate twice',
          fn: async () => {
            const slate = describeSlate({ candidates: CANDIDATES, },);
            expect(slate[1]?.hash,).toBe(hashContent({ content: FRESH_ONE, },),);
          },
        },),
      ],
    },),

    describe({
      name: positionOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'finds the position the shipped text occupies, which is what tells '
            + 'a fallback apart from a win: both ship text, and only one of them '
            + 'was chosen',
          fn: async () => {
            const slate = describeSlate({ candidates: CANDIDATES, },);
            expect(positionOf({
              slate,
              text: FRESH_TWO,
            },),).toBe(3,);
          },
        },),

        it({
          name: 'reports NOT_ON_SLATE for text that was never a candidate, which '
            + 'is exactly what a blank incumbent is: the slice ships unchanged and '
            + 'no position describes that',
          fn: async () => {
            const slate = describeSlate({ candidates: CANDIDATES, },);
            expect(positionOf({
              slate,
              text: '',
            },),).toBe(NOT_ON_SLATE,);
          },
        },),
      ],
    },),

    describe({
      name: rotateCandidates.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'gives the same order for the same slice every time, since a '
            + 'resumed slice replayed under another order would be a different '
            + 'question asked of the judges',
          fn: async () => {
            const once = rotateCandidates({
              candidates: CANDIDATES,
              sourceText: '猫猫在窗台上打盹。',
            },);
            const twice = rotateCandidates({
              candidates: CANDIDATES,
              sourceText: '猫猫在窗台上打盹。',
            },);
            expect(once,).toEqual(twice,);
          },
        },),

        it({
          name: 'MOVES the incumbent off position one across slices, which is the '
            + 'whole point: pinning it there would confound the incumbent win rate '
            + 'with whatever position preference the judges have, equally on every '
            + 'slice and so invisibly',
          fn: async () => {
            /**
             Positions the incumbent lands in across many distinct slices.
             */
            const positions = new Set(
              Array.from(
                { length: 40, },
                function toSlice(
                  _unused,
                  index,
                ): number {
                  return positionOf({
                    slate: describeSlate({ candidates: rotateCandidates({
                      candidates: CANDIDATES,
                      sourceText: `第${String(index,)}段。`,
                    },), },),
                    text: INCUMBENT,
                  },);
                },
              ),
            );
            expect(positions.size,).toBeGreaterThan(1,);
          },
        },),

        it({
          name: 'returns an empty slate unchanged rather than dividing by its '
            + 'length, since a slice where every voice was lost has no candidates '
            + 'at all',
          fn: async () => {
            expect(rotateCandidates({
              candidates: [],
              sourceText: 'anything',
            },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
