/**
 Tests for the pairing stage: what the roster has to agree on before a
 correspondence is kept, and what happens when it agrees on nothing.

 WHY AGREEMENT IS PER PAIR. Two models can agree on nine correspondences and
 differ on the tenth, and discarding both replies over the tenth throws away
 the nine. The stage counts each `source,target` on its own.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  MIN_STAGE_VOICES,
  reachableQuorum,
  shortBenchStageFinding,
} from '@monochromatic-dev/translation-repair-stage';
import { pairBlocksWithRoster, } from '../dist/final/node/index.mjs';
import { refusingSeatsClient, } from '@monochromatic-dev/translation-repair-provider/ts/refusing-seats-client.test-fixture.ts';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { cannedClient, } from '@monochromatic-dev/translation-repair-provider/ts/streaming-reply-client.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Two blocks standing in for an original side.
 */
const SOURCE = [
  {
    index: 0,
    text: '猫睡了。',
  },
  {
    index: 1,
    text: '它喜欢盒子。',
  },
];

/**
 Two blocks standing in for a translation.
 */
const TARGET = [
  {
    index: 0,
    text: 'The cat slept.',
  },
  {
    index: 1,
    text: 'She loves boxes.',
  },
];

/**
 Roster of two, which is the smallest that can agree or disagree.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
] as const;

/**
 Bench of five, whose quorum of three the router can leave out of reach while
 the fewest voices a stage closes on still answer.
 */
const FIVE_SEAT_ROSTER = [
  ...ROSTER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_HYPER_VISION,
] as const;

/**
 Logger for the stage under test.
 */
const l = tagged({ tag: 'pair-blocks-stage-test', },);

