/**
 Tests for the gate deciding whether the rendering this run wrote replaces the
 one that would otherwise ship.

 WHAT THIS FILE EXISTS TO STOP. A consolidation is a third candidate from the
 same kind of instrument that produced the first two, so it can be worse. It
 replaces nothing on a tie, on a refusal, or on a roster too thin to settle.
 Changing what a reader sees on a memorial page needs more evidence than
 leaving it, and the churn reason is already recorded in the translate wire.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  gateConsolidatedSlice,
  settleGateBallots,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { ballot, } from '@monochromatic-dev/translation-repair-test-fixture/ts/structural-ballot.test-fixture.ts';
import { cannedClient, } from '@monochromatic-dev/translation-repair-provider/ts/streaming-reply-client.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 One gated slice, standing in for a corpus passage.
 */
const SUBJECT = {
  lineStructured: false,
  sourceText: '猫睡了一下午。',
  incumbentText: 'The cat slept all afternoon in the sun.',
  consolidatedText: 'The cat slept all afternoon.',
  standingText: 'The cat slept.',
};

/**
 Roster of three, the smallest that can produce a two-to-one split.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 Logger for the stage under test.
 */
const l = tagged({ tag: 'consolidate-gate-stage-test', },);

/**
 Five seats, so three refused leave a bench short of its quorum of three.
 */
const FIVE_SEATS: readonly RosterModelId[] = [
  ...ROSTER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_VISION,
];

/**
 Builds a client whose router refuses some seats for want of a wet
 provider, every other seat backing the consolidation.

 @param refused - seats the router refuses

 @returns Scripted client

 @example
 ```ts
 const client = refusingClient({ refused: FIVE_SEATS.slice(0, 3,), },);
 ```
 */
function refusingClient(
  { refused, }: { readonly refused: readonly RosterModelId[]; },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the gate',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      if (refused.includes(request.modelId,)) {
        throw new NoProviderForModelError({
          modelId: request.modelId,
          reason: 'every provider serving this cat is out of budget',
        },);
      }
      /**
       Ballot backing the consolidation.
       */
      const value: unknown = JSON.parse(ballot({ choice: 'consolidated', },),);
      if (!request.validate(value,))
        throw new Error('scripted gate ballot failed the guard',);
      return {
        kind: 'ok',
        value,
        rawText: JSON.stringify(value,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by the gate',);
    },
  };
}

/**
 Runs one gate over a canned roster.

 @param replyByModel - reply body per model

 @returns What the roster settled and what ships

 @example
 ```ts
 const outcome = await gate({ replyByModel: [], },);
 ```
 */
