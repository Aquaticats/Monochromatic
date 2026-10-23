/**
 Tests for asking the judge bench about every group that needs it: nobody to
 ask reads no seats, a group whose handling breaks outside the bench's own
 containment stops the pass with the failure of the first, the failure of each
 other group is written under the name of the group asked, a failure inside the
 bench is contained unless the caller has stopped the work, and the writers are
 asked only when the seat reading left a writer bench to ask.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { repeatGroupsOf, } from '@monochromatic-dev/translation-repair-corpus-source';
import {
  askAll,
  consistencySlicesOf,
  placesOf,
  type PreparedGroup,
  writersOf,
} from '../../dist/final/node/index.mjs';
import { warnLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import {
  archiveEnglishOf,
  firstPlaceWins,
  PAGE_SOURCE,
  preparedCatPage,
  projectedCatPage,
  scriptedJudgeClient,
  type SeatBehaviour,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';
import { DIGEST, } from './consistency-run.test-fixture.ts';
import { sheetKindOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/consistency-sheet-reading.test-fixture.ts';

/**
 Logger the bench writes to.
 */
const l = tagged({ tag: 'pass-consistency-bench-test', },);

/**
 A group that lists no place, which the pass never builds and the bench cannot
 word.

 @returns The group with its places, both empty

 @example
 ```ts
 const broken = placelessGroup();
 ```
 */
function placelessGroup(): PreparedGroup {
  return {
    group: { kind: 'maximal', length: 12, occurrences: [], unplaced: [], },
    places: [],
  };
}

/**
 The cat page's groups as the pass hands them to the bench, in page order.

 @returns One prepared group for each repeat group of the page

 @example
 ```ts
 const [first,] = catGroups();
 ```
 */
function catGroups(): readonly PreparedGroup[] {
  const prepared = preparedCatPage();
  const slices = consistencySlicesOf({
    prepared,
    projected: projectedCatPage({ prepared, english: archiveEnglishOf({ prepared, },), },),
    contestSlices: [],
    consolidateSlices: [],
    archiveDisputes: new Map(),
  },);
  return repeatGroupsOf({
    text: prepared.sourceText,
    slices: prepared.slices
      .map(function sourceOf(slice,) {
        return slice.source;
      },),
  },)
    .map(function preparedOf(group,): PreparedGroup {
      return { group, places: placesOf({ group, slices, },), };
    },);
}

/**
 Consecutive equal labels counted as one run each, in order.

 @param labels - the labels in the order they came

 @returns Each run as its label and how many times it came in a row

 @example
 ```ts
 const runs = runsOf({ labels: ['decide', 'decide', 'carry',], },); // [['decide', 2], ['carry', 1]]
 ```
 */
