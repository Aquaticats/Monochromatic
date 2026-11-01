/**
 Tests that assembly GIVES EACH SLICE ITS OWN REPAIR when it lists what the
 document carries.

 WHAT THAT LIST IS FOR. The `lintong` example is two NEIGHBOURING slices
 shipping the same wording, which the document-scale repetition check cannot
 see because the duplicated sentence carries no long word. The adjacent check
 exists for exactly that, and it reads a per-slice list assembly builds by
 matching each surviving replacement to the slice it was written for.

 WHAT WAS MEASURED. On 2026-08-25, inverting that match, so each slice takes
 the first replacement written for a DIFFERENT slice, failed no test in this
 package. The assembled document is unaffected, every count still agrees, and
 only the adjacency check is handed a shuffled list; it then reports
 repetitions between slices that share nothing and misses the ones that do.

 THREE SLICES, WHICH IS THE FEWEST THAT CAN SHOW IT. Swapping two adjacent
 slices' wordings leaves the pair unchanged, so a two-slice fixture cannot
 tell a correct match from an inverted one. With three, an inverted match
 lands the same wording on two neighbours and manufactures a repetition.

 THE SECOND CASE IS THE KILL and the first is its liveness control: without
 one showing the check speaks, "no finding" would be satisfied by a check
 that never says anything.

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
import { sliceBuilderFor, } from '@monochromatic-dev/translation-repair-slice-reading/ts/repair-assemble-slice.test-fixture.ts';
import { SEAT_HYPER_OPENROUTER_VISION_EDITOR, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Logger for assembly under test.
 */
const l = tagged({ tag: 'repair-assemble-slice-match-test', },);

//region Fixtures

/**
 Archive wording of the first slice.
 */
const FIRST_ARCHIVE = 'The kitten sleeps on the sill.';

/**
 Archive wording of the second slice.
 */
const SECOND_ARCHIVE = 'Whiskers watches the birds.';

/**
 Archive wording of the third slice.
 */
const THIRD_ARCHIVE = 'The tabby dozes by the heater.';

/**
 Repair the first slice settled on.
 */
const FIRST_REPAIR = 'The kitten sleeps on the windowsill through the afternoon.';

/**
 Repair the second slice settled on, distinct from both its neighbours.
 */
const SECOND_REPAIR = 'Whiskers watches the birds through the window glass.';

/**
 Repair the third slice settled on.
 */
const THIRD_REPAIR = 'The tabby dozes beside the heater until the evening.';

/**
 Document the slice offsets address.
 */
const TARGET_TEXT = `${FIRST_ARCHIVE}\n\n${SECOND_ARCHIVE}\n\n${THIRD_ARCHIVE}`;

/**
 All three slice indices, marked line-structured so the wrap leaves the
 fixture wording byte-identical and the checks read exactly what this wrote.
 */
const LINE_STRUCTURED: ReadonlySet<number> = new Set([
  0,
  1,
  2,
],);

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
    text: FIRST_ARCHIVE,
  },),
  sliceOf({
    sliceIndex: 1,
    text: SECOND_ARCHIVE,
  },),
  sliceOf({
    sliceIndex: 2,
    text: THIRD_ARCHIVE,
  },),
];

/**
 Builds one settled outcome that ships a repair.

 @param sliceIndex - slice this outcome belongs to

 @param repairedText - wording the lane settled on

 @returns Outcome assembly reads

 @example
 ```ts
 const outcome = outcomeOf({ sliceIndex: 0, repairedText: FIRST_REPAIR, },);
 ```
 */
function outcomeOf(
  {
    sliceIndex,
    repairedText,
  }: {
    readonly sliceIndex: number;
    readonly repairedText: string;
  },
): ChunkRepairOutcome {
  return {
    sliceIndex,
    repairedText,
    changed: true,
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
    // that carries anything but the archive's wording.
    heardCriticIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
    claimAttributions: [],
    findings: [],
    silentStages: [],
  };
}

/**
 Assembles three repairs and hands back what assembly found.

 @param second - wording the middle slice settled on, which is what each case
 varies

 @returns Findings assembly recorded, plus the slices it says changed

 @example
 ```ts
 const found = assembleWith({ second: SECOND_REPAIR, },);
 ```
 */
function assembleWith(
  { second, }: { readonly second: string; },
): {
  readonly findings: readonly string[];
  readonly changedSliceIndices: readonly number[];
} {
  /**
   What assembly made of the three repairs.
   */
  const result = assembleRepair({
    targetText: TARGET_TEXT,
    slices: SLICES,
    outcomes: [
      outcomeOf({
        sliceIndex: 0,
        repairedText: FIRST_REPAIR,
      },),
      outcomeOf({
        sliceIndex: 1,
        repairedText: second,
      },),
      outcomeOf({
        sliceIndex: 2,
        repairedText: THIRD_REPAIR,
      },),
    ],
    lineStructuredSlices: LINE_STRUCTURED,
    findings: [],
    l,
  },);

  return {
    findings: result.findings,
    changedSliceIndices: result.changedSliceIndices,
  };
}

/**
 Findings naming an adjacent repetition, which is the only kind read here.

 @param findings - everything assembly recorded

 @returns Those naming an adjacent repetition

 @example
 ```ts
 const repeated = adjacentOnly({ findings, },);
 ```
 */
function adjacentOnly(
  { findings, }: { readonly findings: readonly string[]; },
): readonly string[] {
  return findings.filter(function isAdjacent(finding,): boolean {
    return finding.startsWith('adjacent-repetition',);
  },);
}

//endregion Fixtures

await describe({
  name: assembleRepair.name,
  children: [
    it({
      name: 'NAMES THE TWO NEIGHBOURS that really did ship the same wording, which is the control '
        + 'showing this check speaks at all before the "STAYS SILENT when three neighbours ship three '
        + 'different wordings" case reads its silence',
      fn: async () => {
        /**
         Assembly where the middle slice shipped its neighbour's wording.
         */
        const found = assembleWith({ second: FIRST_REPAIR, },);

        expect(adjacentOnly({ findings: found.findings, },).length,).toBe(1,);
        expect(adjacentOnly({ findings: found.findings, },)[0],)
          .toContain('adjacent-repetition (slices 0 and 1',);
      },
    },),

    it({
      name: 'STAYS SILENT when three neighbours ship three different wordings, since a slice handed '
        + 'another slice\'s repair would put the same wording on two neighbours and manufacture a '
        + 'repetition the document does not have',
      fn: async () => {
        /**
         Assembly where every slice shipped its own distinct repair.
         */
        const found = assembleWith({ second: SECOND_REPAIR, },);

        expect(adjacentOnly({ findings: found.findings, },),).toStrictEqual([],);
        expect(found.changedSliceIndices,).toStrictEqual([
          0,
          1,
          2,
        ],);
      },
    },),
  ],
},);
