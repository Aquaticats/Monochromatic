/**
 Holds every fixture seat's name to the card it maps to: reach, image input
 and holds, so a remapping after a roster change that no longer fits the
 name fails here first.

 Fixtures are the roster itself; no corpus content appears.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  ANTHROPIC_SERVED_IDS,
  ROSTER_MODEL_IDS,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  reachOf,
  readsImages,
} from '../dist/final/node/index.mjs';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_OPENROUTER_DECISIONS,
  SEAT_OPENROUTER_ONLY,
  SEAT_OPENROUTER_ONLY_CHECKER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { cardOf, } from './catalog-lookups.test-fixture.ts';

/**
 What each seat's name claims, checked against the cards.
 */
const CLAIMS: readonly {
  readonly seat: RosterModelId;
  readonly reach: readonly ('synthetic' | 'hyper' | 'openrouter' | 'bedrock')[];
  readonly reads: boolean;
  readonly holds: readonly string[];
}[] = [
  {
    seat: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
    reach: [
      'hyper',
      'openrouter',
    ],
    reads: true,
    holds: [
      'wide-seat-dropped',
      'late-judge-dropped',
      'synthetic-withheld',
    ],
  },
  {
    seat: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    reach: [
      'synthetic',
      'hyper',
    ],
    reads: true,
    holds: [
      'openrouter-dropped',
      'late-judge-dropped',
    ],
  },
  {
    seat: SEAT_SYNTHETIC_VISION_WITHHELD,
    reach: [
      'synthetic',
      'hyper',
    ],
    reads: true,
    holds: [
      'openrouter-withheld',
      'hyper-slow-select',
    ],
  },
  {
    seat: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
    reach: [
      'synthetic',
      'hyper',
      'openrouter',
      'bedrock',
    ],
    reads: false,
    // Culled from every role by the owner on 2026-09-24; the card stays for
    // this fixture and the catalogs.
    holds: [
      'owner-culled',
      'translator-dropped',
    ],
  },
  {
    seat: SEAT_HYPER_VISION,
    reach: [
      'hyper',
      'openrouter',
    ],
    reads: true,
    holds: [],
  },
  {
    seat: SEAT_HYPER_TEXT_BEDROCK,
    reach: [
      'hyper',
      'openrouter',
      'bedrock',
    ],
    reads: true,
    holds: [],
  },
  {
    seat: SEAT_HYPER_ONLY,
    reach: ['hyper',],
    reads: false,
    holds: [
      'wide-seat-dropped',
      'openrouter-dropped',
    ],
  },
  {
    seat: SEAT_HYPER_OPENROUTER_UNMEASURED,
    reach: [
      'hyper',
      'openrouter',
    ],
    reads: true,
    holds: ['reader-unmeasured',],
  },
  {
    seat: SEAT_BEDROCK_ONLY_TEXT,
    reach: ['bedrock',],
    reads: false,
    holds: [],
  },
  {
    seat: SEAT_BEDROCK_ONLY_VISION_UNSEATED,
    reach: ['bedrock',],
    reads: true,
    holds: ['judge-unmeasured',],
  },
  {
    seat: SEAT_OPENROUTER_ONLY,
    reach: ['openrouter',],
    reads: false,
    holds: ['translator-dropped',],
  },
  {
    seat: SEAT_OPENROUTER_ONLY_CHECKER,
    reach: ['openrouter',],
    reads: true,
    holds: [
      'judge-unmeasured',
      'writer-unmeasured',
      'reader-unmeasured',
    ],
  },
  {
    seat: SEAT_OPENROUTER_DECISIONS,
    reach: [],
    reads: false,
    holds: [
      'writer-unmeasured',
      'reader-unmeasured',
    ],
  },
];

await describe({
  name: 'roster seats',
  children: [
    it({
      name: 'MAPS EVERY SEAT TO A DISTINCT ROSTER ID and covers the whole roster, so no test needs a literal',
      fn: async () => {
        /**
         Ids the seats map to.
         */
        const seats = CLAIMS.map(function toSeat(claim,): RosterModelId {
          return claim.seat;
        },);
        // THE EIGHT CLAUDE CARDS NEED NO FIXTURE SEAT: they are served by the owner's Claude key alone and their
        // reach, holds and prices are held by `claude-cards.unit.test.ts` and `claude-reach.unit.test.ts`.
        expect([...seats, ...ANTHROPIC_SERVED_IDS,].toSorted(),).toEqual([...ROSTER_MODEL_IDS,].toSorted(),);
      },
    },),

    it({
      name: 'HOLDS EACH NAME\'S CLAIM about reach (as the router reads it), image input and holds against the card',
      fn: async () => {
        for (const claim of CLAIMS) {
          /**
           Reach the router reads for this seat.
           */
          const reach = reachOf({ modelId: claim.seat, },);
          expect({
            seat: claim.seat,
            synthetic: reach.synthetic,
            hyper: reach.hyper,
            openrouter: reach.openrouter,
            bedrock: reach.bedrock,
          },).toEqual({
            seat: claim.seat,
            synthetic: claim.reach.includes('synthetic',),
            hyper: claim.reach.includes('hyper',),
            openrouter: claim.reach.includes('openrouter',),
            bedrock: claim.reach.includes('bedrock',),
          },);
          expect({
            seat: claim.seat,
            reads: readsImages({ modelId: claim.seat, },),
          },).toEqual({
            seat: claim.seat,
            reads: claim.reads,
          },);
          expect({
            seat: claim.seat,
            holds: [...cardOf({ modelId: claim.seat, },).holds,].toSorted(),
          },).toEqual({
            seat: claim.seat,
            holds: [...claim.holds,].toSorted(),
          },);
        }
      },
    },),
  ],
},);
