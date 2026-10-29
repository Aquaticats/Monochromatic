/**
 Tests for asking the judge bench about one group and deciding it: a group the
 bench answers is decided with the places the chosen rendering is still to
 reach and nothing carried, a failure inside the bench is contained as a group
 left as it stands unless the caller has stopped the work, and a stopped caller
 gets the failure itself.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { repeatGroupsOf, } from '@monochromatic-dev/translation-repair-corpus-source';
import {
  consistencySlicesOf,
  decideAskedGroup,
  openConsistencyCache,
  placesOf,
  type PreparedGroup,
} from '../../dist/final/node/index.mjs';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_ONLY,
  SEAT_HYPER_VISION,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import {
  archiveEnglishOf,
  firstPlaceWins,
  PAGE_SOURCE,
  preparedCatPage,
  projectedCatPage,
  scriptedJudgeClient,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';
import { DIGEST, } from './consistency-run.test-fixture.ts';

/**
 Logger the bench writes to.
 */
const l = tagged({ tag: 'pass-consistency-bench-decide-test', },);

/**
 The judge bench the cases ask.
 */
const BENCH: readonly RosterModelId[] = [SEAT_HYPER_ONLY, SEAT_BEDROCK_ONLY_TEXT, SEAT_HYPER_VISION,];

/**
 The cat page's repeated sentence as the pass hands it to the bench.

 @returns First repeat group of the page with its places

 @example
 ```ts
 const prepared = sentenceGroup();
 ```
 */
function sentenceGroup(): PreparedGroup {
  const prepared = preparedCatPage();
  const slices = consistencySlicesOf({
    prepared,
    projected: projectedCatPage({ prepared, english: archiveEnglishOf({ prepared, },), },),
    contestSlices: [],
    consolidateSlices: [],
    archiveDisputes: new Map(),
  },);
  const group = nonNullishOrThrow(
    repeatGroupsOf({
      text: prepared.sourceText,
      slices: prepared.slices
        .map(function sourceOf(slice,) {
          return slice.source;
        },),
    },)[0],
  );
  return { group, places: placesOf({ group, slices, },), };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: decideAskedGroup.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'DECIDES A GROUP THE BENCH ANSWERS, naming the chosen place and the places the chosen rendering is still to '
            + 'reach, and carries nothing',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-bench-decide-', },);
            const decision = await decideAskedGroup({
              client: scriptedJudgeClient({ behave: firstPlaceWins, asked: [], sheets: [], },),
              modelIds: BENCH,
              text: PAGE_SOURCE,
              prepared: sentenceGroup(),
              cache: await openConsistencyCache({ dir: dir.path, generation: DIGEST, },),
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(decision.kind === 'decided'
              ? {
                kind: decision.kind,
                outcomes: decision.decided.outcomes,
                chosen: decision.decided.chosen.length,
                receivers: decision.decided.receivers.map(function labelOf({ label, },): string {
                  return label;
                },),
                edits: decision.decided.edits.length,
              }
              : { kind: decision.kind, },).toEqual({
                kind: 'decided',
                outcomes: ['chosen', 'kept-current',],
                chosen: 1,
                receivers: ['OCCURRENCE 2',],
                edits: 0,
              },);
          },
        },),
        it({
          name: 'CONTAINS A FAILURE INSIDE THE BENCH WHEN THE CALLER HAS NOT STOPPED THE WORK: a text that is not the page\'s '
            + 'leaves the group as it stands with the failure said, and asks nobody',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-bench-decide-', },);
            /**
             Seats the client was asked.
             */
            const asked: RosterModelId[] = [];
            const decision = await decideAskedGroup({
              client: scriptedJudgeClient({ behave: firstPlaceWins, asked, sheets: [], },),
              modelIds: BENCH,
              text: '',
              prepared: sentenceGroup(),
              cache: await openConsistencyCache({ dir: dir.path, generation: DIGEST, },),
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect({
              decision: decision.kind === 'failed'
                ? [decision.kind, decision.settled.verdict, decision.settled.outcomes,]
                : [decision.kind,],
              asked,
            },).toEqual({
              decision: [
                'failed',
                {
                  kind: 'failed',
                  detail: 'unreachable: the offsets of a place point at other words in the text shown than in the whole original '
                    + '(slice 0), and both are cut from the same offsets',
                },
                ['untouched', 'untouched',],
              ],
              asked: [],
            },);
          },
        },),
        it({
          name: 'THROWS THE FAILURE ITSELF WHEN THE CALLER HAS STOPPED THE WORK: the same text that leaves a group as it stands '
            + 'while the run goes on stops the pass once the signal has fired',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-bench-decide-', },);
            const cache = await openConsistencyCache({ dir: dir.path, generation: DIGEST, },);
            const refusal = await rejectionOf(async function decideAfterStop(): Promise<unknown> {
              return await decideAskedGroup({
                client: scriptedJudgeClient({ behave: firstPlaceWins, asked: [], sheets: [], },),
                modelIds: BENCH,
                text: '',
                prepared: sentenceGroup(),
                cache,
                signal: AbortSignal.abort(new Error('the cat woke the run',),),
                l,
                stageSettings: SCRIPTED_STAGE_SETTINGS,
              },);
            },);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: the offsets of a place point at other words in the text shown than in '
                + 'the whole original (slice 0), and both are cut from the same offsets',
            );
          },
        },),
      ],
    },),
  ],
},);
