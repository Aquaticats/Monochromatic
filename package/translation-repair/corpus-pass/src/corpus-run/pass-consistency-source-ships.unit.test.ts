/**
 Tests that the consistency pass leaves a slice shipping the original of its passage alone.

 A page can carry the original of a passage nobody translated, under a note, at a slice the archive holds no text for
 (`page-check` `source-ships.ts`). The pass rewrites English, so it must neither write English over that slice nor
 carry the original into the English it rewrites at the slices around it: the original is the page writer's to place,
 and the pass reads the slice as one that carries no English.

 The page is the cat page with its last section cut from the archive, so the last slice is an anchor nobody filled and
 the artifact records the original of that slice in the translate lane's delivery rows.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source';
import type { ProjectedLanes, } from '@monochromatic-dev/translation-repair-artifact';
import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  UNTRANSLATED_PASSAGE_NOTE,
  type WouldShipSource,
  wouldShipTextPerSlice,
} from '@monochromatic-dev/translation-repair-page-check';
import {
  archiveEnglishOf,
  firstPlaceWins,
  preparedCatPage,
  projectedCatPage,
  scriptedJudgeClient,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import {
  SCRIPTED_STAGE_SETTINGS,
} from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import {
  consistencySlicesOf,
  runPassConsistency,
} from '../../dist/final/node/index.mjs';
import { DIGEST, } from './consistency-run.test-fixture.ts';

/**
 Index of the slice the archive holds no text for, which is the last of the cat page.
 */
const ANCHOR_INDEX = 3;

/**
 Words only the original of the last section holds, which must not reach any English.
 */
const ANCHOR_SOURCE_WORDS = '院子里的雨声';

/**
 Logger the pass writes to, whose lines are not under test.
 */
const l = tagged({ tag: 'pass-consistency-source-ships-test', },);

/**
 The cat page cut short: the archive stops before the last section, whose slice is an anchor, and the rows of both
 lanes say nobody filled it.

 @returns Preparation and projection of the lanes over it

 @example
 ```ts
 const { prepared, projected, } = pageWithUnfilledLastSection();
 ```
 */
function pageWithUnfilledLastSection(): {
  readonly prepared: PreparedDocumentPair;
  readonly projected: ProjectedLanes;
} {
  /**
   The page with every section in the archive.
   */
  const whole = preparedCatPage();
  /**
   The slices the archive still covers.
   */
  const covered = whole.slices.slice(0, ANCHOR_INDEX,);
  /**
   The last slice the archive covers, after which the archive stops.
   */
  const lastCovered = nonNullishOrThrow(covered.at(-1,),);
  /**
   The slice that loses its archive text.
   */
  const cut = nonNullishOrThrow(whole.slices[ANCHOR_INDEX],);
  /**
   Where the archive stops.
   */
  const stop = lastCovered.target.endOffset;
  /**
   The preparation with the last section cut from the archive.
   */
  const prepared: PreparedDocumentPair = {
    ...whole,
    targetText: whole.targetText.slice(0, stop,),
    slices: [
      ...covered,
      {
        source: cut.source,
        target: makeInsertionChunk({
          sliceIndex: ANCHOR_INDEX,
          offset: stop,
        },),
      },
    ],
  };
  /**
   The lanes over the page with the archive whole.
   */
  const projectedWhole = projectedCatPage({
    prepared: whole,
    english: archiveEnglishOf({ prepared: whole, },),
  },);
  return {
    prepared,
    projected: {
      comparison: projectedWhole.comparison.map(function unfilledAtAnchor(row,): typeof row {
        return (row.sliceIndex === ANCHOR_INDEX)
          ? {
            ...row,
            incumbentKind: 'absent',
            incumbentText: '',
            repairText: '',
            translateText: '',
            laneRelation: 'gap-remains',
            repairOutcome: { kind: 'unfilled', },
            translateOutcome: { kind: 'unfilled', },
            repairDelivery: { kind: 'gap-remains', },
            translateDelivery: { kind: 'gap-remains', },
          }
          : row;
      },),
      delivery: {
        repair: projectedWhole.delivery.repair.map(unfilledDeliveryAtAnchor,),
        translate: projectedWhole.delivery.translate.map(unfilledDeliveryAtAnchor,),
      },
    },
  };
}

/**
 Turns the delivery row of the last slice into one nobody filled and leaves the others.

 @param row - delivery row of one slice

 @returns Row of an unfilled anchor for the last slice, the row itself otherwise

 @example
 ```ts
 const rows = projected.delivery.translate.map(unfilledDeliveryAtAnchor,);
 ```
 */
function unfilledDeliveryAtAnchor(
  row: ProjectedLanes['delivery']['translate'][number],
): ProjectedLanes['delivery']['translate'][number] {
  return (row.sliceIndex === ANCHOR_INDEX)
    ? {
      ...row,
      incumbentKind: 'absent',
      incumbentText: '',
      outcome: { kind: 'unfilled', },
      shippedText: '',
      delivery: { kind: 'gap-remains', },
    }
    : row;
}