await describe({
  name: pairBlocksWithRoster.name,
  children: [
    it({
      name: 'KEEPS a correspondence both voices named',
      fn: async () => {
        const outcome = await pairBlocksWithRoster({
          // Whole bench: this case scripts every seat and reads over the bench it wrote.
          fanOut: 'whole-bench',
          client: cannedClient({
            replyByModel: [
              '{"pairs":[{"source":0,"target":0},{"source":1,"target":1}]}',
              '{"pairs":[{"source":0,"target":0},{"source":1,"target":1}]}',
            ],
          },),
          modelIds: ROSTER,
          sourceBlocks: SOURCE,
          targetBlocks: TARGET,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(outcome.pairs,).toEqual([{ source: 0, target: 0, }, { source: 1, target: 1, },],);
        expect(outcome.heard,).toBe(ROSTER.length,);
        expect(outcome.usable,).toBe(2,);
      },
    },),
    it({
      name: 'KEEPS a split paragraph when enough voices name both target blocks together, rather than treating supported one-to-many correspondence as contested alternatives',
      fn: async () => {
        const outcome = await pairBlocksWithRoster({
          // Whole bench: this case scripts every seat and reads over the bench it wrote.
          fanOut: 'whole-bench',
          client: cannedClient({
            replyByModel: [
              '{"pairs":[{"source":0,"target":0},{"source":0,"target":1}]}',
              '{"pairs":[{"source":0,"target":0},{"source":0,"target":1}]}',
            ],
          },),
          modelIds: ROSTER,
          sourceBlocks: SOURCE,
          targetBlocks: TARGET,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(outcome.pairs,).toEqual([
          { source: 0, target: 0, },
          { source: 0, target: 1, },
        ],);
      },
    },),
    it({
      name: 'KEEPS a merged paragraph when enough voices name both source blocks against one target',
      fn: async () => {
        const outcome = await pairBlocksWithRoster({
          // Whole bench: this case scripts every seat and reads over the bench it wrote.
          fanOut: 'whole-bench',
          client: cannedClient({
            replyByModel: [
              '{"pairs":[{"source":0,"target":0},{"source":1,"target":0}]}',
              '{"pairs":[{"source":0,"target":0},{"source":1,"target":0}]}',
            ],
          },),
          modelIds: ROSTER,
          sourceBlocks: SOURCE,
          targetBlocks: TARGET,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(outcome.pairs,).toEqual([
          { source: 0, target: 0, },
          { source: 1, target: 0, },
        ],);
      },
    },),
    it({
      name: 'KEEPS a correspondence two later voices named though the first voice omitted it, since '
        + 'agreement is per pair and not per reply',
      fn: async () => {
        const outcome = await pairBlocksWithRoster({
          // Whole bench: this case scripts every seat and reads over the bench it wrote.
          fanOut: 'whole-bench',
          client: cannedClient({
            replyByModel: [
              '{"pairs":[{"source":0,"target":0}]}',
              '{"pairs":[{"source":0,"target":0},{"source":1,"target":1}]}',
              '{"pairs":[{"source":0,"target":0},{"source":1,"target":1}]}',
            ],
          },),
          modelIds: [...ROSTER, SEAT_SYNTHETIC_TEXT_EVERYWHERE,],
          sourceBlocks: SOURCE,
          targetBlocks: TARGET,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(outcome.usable,).toBe(3,);
        expect(outcome.pairs.length,).toBe(2,);
        expect(outcome.pairs[1]?.source,).toBe(1,);
      },
    },),
    it({
      name: 'DROPS a correspondence only one voice named',
      fn: async () => {
        const outcome = await pairBlocksWithRoster({
          // Whole bench: this case scripts every seat and reads over the bench it wrote.
          fanOut: 'whole-bench',
          client: cannedClient({
            replyByModel: [
              '{"pairs":[{"source":0,"target":0},{"source":1,"target":1}]}',
              '{"pairs":[{"source":0,"target":0}]}',
            ],
          },),
          modelIds: ROSTER,
          sourceBlocks: SOURCE,
          targetBlocks: TARGET,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(outcome.pairs.length,).toBe(1,);
        expect(outcome.pairs[0]?.source,).toBe(0,);
      },
    },),
    it({
      name: 'RETURNS no pairs and says so when every reply is unusable',
      fn: async () => {
        const outcome = await pairBlocksWithRoster({
          // Whole bench: this case scripts every seat and reads over the bench it wrote.
          fanOut: 'whole-bench',
          client: cannedClient({
            replyByModel: [
              '{"pairs":[{"source":9,"target":0}]}',
              '{"pairs":[{"source":9,"target":0}]}',
            ],
          },),
          modelIds: ROSTER,
          sourceBlocks: SOURCE,
          targetBlocks: TARGET,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(outcome.pairs.length,).toBe(0,);
        expect(outcome.usable,).toBe(0,);
        expect(outcome.findings.join(' ',),).toContain('no-usable-voice',);
        // HEARD BUT UNUSABLE, each seat named: the replies arrived in shape and
        // were refused by the reader, not lost on the way.
        expect(outcome.heard,).toBe(ROSTER.length,);
        expect(outcome.findings.filter(finding => finding.startsWith('block-pairing unusable',),),)
          .toHaveLength(ROSTER.length,);
      },
    },),
    it({
      name: 'counts an off-shape reply as unheard rather than as a ballot',
      fn: async () => {
        const outcome = await pairBlocksWithRoster({
          fanOut: 'whole-bench',
          client: cannedClient({ replyByModel: ['{"noPairs":true}',], },),
          modelIds: ROSTER,
          sourceBlocks: SOURCE,
          targetBlocks: TARGET,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(outcome.pairs,).toEqual([],);
        expect(outcome.heard,).toBe(0,);
        expect(outcome.usable,).toBe(0,);
        expect(outcome.findings.join(' ',),).toContain(`no-usable-voice (0 heard of ${String(ROSTER.length,)})`,);
      },
    },),
    it({
      name: 'SAYS a short bench in the findings when the router leaves the bench quorum out of reach, '
        + 'and still keeps what the seats that answered agreed on (ledger X8)',
      fn: async () => {
        /**
         Seats the router refuses, leaving the fewest voices a stage closes on.
         */
        const refused = FIVE_SEAT_ROSTER.slice(MIN_STAGE_VOICES,);
        /**
         The bench's quorum once those seats are out of reach.
         */
        const quorum = reachableQuorum({
          benchSize: FIVE_SEAT_ROSTER.length,
          unreachable: refused.length,
        },);
        expect(quorum.short,).toBe(true,);

        const outcome = await pairBlocksWithRoster({
          // Whole bench: this case scripts every seat and reads over the bench it wrote.
          fanOut: 'whole-bench',
          client: refusingSeatsClient({
            reply: '{"pairs":[{"source":0,"target":0},{"source":1,"target":1}]}',
            refused,
          },),
          modelIds: FIVE_SEAT_ROSTER,
          sourceBlocks: SOURCE,
          targetBlocks: TARGET,
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(outcome.usable,).toBe(MIN_STAGE_VOICES,);
        expect(outcome.pairs,).toEqual([{ source: 0, target: 0, }, { source: 1, target: 1, },],);
        expect(outcome.findings,).toStrictEqual([
          shortBenchStageFinding({
            stage: 'block-pairing',
            quorum,
            benchSize: FIVE_SEAT_ROSTER.length,
          },),
        ],);
      },
    },),
  ],
},);
