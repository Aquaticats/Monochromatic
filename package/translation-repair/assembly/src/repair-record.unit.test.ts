/**
 Tests for flattening slice outcomes into the whole-document issue report,
 and for the disposition that says what became of each issue's repair.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RepairRegion, } from '@monochromatic-dev/translation-repair-adjudication';
import type { AdjudicatedIssue, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  buildIssueRecords,
  type ChunkRepairOutcome,
} from '../dist/final/node/index.mjs';
import { catIssue, } from '@monochromatic-dev/translation-repair-shared-contract/ts/accepted-cat-issue.test-fixture.ts';

/**
 Slice text after the naturalness lane rewrote it.
 */
const REFINED_TEXT = 'The cat naps in the sun and chases butterflies.';

/**
 Slice text a shipped, unrefined repair returns: it CONTAINS the region's
 replacement verbatim, which is the property the conditional `finalSliceText`
 rests on. Using the refined wording here regardless would let the conditional
 test pass while the fixture modelled a state that cannot occur.
 */
const PATCHED_TEXT = 'The cat is asleep. She wakes at dusk.';

/**
 Replacement the fixture region writes, and a substring of
 {@link PATCHED_TEXT}.
 */
const REPLACEMENT = 'The cat is asleep.';

/**
 Builds one region serving the given issues.

 @param issueIds - accepted issues the envelope was cut for

 @returns Region the report filters by issue

 @example
 ```ts
 const region = catRegion({ issueIds: ['adjudicated/nap',], },);
 ```
 */
function catRegion(
  { issueIds, }: { readonly issueIds: readonly string[]; },
): RepairRegion {
  return {
    envelopeId: `envelope/${issueIds.join('-',)}`,
    issueIds,
    before: 'The cat is doing the sleeping.',
    editorAfter: REPLACEMENT,
  };
}

/**
 Builds one settled slice outcome.

 @param issues - adjudicated issues of this slice

 @param repairRegions - regions the accuracy stage replaced

 @param accuracyPatchSelected - whether the patched candidate won its slice

 @param resolvedIssueIds - issues the checkers confirmed fixed

 @param refined - whether the naturalness lane rewrote this slice

 @returns Outcome the builder flattens

 @example
 ```ts
 const outcome = catOutcome({ issues: [], repairRegions: [], },);
 ```
 */
function catOutcome(
  {
    issues,
    repairRegions,
    accuracyPatchSelected = true,
    resolvedIssueIds = [],
    refined = false,
  }: {
    readonly issues: readonly AdjudicatedIssue[];
    readonly repairRegions: readonly RepairRegion[];
    readonly accuracyPatchSelected?: boolean;
    readonly resolvedIssueIds?: readonly string[];
    readonly refined?: boolean;
  },
): ChunkRepairOutcome {
  return {
    sliceIndex: 3,
    // Modelled the way the pipeline actually assembles it: a refined slice
    // returns the rewritten text, a shipped unrefined slice returns the patched
    // text carrying the replacement verbatim, and an unselected unrefined slice
    // returns the original.
    repairedText: refined
      ? REFINED_TEXT
      : accuracyPatchSelected
        ? PATCHED_TEXT
        : 'The cat is doing the sleeping. She wakes at dusk.',
    changed: accuracyPatchSelected || refined,
    issues,
    resolvedIssueIds,
    candidateResolvedIssueIds: resolvedIssueIds,
    // No checker round in this fixture, so nothing was said about any issue.
    checkerReadings: {},
    recheckReadings: {},
    repairRegions,
    accuracyPatchSelected,
    refined,
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
    heardCritics: 7,
    heardCriticIds: [],
    claimAttributions: [],
    findings: [],
    silentStages: [],
  };
}

