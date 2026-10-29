/**
 Tests for lane texts offered on a consolidation slate (class forty,
 2026-09-17).

 WHY THIS FILE EXISTS. Mio24 slice 17: the contest settled `neither` with
 every ballot judging the archive flawed, the archive's one-line rendering
 stood, it failed the deterministic publication rule (the original's poem
 carries a link the line does not), the writers' proposals flattened the
 poem's blockquote, three of five slate judges abstained, the two who voted
 tied, and the entry stopped with two valid lane texts carrying the poem,
 its line breaks and its link that no slate judge ever saw. A lane text on
 the slate is provenance no model on the roster owns as a whole, so it needs
 its own producer kind, and a model whose proposal reproduces it gains a
 stake in it exactly as one reproducing the incumbent does.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { HeardVoice, } from '@monochromatic-dev/translation-repair-stage';
import {
  type CandidateProducer,
  describeProducer,
  mergeProducers,
  producerModelIds,
} from '@monochromatic-dev/translation-repair-house-rule';
import {
  buildTranslateCandidates,
  type TranslateReportWire,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

//region Fixtures

/**
 Two seated translators, in roster order.
 */
const TRANSLATORS = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
] as const satisfies readonly RosterModelId[];

/**
 What the repair lane would ship.
 */
const REPAIR_TEXT = '> The cat naps by the [window](https://example.invalid/cat).';

/**
 What the translate lane would ship.
 */
const TRANSLATE_TEXT = '> The cat dozes beside the [window](https://example.invalid/cat).';

/**
 Builds one heard reply.

 @param at - roster position that answered

 @param translation - wording it proposed

 @returns Voice shaped as the gather returns one

 @example
 ```ts
 const voice = voiceOf({ at: 0, translation: 'The cat naps.', },);
 ```
 */
function voiceOf(
  {
    at,
    translation,
  }: {
    readonly at: number;
    readonly translation: string;
  },
): HeardVoice<TranslateReportWire> {
  return {
    modelId: TRANSLATORS[at] ?? TRANSLATORS[0],
    value: { translation, },
  };
}

//endregion Fixtures

