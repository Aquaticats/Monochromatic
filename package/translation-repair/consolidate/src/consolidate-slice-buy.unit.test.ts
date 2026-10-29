/**
 Tests for when a single consolidation attempt ships a standing the lane
 contest never endorsed, with the non-endorsement recorded.

 The provider-identity anonymizer these cases once shared a file with built
 evidence for a stage-local recovery the single attempt ended (1ba8f713a);
 nothing called it, and it went on 2026-09-29 (ledger B30).

 Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { LaneText, } from '@monochromatic-dev/translation-repair-translate-floor';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  buyConsolidationSlice,
  type ConsolidationSettlement,
  standingKeptUnendorsed,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Writers and judges of the empty-standing cases.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 A settlement ending in one terminal, the only field the rule reads.

 @param terminal - how the attempt ended

 @returns Settlement cast past the fields the rule never reads
 */
function settlementEnding(
  { terminal, }: { readonly terminal: ConsolidationSettlement['terminal']; },
): ConsolidationSettlement {
  return {
    terminal,
    text: 'The cat sleeps.',
    findings: [],
  } as unknown as ConsolidationSettlement;
}

/**
 Marker only the consolidate gate's sheet carries.
 */
const GATE_MARKER = 'Return JSON: assessment one sentence';

/**
 The slice the empty-standing cases buy for.
 */
const SUBJECT = {
  sourceText: '猫在窗边睡着了。她四点醒来。',
  incumbentText: 'The cat fell asleep by the window and woke at four in the afternoon.',
  repairText: '',
  ballots: [],
  lineStructured: false,
} as const;

/**
 Builds a client that counts who was asked: the gate by its sheet, a slate
 judge by its ballot schema, and a producer as anything else, which it
 refuses so a producer asked would show.

 @returns Client and what it was asked, by role

 @example
 ```ts
 const { client, asked, } = countingClient();
 ```
 */
function countingClient(): {
  readonly client: ModelClient;
  readonly asked: { producer: number; judge: number; gate: number; };
} {
  /**
   Calls by role.
   */
  const asked = {
    producer: 0,
    judge: 0,
    gate: 0,
  };
  return {
    asked,
    client: {
      chatText: async () => {
        asked.producer += 1;
        throw new Error('a producer was asked over an empty standing',);
      },
      chatJson: async <ValueT,>(
        request: ChatJsonRequest<ValueT>,
      ): Promise<ChatJsonOutcome<ValueT>> => {
        /**
         Everything the call sends, where the gate's sheet shows.
         */
        const sent = JSON.stringify(request.messages,);
        /**
         Scripted reply for the role this call plays.
         */
        const value: unknown = sent.includes(GATE_MARKER,)
          ? { verdict: 'consolidated', unsupported: [], dropped: [], assessment: 'the lane text is faithful', }
          : (request.responseFormat?.json_schema.name === 'candidate_ballot')
            ? { best: 1, assessment: 'the only candidate reads well', }
            : undefined;
        if (value === undefined) {
          asked.producer += 1;
          throw new Error('a producer was asked over an empty standing',);
        }
        if (sent.includes(GATE_MARKER,))
          asked.gate += 1;
        else
          asked.judge += 1;
        if (!request.validate(value,))
          throw new Error('scripted reply failed validation',);
        return {
          kind: 'ok',
          value,
          rawText: JSON.stringify(value,),
        };
      },
      quotas: async () => {
        throw new Error('quotas unused by the consolidation',);
      },
    },
  };
}

/**
 Buys one slice over an empty standing.

 @param client - scripted client

 @param laneTexts - lane texts offered beside the standing

 @returns Settlement
 */
async function buyOverEmptyStanding(
  {
    client,
    laneTexts,
  }: {
    readonly client: ModelClient;
    readonly laneTexts: readonly LaneText[];
  },
): Promise<ConsolidationSettlement> {
  /**
   The translate lane's text, as the subject carries it.
   */
  const translateText = laneTexts.find(function isTranslate(laneText,): boolean {
    return laneText.lane === 'translate';
  },)?.text ?? '';
  return await buyConsolidationSlice({
    client,
    roster: ROSTER,
    judgeModelIds: ROSTER,
    subject: {
      ...SUBJECT,
      translateText,
    },
    standingText: '',
    lineStructured: false,
    sliceIndex: 0,
    standingMayShip: false,
    standingEligible: false,
    laneTexts,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l: tagged({ tag: 'consolidate-slice-buy-test', },),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: standingKeptUnendorsed.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES an unendorsed standing the attempt kept, and neither an endorsed one nor a consolidated '
            + 'settlement',
          fn: async () => {
            expect(standingKeptUnendorsed({
              settlement: settlementEnding({ terminal: 'gate-kept-standing', },),
              standingMayShip: false,
            },),).toBe(true,);
            expect(standingKeptUnendorsed({
              settlement: settlementEnding({ terminal: 'gate-kept-standing', },),
              standingMayShip: true,
            },),).toBe(false,);
            expect(standingKeptUnendorsed({
              settlement: settlementEnding({ terminal: 'consolidated', },),
              standingMayShip: false,
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: buyConsolidationSlice.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS NOBODY over an empty standing with no lane text, and settles on the slice having nothing to '
            + 'consolidate against',
          fn: async () => {
            const { client, asked, } = countingClient();
            const settled = await buyOverEmptyStanding({
              client,
              laneTexts: [],
            },);
            expect({
              terminal: settled.terminal,
              asked,
            },).toEqual({
              terminal: 'no-standing-text',
              asked: { producer: 0, judge: 0, gate: 0, },
            },);
          },
        },),
        it({
          name: 'ASKS NO PRODUCER over an empty standing, since producers write from it, and takes the lane text '
            + 'to the slate judges (class eighty-seven)',
          fn: async () => {
            const { client, asked, } = countingClient();
            const settled = await buyOverEmptyStanding({
              client,
              laneTexts: [{
                lane: 'translate',
                text: 'The cat fell asleep by the window.\nShe woke at four.',
              },],
            },);
            expect(asked.producer,).toBe(0,);
            expect(asked.judge,).toBeGreaterThan(0,);
            expect(settled.terminal,).not.toBe('no-standing-text',);
          },
        },),
      ],
    },),
  ],
},);