function runsOf({ labels, }: { readonly labels: readonly string[]; },): readonly (readonly [string, number])[] {
  /**
   The runs counted so far.
   */
  const runs: [string, number][] = [];
  for (const label of labels) {
    /**
     The run the label may continue.
     */
    const last = runs.at(-1,);
    if (last?.[0] === label)
      last[1] += 1;
    else
      runs.push([label, 1,],);
  }
  return runs;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: writersOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS THE WRITER BENCH AS THE SEAT READING LEFT IT when no writing bench is below its floor',
          fn: async () => {
            expect(writersOf({ translators: ['glm-5.3', 'minimax-m3',], unreachableWriters: [], },),).toEqual({
              kind: 'asked',
              modelIds: ['glm-5.3', 'minimax-m3',],
            },);
          },
        },),
        it({
          name: 'DOES NOT ASK THE WRITERS WHEN A WRITING BENCH IS BELOW ITS FLOOR, naming each such bench and saying no '
            + 'provider named its return, even when the reading still lists translators',
          fn: async () => {
            expect([
              writersOf({
                translators: ['glm-5.3', 'minimax-m3',],
                unreachableWriters: ['translators 0 of 8 reachable, floor 2',],
              },),
              writersOf({
                translators: [],
                unreachableWriters: ['translators 1 of 8 reachable, floor 2', 'editors 0 of 4 reachable, floor 2',],
              },),
            ],).toEqual([
              {
                kind: 'not-asked',
                why: 'translators 0 of 8 reachable, floor 2; no provider has named a return the pass can wait for',
              },
              {
                kind: 'not-asked',
                why: 'translators 1 of 8 reachable, floor 2; editors 0 of 4 reachable, floor 2; '
                  + 'no provider has named a return the pass can wait for',
              },
            ],);
          },
        },),
        it({
          name: 'DOES NOT ASK THE WRITERS WHEN THE BENCH SEATS NO MODEL, whatever the reading said of the floor',
          fn: async () => {
            expect(writersOf({ translators: [], unreachableWriters: [], },),).toEqual({
              kind: 'not-asked',
              why: 'the translators bench seats no model',
            },);
          },
        },),
      ],
    },),
    describe({
      name: askAll.name,
      concurrency: 1,
      children: [
        it({
          name: 'ASKS NOBODY AND READS NO SEATS FOR NO GROUP TO ASK, since a page that needs no judge waits out no hold',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-bench-', },);
            /**
             Seats the client was asked.
             */
            const asked: RosterModelId[] = [];
            expect(await askAll({
              client: scriptedJudgeClient({ behave: firstPlaceWins, asked, sheets: [], },),
              text: PAGE_SOURCE,
              toAsk: [],
              declared: [],
              declaredNames: [],
              entryCacheDir: dir.path,
              pipelineDigest: DIGEST,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              overlap: 2,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },),).toEqual([],);
            expect(asked,).toEqual([],);
          },
        },),
        it({
          name: 'DECIDES EVERY GROUP BEFORE ANY GROUP IS CARRIED: with one group in flight at a time, every ballot of both '
            + 'groups is asked before the first writer or choice sheet, and no ballot follows a carry',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-bench-', },);
            /**
             The kind of every sheet the client was shown, in the order the calls were made.
             */
            const kinds: string[] = [];
            await askAll({
              client: scriptedJudgeClient({
                behave: function recordKind({ sheet, },): SeatBehaviour {
                  kinds.push(sheetKindOf({ sheet, },),);
                  return firstPlaceWins({ sheet, },);
                },
                asked: [],
                sheets: [],
              },),
              text: PAGE_SOURCE,
              toAsk: catGroups(),
              declared: [],
              declaredNames: [],
              entryCacheDir: dir.path,
              pipelineDigest: DIGEST,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              overlap: 1,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect({
              phases: runsOf({
                labels: kinds.map(function phaseOf(kind,): string {
                  return ((kind === 'ballot') || (kind === 'run-off')) ? 'decide' : 'carry';
                },),
              },),
              ballots: kinds.filter(function isBallot(kind,): boolean {
                return kind === 'ballot';
              },).length,
            },).toEqual({
              phases: [['decide', 10,], ['carry', 40,],],
              ballots: 10,
            },);
          },
        },),
        it({
          name: 'THROWS THE FIRST GROUP\'S FAILURE AND WRITES THE SECOND\'S UNDER ITS NAME when two groups break outside '
            + 'the bench\'s containment',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-bench-', },);
            const { result: refusal, warned, } = await warnLinesDuring({
              run: async function askBoth(): Promise<string> {
                try {
                  await askAll({
                    client: scriptedJudgeClient({ behave: firstPlaceWins, asked: [], sheets: [], },),
                    text: PAGE_SOURCE,
                    toAsk: [placelessGroup(), placelessGroup(),],
                    declared: [],
                    declaredNames: [],
                    entryCacheDir: dir.path,
                    pipelineDigest: DIGEST,
                    signal: AbortSignal.timeout(HANG_STOP_MS,),
                    overlap: 2,
                    l,
                    stageSettings: SCRIPTED_STAGE_SETTINGS,
                  },);
                }
                catch (error) {
                  return String(error,);
                }
                return 'settled';
              },
            },);
            expect(refusal,).toBe(
              'ConsistencyInvariantError: unreachable: a repeat group lists no place, and the detector reports a repeat only '
                + 'where it stands in two',
            );
            expect(warned.some(function namesTheSecond(line,): boolean {
              return line.includes(
                'group 2 asked failed (unreachable: a repeat group lists no place, and the detector reports a repeat only where '
                  + "it stands in two), and the failure thrown is group 1 asked's",
              );
            },),).toBe(true,);
          },
        },),
        it({
          name: 'CONTAINS A FAILURE INSIDE THE BENCH WHEN THE CALLER HAS NOT STOPPED THE WORK: a text that is not the page\'s '
            + 'settles the group as failed with its places left as they stand, and asks nobody',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-bench-', },);
            /**
             Seats the client was asked.
             */
            const asked: RosterModelId[] = [];
            const settled = await askAll({
              client: scriptedJudgeClient({ behave: firstPlaceWins, asked, sheets: [], },),
              text: '',
              toAsk: [nonNullishOrThrow(catGroups()[0],),],
              declared: [],
              declaredNames: [],
              entryCacheDir: dir.path,
              pipelineDigest: DIGEST,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              overlap: 2,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect({
              verdicts: settled.map(function verdictOf({ verdict, },): unknown {
                return verdict.kind === 'failed' ? [verdict.kind, verdict.detail,] : verdict.kind;
              },),
              outcomes: settled.map(function outcomesOf({ outcomes, },): readonly string[] {
                return outcomes;
              },),
              edits: settled.map(function editsOf({ edits, },): number {
                return edits.length;
              },),
              asked,
            },).toEqual({
              verdicts: [
                [
                  'failed',
                  'unreachable: the offsets of a place point at other words in the text shown than in the whole original '
                    + '(slice 0), and both are cut from the same offsets',
                ],
              ],
              outcomes: [['untouched', 'untouched',],],
              edits: [0,],
              asked: [],
            },);
          },
        },),
        it({
          name: 'THROWS THE FAILURE ITSELF INSTEAD OF CONTAINING IT WHEN THE CALLER HAS STOPPED THE WORK: the same text that '
            + 'settles a group as failed while the run goes on stops the pass once the signal has fired',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-consistency-bench-', },);
            const refusal = await rejectionOf(async function askAfterStop(): Promise<unknown> {
              return await askAll({
                client: scriptedJudgeClient({ behave: firstPlaceWins, asked: [], sheets: [], },),
                text: '',
                toAsk: [nonNullishOrThrow(catGroups()[0],),],
                declared: [],
                declaredNames: [],
                entryCacheDir: dir.path,
                pipelineDigest: DIGEST,
                signal: AbortSignal.abort(new Error('the cat woke the run',),),
                overlap: 2,
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