/**
 What the page checks read of the page: the rows of the lanes, the contest asked of nothing, and the original of every
 slice as the translate lane recorded it.

 @param projected - rows of both lanes

 @returns Artifact whose last slice ships its original

 @example
 ```ts
 const artifact = artifactOf({ projected, },);
 ```
 */
function artifactOf({ projected, }: { readonly projected: ProjectedLanes; },): WouldShipSource {
  return {
    comparison: projected.comparison,
    consolidation: { kind: 'not-run', },
    laneSelection: {
      kind: 'contested',
      slices: [],
    },
    lanes: {
      translate: {
        delivery: projected.delivery.translate.map(function toSource({
          sliceIndex,
          sourceText,
        },) {
          return {
            sliceIndex,
            sourceText,
          };
        },),
      },
    },
  };
}

await describe({
  name: 'consistency pass over a page that ships the original of a passage',
  children: [
    it({
      name: 'READS THE LAST SLICE AS SHIPPING ITS ORIGINAL, the positive control: the page writer would place the '
        + 'original there, so the other cases are about a slice that does ship it',
      fn: async () => {
        const { projected, } = pageWithUnfilledLastSection();
        const { reading, } = nonNullishOrThrow(
          wouldShipTextPerSlice({ artifact: artifactOf({ projected, },), },)
            .find(function isLast({ sliceIndex, },): boolean {
              return sliceIndex === ANCHOR_INDEX;
            },),
        );
        expect(reading.kind,).toBe('source-ships',);
        expect(reading.kind === 'source-ships' ? reading.text : '',).toContain(ANCHOR_SOURCE_WORDS,);
      },
    },),

    it({
      name: 'READS THAT SLICE AS CARRYING NO ENGLISH, with no page of its own, and puts neither the original nor the '
        + 'note into the page the other slices are read on',
      fn: async () => {
        const { prepared, projected, } = pageWithUnfilledLastSection();
        const facts = consistencySlicesOf({
          prepared,
          projected,
          contestSlices: [],
          consolidateSlices: [],
          archiveDisputes: new Map(),
        },);
        const anchor = nonNullishOrThrow(facts[ANCHOR_INDEX],);
        expect(anchor.current,).toEqual({ kind: 'absent', },);
        expect(anchor.page,).toBeUndefined();
        expect(anchor.sourceText,).toContain(ANCHOR_SOURCE_WORDS,);

        // Every other slice is read on the page the publishing would write, which holds the archive's English alone.
        const pageText = nonNullishOrThrow(nonNullishOrThrow(facts[0],).page,).text;
        expect(pageText,).not.toContain(ANCHOR_SOURCE_WORDS,);
        expect(pageText,).not.toContain(UNTRANSLATED_PASSAGE_NOTE,);
      },
    },),

    it({
      name: 'WRITES NO ENGLISH OVER THAT SLICE AND CARRIES NO ORIGINAL INTO THE ENGLISH IT REWRITES, so the slice '
        + 'still ships its original after the pass',
      fn: async () => {
        const { prepared, projected, } = pageWithUnfilledLastSection();
        await using dir = await scratchDir({ prefix: 'pass-consistency-source-ships-', },);
        const consistency = await runPassConsistency({
          client: scriptedJudgeClient({
            behave: function byQuestion(call,) {
              return firstPlaceWins({ sheet: call.sheet, },);
            },
            asked: [],
            sheets: [],
          },),
          prepared,
          projected,
          contestSlices: [],
          consolidateSlices: [],
          archiveDisputes: new Map(),
          entryCacheDir: dir.path,
          pipelineDigest: DIGEST,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          overlap: 2,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        // The positive control: the pass did write English elsewhere on the page, so the other cases are not about a
        // pass that wrote nothing.
        expect(consistency.slices.length,).toBeGreaterThan(0,);

        // The place the refrain has in that slice is left unlocated and never replaced.
        const outcomesAtAnchor = consistency.groups
          .flatMap(function placesOf({ occurrences, },) {
            return occurrences;
          },)
          .filter(function isAtAnchor({ sliceIndex, },): boolean {
            return sliceIndex === ANCHOR_INDEX;
          },)
          .map(function outcomeOf({ outcome, },): string {
            return outcome;
          },);
        expect(outcomesAtAnchor,).toEqual(['unlocated',],);

        // Nothing is written at the slice, and nothing written elsewhere holds the original or the note.
        expect(consistency.slices.map(function indexOf({ sliceIndex, },): number {
          return sliceIndex;
        },),).not.toContain(ANCHOR_INDEX,);
        for (const { text, } of consistency.slices) {
          expect(text,).not.toContain(ANCHOR_SOURCE_WORDS,);
          expect(text,).not.toContain(UNTRANSLATED_PASSAGE_NOTE,);
        }

        // The artifact with the pass recorded still reads that slice as shipping its original.
        const after = wouldShipTextPerSlice({
          artifact: {
            ...artifactOf({ projected, },),
            consistency,
          },
        },);
        expect(nonNullishOrThrow(after[ANCHOR_INDEX],).reading.kind,).toBe('source-ships',);
      },
    },),
  ],
},);
