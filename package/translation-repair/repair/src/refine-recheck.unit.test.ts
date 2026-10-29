/**
 Tests the refinement recheck, the checker round a refined slice passes
 before its rewrite may ship (ledger L11), by the verdict it returns for each
 kind of round: none to buy, one short of the checker stage's quorum, one
 that passes, and one that rolls the slice back.

 A ROUND SHORT OF ITS QUORUM IS THE CASE THIS FILE EXISTS FOR. With no ballot
 cast, no tally says worse, so such a round returned the findings of a round
 every checker answered and the rewrite shipped as rechecked. The same holds
 for one issue a round that met its quorum cast too few ballots on, so cases
 rule on two issues and hear a different number of ballots on each, on the
 three-seat bench and on a five-seat bench the router left short.

 Each case asserts the whole verdict: whether the rewrite is retained, every
 finding, and every issue's reading. The cases for what the slice settler
 then does with a verdict are in `refine-slice-settle.unit.test.ts`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  IssueCheckerReading,
  ResolutionVerdict,
} from '@monochromatic-dev/translation-repair-adjudication';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import type {
  AdjudicatedIssue,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import { retainsResolvedIssues, } from '../dist/final/node/index.mjs';
import {
  type AskedSheet,
  askedSheetOf,
  stagesAsked,
} from '@monochromatic-dev/translation-repair-provider/ts/asked-sheets.test-fixture.ts';
import {
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_OPENROUTER_ONLY_CHECKER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  AFTERNOON_ISSUE,
  CHECKERS,
  everyCheckerSays,
  REWRITTEN_TEXT,
  SOURCE_TEXT,
  SUNBATHING_ISSUE,
  sunbathingOutcome,
} from '@monochromatic-dev/translation-repair-assembly/ts/sunbathing-recheck.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the recheck under test.
 */
const l = tagged({ tag: 'refine-recheck-test', },);

/**
 Message the scripted bench throws with for an exchange the recheck never
 sends.
 */
const NOT_A_CHECKER_EXCHANGE = 'the recheck asked for something other than a checker report';

/**
 The three lost voices and the unmet quorum the checker stage reports for a
 round nobody answered, in the bench's seat order.
 */
const NOBODY_HEARD_FINDINGS: readonly string[] = [
  `stage-voice-lost (checker ${SEAT_SYNTHETIC_VISION_NO_OPENROUTER})`,
  `stage-voice-lost (checker ${SEAT_SYNTHETIC_VISION_WITHHELD})`,
  `stage-voice-lost (checker ${SEAT_SYNTHETIC_TEXT_EVERYWHERE})`,
  'stage-quorum-unmet (checker 0/3)',
];

/**
 The lost voice and the short roster the checker stage reports for a round
 that met its quorum on the first two seats and never heard the third.
 */
const THIRD_SEAT_LOST_FINDINGS: readonly string[] = [
  `stage-voice-lost (checker ${SEAT_SYNTHETIC_TEXT_EVERYWHERE})`,
  'stage-roster-incomplete (checker 2/3)',
];

/**
 Two open issues the sheet numbers in this order, for a round that hears a
 different number of ballots on each.
 */
const TWO_OPEN_ISSUES: readonly AdjudicatedIssue[] = [
  SUNBATHING_ISSUE,
  AFTERNOON_ISSUE,
];

/**
 Checker bench of five seats, the three of `CHECKERS` first, for a round the
 router leaves short: the last three seats no provider serves, so two
 reachable seats are short of the bench's quorum of three and the checker
 stage closes on two, the fewest voices any stage closes on.
 */
const FIVE_SEATS: readonly RosterModelId[] = [
  ...CHECKERS,
  SEAT_HYPER_ONLY,
  SEAT_OPENROUTER_ONLY_CHECKER,
];

/**
 Seats of that bench no provider serves.
 */
const REFUSED_SEATS: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_ONLY,
  SEAT_OPENROUTER_ONLY_CHECKER,
];

/**
 What the checker stage reports for that short bench's round, heard on its
 first two seats: the short bench, the three refused seats as lost voices,
 and the short roster.
 */
const SHORT_BENCH_FINDINGS: readonly string[] = [
  'stage-short-bench (checker reachable 2 of 5, quorum 2)',
  `stage-voice-lost (checker ${SEAT_SYNTHETIC_TEXT_EVERYWHERE})`,
  `stage-voice-lost (checker ${SEAT_HYPER_ONLY})`,
  `stage-voice-lost (checker ${SEAT_OPENROUTER_ONLY_CHECKER})`,
  'stage-roster-incomplete (checker 2/5)',
];