await describe({
  name: buildIssueRecords.name,
  children: [
    it({
      name: 'reports a shipped repair, keeping only the regions serving this '
        + 'issue and carrying the chunk it came from',
      fn: async () => {
        const records = buildIssueRecords({
          outcomes: [
            catOutcome({
              issues: [
                catIssue({ issueId: 'adjudicated/nap', },),
                catIssue({ issueId: 'adjudicated/chase', },),
              ],
              repairRegions: [catRegion({ issueIds: ['adjudicated/nap',], },),],
              resolvedIssueIds: ['adjudicated/nap',],
            },),
          ],
          blocked: false,
        },);
        expect(records,).toHaveLength(2,);
        expect(records[0]?.repairDisposition,).toBe('shipped',);
        expect(records[0]?.repairRegions,).toHaveLength(1,);
        expect(records[0]?.resolved,).toBe(true,);
        expect(records[0]?.sliceIndex,).toBe(3,);
        // The second issue shares the slice but not the envelope, so it has no
        // repair of its own even though the slice was repaired.
        expect(records[1]?.repairDisposition,).toBe('no-region',);
        expect(records[1]?.repairRegions,).toHaveLength(0,);
      },
    },),

    it({
      name: 'separates a repair that lost its slice selection from one that '
        + 'was never written, since only the first indicts the stage',
      fn: async () => {
        const records = buildIssueRecords({
          outcomes: [
            catOutcome({
              issues: [catIssue({ issueId: 'adjudicated/nap', },),],
              repairRegions: [catRegion({ issueIds: ['adjudicated/nap',], },),],
              accuracyPatchSelected: false,
            },),
          ],
          blocked: false,
        },);
        expect(records[0]?.repairDisposition,).toBe('not-selected',);
        // The attempt survives even though it never shipped, which is the
        // whole reason regions are recorded regardless of selection.
        expect(records[0]?.repairRegions[0]?.editorAfter,)
          .toBe('The cat is asleep.',);
      },
    },),

    it({
      name: 'withdraws every slice repair when the document was blocked, '
        + 'because a blocked run returns its input whatever each slice decided',
      fn: async () => {
        const records = buildIssueRecords({
          outcomes: [
            catOutcome({
              issues: [catIssue({ issueId: 'adjudicated/nap', },),],
              repairRegions: [catRegion({ issueIds: ['adjudicated/nap',], },),],
              resolvedIssueIds: ['adjudicated/nap',],
            },),
          ],
          blocked: true,
        },);
        expect(records[0]?.repairDisposition,).toBe('withdrawn',);
        // Checkers confirmed it against a candidate nobody ever read.
        expect(records[0]?.resolved,).toBe(false,);
      },
    },),

    it({
      name: 'carries the final slice text exactly when refinement made the '
        + 'recorded replacement stale, and never otherwise',
      fn: async () => {
        const refined = buildIssueRecords({
          outcomes: [
            catOutcome({
              issues: [catIssue({ issueId: 'adjudicated/nap', },),],
              repairRegions: [catRegion({ issueIds: ['adjudicated/nap',], },),],
              refined: true,
            },),
          ],
          blocked: false,
        },);
        expect(refined[0]?.refined,).toBe(true,);
        expect(refined[0]?.finalSliceText,).toBe(REFINED_TEXT,);

        /** Same slice with the naturalness lane leaving it alone. */
        const untouched = buildIssueRecords({
          outcomes: [
            catOutcome({
              issues: [catIssue({ issueId: 'adjudicated/nap', },),],
              repairRegions: [catRegion({ issueIds: ['adjudicated/nap',], },),],
            },),
          ],
          blocked: false,
        },);
        expect(untouched[0]?.refined,).toBe(false,);
        expect(untouched[0]?.finalSliceText,).toBeUndefined();
      },
    },),

    it({
      name: 'carries the naturalness rewrite\'s probe report onto each issue of its slice field by '
        + 'field, leaving out the findings a record declares no field for',
      fn: async () => {
        const records = buildIssueRecords({
          outcomes: [
            {
              ...catOutcome({
                issues: [catIssue({ issueId: 'adjudicated/nap', },),],
                repairRegions: [catRegion({ issueIds: ['adjudicated/nap',], },),],
                refined: true,
              },),
              refinementDefects: {
                heardProbers: 2,
                configuredProbers: 3,
                regions: [],
                findings: ['stage-voice-lost (prober tabby-large)',],
              },
            },
          ],
          blocked: false,
        },);
        expect(records[0]?.refinementDefects,).toStrictEqual({
          heardProbers: 2,
          configuredProbers: 3,
          regions: [],
        },);
      },
    },),

    it({
      name: 'omits the final slice text only where the replacement really is '
        + 'verbatim in the returned slice, which is the claim that justifies '
        + 'omitting it',
      fn: async () => {
        /** Shipped, unrefined: the returned slice must carry the replacement. */
        const shipped = buildIssueRecords({
          outcomes: [
            catOutcome({
              issues: [catIssue({ issueId: 'adjudicated/nap', },),],
              repairRegions: [catRegion({ issueIds: ['adjudicated/nap',], },),],
            },),
          ],
          blocked: false,
        },);
        expect(shipped[0]?.repairDisposition,).toBe('shipped',);
        expect(shipped[0]?.finalSliceText,).toBeUndefined();
        expect(
          PATCHED_TEXT.includes(
            shipped[0]
              ?.repairRegions[0]
              ?.editorAfter
              ?? 'absent',
          ),
        ).toBe(true,);
      },
    },),

    it({
      name: 'still carries the final slice text when the naturalness lane '
        + 'rewrote a slice whose accuracy patch was NOT selected, since the '
        + 'returned text changed even though no targeted repair shipped',
      fn: async () => {
        // The two stages decide independently: refinement runs whatever the
        // accuracy selection did, so this pairing is reachable and is the case
        // where "nothing reached the reader" is true of the repair and false of
        // the text.
        const records = buildIssueRecords({
          outcomes: [
            catOutcome({
              issues: [catIssue({ issueId: 'adjudicated/nap', },),],
              repairRegions: [catRegion({ issueIds: ['adjudicated/nap',], },),],
              accuracyPatchSelected: false,
              refined: true,
            },),
          ],
          blocked: false,
        },);
        expect(records[0]?.repairDisposition,).toBe('not-selected',);
        expect(records[0]?.refined,).toBe(true,);
        expect(records[0]?.finalSliceText,).toBe(REFINED_TEXT,);
      },
    },),

    it({
      name: 'DROPS the final slice text when the document does not carry the '
        + 'rewritten slice, which a withdrawn slice and a blocked run both '
        + 'mean. The field names what shipped, so a refined slice nobody read '
        + 'must not name its rewrite as the shipped wording',
      fn: async () => {
        /**
         Refined slice the assembly guard took back to keep a footnote whole.
         */
        const withdrawn = buildIssueRecords({
          outcomes: [
            catOutcome({
              issues: [catIssue({ issueId: 'adjudicated/nap', },),],
              repairRegions: [catRegion({ issueIds: ['adjudicated/nap',], },),],
              refined: true,
            },),
          ],
          blocked: false,
          withdrawnSliceIndices: [3,],
        },);
        expect(withdrawn[0]?.repairDisposition,).toBe('withdrawn',);
        // Still true, and still worth disclosing: the lane did rewrite this
        // slice, which is why `editorAfter` is stale wording either way.
        expect(withdrawn[0]?.refined,).toBe(true,);
        expect(withdrawn[0]?.finalSliceText,).toBeUndefined();

        /** Same slice under a run that returned its input for non-translation. */
        const blocked = buildIssueRecords({
          outcomes: [
            catOutcome({
              issues: [catIssue({ issueId: 'adjudicated/nap', },),],
              repairRegions: [catRegion({ issueIds: ['adjudicated/nap',], },),],
              refined: true,
            },),
          ],
          blocked: true,
        },);
        expect(blocked[0]?.repairDisposition,).toBe('withdrawn',);
        expect(blocked[0]?.finalSliceText,).toBeUndefined();
      },
    },),

    it({
      name: 'gives every issue of a shared envelope the same region, each '
        + 'still naming the others',
      fn: async () => {
        const records = buildIssueRecords({
          outcomes: [
            catOutcome({
              issues: [
                catIssue({ issueId: 'adjudicated/nap', },),
                catIssue({ issueId: 'adjudicated/chase', },),
              ],
              repairRegions: [
                catRegion({
                  issueIds: [
                    'adjudicated/nap',
                    'adjudicated/chase',
                  ],
                },),
              ],
            },),
          ],
          blocked: false,
        },);
        expect(records[0]?.repairRegions,).toHaveLength(1,);
        expect(records[1]?.repairRegions,).toHaveLength(1,);
        expect(records[1]?.repairRegions[0]?.issueIds,).toEqual([
          'adjudicated/nap',
          'adjudicated/chase',
        ],);
      },
    },),
  ],
},);
