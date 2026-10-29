/**
 Tests for turning heard editor voices into comparable candidates.

 `buildEditorCandidates` had no test, and the property it exists to hold is
 determinism. Voices come back in whatever order the provider answered. If the
 candidate list followed arrival order, then the anonymized candidate
 numbering the judges see, the winner of a duplicate collapse, and the
 fallback choice would all vary between runs over identical inputs, and two
 runs of the same chunk could ship different text for no reason anyone could
 reconstruct.

 So the cases in this file feed voices in orders that disagree with the roster and
 assert the output does not move.

 Envelopes are built with the real `hashContent`, so the patches genuinely
 pass the apply gate rather than arriving pre-rejected. Cat-themed invention
 throughout.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { EditableEnvelope, } from '@monochromatic-dev/translation-repair-shared-contract';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import { buildEditorCandidates, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Translation chunk the editors propose against.
 */
const TARGET_TEXT = 'The cat is doing the sleeping.\n\nShe is doing the chasing of butterflies.';

/**
 First editable paragraph.
 */
const FIRST_TEXT = 'The cat is doing the sleeping.';

/**
 Second editable paragraph.
 */
const SECOND_TEXT = 'She is doing the chasing of butterflies.';

/**
 Envelopes in prompt numbering order, hashed the way the apply gate expects.
 */
const ENVELOPES: readonly EditableEnvelope[] = [
  {
    envelopeId: 'envelope/0',
    startOffset: 0,
    endOffset: FIRST_TEXT.length,
    baseText: FIRST_TEXT,
    baseHash: hashContent({ content: FIRST_TEXT, },),
    issueIds: ['issue/tense',],
  },
  {
    envelopeId: 'envelope/1',
    startOffset: TARGET_TEXT.indexOf(SECOND_TEXT,),
    endOffset: TARGET_TEXT.indexOf(SECOND_TEXT,) + SECOND_TEXT.length,
    baseText: SECOND_TEXT,
    baseHash: hashContent({ content: SECOND_TEXT, },),
    issueIds: ['issue/gloss',],
  },
];

/**
 Roster fixing candidate order, deliberately not alphabetical so a sort by id
 rather than by roster position would be visible.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
] as const;

/**
 Builds one heard editor voice proposing a single region rewrite.

 @param modelId - editor that answered

 @param region - one-based region number from the prompt sheet

 @param newText - replacement for that region

 @returns Heard voice carrying that reply

 @example
 ```ts
 const voice = heard({ modelId: ROSTER[0], region: 1, newText: 'The cat sleeps.', },);
 ```
 */
function heard(
  {
    modelId,
    region,
    newText,
  }: {
    readonly modelId: typeof ROSTER[number];
    readonly region: number;
    readonly newText: string;
  },
) {
  return {
    modelId,
    value: {
      edits: [
        {
          region,
          newText,
        },
      ],
    },
  };
}