/**
 Client standing in for the checker bench, which is all the recheck asks.

 Each checker answers with its scripted verdicts on the issues the sheet
 numbers first and second. One given no verdict on either sends an empty
 report, which the checker stage's guard refuses (ledger L8), so the stage
 never hears it and the client hands the refusal back as a schema mismatch,
 as a provider's would. A refused seat throws the router's refusal, which
 the stage reads as a seat no provider serves.

 @param verdicts - verdict each checker casts on the issue the sheet numbers
 first, by model id

 @param secondIssueVerdicts - verdict each checker casts on the issue the
 sheet numbers second, by model id; a checker it leaves out skips that issue

 @param refused - seats no provider serves

 @param asked - sink receiving every exchange the recheck sent

 @param verdictsByCall - verdict on the issue the sheet numbers first for the
 first call, the second call and so on, in place of `verdicts`, so a case does
 not depend on which seats the window asks: the window starts where a hash of
 the sheet's text rotates the bench, and an edit to that text moves it. A call
 past the end of the list is heard as a checker with no verdict on the issue

 @returns Client usable by the recheck

 @example
 ```ts
 const client = checkerBench({ verdicts: everyCheckerSays({ verdict: 'fixed', },), secondIssueVerdicts: new Map(), refused: [], asked: [], },);
 ```
 */
function checkerBench(
  {
    verdicts,
    secondIssueVerdicts,
    refused,
    asked,
    verdictsByCall,
  }: {
    readonly verdicts: ReadonlyMap<RosterModelId, ResolutionVerdict>;
    readonly secondIssueVerdicts: ReadonlyMap<RosterModelId, ResolutionVerdict>;
    readonly refused: readonly RosterModelId[];
    readonly asked: AskedSheet[];
    readonly verdictsByCall?: readonly ResolutionVerdict[];
  },
): ModelClient {
  /**
   Verdicts handed out by call, in the order they are given, absent when the
   case keys its verdicts by seat.
   */
  const callOrder = verdictsByCall?.values();
  return {
    chatText(): never {
      throw new Error(NOT_A_CHECKER_EXCHANGE,);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      asked.push(askedSheetOf({ request, },),);
      if (refused.includes(request.modelId,)) {
        throw new NoProviderForModelError({
          modelId: request.modelId,
          reason: 'every provider serving this cat is out of treats',
        },);
      }
      /**
       Verdict this checker casts on the first issue, absent for one never
       heard or one that skips it.
       */
      const verdict = (callOrder === undefined)
        ? verdicts.get(request.modelId,)
        : callOrder.next()
          .value;
      /**
       Verdict this checker casts on the second issue, absent for one that
       skips it.
       */
      const secondVerdict = secondIssueVerdicts.get(request.modelId,);
      /**
       Report the checker sends: a check per issue it rules on, or none.
       */
      const report: unknown = {
        checks: [
          ...((verdict === undefined)
            ? []
            : [{
              issue: 1,
              verdict,
            },]),
          ...((secondVerdict === undefined)
            ? []
            : [{
              issue: 2,
              verdict: secondVerdict,
            },]),
        ],
      };
      if (request.validate(report,))
        return {
          kind: 'ok',
          value: report,
          rawText: JSON.stringify(report,),
        };
      return {
        kind: 'schema-mismatch',
        rawText: JSON.stringify(report,),
        detail: 'the guard refused a report with no usable check',
      };
    },
    quotas(): never {
      throw new Error(NOT_A_CHECKER_EXCHANGE,);
    },
  };
}

/**
 Ballots a reading holds when each scripted checker's verdict was heard:
 one per verdict, in the order given, none of the checkers having written
 the text.

 @param verdicts - verdict by checker, in seat order

 @returns Ballots as the checker stage records them

 @example
 ```ts
 const ballots = ballotsCast({ verdicts: everyCheckerSays({ verdict: 'fixed', },), },);
 ```
 */
function ballotsCast(
  { verdicts, }: { readonly verdicts: ReadonlyMap<RosterModelId, ResolutionVerdict>; },
): IssueCheckerReading['ballots'] {
  return [...verdicts,].map(function toBallot([modelId, verdict,],) {
    return {
      modelId,
      verdict,
      wroteTheText: false,
    };
  },);
}

/**
 The finding a checker's report leaves for an issue it did not answer, as the
 checker stage spells it for a checker that sent no check on the issue.

 @param issueId - issue the checker left unanswered

 @param modelId - checker that left it

 @returns Unanswered finding

 @example
 ```ts
 const finding = missingCheck({ issueId: SUNBATHING_ISSUE.issueId, modelId: SEAT_SYNTHETIC_VISION_WITHHELD, },);
 ```
 */
function missingCheck(
  {
    issueId,
    modelId,
  }: {
    readonly issueId: string;
    readonly modelId: RosterModelId;
  },
): string {
  return `missing-check (${issueId}, ${modelId}, unanswered)`;
}

/**
 Runs the recheck over the sunbathing slice's rewrite.

 @param issues - issues the accuracy lane settled for the slice

 @param resolvedIssueIds - issues among them its checkers had confirmed fixed

 @param verdicts - verdict each checker casts this round on the issue the
 sheet numbers first, confirmed issues being numbered before open ones; a
 checker it and `secondIssueVerdicts` both leave out is never heard

 @param secondIssueVerdicts - verdict each checker casts on the issue the
 sheet numbers second; a checker it leaves out skips that issue

 @param checkerModelIds - bench the round seats, `CHECKERS` unless a case
 seats another

 @param refused - seats of that bench no provider serves

 @param asked - sink receiving every exchange the recheck sent

 @param logger - logger the recheck writes to, `l` unless a case reads its warnings

 @param verdictsByCall - verdict on the first issue by order of call, in place of `verdicts`, for a case that must not
 depend on which seats the window asks

 @returns Recheck's verdict exactly as it came back: its ballots and its
 findings in the order the roster seats the checkers, which the stage
 guarantees and the cases assert

 @example
 ```ts
 const verdict = await recheck({ issues: [SUNBATHING_ISSUE,], },);
 ```
 */
