import type { ResolutionVerdict, } from '@monochromatic-dev/translation-repair-adjudication';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import type {
  AdjudicatedIssue,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import type { ChunkRepairOutcome, } from '../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

//region Sunbathing recheck
// THE SLICE, ITS OPEN ISSUE AND ITS CHECKER BENCH that the refinement
// recheck's cases share: one repaired paragraph about a cat sunbathing, the
// rewrite a refiner proposes for it, an accepted issue quoting it (and a
// second, for a round that rules on two), and three checkers.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The slice settler's test file and the
// recheck's own both rule on this slice, the first through the whole lane
// and the second through the gate alone, so the two read one fixture rather
// than each keeping a copy of the outcome and the bench.
//
// Fixtures are cat-themed invention. No corpus content appears here.

/**
 Repaired slice text, one long single-line paragraph so the lane finds it
 eligible and would reach a rewriter.
 */
export const REPAIRED_TEXT =
  'The cat is doing the sunbathing on the windowsill in every afternoon, and when the light is moving across the floor she is following it without any hurry at all.';

/**
 Wording the rewrite appends to the repaired text: present after the rewrite
 and absent before it, which is what a damage prober's claim of added damage
 quotes for the probe's screen to admit it.
 */
export const ADDED_WORDING = 'Rewritten for flow.';

/**
 Rewrite a scripted refiner proposes for that one paragraph.
 */
export const REWRITTEN_TEXT: string = `${REPAIRED_TEXT} ${ADDED_WORDING}`;

/**
 Original this slice was repaired against.
 */
export const SOURCE_TEXT = '猫猫每天下午都在窗台上晒太阳。';

/**
 Checker bench, in seat order: three seats, the fewest the refine phase lets
 a run configure (`assertCheckerQuorumReachable`), so the checker stage's
 quorum is two and one heard checker is short of it.
 */
export const CHECKERS: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
];

/**
 Builds an accepted issue about the repaired text, with the one member claim
 the panel accepted, quoting the text, so the checker sheet shows a checker
 what it rules on.

 @param issueId - id the checker sheet numbers and the readings key the
 issue by

 @param claimId - id of its one member claim

 @param quote - wording of the repaired text the claim complains about

 @param summary - claim as the panel read it

 @returns Accepted issue a recheck rules on

 @example
 ```ts
 const issue = acceptedIssueQuoting({ issueId: 'adjudicated/nap', claimId: 'issue/nap', quote: 'the cat', summary: 'Too plain.', },);
 ```
 */
function acceptedIssueQuoting(
  {
    issueId,
    claimId,
    quote,
    summary,
  }: {
    readonly issueId: string;
    readonly claimId: string;
    readonly quote: string;
    readonly summary: string;
  },
): AdjudicatedIssue {
  return {
    issueId,
    status: 'accepted',
    severity: 'minor',
    claims: [
      {
        claimId,
        claim: {
          category: 'fluency/grammar',
          severity: 'minor',
          summary,
          spans: [
            {
              side: 'target',
              nodeId: 'block/1',
              nodeHash: hashContent({ content: REPAIRED_TEXT, },),
              startOffset: REPAIRED_TEXT.indexOf(quote,),
              endOffset: REPAIRED_TEXT.indexOf(quote,) + quote.length,
              quotedText: quote,
            },
          ],
        },
      },
    ],
    tallies: {},
  };
}

/**
 Accepted issue about the repaired text's "is doing the sunbathing". A case
 decides whether the accuracy lane's checkers confirmed it fixed or left it
 open.
 */
export const SUNBATHING_ISSUE: AdjudicatedIssue = acceptedIssueQuoting({
  issueId: 'adjudicated/sunbathing',
  claimId: 'issue/sunbathing',
  quote: 'is doing the sunbathing',
  summary: 'The cat "is doing the sunbathing", which no speaker says.',
},);

/**
 Second accepted issue about the same text, numbered after the sunbathing one
 on the checker sheet, for a round that rules on two issues and hears a
 different number of ballots on each.
 */
export const AFTERNOON_ISSUE: AdjudicatedIssue = acceptedIssueQuoting({
  issueId: 'adjudicated/afternoon',
  claimId: 'issue/afternoon',
  quote: 'in every afternoon',
  summary: 'The cat naps "in every afternoon", where English drops the preposition.',
},);

/**
 Builds the settled accuracy outcome of the slice, standing as a translation
 or not, with no issue on record.

 @param nonTranslationStanding - whether the critics' non-translation ruling
 survived contradiction

 @returns Outcome the naturalness lane would refine

 @example
 ```ts
 const outcome = sunbathingOutcome({ nonTranslationStanding: true, },);
 ```
 */
export function sunbathingOutcome(
  { nonTranslationStanding, }: { readonly nonTranslationStanding: boolean; },
): ChunkRepairOutcome {
  return {
    sliceIndex: 0,
    repairedText: REPAIRED_TEXT,
    changed: false,
    issues: [],
    resolvedIssueIds: [],
    candidateResolvedIssueIds: [],
    checkerReadings: {},
    recheckReadings: {},
    repairRegions: [],
    authorship: {
      perIssue: {},
      everyIssue: [],
    },
    accuracyPatchSelected: false,
    refined: false,
    rounds: [],
    droppedDeclaredNames: [],
    nonTranslationVotes: nonTranslationStanding ? 2 : 0,
    nonTranslationContradicted: false,
    nonTranslationStanding,
    heardCritics: 1,
    heardCriticIds: [],
    claimAttributions: [],
    findings: [],
    silentStages: [],
  };
}

/**
 Verdicts of a round in which every checker of the bench casts the same one.

 @param verdict - what each checker answers on the one issue

 @returns Verdict by checker, for a scripted client

 @example
 ```ts
 const verdicts = everyCheckerSays({ verdict: 'not-fixed', },);
 ```
 */
export function everyCheckerSays(
  { verdict, }: { readonly verdict: ResolutionVerdict; },
): ReadonlyMap<RosterModelId, ResolutionVerdict> {
  return new Map(CHECKERS.map(function toEntry(modelId,): readonly [
    RosterModelId,
    ResolutionVerdict,
  ] {
    return [
      modelId,
      verdict,
    ];
  },),);
}

//endregion Sunbathing recheck
