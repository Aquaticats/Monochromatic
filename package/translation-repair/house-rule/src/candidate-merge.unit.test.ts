/**
 Tests for collapsing identical candidates while keeping every author.

 The defect these exist for is silent in every log: when duplicates stand as
 separate candidates, the ballot splits and the self-vote discount stops
 applying, because each copy is credited to one model and the others look
 disinterested in text they wrote themselves.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type Candidate,
  mergeIdenticalCandidates,
  producerModelIds,
} from '../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Text two models happened to write identically.
 */
const SHARED = 'The cat naps on the windowsill.';

/**
 A different rendering of the same passage.
 */
const OTHER = 'The cat is napping on the windowsill.';

/**
 Builds one model's candidate.

 @param modelId - model credited with it

 @param text - what it wrote

 @returns Candidate as a lane would assemble it

 @example
 ```ts
 const candidate = from({ modelId: 'hf:x', text: SHARED, },);
 ```
 */
function from(
  {
    modelId,
    text,
  }: {
    readonly modelId: RosterModelId;
    readonly text: string;
  },
): Candidate<string> {
  return {
    producer: {
      kind: 'model',
      modelId,
    },
    value: text,
    rendered: text,
  };
}

await describe({
  name: mergeIdenticalCandidates.name,
  children: [
    it({
      name: 'CREDITS every model that wrote the same text to one candidate, '
        + 'which is what makes a later ballot for it count as that model\'s own '
        + 'work. Dropping the second author instead left the discount silently '
        + 'inapplicable on exactly the slices where the ensemble agreed',
      fn: async () => {
        const merged = mergeIdenticalCandidates({
          candidates: [
            from({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              text: SHARED,
            },),
            from({
              modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              text: SHARED,
            },),
          ],
        },);
        expect(merged,).toHaveLength(1,);
        expect(producerModelIds(merged[0]?.producer
          ?? {
            kind: 'model',
            modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
          },),)
          .toEqual([
            SEAT_HYPER_OPENROUTER_VISION_EDITOR,
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          ],);
      },
    },),

    it({
      name: 'keeps distinct proposals apart and in FIRST-SEEN order, since the '
        + 'lane assembling them puts them in roster order and candidate '
        + 'numbering must not vary between runs over identical inputs',
      fn: async () => {
        const merged = mergeIdenticalCandidates({
          candidates: [
            from({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              text: OTHER,
            },),
            from({
              modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              text: SHARED,
            },),
            from({
              modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
              text: OTHER,
            },),
          ],
        },);
        expect(merged,).toHaveLength(2,);
        expect(merged[0]?.rendered,).toBe(OTHER,);
        expect(merged[1]?.rendered,).toBe(SHARED,);
      },
    },),

    it({
      name: 'returns nothing for no candidates, which is a stage whose whole '
        + 'roster was lost rather than a fault',
      fn: async () => {
        expect(mergeIdenticalCandidates({ candidates: [], },),).toEqual([],);
      },
    },),
  ],
},);