async function recheck(
  {
    issues,
    resolvedIssueIds = [],
    verdicts = new Map<RosterModelId, ResolutionVerdict>(),
    secondIssueVerdicts = new Map<RosterModelId, ResolutionVerdict>(),
    checkerModelIds = CHECKERS,
    refused = [],
    asked = [],
    logger = l,
    verdictsByCall,
  }: {
    readonly issues: readonly AdjudicatedIssue[];
    readonly resolvedIssueIds?: readonly string[];
    readonly verdicts?: ReadonlyMap<RosterModelId, ResolutionVerdict>;
    readonly secondIssueVerdicts?: ReadonlyMap<RosterModelId, ResolutionVerdict>;
    readonly checkerModelIds?: readonly RosterModelId[];
    readonly refused?: readonly RosterModelId[];
    readonly asked?: AskedSheet[];
    readonly logger?: Logger;
    readonly verdictsByCall?: readonly ResolutionVerdict[];
  },
): Promise<Awaited<ReturnType<typeof retainsResolvedIssues>>> {
  /**
   Verdict as the recheck returned it.
   */
  const verdict = await retainsResolvedIssues({
    client: checkerBench({
      verdicts,
      secondIssueVerdicts,
      refused,
      asked,
      // Conditional spread keeps the field absent where the case keys its verdicts by seat.
      ...((verdictsByCall === undefined) ? {} : { verdictsByCall, }),
    },),
    checkerModelIds,
    outcome: {
      ...sunbathingOutcome({ nonTranslationStanding: false, },),
      issues,
      resolvedIssueIds,
    },
    refineContributors: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
    sourceText: SOURCE_TEXT,
    refinedText: REWRITTEN_TEXT,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l: logger,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return verdict;
}

await describe({
  name: retainsResolvedIssues.name,
  children: [
    it({
      name: 'BUYS NO ROUND for a slice whose one issue the panel rejected: the rewrite is retained with no '
        + 'finding and no reading, and no checker is asked',
      fn: async () => {
        /**
         Exchanges the recheck sent.
         */
        const asked: AskedSheet[] = [];
        expect(
          await recheck({
            issues: [{
              ...SUNBATHING_ISSUE,
              status: 'rejected',
            },],
            verdicts: everyCheckerSays({ verdict: 'worse', },),
            asked,
          },),
        ).toEqual({
          retained: true,
          findings: [],
          readings: {},
        },);
        expect(stagesAsked({ asked, },),).toEqual([],);
      },
    },),

    it({
      name: 'REFUSES a round that heard none of the three checkers: the rewrite is not retained, under the '
        + 'checker stage\'s three lost voices, its unmet quorum and refine-recheck-unheard at 0 of 3, with '
        + 'no ballot in the open issue\'s reading',
      fn: async () => {
        expect(await recheck({ issues: [SUNBATHING_ISSUE,], },),).toEqual({
          retained: false,
          findings: [
            ...NOBODY_HEARD_FINDINGS,
            'refine-recheck-unheard (0 of 3 checkers heard)',
          ],
          readings: {
            [SUNBATHING_ISSUE.issueId]: {
              ballots: [],
              configuredCheckers: 3,
              tally: {
                fixed: 0,
                notFixed: 0,
                worse: 0,
                resolved: false,
                regressed: false,
              },
            },
          },
        },);
      },
    },),

    it({
      name: 'REFUSES a round that heard one checker of three, short of the stage\'s quorum of two, though '
        + 'the one ballot cast says not-fixed: refine-recheck-unheard at 1 of 3, with that ballot in the '
        + 'reading',
      fn: async () => {
        /**
         Round in which the first seat alone answers.
         */
        const verdicts = new Map<RosterModelId, ResolutionVerdict>([
          [
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            'not-fixed',
          ],
        ],);
        expect(
          await recheck({
            issues: [SUNBATHING_ISSUE,],
            verdicts,
          },),
        ).toEqual({
          retained: false,
          findings: [
            `stage-voice-lost (checker ${SEAT_SYNTHETIC_VISION_WITHHELD})`,
            `stage-voice-lost (checker ${SEAT_SYNTHETIC_TEXT_EVERYWHERE})`,
            'stage-quorum-unmet (checker 1/3)',
            'refine-recheck-unheard (1 of 3 checkers heard)',
          ],
          readings: {
            [SUNBATHING_ISSUE.issueId]: {
              ballots: ballotsCast({ verdicts, },),
              configuredCheckers: 3,
              tally: {
                fixed: 0,
                notFixed: 1,
                worse: 0,
                resolved: false,
                regressed: false,
              },
            },
          },
        },);
      },
    },),

    it({
      name: 'NAMES a round short of quorum unheard, not its issue worsened, where the one ballot cast says '
        + 'worse: the findings name no issue, and the worse ballot stays in the reading, which does not flag the '
        + 'issue regressed since one worse ballot is one voice (re-check 3, M1)',
      fn: async () => {
        /**
         Round in which the second seat alone answers, and answers worse.
         */
        const verdicts = new Map<RosterModelId, ResolutionVerdict>([
          [
            SEAT_SYNTHETIC_VISION_WITHHELD,
            'worse',
          ],
        ],);
        expect(
          await recheck({
            issues: [SUNBATHING_ISSUE,],
            verdicts,
          },),
        ).toEqual({
          retained: false,
          findings: [
            `stage-voice-lost (checker ${SEAT_SYNTHETIC_VISION_NO_OPENROUTER})`,
            `stage-voice-lost (checker ${SEAT_SYNTHETIC_TEXT_EVERYWHERE})`,
            'stage-quorum-unmet (checker 1/3)',
            'refine-recheck-unheard (1 of 3 checkers heard)',
          ],
          readings: {
            [SUNBATHING_ISSUE.issueId]: {
              ballots: ballotsCast({ verdicts, },),
              configuredCheckers: 3,
              tally: {
                fixed: 0,
                notFixed: 0,
                worse: 1,
                resolved: false,
                regressed: false,
              },
            },
          },
        },);
      },
    },),

    it({
      name: 'NAMES a round nobody answered unheard, not its issue regressed, for an issue the checkers had '
        + 'confirmed fixed: no ballot says the rewrite broke it',
      fn: async () => {
        expect(
          await recheck({
            issues: [SUNBATHING_ISSUE,],
            resolvedIssueIds: [SUNBATHING_ISSUE.issueId,],
          },),
        ).toEqual({
          retained: false,
          findings: [
            ...NOBODY_HEARD_FINDINGS,
            'refine-recheck-unheard (0 of 3 checkers heard)',
          ],
          readings: {
            [SUNBATHING_ISSUE.issueId]: {
              ballots: [],
              configuredCheckers: 3,
              tally: {
                fixed: 0,
                notFixed: 0,
                worse: 0,
                resolved: false,
                regressed: false,
              },
            },
          },
        },);
      },
    },),

    it({
      name: 'PASSES a round every checker answered not-fixed on an open issue: the rewrite is retained '
        + 'under refine-recheck-passed (1 issue) and no other finding',
      fn: async () => {
        /**
         Round in which every checker finds the open issue as it stood.
         */
        const verdicts = everyCheckerSays({ verdict: 'not-fixed', },);
        expect(
          await recheck({
            issues: [SUNBATHING_ISSUE,],
            verdicts,
          },),
        ).toEqual({
          retained: true,
          findings: ['refine-recheck-passed (1 issue)',],
          readings: {
            [SUNBATHING_ISSUE.issueId]: {
              ballots: ballotsCast({ verdicts, },),
              configuredCheckers: 3,
              tally: {
                fixed: 0,
                notFixed: 3,
                worse: 0,
                resolved: false,
                regressed: false,
              },
            },
          },
        },);
      },
    },),

    it({
      name: 'PASSES a round every checker confirmed a confirmed issue fixed again',
      fn: async () => {
        /**
         Round in which every checker calls the confirmed issue fixed.
         */
        const verdicts = everyCheckerSays({ verdict: 'fixed', },);
        expect(
          await recheck({
            issues: [SUNBATHING_ISSUE,],
            resolvedIssueIds: [SUNBATHING_ISSUE.issueId,],
            verdicts,
          },),
        ).toEqual({
          retained: true,
          findings: ['refine-recheck-passed (1 issue)',],
          readings: {
            [SUNBATHING_ISSUE.issueId]: {
              ballots: ballotsCast({ verdicts, },),
              configuredCheckers: 3,
              tally: {
                fixed: 3,
                notFixed: 0,
                worse: 0,
                resolved: true,
                regressed: false,
              },
            },
          },
        },);
      },
    },),

    it({
      name: 'PASSES a round with one worse ballot of three on an open issue, though the other two say '
        + 'not-fixed: one worse ballot is one voice, and one voice rolls nothing back (owner, 2026-10-25, '
        + '"Two ballots for undo too")',
      fn: async () => {
        /**
         Round with one worse ballot, from the second seat.
         */
        const verdicts = new Map<RosterModelId, ResolutionVerdict>([
          [
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            'not-fixed',
          ],
          [
            SEAT_SYNTHETIC_VISION_WITHHELD,
            'worse',
          ],
          [
            SEAT_SYNTHETIC_TEXT_EVERYWHERE,
            'not-fixed',
          ],
        ],);
        expect(
          await recheck({
            issues: [SUNBATHING_ISSUE,],
            verdicts,
          },),
        ).toEqual({
          retained: true,
          findings: ['refine-recheck-passed (1 issue)',],
          readings: {
            [SUNBATHING_ISSUE.issueId]: {
              ballots: ballotsCast({ verdicts, },),
              configuredCheckers: 3,
              tally: {
                fixed: 0,
                notFixed: 2,
                worse: 1,
                resolved: false,
                regressed: false,
              },
            },
          },
        },);
      },
    },),

    it({
      name: 'PASSES a round with one worse ballot beside one fixed and one not-fixed on an open issue: the '
        + 'worse ballot does not outweigh the rest, and a fixed ballot beside it does not make it two',
      fn: async () => {
        /**
         Round with one worse ballot, from the first seat.
         */
        const verdicts = new Map<RosterModelId, ResolutionVerdict>([
          [
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            'worse',
          ],
          [
            SEAT_SYNTHETIC_VISION_WITHHELD,
            'fixed',
          ],
          [
            SEAT_SYNTHETIC_TEXT_EVERYWHERE,
            'not-fixed',
          ],
        ],);
        expect(
          await recheck({
            issues: [SUNBATHING_ISSUE,],
            verdicts,
          },),
        ).toEqual({
          retained: true,
          findings: ['refine-recheck-passed (1 issue)',],
          readings: {
            [SUNBATHING_ISSUE.issueId]: {
              ballots: ballotsCast({ verdicts, },),
              configuredCheckers: 3,
              tally: {
                fixed: 1,
                notFixed: 1,
                worse: 1,
                resolved: false,
                regressed: false,
              },
            },
          },
        },);
      },
    },),

    it({
      name: 'ROLLS BACK on two worse ballots of three on an open issue, naming the issue, though the third '
        + 'says not-fixed: two voices name the rewrite damaging and outweigh the rest',
      fn: async () => {
        /**
         Round with two worse ballots, from the first two seats.
         */
        const verdicts = new Map<RosterModelId, ResolutionVerdict>([
          [
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            'worse',
          ],
          [
            SEAT_SYNTHETIC_VISION_WITHHELD,
            'worse',
          ],
          [
            SEAT_SYNTHETIC_TEXT_EVERYWHERE,
            'not-fixed',
          ],
        ],);
        expect(
          await recheck({
            issues: [SUNBATHING_ISSUE,],
            verdicts,
          },),
        ).toEqual({
          retained: false,
          findings: [`refine-rolled-back (${SUNBATHING_ISSUE.issueId})`,],
          readings: {
            [SUNBATHING_ISSUE.issueId]: {
              ballots: ballotsCast({ verdicts, },),
              configuredCheckers: 3,
              tally: {
                fixed: 0,
                notFixed: 1,
                worse: 2,
                resolved: false,
                regressed: true,
              },
            },
          },
        },);
      },
    },),

    it({
      name: 'ROLLS BACK on two worse ballots against one fixed on an open issue, the control that a fixed '
        + 'ballot beside the worse ones does not stop the rollback',
      fn: async () => {
        /**
         Round with two worse ballots and one fixed.
         */
        const verdicts = new Map<RosterModelId, ResolutionVerdict>([
          [
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            'worse',
          ],
          [
            SEAT_SYNTHETIC_VISION_WITHHELD,
            'fixed',
          ],
          [
            SEAT_SYNTHETIC_TEXT_EVERYWHERE,
            'worse',
          ],
        ],);
        expect(
          await recheck({
            issues: [SUNBATHING_ISSUE,],
            verdicts,
          },),
        ).toEqual({
          retained: false,
          findings: [`refine-rolled-back (${SUNBATHING_ISSUE.issueId})`,],
          readings: {
            [SUNBATHING_ISSUE.issueId]: {
              ballots: ballotsCast({ verdicts, },),
              configuredCheckers: 3,
              tally: {
                fixed: 1,
                notFixed: 0,
                worse: 2,
                resolved: false,
                regressed: true,
              },
            },
          },
        },);
      },
    },),

    it({
      name: 'PASSES a round of two worse ballots level with two fixed ballots on an open issue: the worse '
        + 'weight has to strictly outweigh the rest as well as come from two checkers',
      fn: async () => {
        // THE FIVE-SEAT BENCH ASKS FOUR SEATS, quorum of three plus the spare,
        // and all four can answer: this is the round production reaches, and
        // the case that a rollback written as "two worse ballots" alone
        // (`worse >= 2`) passes for the wrong reason. WHICH FOUR is decided by
        // a hash of the checker sheet's text, so a seat-keyed script made this
        // case fail falsely on a one-word edit to the sheet (re-check 6, M-A).
        // The verdicts are therefore handed out by order of call: the first
        // four calls hear worse, fixed, worse, fixed, whichever seats they
        // are. The client reads its verdict at entry, before any await, so a
        // reordering of the launch moves only which seat holds which verdict,
        // and the asserted count of exchanges ties the script to a window of
        // four.
        /**
         Exchanges the recheck sent, one per seat asked.
         */
        const asked: AskedSheet[] = [];
        /**
         What the recheck returned.
         */
        const verdict = await recheck({
          issues: [SUNBATHING_ISSUE,],
          checkerModelIds: FIVE_SEATS,
          verdictsByCall: [
            'worse',
            'fixed',
            'worse',
            'fixed',
          ],
          asked,
        },);
        expect(asked.length,).toBe(4,);
        expect({
          retained: verdict.retained,
          tally: verdict.readings[SUNBATHING_ISSUE.issueId]?.tally,
        },).toEqual({
          retained: true,
          tally: {
            fixed: 2,
            notFixed: 0,
            worse: 2,
            resolved: false,
            regressed: false,
          },
        },);
      },
    },),

    it({
      name: 'WARNS ONCE when it rolls a slice back on two worse ballots, naming the slice, the refiner, the '
        + 'issue and every ballot cast on that issue',
      fn: async () => {
        /**
         Warnings the recheck wrote, in order.
         */
        const warnings: string[] = [];
        /**
         Logger keeping the warnings and forwarding nothing else.
         */
        const capturing: Logger = {
          ...l,
          warn: function keep(message: string,): void {
            warnings.push(message,);
          },
        };
        await recheck({
          issues: [SUNBATHING_ISSUE,],
          verdicts: new Map<RosterModelId, ResolutionVerdict>([
            [
              SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              'not-fixed',
            ],
            [
              SEAT_SYNTHETIC_VISION_WITHHELD,
              'worse',
            ],
            [
              SEAT_SYNTHETIC_TEXT_EVERYWHERE,
              'worse',
            ],
          ],),
          logger: capturing,
        },);
        expect(warnings,).toEqual([
          `slice 0: refine-rolled-back (${SUNBATHING_ISSUE.issueId}); rewrite by `
            + `${SEAT_HYPER_OPENROUTER_VISION_EDITOR}; ballots on ${SUNBATHING_ISSUE.issueId}: `
            + `${SEAT_SYNTHETIC_VISION_NO_OPENROUTER} not-fixed, ${SEAT_SYNTHETIC_VISION_WITHHELD} worse, `
            + `${SEAT_SYNTHETIC_TEXT_EVERYWHERE} worse; keeping the text before the rewrite`,
        ],);
      },
    },),

    it({
      name: 'ROLLS BACK when a round every checker answered does not confirm a confirmed issue again, '
        + 'naming the issue',
      fn: async () => {
        /**
         Round in which every checker finds the confirmed issue present again.
         */
        const verdicts = everyCheckerSays({ verdict: 'not-fixed', },);
        expect(
          await recheck({
            issues: [SUNBATHING_ISSUE,],
            resolvedIssueIds: [SUNBATHING_ISSUE.issueId,],
            verdicts,
          },),
        ).toEqual({
          retained: false,
          findings: [`refine-rolled-back (${SUNBATHING_ISSUE.issueId})`,],
          readings: {
            [SUNBATHING_ISSUE.issueId]: {
              ballots: ballotsCast({ verdicts, },),
              configuredCheckers: 3,
              tally: {
                fixed: 0,
                notFixed: 3,
                worse: 0,
                resolved: false,
                regressed: false,
              },
            },
          },
        },);
      },
    },),

    it({
      name: 'CARRIES the checker stage\'s own findings into a pass: two checkers of three answering '
        + 'not-fixed meet the quorum, and the lost checker and the short roster are named beside '
        + 'refine-recheck-passed',
      fn: async () => {
        /**
         Round the first two seats answer and the third never does.
         */
        const verdicts = new Map<RosterModelId, ResolutionVerdict>([
          [
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            'not-fixed',
          ],
          [
            SEAT_SYNTHETIC_VISION_WITHHELD,
            'not-fixed',
          ],
        ],);
        expect(
          await recheck({
            issues: [SUNBATHING_ISSUE,],
            verdicts,
          },),
        ).toEqual({
          retained: true,
          findings: [
            ...THIRD_SEAT_LOST_FINDINGS,
            'refine-recheck-passed (1 issue)',
          ],
          readings: {
            [SUNBATHING_ISSUE.issueId]: {
              ballots: ballotsCast({ verdicts, },),
              configuredCheckers: 3,
              tally: {
                fixed: 0,
                notFixed: 2,
                worse: 0,
                resolved: false,
                regressed: false,
              },
            },
          },
        },);
      },
    },),

    it({
      name: 'CARRIES the checker stage\'s own findings into a rollback: two worse ballots of the two heard '
        + 'roll back, and the lost checker and the short roster are named beside the issue',
      fn: async () => {
        /**
         Round the first two seats call worse and the third never answers.
         */
        const verdicts = new Map<RosterModelId, ResolutionVerdict>([
          [
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            'worse',
          ],
          [
            SEAT_SYNTHETIC_VISION_WITHHELD,
            'worse',
          ],
        ],);
        expect(
          await recheck({
            issues: [SUNBATHING_ISSUE,],
            verdicts,
          },),
        ).toEqual({
          retained: false,
          findings: [
            ...THIRD_SEAT_LOST_FINDINGS,
            `refine-rolled-back (${SUNBATHING_ISSUE.issueId})`,
          ],
          readings: {
            [SUNBATHING_ISSUE.issueId]: {
              ballots: ballotsCast({ verdicts, },),
              configuredCheckers: 3,
              tally: {
                fixed: 0,
                notFixed: 0,
                worse: 2,
                resolved: false,
                regressed: true,
              },
            },
          },
        },);
      },
    },),

    it({
      name: 'REFUSES a round that met its quorum but heard one ballot on the second of two open issues, '
        + 'short of the quorum of two: refine-recheck-unheard names that issue with its one ballot beside '
        + 'the two missing checks, though every ballot cast says not-fixed',
      fn: async () => {
        /**
         Round every checker answers on the first issue, the first seat alone
         on the second.
         */
        const secondIssueVerdicts = new Map<RosterModelId, ResolutionVerdict>([
          [
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            'not-fixed',
          ],
        ],);
        expect(
          await recheck({
            issues: TWO_OPEN_ISSUES,
            verdicts: everyCheckerSays({ verdict: 'not-fixed', },),
            secondIssueVerdicts,
          },),
        ).toEqual({
          retained: false,
          findings: [
            missingCheck({ issueId: AFTERNOON_ISSUE.issueId, modelId: SEAT_SYNTHETIC_VISION_WITHHELD, },),
            missingCheck({ issueId: AFTERNOON_ISSUE.issueId, modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, },),
            `refine-recheck-unheard (${AFTERNOON_ISSUE.issueId}: 1 ballot, quorum 2)`,
          ],
          readings: {
            [SUNBATHING_ISSUE.issueId]: {
              ballots: ballotsCast({ verdicts: everyCheckerSays({ verdict: 'not-fixed', },), },),
              configuredCheckers: 3,
              tally: {
                fixed: 0,
                notFixed: 3,
                worse: 0,
                resolved: false,
                regressed: false,
              },
            },
            [AFTERNOON_ISSUE.issueId]: {
              ballots: ballotsCast({ verdicts: secondIssueVerdicts, },),
              configuredCheckers: 3,
              tally: {
                fixed: 0,
                notFixed: 1,
                worse: 0,
                resolved: false,
                regressed: false,
              },
            },
          },
        },);
      },
    },),

    it({
      name: 'NAMES a confirmed issue the round heard one ballot on unheard, not regressed, though that '
        + 'ballot says fixed: the round met its quorum on the open issue, and no ballot says the rewrite '
        + 'broke the confirmed one',
      fn: async () => {
        /**
         The first seat's verdict on the confirmed issue, which the sheet
         numbers first; the other two skip it.
         */
        const verdicts = new Map<RosterModelId, ResolutionVerdict>([
          [
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            'fixed',
          ],
        ],);
        expect(
          await recheck({
            issues: TWO_OPEN_ISSUES,
            resolvedIssueIds: [SUNBATHING_ISSUE.issueId,],
            verdicts,
            secondIssueVerdicts: everyCheckerSays({ verdict: 'not-fixed', },),
          },),
        ).toEqual({
          retained: false,
          findings: [
            missingCheck({ issueId: SUNBATHING_ISSUE.issueId, modelId: SEAT_SYNTHETIC_VISION_WITHHELD, },),
            missingCheck({ issueId: SUNBATHING_ISSUE.issueId, modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, },),
            `refine-recheck-unheard (${SUNBATHING_ISSUE.issueId}: 1 ballot, quorum 2)`,
          ],
          readings: {
            [SUNBATHING_ISSUE.issueId]: {
              ballots: ballotsCast({ verdicts, },),
              configuredCheckers: 3,
              tally: {
                fixed: 1,
                notFixed: 0,
                worse: 0,
                resolved: false,
                regressed: false,
              },
            },
            [AFTERNOON_ISSUE.issueId]: {
              ballots: ballotsCast({ verdicts: everyCheckerSays({ verdict: 'not-fixed', },), },),
              configuredCheckers: 3,
              tally: {
                fixed: 0,
                notFixed: 3,
                worse: 0,
                resolved: false,
                regressed: false,
              },
            },
          },
        },);
      },
    },),

    it({
      name: 'NAMES both an issue heard short of quorum and an issue two checkers found worse, in one round: '
        + 'refine-recheck-unheard for the second issue and refine-rolled-back for the first',
      fn: async () => {
        /**
         Round with two worse ballots on the first issue, from the last two seats.
         */
        const verdicts = new Map<RosterModelId, ResolutionVerdict>([
          [
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            'not-fixed',
          ],
          [
            SEAT_SYNTHETIC_VISION_WITHHELD,
            'worse',
          ],
          [
            SEAT_SYNTHETIC_TEXT_EVERYWHERE,
            'worse',
          ],
        ],);
        /**
         The first seat alone rules on the second issue.
         */
        const secondIssueVerdicts = new Map<RosterModelId, ResolutionVerdict>([
          [
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            'not-fixed',
          ],
        ],);
        expect(
          await recheck({
            issues: TWO_OPEN_ISSUES,
            verdicts,
            secondIssueVerdicts,
          },),
        ).toEqual({
          retained: false,
          findings: [
            missingCheck({ issueId: AFTERNOON_ISSUE.issueId, modelId: SEAT_SYNTHETIC_VISION_WITHHELD, },),
            missingCheck({ issueId: AFTERNOON_ISSUE.issueId, modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, },),
            `refine-recheck-unheard (${AFTERNOON_ISSUE.issueId}: 1 ballot, quorum 2)`,
            `refine-rolled-back (${SUNBATHING_ISSUE.issueId})`,
          ],
          readings: {
            [SUNBATHING_ISSUE.issueId]: {
              ballots: ballotsCast({ verdicts, },),
              configuredCheckers: 3,
              tally: {
                fixed: 0,
                notFixed: 1,
                worse: 2,
                resolved: false,
                regressed: true,
              },
            },
            [AFTERNOON_ISSUE.issueId]: {
              ballots: ballotsCast({ verdicts: secondIssueVerdicts, },),
              configuredCheckers: 3,
              tally: {
                fixed: 0,
                notFixed: 1,
                worse: 0,
                resolved: false,
                regressed: false,
              },
            },
          },
        },);
      },
    },),

    it({
      name: 'PASSES a round that heard two ballots on the second of two open issues, the quorum of two, '
        + 'one checker skipping it: refine-recheck-passed (2 issues) beside the one missing check',
      fn: async () => {
        /**
         The first two seats rule on the second issue, the third skips it.
         */
        const secondIssueVerdicts = new Map<RosterModelId, ResolutionVerdict>([
          [
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            'not-fixed',
          ],
          [
            SEAT_SYNTHETIC_VISION_WITHHELD,
            'not-fixed',
          ],
        ],);
        expect(
          await recheck({
            issues: TWO_OPEN_ISSUES,
            verdicts: everyCheckerSays({ verdict: 'not-fixed', },),
            secondIssueVerdicts,
          },),
        ).toEqual({
          retained: true,
          findings: [
            missingCheck({ issueId: AFTERNOON_ISSUE.issueId, modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, },),
            'refine-recheck-passed (2 issues)',
          ],
          readings: {
            [SUNBATHING_ISSUE.issueId]: {
              ballots: ballotsCast({ verdicts: everyCheckerSays({ verdict: 'not-fixed', },), },),
              configuredCheckers: 3,
              tally: {
                fixed: 0,
                notFixed: 3,
                worse: 0,
                resolved: false,
                regressed: false,
              },
            },
            [AFTERNOON_ISSUE.issueId]: {
              ballots: ballotsCast({ verdicts: secondIssueVerdicts, },),
              configuredCheckers: 3,
              tally: {
                fixed: 0,
                notFixed: 2,
                worse: 0,
                resolved: false,
                regressed: false,
              },
            },
          },
        },);
      },
    },),

    it({
      name: 'REFUSES a round on a bench the router left short, closed at its quorum of two, that heard one '
        + 'ballot on the second issue: refine-recheck-unheard names that issue against the quorum of two '
        + 'the stage closed on, not the bench quorum of three',
      fn: async () => {
        /**
         The two reachable seats' verdicts on the first issue.
         */
        const verdicts = new Map<RosterModelId, ResolutionVerdict>([
          [
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            'not-fixed',
          ],
          [
            SEAT_SYNTHETIC_VISION_WITHHELD,
            'not-fixed',
          ],
        ],);
        /**
         The first seat alone rules on the second issue.
         */
        const secondIssueVerdicts = new Map<RosterModelId, ResolutionVerdict>([
          [
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            'not-fixed',
          ],
        ],);
        expect(
          await recheck({
            issues: TWO_OPEN_ISSUES,
            verdicts,
            secondIssueVerdicts,
            checkerModelIds: FIVE_SEATS,
            refused: REFUSED_SEATS,
          },),
        ).toEqual({
          retained: false,
          findings: [
            ...SHORT_BENCH_FINDINGS,
            missingCheck({ issueId: AFTERNOON_ISSUE.issueId, modelId: SEAT_SYNTHETIC_VISION_WITHHELD, },),
            `refine-recheck-unheard (${AFTERNOON_ISSUE.issueId}: 1 ballot, quorum 2)`,
          ],
          readings: {
            [SUNBATHING_ISSUE.issueId]: {
              ballots: ballotsCast({ verdicts, },),
              configuredCheckers: 5,
              tally: {
                fixed: 0,
                notFixed: 2,
                worse: 0,
                resolved: false,
                regressed: false,
              },
            },
            [AFTERNOON_ISSUE.issueId]: {
              ballots: ballotsCast({ verdicts: secondIssueVerdicts, },),
              configuredCheckers: 5,
              tally: {
                fixed: 0,
                notFixed: 1,
                worse: 0,
                resolved: false,
                regressed: false,
              },
            },
          },
        },);
      },
    },),

    it({
      name: 'PASSES a round on a bench the router left short whose two reachable checkers rule on both '
        + 'issues: two ballots an issue meet the quorum of two the stage closed on, below the bench quorum '
        + 'of three',
      fn: async () => {
        /**
         The two reachable seats' verdicts, the same on both issues.
         */
        const verdicts = new Map<RosterModelId, ResolutionVerdict>([
          [
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            'not-fixed',
          ],
          [
            SEAT_SYNTHETIC_VISION_WITHHELD,
            'not-fixed',
          ],
        ],);
        /**
         Reading each issue holds: the two reachable seats' ballots.
         */
        const reading: IssueCheckerReading = {
          ballots: ballotsCast({ verdicts, },),
          configuredCheckers: 5,
          tally: {
            fixed: 0,
            notFixed: 2,
            worse: 0,
            resolved: false,
            regressed: false,
          },
        };
        expect(
          await recheck({
            issues: TWO_OPEN_ISSUES,
            verdicts,
            secondIssueVerdicts: verdicts,
            checkerModelIds: FIVE_SEATS,
            refused: REFUSED_SEATS,
          },),
        ).toEqual({
          retained: true,
          findings: [
            ...SHORT_BENCH_FINDINGS,
            'refine-recheck-passed (2 issues)',
          ],
          readings: {
            [SUNBATHING_ISSUE.issueId]: reading,
            [AFTERNOON_ISSUE.issueId]: reading,
          },
        },);
      },
    },),
  ],
},);