async function gate(
  { replyByModel, }: { readonly replyByModel: readonly string[]; },
) {
  return await gateConsolidatedSlice({
    // Whole bench: this case scripts every seat and reads over the bench it wrote.
    fanOut: 'whole-bench',
    client: cannedClient({ replyByModel, },),
    modelIds: ROSTER,
    subject: SUBJECT,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    exchangeTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'front matter consolidation gate',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SETTLES SYNTAX-BEARING METADATA through gate stage',
          fn: async () => {
            const outcome = await gateConsolidatedSlice({
              // Whole bench: this case scripts every seat and reads over the bench it wrote.
              fanOut: 'whole-bench',
              client: cannedClient({
                replyByModel: [
                  ballot({ choice: 'consolidated', },),
                  ballot({ choice: 'consolidated', },),
                  ballot({ choice: 'consolidated', },),
                ],
              },),
              modelIds: ROSTER,
              subject: {
                lineStructured: false,
                sourceText: '---\nname: 猫猫\n---\n',
                incumbentText: '---\nname: EntryId\n---\n',
                consolidatedText: '---\nname: Maomao Cat\n---\n',
                standingText: '---\nname: Maomao\n---\n',
                syntax: 'front-matter',
              },
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(outcome.choice,).toBe('consolidated',);
            expect(outcome.ships,).toBe('consolidated',);
          },
        },),
      ],
    },),

    describe({
      name: settleGateBallots.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a clear win for the rendering this run wrote',
          fn: async () => {
            expect(settleGateBallots({
              ballots: [
                { choice: 'consolidated', unsupported: [], unsupportedRaw: [], dropped: [], droppedRaw: [], reason: '', },
                { choice: 'consolidated', unsupported: [], unsupportedRaw: [], dropped: [], droppedRaw: [], reason: '', },
                { choice: 'standing', unsupported: [], unsupportedRaw: [], dropped: [], droppedRaw: [], reason: '', },
              ],
            },),).toBe('consolidated',);
          },
        },),
        it({
          name: 'REFUSES a one-voice majority, since one judge is an opinion',
          fn: async () => {
            expect(settleGateBallots({
              ballots: [
                { choice: 'consolidated', unsupported: [], unsupportedRaw: [], dropped: [], droppedRaw: [], reason: '', },
              ],
            },),).toBe('neither',);
          },
        },),
        it({
          name: 'REFUSES a tie rather than picking by list order',
          fn: async () => {
            expect(settleGateBallots({
              ballots: [
                { choice: 'consolidated', unsupported: [], unsupportedRaw: [], dropped: [], droppedRaw: [], reason: '', },
                { choice: 'consolidated', unsupported: [], unsupportedRaw: [], dropped: [], droppedRaw: [], reason: '', },
                { choice: 'standing', unsupported: [], unsupportedRaw: [], dropped: [], droppedRaw: [], reason: '', },
                { choice: 'standing', unsupported: [], unsupportedRaw: [], dropped: [], droppedRaw: [], reason: '', },
              ],
            },),).toBe('neither',);
          },
        },),
      ],
    },),

    describe({
      name: gateConsolidatedSlice.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a consolidation two voices back over one',
          fn: async () => {
            const outcome = await gate({
              replyByModel: [
                ballot({ choice: 'consolidated', },),
                ballot({ choice: 'consolidated', },),
                ballot({ choice: 'standing', },),
              ],
            },);
            expect(outcome.choice,).toBe('consolidated',);
            expect(outcome.ships,).toBe('consolidated',);
            expect(outcome.usable,).toBe(3,);
          },
        },),
        it({
          name: 'KEEPS the standing text when the roster refuses',
          fn: async () => {
            const outcome = await gate({
              replyByModel: [
                ballot({ choice: 'neither', },),
                ballot({ choice: 'neither', },),
                ballot({ choice: 'neither', },),
              ],
            },);
            expect(outcome.choice,).toBe('neither',);
            expect(outcome.ships,).toBe('standing',);
          },
        },),
        it({
          name: 'KEEPS the standing text on a tie, rather than churning the page',
          fn: async () => {
            const outcome = await gate({
              replyByModel: [
                ballot({ choice: 'consolidated', },),
                ballot({ choice: 'standing', },),
                ballot({ choice: 'neither', },),
              ],
            },);
            expect(outcome.ships,).toBe('standing',);
          },
        },),
        it({
          name: 'KEEPS the standing text when too few voices arrived to settle',
          fn: async () => {
            const outcome = await gate({
              replyByModel: [
                ballot({ choice: 'consolidated', },),
                'not json at all',
                'also not json',
              ],
            },);
            expect(outcome.usable,).toBe(1,);
            expect(outcome.ships,).toBe('standing',);
            expect(outcome.findings.length,).toBe(1,);
          },
        },),
        it({
          name: 'SAYS THE BENCH WAS SHORT when the router refused seats (ledger X8): three of five refused, two '
            + 'ballots settle the gate, and the finding records the reachable share the gathers already record',
          fn: async () => {
            const outcome = await gateConsolidatedSlice({
              fanOut: 'whole-bench',
              client: refusingClient({ refused: FIVE_SEATS.slice(0, 3,), },),
              modelIds: FIVE_SEATS,
              subject: SUBJECT,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(outcome.ships,).toBe('consolidated',);
            expect(outcome.findings,).toContain('stage-short-bench (consolidate-gate reachable 2 of 5, quorum 2)',);
          },
        },),
        it({
          name: 'KEEPS every usable ballot, including the ones that ship nothing',
          fn: async () => {
            const outcome = await gate({
              replyByModel: [
                ballot({ choice: 'neither', },),
                ballot({ choice: 'neither', },),
                ballot({ choice: 'standing', },),
              ],
            },);
            // A READER ASKING WHY A SLICE KEPT ITS TEXT looks exactly where a
            // record without ballots would be silent.
            expect(outcome.ballots.length,).toBe(3,);
          },
        },),
      ],
    },),
  ],
},);
