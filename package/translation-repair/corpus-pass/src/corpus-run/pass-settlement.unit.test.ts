/**
 Tests for the two stages that settle what the page says, in the order they
 run: the third rendering slice by slice, then the page-level consistency of
 the slices it settled. One call hands the entry pipeline both results.

 WHAT IS UNDER TEST is the hand-over. The consolidation runs over scripted
 judges that endorse one new rendering for the second slice, and the cases read
 that the consistency pass is shown that rendering, is told what the lane contest
 settled, runs after the last consolidation call, and that the settlement
 carries both results as the stages returned them.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ProjectedLanes, } from '@monochromatic-dev/translation-repair-artifact';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
} from '@monochromatic-dev/translation-repair-provider';
import type { ArtifactContestSlice, } from '@monochromatic-dev/translation-repair-lane-contest';
import {
  runPassConsolidation,
  runPassSettlement,
} from '../../dist/final/node/index.mjs';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import {
  archiveEnglishOf,
  preparedCatPage,
  projectedCatPage,
  scriptedJudgeClient,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';
import { DIGEST, } from './consistency-run.test-fixture.ts';

/**
 Logger the stages write to, whose lines are not under test.
 */
const l = tagged({ tag: 'pass-settlement-test', },);

/**
 The wording the scripted writers give the second slice, which differs from the archive's by two words.
 */
const REWORDED = '## Two\n\nThe smell of fish in the kitchen woke the kitten up. '
  + 'So this cat is surely a lazy little treasure.';

/**
 The second slice as the consolidation ships it: the wording of `REWORDED`, wrapped at the sentence.
 */
const CONSOLIDATED = '## Two\n\nThe smell of fish in the kitchen woke the kitten up.\n'
  + 'So this cat is surely a lazy little treasure.';

/**
 What the scripted writers give each slice, by the heading its original carries.
 */
const WRITTEN: readonly string[] = [
  '## One\n\nThe sunny window put the kitten to sleep. So this cat must be a chubby little treasure.',
  REWORDED,
  '## Three\n\nIn the evening, the sunlight on the sill lulled the kitten to sleep, and the fish snacks smiled too. '
    + 'So this cat has to be a sweet little darling.',
  '## Four\n\nThe rain in the yard sent the kitten running home. So this cat is certainly a warm little dumpling.',
];

/**
 The headings of the cat page's original, in slice order.
 */
const HEADINGS: readonly string[] = ['## 一', '## 二', '## 三', '## 四',];

/**
 What the scripted judges reply to a stage, by the name of the schema it asks for.

 @param schema - schema name the stage asked for

 @param sheet - what the stage sent

 @returns Wire value for that stage

 @example
 ```ts
 const value = replyFor({ schema: 'refine_report', sheet, },);
 ```
 */
function replyFor({ schema, sheet, }: { readonly schema: string; readonly sheet: string; },): unknown {
  if (schema === 'translation_report') {
    return {
      translation: WRITTEN[HEADINGS.findIndex(function namedIn(heading,): boolean {
        return sheet.includes(heading,);
      },)] ?? 'none',
    };
  }
  if (schema === 'candidate_ballot')
    return { best: 2, assessment: 'The new rendering reads better.', };
  if (schema === 'consolidate_gate')
    return { verdict: 'consolidated', unsupported: [], dropped: [], assessment: 'The new rendering stands.', };
  if (schema === 'refine_report')
    return { rewrites: [], };
  if (schema === 'absolute_naturalness_review')
    return { isAcceptable: true, findings: [], assessment: 'The whole page reads well.', };
  if (schema === 'page_consistency')
    return { reason: 'No place needs to follow another.', quotes: [], choice: 'OCCURRENCE 1', keep: [], };
  throw new Error(`no script for ${schema}`,);
}

/**
 A run client whose judges reply as scripted, recording every schema asked and every sheet shown.

 @param calls - sink for the schema and sheet of every structured call, in order

 @returns Client to settle the page with

 @example
 ```ts
 const client = settlementClient({ calls: [], },);
 ```
 */
function settlementClient(
  { calls, }: { readonly calls: { readonly schema: string; readonly sheet: string; }[]; },
): ReturnType<typeof scriptedJudgeClient> {
  return {
    ...scriptedJudgeClient({ behave: function never(): never { throw new Error('the structured calls are scripted here',); }, asked: [], sheets: [], },),
    chatJson: function chatJson<ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> {
      /**
       Name of the schema the stage asked for.
       */
      const schema = request.responseFormat?.json_schema.name ?? '';
      /**
       What the stage sent, as one text.
       */
      const sheet = request.messages
        .map(function textOf(message,): string {
          return ((typeof message.content) === 'string') ? message.content : '';
        },)
        .join('\n',);
      calls.push({ schema, sheet, },);
      /**
       Reply as the stage parses it.
       */
      const value: unknown = replyFor({ schema, sheet, },);
      return Promise.resolve(request.validate(value,)
        ? { kind: 'ok', value, rawText: JSON.stringify(value,), }
        : { kind: 'schema-mismatch', rawText: JSON.stringify(value,), detail: 'the scripted reply is not usable', },);
    },
  };
}

/**
 The cat page prepared and projected with the archive standing in every slice.

 @returns Preparation, both lanes' rows, and the contest's record for every slice

 @example
 ```ts
 const { prepared, projected, contests, } = catPage();
 ```
 */
