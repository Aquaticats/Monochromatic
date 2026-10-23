/**
 Tests for the lane contest's floor on the reachable bench: a round is not
 opened for a slice when the seats a wet provider would serve are fewer than
 the voices the contest needs to settle anything.

 XingZ6020 (2026-10-22) opened 63 rounds on a bench that read one seat of seven
 reachable; every round heard one ballot, settled on `neither`, and cost a
 NeuralWatt call (17.501 Wh in all) for a verdict the quorum could not give.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

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
  reachableQuorum,
  shortBenchStageFinding,
} from '@monochromatic-dev/translation-repair-stage';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { ballot, } from '@monochromatic-dev/translation-repair-test-fixture/ts/structural-ballot.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import {
  contestLaneSlice,
  LANE_CONTEST_QUORUM,
} from '../dist/final/node/index.mjs';

/**
 One contested slice, standing in for a corpus passage.
 */
const SUBJECT = {
  lineStructured: false,
  sourceText: '猫睡了。',
  incumbentText: 'The cat slept all afternoon in the sun.',
  repairText: 'The cat slept all afternoon.',
  translateText: 'The cat slept.',
};

/**
 Bench of four, enough for the floor to be met by two reachable seats.
 */
const BENCH = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
] as const;

/**
 Builds a client that answers every ask with a repair ballot and counts the asks.

 @param asked - sink for the seat of every call

 @returns Client over a scripted transport

 @example
 ```ts
 const client = countingClient({ asked: [], },);
 ```
 */
function countingClient({ asked, }: { readonly asked: string[]; },): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the lane contest',);
    },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      asked.push(request.modelId,);
      /**
       This seat's ballot.
       */
      const value: unknown = JSON.parse(ballot({ choice: 'repair', },),);
      if (!request.validate(value,))
        throw new Error('scripted lane contest ballot failed validation',);
      return {
        kind: 'ok',
        value,
        rawText: JSON.stringify(value,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by the lane contest',);
    },
  };
}

/**
 Runs one contest over the bench with a stated reachable count.

 @param asked - sink for the seat of every call

 @param lines - sink for every line the stage wrote

 @param reachable - seats a wet provider would serve, absent for a caller that did not count

 @returns What the roster settled on

 @example
 ```ts
 const outcome = await contestWith({ asked: [], lines: [], reachable: 1, },);
 ```
 */
async function contestWith(
  {
    asked,
    lines,
    reachable,
  }: {
    readonly asked: string[];
    readonly lines: string[];
    readonly reachable?: number;
  },
) {
  return await contestLaneSlice({
    // Whole bench: this case scripts every seat and reads over the bench it wrote.
    fanOut: 'whole-bench',
    client: countingClient({ asked, },),
    modelIds: BENCH,
    subject: SUBJECT,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    exchangeTimeoutMs: HANG_STOP_MS,
    l: capturingLogger({ messages: lines, },),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
    ...((reachable === undefined) ? {} : { reachable, }),
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: `${contestLaneSlice.name} floor on the reachable bench`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'OPENS NO ROUND when fewer seats are reachable than the voices the contest needs, and settles on '
            + 'neither with the reason in the findings (XingZ6020: 63 rounds on one reachable seat of seven)',
          fn: async () => {
            const asked: string[] = [];
            const lines: string[] = [];
            const outcome = await contestWith({ asked, lines, reachable: LANE_CONTEST_QUORUM - 1, },);
            expect({
              calls: asked.length,
              choice: outcome.choice,
              ballots: outcome.ballots.length,
              usable: outcome.usable,
              findings: outcome.findings,
            },).toEqual({
              calls: 0,
              choice: 'neither',
              ballots: 0,
              usable: 0,
              findings: [
                'lane-contest heard 0 usable ballots, below the 2 needed to settle',
                shortBenchStageFinding({
                  stage: 'lane-contest',
                  quorum: reachableQuorum({ benchSize: BENCH.length, unreachable: BENCH.length - 1, },),
                  benchSize: BENCH.length,
                },),
                'stage-bench-below-floor (lane-contest reachable 1 of 4, floor 2, no call made)',
              ],
            },);
          },
        },),
        it({
          name: 'SAYS ONCE WHY NO ROUND WAS OPENED, naming the reachable seats, the bench and the floor',
          fn: async () => {
            const lines: string[] = [];
            await contestWith({ asked: [], lines, reachable: 0, },);
            expect(
              lines.filter(function namesTheFloor(line,): boolean {
                return line.includes('below the floor',);
              },),
            ).toEqual([
              '[contestLaneSlice] lane contest: 0 of 4 seats reachable, below the floor of 2; no round is opened and the slice keeps its incumbent',
            ],);
          },
        },),
        it({
          name: 'ASKS THE BENCH AS BEFORE when the floor is met, and when the caller did not count',
          fn: async () => {
            const metAsked: string[] = [];
            const met = await contestWith({ asked: metAsked, lines: [], reachable: LANE_CONTEST_QUORUM, },);
            const uncountedAsked: string[] = [];
            const uncounted = await contestWith({ asked: uncountedAsked, lines: [], },);
            expect({
              metCalls: metAsked.length,
              metChoice: met.choice,
              uncountedCalls: uncountedAsked.length,
              uncountedChoice: uncounted.choice,
            },).toEqual({
              metCalls: BENCH.length,
              metChoice: 'repair',
              uncountedCalls: BENCH.length,
              uncountedChoice: 'repair',
            },);
          },
        },),
        it({
          name: 'MARKS THE OUTCOME OF A ROUND NEVER OPENED, and only that one, so the artifact tells it from a roster '
            + 'that was asked and heard nobody',
          fn: async () => {
            const skipped = await contestWith({ asked: [], lines: [], reachable: LANE_CONTEST_QUORUM - 1, },);
            const met = await contestWith({ asked: [], lines: [], reachable: LANE_CONTEST_QUORUM, },);
            const uncounted = await contestWith({ asked: [], lines: [], },);
            expect({
              skipped: skipped.benchBelowFloor,
              metHasKey: 'benchBelowFloor' in met,
              uncountedHasKey: 'benchBelowFloor' in uncounted,
            },).toEqual({
              skipped: true,
              metHasKey: false,
              uncountedHasKey: false,
            },);
          },
        },),
      ],
    },),
  ],
},);
