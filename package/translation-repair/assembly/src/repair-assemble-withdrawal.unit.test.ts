/**
 Tests for what assembly SAYS when it takes a repair back.

 WHY THIS MATTERS ENOUGH TO PIN. The assembly guard is the only layer that can
 see a footnote, because a footnote is a relation BETWEEN slices and every
 other stage works inside one. When it withdraws a repair the run already paid
 for, the single line it logs is the only place an operator watching a pass
 learns that it happened; the findings say why, and this says that.

 WHAT WAS MEASURED. On 2026-08-25, loosening the guard on that warning so it
 fires on every assembly, withdrawal or not, failed no test in this package.
 An operator would then read `withdrew 0 slice repairs` on every clean entry
 of a corpus pass, which is how a real withdrawal stops being noticed.

 THE SILENT CASE IS THE ONE THAT PINS IT. Asserting the warning fires when a
 repair is withdrawn cannot see that loosening; asserting nothing is said when
 none is withdrawn is what catches it, and the noisy case is what proves the
 quiet one is not simply a run where the guard never looked.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  assembleRepair,
  type ChunkRepairOutcome,
} from '../dist/final/node/index.mjs';
import { warningRecordingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { sliceBuilderFor, } from '@monochromatic-dev/translation-repair-slice-reading/ts/repair-assemble-slice.test-fixture.ts';
import { SEAT_HYPER_OPENROUTER_VISION_EDITOR, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Logger every case replaces with one that keeps what it was told.
 */
const l = tagged({ tag: 'repair-assemble-withdrawal-test', },);

//region Fixtures

/**
 Archive wording of the slice carrying the footnote reference.
 */
const REFERENCING_SLICE = 'The cat naps on the windowsill.[^1]';

/**
 Archive wording of the slice carrying its definition.
 */
const DEFINING_SLICE = '[^1]: She has done so since the spring.';

/**
 Repair that keeps the reference, which the guard has no reason to withdraw.
 */
const KEPT_REPAIR = 'Mittens naps on the windowsill through the afternoon.[^1]';

/**
 Repair that drops the reference, orphaning a definition the archive resolves.
 */
const ORPHANING_REPAIR = 'Mittens naps on the windowsill through the afternoon.';

/**
 Document the slice offsets address.
 */
const TARGET_TEXT = `${REFERENCING_SLICE}\n\n${DEFINING_SLICE}`;

/**
 Prepares one slice over the span of {@link TARGET_TEXT}.
 */
const sliceOf = sliceBuilderFor({ targetText: TARGET_TEXT, },);

/**
 Slices every case assembles.
 */
const SLICES: readonly ChunkPair[] = [
  sliceOf({
    sliceIndex: 0,
    text: REFERENCING_SLICE,
  },),
  sliceOf({
    sliceIndex: 1,
    text: DEFINING_SLICE,
  },),
];

/**
 Both slice indices, marked line-structured so the wrap leaves the fixture
 wording byte-identical and the guard reads exactly what this file wrote.
 */
const LINE_STRUCTURED: ReadonlySet<number> = new Set([
  0,
  1,
],);

/**
 Builds one settled outcome.

 @param sliceIndex - slice this outcome belongs to

 @param repairedText - wording the lane settled on

 @param changed - whether that wording replaces the archive

 @returns Outcome assembly reads

 @example
 ```ts
 const outcome = outcomeOf({ sliceIndex: 0, repairedText: KEPT_REPAIR, changed: true, },);
 ```
 */
function outcomeOf(
  {
    sliceIndex,
    repairedText,
    changed,
  }: {
    readonly sliceIndex: number;
    readonly repairedText: string;
    readonly changed: boolean;
  },
): ChunkRepairOutcome {
  return {
    sliceIndex,
    repairedText,
    changed,
    issues: [],
    resolvedIssueIds: [],
    candidateResolvedIssueIds: [],
    // No checker round in this fixture, so nothing was said about any issue.
    checkerReadings: {},
    recheckReadings: {},
    repairRegions: [],
    accuracyPatchSelected: false,
    refined: false,
    rounds: [],
    droppedDeclaredNames: [],
    // Hand-written fixture text, so nothing here has a model author.
    authorship: {
      perIssue: {},
      everyIssue: [],
    },
    nonTranslationVotes: 0,
    nonTranslationContradicted: false,
    nonTranslationStanding: false,
    heardCritics: 1,
    // NAMED, not just counted: the lane refuses a slice it heard nobody about
    // that carries anything but the archive's wording, and it reads the id list
    // rather than the count.
    heardCriticIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
    claimAttributions: [],
    findings: [],
    silentStages: [],
  };
}

/**
 Assembles one repair of the referencing slice.

 @param repairedText - wording that slice settled on

 @returns Result plus every warning assembly emitted

 @example
 ```ts
 const { result, warnings, } = assembleWith({ repairedText: KEPT_REPAIR, },);
 ```
 */
function assembleWith(
  { repairedText, }: { readonly repairedText: string; },
): {
  readonly result: ReturnType<typeof assembleRepair>;
  readonly warnings: readonly string[];
} {
  const warnings: string[] = [];
  const logger = warningRecordingLogger({ base: l, warnings, },);

  return {
    result: assembleRepair({
      targetText: TARGET_TEXT,
      slices: SLICES,
      outcomes: [
        outcomeOf({
          sliceIndex: 0,
          repairedText,
          changed: true,
        },),
        outcomeOf({
          sliceIndex: 1,
          repairedText: DEFINING_SLICE,
          changed: false,
        },),
      ],
      lineStructuredSlices: LINE_STRUCTURED,
      findings: [],
      l: logger,
    },),
    warnings,
  };
}

//endregion Fixtures

await describe({
  name: assembleRepair.name,
  children: [
    it({
      name: 'SAYS SO WHEN IT TAKES A REPAIR BACK, naming how many and then each finding, since those '
        + 'lines are where an operator learns that work the run already paid for did not reach the reader '
        + 'and why (the twenty-first hakureico pass of 2026-09-09 left only the count)',
      fn: async () => {
        const { result, warnings, } = assembleWith({ repairedText: ORPHANING_REPAIR, },);

        expect(result.withdrawnSliceIndices,).toEqual([ 0, ],);
        expect(warnings,).toHaveLength(2,);
        expect(warnings[0],).toBe('withdrew 1 slice repair at assembly; the findings say why',);
        expect(warnings[1],).toContain('assembly-footnote-reverted',);
      },
    },),
    it({
      name: 'STAYS QUIET when it withdrew nothing, which is what keeps the withdrawal announcement worth reading: a '
        + 'guard announcing every clean assembly is how a real withdrawal stops being noticed',
      fn: async () => {
        const { result, warnings, } = assembleWith({ repairedText: KEPT_REPAIR, },);

        expect(result.withdrawnSliceIndices,).toEqual([],);
        expect(result.changedSliceIndices,).toEqual([ 0, ],);
        expect(warnings,).toEqual([],);
      },
    },),
  ],
},);