await describe({
  name: 'translate candidates from lane texts (class forty, 2026-09-17)',
  children: [
    it({
      name: 'OFFERS each lane text as its own candidate after the incumbent and before the fresh '
        + 'proposals, under a lane producer nobody on the roster is discounted for',
      fn: async () => {
        const set = buildTranslateCandidates({
          voices: [
            voiceOf({
              at: 0,
              translation: 'The cat sleeps by the window.',
            },),
          ],
          translatorModelIds: [...TRANSLATORS,],
          incumbentText: 'The cat naps.',
          lineStructured: false,
          laneTexts: [
            {
              lane: 'repair',
              text: REPAIR_TEXT,
            },
            {
              lane: 'translate',
              text: TRANSLATE_TEXT,
            },
          ],
        },);
        expect(set.candidates,).toHaveLength(4,);
        expect(set.candidates
          .map(function toRendering(candidate,): string {
            return candidate.rendered;
          },),).toEqual([
            'The cat naps.',
            REPAIR_TEXT,
            TRANSLATE_TEXT,
            'The cat sleeps by the window.',
          ],);
        expect(set.candidates[1]?.producer,).toEqual({
          kind: 'lane',
          lane: 'repair',
          matched: [],
        },);
        expect(set.candidates[1]?.value
          .origin,).toBe('fresh',);
        expect(set.candidates[2]?.producer,).toEqual({
          kind: 'lane',
          lane: 'translate',
          matched: [],
        },);
      },
    },),

    it({
      name: 'COLLAPSES a fresh proposal reproducing a lane text into the lane candidate, so the '
        + 'model gains a stake in it and its own ballot for it is discounted',
      fn: async () => {
        const set = buildTranslateCandidates({
          voices: [
            voiceOf({
              at: 1,
              translation: `${REPAIR_TEXT}\n`,
            },),
          ],
          translatorModelIds: [...TRANSLATORS,],
          incumbentText: '',
          lineStructured: false,
          laneTexts: [
            {
              lane: 'repair',
              text: REPAIR_TEXT,
            },
          ],
        },);
        expect(set.candidates,).toHaveLength(1,);
        expect(set.collapsed,).toBe(1,);
        expect(set.candidates[0]?.producer,).toEqual({
          kind: 'lane',
          lane: 'repair',
          matched: [TRANSLATORS[1],],
        },);
        expect(producerModelIds({
          kind: 'lane',
          lane: 'repair',
          matched: [TRANSLATORS[1],],
        },),).toEqual([TRANSLATORS[1],],);
      },
    },),

    it({
      name: 'COLLAPSES a lane text matching the incumbent into the incumbent, since the text that '
        + 'was already there stays the text that was already there',
      fn: async () => {
        const set = buildTranslateCandidates({
          voices: [],
          translatorModelIds: [...TRANSLATORS,],
          incumbentText: REPAIR_TEXT,
          lineStructured: false,
          laneTexts: [
            {
              lane: 'repair',
              text: REPAIR_TEXT,
            },
          ],
        },);
        expect(set.candidates,).toHaveLength(1,);
        expect(set.candidates[0]?.producer,).toEqual({
          kind: 'incumbent',
          matched: [],
        },);
      },
    },),

    it({
      name: 'OFFERS no lane candidate when none is given, so every slate built before this '
        + 'class is built exactly as it was',
      fn: async () => {
        const without = buildTranslateCandidates({
          voices: [voiceOf({
            at: 0,
            translation: 'The cat sleeps by the window.',
          },),],
          translatorModelIds: [...TRANSLATORS,],
          incumbentText: 'The cat naps.',
          lineStructured: false,
        },);
        const empty = buildTranslateCandidates({
          voices: [voiceOf({
            at: 0,
            translation: 'The cat sleeps by the window.',
          },),],
          translatorModelIds: [...TRANSLATORS,],
          incumbentText: 'The cat naps.',
          lineStructured: false,
          laneTexts: [],
        },);
        expect(empty,).toEqual(without,);
        expect(without.candidates,).toHaveLength(2,);
      },
    },),

    it({
      name: 'DESCRIBES a lane producer by its lane, with the models that reproduced it when any did',
      fn: async () => {
        expect(describeProducer({
          kind: 'lane',
          lane: 'translate',
          matched: [],
        },),).toBe('lane(translate)',);
        expect(describeProducer({
          kind: 'lane',
          lane: 'translate',
          matched: [TRANSLATORS[0],],
        },),).toBe(`lane(translate, matched by ${TRANSLATORS[0]})`,);
      },
    },),

    it({
      name: 'MERGES a lane producer over a model or composite and under the incumbent, in either order',
      fn: async () => {
        /**
         Lane producer with no stake yet.
         */
        const lane: CandidateProducer = {
          kind: 'lane',
          lane: 'repair',
          matched: [],
        };
        /**
         One model's own proposal.
         */
        const model: CandidateProducer = {
          kind: 'model',
          modelId: TRANSLATORS[0],
        };
        expect(mergeProducers({
          left: lane,
          right: model,
        },),).toEqual({
          kind: 'lane',
          lane: 'repair',
          matched: [TRANSLATORS[0],],
        },);
        expect(mergeProducers({
          left: model,
          right: lane,
        },),).toEqual({
          kind: 'lane',
          lane: 'repair',
          matched: [TRANSLATORS[0],],
        },);
        expect(mergeProducers({
          left: {
            kind: 'incumbent',
            matched: [],
          },
          right: {
            kind: 'lane',
            lane: 'repair',
            matched: [TRANSLATORS[1],],
          },
        },),).toEqual({
          kind: 'incumbent',
          matched: [TRANSLATORS[1],],
        },);
      },
    },),
  ],
},);