function catPage(): {
  readonly prepared: PreparedDocumentPair;
  readonly projected: ProjectedLanes;
  readonly contests: readonly ArtifactContestSlice[];
} {
  const prepared = preparedCatPage();
  return {
    prepared,
    projected: projectedCatPage({ prepared, english: archiveEnglishOf({ prepared, },), },),
    contests: prepared.slices
      .map(function wonByRepair(slice,): ArtifactContestSlice {
        return {
          sliceIndex: slice.target.sliceIndex,
          verdict: { kind: 'lane-won', lane: 'repair', },
          ballots: [],
          usable: 3,
        };
      },),
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: runPassSettlement.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHOWS THE CONSISTENCY PASS THE RENDERING THE CONSOLIDATION SETTLED, not the archive\'s wording of the slice',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-settlement-', },);
            const calls: { schema: string; sheet: string; }[] = [];
            const settlement = await runPassSettlement({
              client: settlementClient({ calls, },),
              ...catPage(),
              archiveDisputes: new Map(),
              frontMatterSlices: new Set(),
              pictureReadings: new Map(),
              entryCacheDir: dir.path,
              pipelineDigest: DIGEST,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              overlap: 2,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(settlement.consolidateSlices.map(function shippedOf(slice,): unknown {
              return [slice.sliceIndex, slice.shipped,];
            },),).toEqual([
              [0, { kind: 'unchanged', },],
              [1, { kind: 'consolidated', text: CONSOLIDATED, },],
              [2, { kind: 'unchanged', },],
              [3, { kind: 'unchanged', },],
            ],);
            expect(settlement.consistency.groups.length,).toBe(2,);
            const consistencySheets = calls.filter(function asksConsistency(call,): boolean {
              return call.schema === 'page_consistency';
            },);
            const refrainSheets = consistencySheets.filter(function showsRefrain(call,): boolean {
              return call.sheet.includes('FRAME BEGINS WITH',);
            },);
            expect(refrainSheets.length > 0,).toBe(true,);
            expect(refrainSheets.map(function showsEnglish(call,): readonly boolean[] {
              return [call.sheet.includes('woke the kitten up.',), call.sheet.includes('woke the kitten. So',),];
            },),).toEqual(refrainSheets.map(function consolidatedNotArchive(): readonly boolean[] {
              return [true, false,];
            },),);
          },
        },),
        it({
          name: 'TELLS THE CONSISTENCY PASS WHAT THE LANE CONTEST SETTLED, so a slice where the lanes differ ships the lane that won',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-settlement-', },);
            const page = catPage();
            const calls: { schema: string; sheet: string; }[] = [];
            const settlement = await runPassSettlement({
              client: settlementClient({ calls, },),
              ...page,
              projected: {
                ...page.projected,
                comparison: page.projected.comparison
                  .map(function lanesDifferAtTheFourth(row,) {
                    return (row.sliceIndex === 3)
                      ? {
                        ...row,
                        laneRelation: 'both-differ' as const,
                        translateText: '## Four\n\nThe rain in the yard sent the kitten home. So this cat is a warm little dumpling.',
                      }
                      : row;
                  },),
              },
              archiveDisputes: new Map(),
              frontMatterSlices: new Set(),
              pictureReadings: new Map(),
              entryCacheDir: dir.path,
              pipelineDigest: DIGEST,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              overlap: 2,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(settlement.consistency.findings.filter(function stopped(finding,): boolean {
              return finding.startsWith('the consistency pass stopped',);
            },),).toEqual([],);
            expect(settlement.consistency.groups.length,).toBe(2,);
          },
        },),
        it({
          name: 'ASKS THE CONSISTENCY BENCH ONLY AFTER THE LAST CONSOLIDATION CALL',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'pass-settlement-', },);
            const calls: { schema: string; sheet: string; }[] = [];
            await runPassSettlement({
              client: settlementClient({ calls, },),
              ...catPage(),
              archiveDisputes: new Map(),
              frontMatterSlices: new Set(),
              pictureReadings: new Map(),
              entryCacheDir: dir.path,
              pipelineDigest: DIGEST,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              overlap: 2,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            const schemas = calls.map(function schemaOf(call,): string {
              return call.schema;
            },);
            expect(schemas.indexOf('page_consistency',) > schemas.lastIndexOf('absolute_naturalness_review',),).toBe(true,);
            expect(schemas.indexOf('page_consistency',) > schemas.lastIndexOf('translation_report',),).toBe(true,);
          },
        },),
        it({
          name: 'CARRIES THE CONSOLIDATION\'S RECORDS AS THE STAGE ALONE RETURNS THEM',
          fn: async () => {
            await using together = await scratchDir({ prefix: 'pass-settlement-', },);
            await using alone = await scratchDir({ prefix: 'pass-settlement-', },);
            const page = catPage();
            const settlement = await runPassSettlement({
              client: settlementClient({ calls: [], },),
              ...page,
              archiveDisputes: new Map(),
              frontMatterSlices: new Set(),
              pictureReadings: new Map(),
              entryCacheDir: together.path,
              pipelineDigest: DIGEST,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              overlap: 2,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(settlement.consolidateSlices,).toEqual(await runPassConsolidation({
              client: settlementClient({ calls: [], },),
              ...page,
              frontMatterSlices: new Set(),
              pictureReadings: new Map(),
              entryCacheDir: alone.path,
              pipelineDigest: DIGEST,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              overlap: 2,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },),);
          },
        },),
      ],
    },),
  ],
},);
