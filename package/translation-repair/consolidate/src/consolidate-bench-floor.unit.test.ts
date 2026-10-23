/**
 Tests for the consolidation's floor on the reachable slate bench: a slice
 whose slate judges are fewer than the ballots a winner needs buys nothing,
 because every slate it could write would end with its winner named by one
 judge alone.

 XingZ6020 (2026-10-22) bought 54 such slices anyway: 164 NeuralWatt calls,
 39.746 Wh, and every slate settled on the standing it started from.

 Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  buyConsolidationSlice,
  type ConsolidationSettlement,
  consolidationWorthResuming,
} from '../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Writers and slate judges of the cases.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 The wording the slice stands on.
 */
const STANDING = 'The cat fell asleep by the window and woke at four in the afternoon.';

/**
 The slice the cases buy for.
 */
const SUBJECT = {
  sourceText: '猫在窗边睡着了。她四点醒来。',
  incumbentText: STANDING,
  repairText: '',
  translateText: '',
  ballots: [],
  lineStructured: false,
} as const;

/**
 Builds a client that refuses every call and counts them.

 @returns The client and its call counter

 @example
 ```ts
 const { client, calls, } = refusingClient();
 ```
 */
function refusingClient(): { readonly client: ModelClient; readonly calls: { count: number; }; } {
  /**
   Calls made so far, of any kind.
   */
  const calls = { count: 0, };
  return {
    calls,
    client: {
      chatText: async () => {
        calls.count += 1;
        throw new Error('scripted refusal',);
      },
      chatJson: async <ValueT,>(_request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
        calls.count += 1;
        throw new Error('scripted refusal',);
      },
      quotas: async () => {
        throw new Error('quotas unused by the consolidation',);
      },
    },
  };
}

/**
 Buys one slice on an eligible standing with a stated slate reach.

 @param client - scripted client

 @param lines - sink for every line written

 @param slateReachable - slate judges a wet provider would serve, absent when not counted

 @param standingEligible - whether the standing passed the deterministic gate

 @param standingMayShip - whether the contest endorsed the standing

 @returns The settlement

 @example
 ```ts
 const settlement = await buyWithReach({ client, lines: [], slateReachable: 1, },);
 ```
 */
async function buyWithReach(
  {
    client,
    lines,
    slateReachable,
    standingEligible = true,
    standingMayShip = false,
  }: {
    readonly client: ModelClient;
    readonly lines: string[];
    readonly slateReachable?: number;
    readonly standingEligible?: boolean;
    readonly standingMayShip?: boolean;
  },
): Promise<ConsolidationSettlement> {
  return await buyConsolidationSlice({
    client,
    roster: ROSTER,
    judgeModelIds: ROSTER,
    subject: SUBJECT,
    standingText: STANDING,
    lineStructured: false,
    sliceIndex: 7,
    standingMayShip,
    standingEligible,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l: tagged({ tag: 'consolidate-bench-floor-test', l: capturingLogger({ messages: lines, },), },),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
    ...((slateReachable === undefined) ? {} : { slateReachable, }),
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: `${buyConsolidationSlice.name} floor on the reachable slate bench`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'BUYS NOTHING for an eligible standing when fewer slate judges are reachable than the ballots a '
            + 'winner needs, keeps the standing with the reason in the findings, and says so once (XingZ6020: '
            + '54 slices, 164 calls)',
          fn: async () => {
            const { client, calls, } = refusingClient();
            const lines: string[] = [];
            const settlement = await buyWithReach({ client, lines, slateReachable: 1, },);
            expect({
              calls: calls.count,
              terminal: settlement.terminal,
              text: settlement.text,
              benchBelowFloor: settlement.benchBelowFloor,
              floorFinding: settlement.findings.includes(
                'stage-bench-below-floor (consolidate-slate reachable 1 of 3, floor 2, no call made)',
              ),
              said: lines.filter(function namesTheFloor(line,): boolean {
                return line.includes('below the floor',);
              },),
            },).toEqual({
              calls: 0,
              terminal: 'incumbent-only',
              text: STANDING,
              benchBelowFloor: true,
              floorFinding: true,
              said: [
                '[consolidate-bench-floor-test] slice 7: 1 of 3 slate judges reachable, '
                + 'below the floor of 2; no slate is bought and the standing is kept',
              ],
            },);
          },
        },),
        it({
          name: 'NEVER KEEPS SUCH A SETTLEMENT FOR A LATER RUN, even where the standing is endorsed, since a later '
            + 'pass on a healthier bench must ask the slice again',
          fn: async () => {
            const { client, } = refusingClient();
            const settlement = await buyWithReach({
              client,
              lines: [],
              slateReachable: 1,
              standingMayShip: true,
            },);
            expect(consolidationWorthResuming({ settlement, standingMayShip: true, },),).toBe(false,);
          },
        },),
        it({
          name: 'ASKS AS BEFORE once the slate bench meets the floor, when the caller did not count, and when the '
            + 'standing is ineligible, since an ineligible standing is withheld and its slate may still ship',
          fn: async () => {
            const met = refusingClient();
            const uncounted = refusingClient();
            const ineligible = refusingClient();
            await buyWithReach({ client: met.client, lines: [], slateReachable: 2, },).catch(() => undefined);
            await buyWithReach({ client: uncounted.client, lines: [], },).catch(() => undefined);
            await buyWithReach({
              client: ineligible.client,
              lines: [],
              slateReachable: 1,
              standingEligible: false,
            },).catch(() => undefined);
            expect({
              metAsked: met.calls.count > 0,
              uncountedAsked: uncounted.calls.count > 0,
              ineligibleAsked: ineligible.calls.count > 0,
            },).toEqual({
              metAsked: true,
              uncountedAsked: true,
              ineligibleAsked: true,
            },);
          },
        },),
      ],
    },),
  ],
},);
