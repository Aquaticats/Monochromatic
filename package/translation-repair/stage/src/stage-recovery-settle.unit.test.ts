/**
 Tests for what the quorum loop makes of the outcomes of the recovery round (ledger B1371): a voice heard joins the
 voices, a seat lost as unreachable joins the unreachable seats and leaves the unreadable ones, and a seat still
 unreadable stays where it was.

 Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  type RoundOutcome,
  settleRecovered,
} from '../dist/final/node/index.mjs';

//region Recovery settlement tests

/**
 Logger of the stage the voices are recorded under.
 */
const l = tagged({ tag: 'stage-recovery-settle-test', },);

/**
 Value a heard voice carries.
 */
type Meow = { readonly meow: string; };

/**
 Outcomes of a recovery round over three seats: one heard, one lost as unreachable and one still unreadable.
 */
const RECOVERED: readonly RoundOutcome<Meow>[] = [
  {
    modelId: SEAT_HYPER_TEXT_BEDROCK,
    voice: { heard: true, value: { meow: 'heard', }, },
  },
  {
    modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    voice: { heard: false, answered: false, unreachable: true, },
  },
  {
    modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
    voice: { heard: false, answered: true, unreachable: false, unreadable: 'off-shape', },
  },
];

/**
 Settles the outcomes over seats that were all unreadable before the round.

 @returns Voices collected, and the unreadable and unreachable seats after the settlement

 @example
 ```ts
 const settled = settleRecoveredOutcomes();
 ```
 */
function settleRecoveredOutcomes() {
  const collected: Parameters<typeof settleRecovered<Meow>>[0]['collected'] = [];
  const unreadableSeats = new Map<RosterModelId, unknown>(RECOVERED.map(function unreadableBefore(outcome,) {
    return [outcome.modelId, 'off-shape',] as const;
  },),);
  const unreachableSeats = new Set<RosterModelId>();
  settleRecovered<Meow>({
    recovered: RECOVERED,
    stage: 'gate',
    l,
    collected,
    unreadableSeats,
    unreachableSeats,
  },);
  return {
    collected,
    unreadable: [...unreadableSeats.keys(),],
    unreachable: [...unreachableSeats,],
  };
}

await describe({
  name: settleRecovered.name,
  children: [
    it({
      name: 'ADDS A HEARD VOICE TO THE VOICES AND TAKES ITS SEAT OFF THE UNREADABLE ONES',
      fn: async () => {
        const settled = settleRecoveredOutcomes();
        expect(settled.collected.map(function modelOf(voice,): RosterModelId {
          return voice.modelId;
        },),).toEqual([SEAT_HYPER_TEXT_BEDROCK,],);
        expect(settled.unreadable,).not.toContain(SEAT_HYPER_TEXT_BEDROCK,);
        expect(settled.unreachable,).not.toContain(SEAT_HYPER_TEXT_BEDROCK,);
      },
    },),

    it({
      name: 'MOVES A SEAT LOST AS UNREACHABLE FROM THE UNREADABLE SEATS TO THE UNREACHABLE ONES (ledger B1371), so the '
        + 'quorum is sized without it and no reader counts it as one that answered',
      fn: async () => {
        const settled = settleRecoveredOutcomes();
        expect(settled.unreachable,).toEqual([SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],);
        expect(settled.unreadable,).not.toContain(SEAT_SYNTHETIC_VISION_NO_OPENROUTER,);
      },
    },),

    it({
      name: 'LEAVES A SEAT THAT IS STILL UNREADABLE UNREADABLE, and neither unreachable nor heard',
      fn: async () => {
        const settled = settleRecoveredOutcomes();
        expect(settled.unreadable,).toEqual([SEAT_HYPER_OPENROUTER_UNMEASURED,],);
        expect(settled.unreachable,).not.toContain(SEAT_HYPER_OPENROUTER_UNMEASURED,);
        expect(settled.collected.map(function modelOf(voice,): RosterModelId {
          return voice.modelId;
        },),).not.toContain(SEAT_HYPER_OPENROUTER_UNMEASURED,);
      },
    },),
  ],
},);

//endregion Recovery settlement tests
