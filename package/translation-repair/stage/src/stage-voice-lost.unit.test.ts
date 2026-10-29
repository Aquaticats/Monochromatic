/**
 Tests for the per-model findings a gather leaves for the voices it asked and never heard.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  lostVoiceFindings,
  rosterShortfallFindings,
} from '../dist/final/node/index.mjs';

/**
 Roster the cases seat, in roster order.
 */
const SEATED: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
];

await describe({
  name: '',
  children: [
    describe({
      name: lostVoiceFindings.name,
      children: [
        it({
          name: 'NAMES EACH MODEL ASKED AND NEVER HEARD, one finding per model in roster order, and none for a model '
            + 'that was never asked or that answered',
          fn: async () => {
            expect(lostVoiceFindings({
              stage: 'critic',
              seated: SEATED,
              asked: new Set([SEAT_SYNTHETIC_VISION_WITHHELD, SEAT_HYPER_OPENROUTER_VISION_EDITOR,],),
              voices: [{ modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, value: {}, },],
            },),).toStrictEqual([
              `stage-voice-lost (critic ${SEAT_HYPER_OPENROUTER_VISION_EDITOR})`,
              `stage-voice-lost (critic ${SEAT_SYNTHETIC_VISION_WITHHELD})`,
            ],);
          },
        },),

        it({
          name: 'NAMES NOBODY when every model asked answered',
          fn: async () => {
            expect(lostVoiceFindings({
              stage: 'panel',
              seated: SEATED,
              asked: new Set(SEATED,),
              voices: SEATED.map(function heard(modelId,) {
                return { modelId, value: {}, };
              },),
            },),).toStrictEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: rosterShortfallFindings.name,
      children: [
        it({
          name: 'SAYS QUORUM UNMET, with heard against asked, when the gather did not reach it',
          fn: async () => {
            expect(rosterShortfallFindings({
              stage: 'critic',
              quorumMet: false,
              asked: new Set(SEATED,),
              voices: [{ modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, value: {}, },],
            },),).toStrictEqual(['stage-quorum-unmet (critic 1/3)',],);
          },
        },),

        it({
          name: 'SAYS THE ROSTER WAS INCOMPLETE when quorum was met with a voice missing, and nothing when every asked '
            + 'model answered',
          fn: async () => {
            expect(rosterShortfallFindings({
              stage: 'panel',
              quorumMet: true,
              asked: new Set(SEATED,),
              voices: SEATED.slice(0, 2,).map(function heard(modelId,) {
                return { modelId, value: {}, };
              },),
            },),).toStrictEqual(['stage-roster-incomplete (panel 2/3)',],);
            expect(rosterShortfallFindings({
              stage: 'panel',
              quorumMet: true,
              asked: new Set(SEATED,),
              voices: SEATED.map(function heard(modelId,) {
                return { modelId, value: {}, };
              },),
            },),).toStrictEqual([],);
          },
        },),
      ],
    },),
  ],
},);