await describe({
  name: buildEditorCandidates.name,
  children: [
    it({
      name: 'orders candidates by ROSTER POSITION rather than arrival, which '
        + 'is what keeps two runs over identical inputs from shipping '
        + 'different text: arrival order decides the anonymized numbering the '
        + 'judges see, the survivor of a duplicate collapse, and the fallback',
      fn: async () => {
        /**
         Voices arriving in exactly reverse roster order.
         */
        const { candidates, } = buildEditorCandidates({
          voices: [
            heard({
              modelId: ROSTER[2],
              region: 1,
              newText: 'The cat naps.',
            },),
            heard({
              modelId: ROSTER[1],
              region: 1,
              newText: 'The cat dozes.',
            },),
            heard({
              modelId: ROSTER[0],
              region: 1,
              newText: 'The cat sleeps.',
            },),
          ],
          editorModelIds: ROSTER,
          promptEnvelopes: ENVELOPES,
          targetText: TARGET_TEXT,
          envelopes: ENVELOPES,
          preservation: { mode: 'skip', },
        },);

        expect(candidates.map(function toId(candidate,) {
          return candidate.modelId;
        },),).toStrictEqual([...ROSTER,],);
      },
    },),

    it({
      name: 'produces the SAME candidate order for every arrival permutation, '
        + 'so determinism does not depend on which permutation happened to be '
        + 'tried',
      fn: async () => {
        /**
         Every voice, built once and reordered per permutation.
         */
        const voices = ROSTER.map(function toVoice(modelId, index,) {
          return heard({
            modelId,
            region: 1,
            newText: `The cat rests ${String(index,)}.`,
          },);
        },);

        for (const order of [
          [
            0,
            1,
            2,
          ],
          [
            2,
            1,
            0,
          ],
          [
            1,
            2,
            0,
          ],
          [
            0,
            2,
            1,
          ],
        ]) {
          /**
           Candidates for this arrival permutation.
           */
          const { candidates, } = buildEditorCandidates({
            voices: order.map(function pick(index,) {
              return voices[index] ?? voices[0];
            },).filter(function isPresent(voice,) {
              return voice !== undefined;
            },),
            editorModelIds: ROSTER,
            promptEnvelopes: ENVELOPES,
            targetText: TARGET_TEXT,
            envelopes: ENVELOPES,
            preservation: { mode: 'skip', },
          },);

          expect(candidates.map(function toId(candidate,) {
            return candidate.modelId;
          },),).toStrictEqual([...ROSTER,],);
        }
      },
    },),

    it({
      name: 'produces one candidate per HEARD voice rather than per roster '
        + 'entry, so an editor that never answered leaves no empty proposal '
        + 'for judges to rank',
      fn: async () => {
        /**
         Only the last two of three editors answered.
         */
        const { candidates, } = buildEditorCandidates({
          voices: [
            heard({
              modelId: ROSTER[2],
              region: 1,
              newText: 'The cat naps.',
            },),
            heard({
              modelId: ROSTER[1],
              region: 1,
              newText: 'The cat dozes.',
            },),
          ],
          editorModelIds: ROSTER,
          promptEnvelopes: ENVELOPES,
          targetText: TARGET_TEXT,
          envelopes: ENVELOPES,
          preservation: { mode: 'skip', },
        },);

        expect(candidates.length,).toBe(2,);
        // Still roster order among those heard.
        expect(candidates.map(function toId(candidate,) {
          return candidate.modelId;
        },),).toStrictEqual([
          ROSTER[1],
          ROSTER[2],
        ],);
      },
    },),

    it({
      name: 'runs every voice through the apply gate, so a well-formed edit '
        + 'lands as applied text and the candidates are directly comparable '
        + 'rather than each carrying its own notion of what applied',
      fn: async () => {
        /**
         One editor rewriting the first region.
         */
        const { candidates, findings, } = buildEditorCandidates({
          voices: [
            heard({
              modelId: ROSTER[0],
              region: 1,
              newText: 'The cat sleeps.',
            },),
          ],
          editorModelIds: ROSTER,
          promptEnvelopes: ENVELOPES,
          targetText: TARGET_TEXT,
          envelopes: ENVELOPES,
          preservation: { mode: 'skip', },
        },);

        expect(findings,).toStrictEqual([],);
        expect(candidates[0]?.patch.applied.length,).toBe(1,);
        expect(candidates[0]?.patch.patchedText,).toBe(
          `The cat sleeps.\n\n${SECOND_TEXT}`,
        );
      },
    },),

    it({
      name: 'ATTRIBUTES each wire irregularity to the editor that produced it, '
        + 'because an unattributed finding cannot tell you which model to stop '
        + 'trusting',
      fn: async () => {
        /**
         One good editor and one naming a region that is not on the sheet.
         */
        const { findings, } = buildEditorCandidates({
          voices: [
            heard({
              modelId: ROSTER[1],
              region: 9,
              newText: 'The dog barks.',
            },),
            heard({
              modelId: ROSTER[0],
              region: 1,
              newText: 'The cat sleeps.',
            },),
          ],
          editorModelIds: ROSTER,
          promptEnvelopes: ENVELOPES,
          targetText: TARGET_TEXT,
          envelopes: ENVELOPES,
          preservation: { mode: 'skip', },
        },);

        expect(findings.length,).toBe(1,);
        expect(findings[0],).toContain(ROSTER[1],);
      },
    },),

    it({
      name: 'collects findings in ROSTER order too, so a scorecard reads the '
        + 'same way across runs even when several editors misbehaved',
      fn: async () => {
        /**
         Two editors both naming regions off the sheet, arriving reversed.
         */
        const { findings, } = buildEditorCandidates({
          voices: [
            heard({
              modelId: ROSTER[2],
              region: 8,
              newText: 'The bird sings.',
            },),
            heard({
              modelId: ROSTER[0],
              region: 9,
              newText: 'The dog barks.',
            },),
          ],
          editorModelIds: ROSTER,
          promptEnvelopes: ENVELOPES,
          targetText: TARGET_TEXT,
          envelopes: ENVELOPES,
          preservation: { mode: 'skip', },
        },);

        expect(findings.length,).toBe(2,);
        expect(findings[0],).toContain(ROSTER[0],);
        expect(findings[1],).toContain(ROSTER[2],);
      },
    },),

    it({
      name: 'returns nothing at all when no editor was heard, rather than '
        + 'throwing, since a stage that lost every voice is a degraded run the '
        + 'caller handles rather than a malformed input',
      fn: async () => {
        /**
         Result of a stage where every editor was lost.
         */
        const { candidates, findings, } = buildEditorCandidates({
          voices: [],
          editorModelIds: ROSTER,
          promptEnvelopes: ENVELOPES,
          targetText: TARGET_TEXT,
          envelopes: ENVELOPES,
          preservation: { mode: 'skip', },
        },);

        expect(candidates,).toStrictEqual([],);
        expect(findings,).toStrictEqual([],);
      },
    },),
  ],
},);
