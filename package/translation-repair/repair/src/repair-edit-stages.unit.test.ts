/**
 Tests for the resolution checker stage.

 `runCheckerStage` had no test, and it produces `tallies`, which decide
 `resolvedIssueIds`. Those feed candidate selection AND the milestone's
 headline resolution rate, so a defect here does not break a run; it moves the
 number the milestone is judged on.

 The stage's own arithmetic lives in `tallyResolutionChecks`, which is tested
 separately. What is untested here is the wiring: that every heard checker
 becomes exactly one ballot, that a lost voice reduces the count rather than
 silently counting as agreement, and that ballot irregularities reach the
 findings rather than being dropped between the two halves.

 Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type IssueAuthorship,
  UNATTRIBUTED_TEXT,
} from '@monochromatic-dev/translation-repair-house-rule';
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
import { runCheckerStage, } from '../dist/final/node/index.mjs';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_OPENROUTER_ONLY_CHECKER,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { catIssue, } from '@monochromatic-dev/translation-repair-shared-contract/ts/accepted-cat-issue.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the stages under test.
 */
const l = tagged({ tag: 'checker-stage-test', },);

/**
 Original the checkers judge against.
 */
const SOURCE_TEXT = '猫猫在窗台上睡觉。';

/**
 Candidate under check.
 */
const PATCHED_TEXT = 'The cat sleeps on the windowsill.';

/**
 Checker roster, larger than a majority so quorum arithmetic is visible.
 */
const CHECKERS = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
] as const;

/**
 Client answering each checker with a scripted report, or losing its voice.

 @param reportFor - report per model; returning undefined loses that voice

 @returns Client honoring that script

 @example
 ```ts
 const client = checkerClient({ reportFor: () => ({ checks: [], }), },);
 ```
 */
function checkerClient(
  { reportFor, }: { readonly reportFor: (modelId: string,) => unknown; },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Scripted report for the answering model.
       */
      const scripted = reportFor(request.modelId,);
      if (scripted === undefined) {
        return {
          kind: 'schema-mismatch',
          rawText: '',
          detail: 'scripted voice loss',
        };
      }
      // A REPORT THE STAGE'S GUARD REJECTS comes back as the real clients
      // return it, a schema mismatch, so the gather reads it as unreadable.
      if (!request.validate(scripted,)) {
        return {
          kind: 'schema-mismatch',
          rawText: JSON.stringify(scripted,),
          reason: 'caller-guard-rejected',
          detail: 'content parsed as JSON but failed the caller schema guard',
        };
      }
      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
}

/**
 Bench of five checkers, the three of `CHECKERS` first.
 */
const FIVE_CHECKERS: readonly RosterModelId[] = [
  ...CHECKERS,
  SEAT_HYPER_ONLY,
  SEAT_OPENROUTER_ONLY_CHECKER,
];

/**
 Seats of that bench no provider serves, leaving two reachable.
 */
const REFUSED_CHECKERS: readonly RosterModelId[] = [
  CHECKERS[2],
  SEAT_HYPER_ONLY,
  SEAT_OPENROUTER_ONLY_CHECKER,
];

/**
 Client that refuses the given seats as the router does, for a seat no
 provider serves, and hands every other exchange to the client it wraps.

 @param client - client answering the seats that are served

 @param refused - seats no provider serves

 @returns Client refusing those seats

 @example
 ```ts
 const client = refusingSeats({ client: checkerClient({ reportFor, },), refused: REFUSED_CHECKERS, },);
 ```
 */
function refusingSeats(
  {
    client,
    refused,
  }: {
    readonly client: ModelClient;
    readonly refused: readonly RosterModelId[];
  },
): ModelClient {
  return {
    ...client,
    chatJson: async function chatJson<ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> {
      if (refused.includes(request.modelId,)) {
        throw new NoProviderForModelError({
          modelId: request.modelId,
          reason: 'every provider serving this cat is out of treats',
        },);
      }
      return await client.chatJson(request,);
    },
  };
}

/**
 Runs the checker stage against a scripted client.

 @param client - scripted checker client

 @param issues - accepted issues under check

 @param checkerModelIds - bench the stage seats, the three-seat roster unless
 a case seats another

 @returns Stage result

 @example
 ```ts
 const result = await runStage({ client, issues, },);
 ```
 */
async function runStage(
  {
    client,
    issues,
    authorship = UNATTRIBUTED_TEXT,
    logger = l,
    checkerModelIds = CHECKERS,
  }: {
    readonly client: ModelClient;
    readonly issues: readonly AdjudicatedIssue[];
    readonly authorship?: IssueAuthorship;
    readonly logger?: typeof l;
    readonly checkerModelIds?: readonly RosterModelId[];
  },
) {
  return await runCheckerStage({
    client,
    checkerModelIds,
    sourceText: SOURCE_TEXT,
    patchedText: PATCHED_TEXT,
    issues,
    authorship,
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l: logger,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: runCheckerStage.name,
  children: [
    it({
      name: 'resolves an issue every checker called fixed, which is the '
        + 'ordinary case the resolution rate is built from',
      fn: async () => {
        /**
         Stage where all three checkers agree the defect is gone.
         */
        const result = await runStage({
          client: checkerClient({
            reportFor: () => ({
              checks: [
                {
                  issue: 1,
                  verdict: 'fixed',
                },
              ],
            }),
          },),
          issues: [catIssue({ issueId: 'adjudicated/tense', },),],
        },);

        expect(result.heardCheckers,).toBe(CHECKERS.length,);
        expect(result.tallies['adjudicated/tense']?.resolved,).toBe(true,);
      },
    },),

    it({
      name: 'PUBLISHES one line per ballot naming checker, issue, verdict and whether that checker '
        + 'wrote the text, because the tally it returns is a sum and a sum cannot be taken apart: '
        + 'without these lines no settled run can be re-read at another roster width',
      fn: async () => {
        const {
          lines,
          logger,
        } = capturingLoggerPair();

        /**
         Authorship making the first checker an author of this issue's text
         and leaving the other two outsiders, which is the mixed shape a
         roster permitted to self-certify produces.
         */
        const mixedAuthorship: IssueAuthorship = {
          perIssue: { 'adjudicated/tense': [CHECKERS[0],], },
          everyIssue: [],
        };
        await runStage({
          client: checkerClient({
            reportFor: () => ({
              checks: [
                {
                  issue: 1,
                  verdict: 'fixed',
                },
              ],
            }),
          },),
          issues: [catIssue({ issueId: 'adjudicated/tense', },),],
          authorship: mixedAuthorship,
          logger,
        },);

        /** Ballot lines the stage published. */
        const published = lines.filter(function isBallot(line,): boolean {
          return line.startsWith('checker-ballot ',);
        },);
        expect(published.length,).toBe(CHECKERS.length,);
        expect(published,).toContain(
          `checker-ballot ${CHECKERS[0]} adjudicated/tense fixed author`,
        );
        expect(published,).toContain(
          `checker-ballot ${CHECKERS[1]} adjudicated/tense fixed outsider`,
        );
      },
    },),

    it({
      name: 'CARRIES NO DOCUMENT TEXT into those lines, since they are published on every run over '
        + 'an archive nobody licensed us to copy',
      fn: async () => {
        const {
          lines,
          logger,
        } = capturingLoggerPair();
        await runStage({
          client: checkerClient({
            reportFor: () => ({
              checks: [
                {
                  issue: 1,
                  verdict: 'fixed',
                },
              ],
            }),
          },),
          issues: [catIssue({ issueId: 'adjudicated/tense', },),],
          logger,
        },);
        /** Ballot lines the stage published. */
        const published = lines.filter(function isBallot(candidate,): boolean {
          return candidate.startsWith('checker-ballot ',);
        },);

        // COUNTED BEFORE SCANNED, so this case cannot pass by there being
        // nothing to scan. A content check over an empty list is a check that
        // reports clean exactly when the thing it guards has disappeared.
        expect(published.length,).toBe(CHECKERS.length,);
        for (const line of published) {
          expect(line.includes(SOURCE_TEXT,),).toBe(false,);
          expect(line.includes(PATCHED_TEXT,),).toBe(false,);
        }
      },
    },),

    it({
      name: 'refuses to resolve an issue the checkers split on, since a '
        + 'resolution rate built from ties would credit repairs no majority '
        + 'agreed had landed',
      fn: async () => {
        /**
         Checkers that call the issue fixed; the rest disagree.
         */
        const agreeing: ReadonlySet<string> = new Set([CHECKERS[0],],);

        /**
         Stage where one checker says fixed and two say not.
         */
        const result = await runStage({
          client: checkerClient({
            reportFor: (modelId,) => ({
              checks: [
                {
                  issue: 1,
                  verdict: agreeing.has(modelId,)
                    ? 'fixed'
                    : 'not-fixed',
                },
              ],
            }),
          },),
          issues: [catIssue({ issueId: 'adjudicated/tense', },),],
        },);

        expect(result.tallies['adjudicated/tense']?.resolved,).toBe(false,);
      },
    },),

    it({
      name: 'COUNTS ONLY HEARD CHECKERS, so a lost voice reduces the count '
        + 'rather than passing silently as agreement: a stage that reported a '
        + 'full roster while two voices were lost would let one checker decide '
        + 'the resolution rate',
      fn: async () => {
        /**
         Checker that answers; the others lose their voices.
         */
        const answering: ReadonlySet<string> = new Set([CHECKERS[0],],);

        /**
         Stage where only one of three checkers replied.
         */
        const result = await runStage({
          client: checkerClient({
            reportFor: (modelId,) =>
              answering.has(modelId,)
                ? {
                  checks: [
                    {
                      issue: 1,
                      verdict: 'fixed',
                    },
                  ],
                }
                : undefined
            ,
          },),
          issues: [catIssue({ issueId: 'adjudicated/tense', },),],
        },);

        expect(result.heardCheckers,).toBe(1,);
        expect(result.findings.length,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'produces a tally for EVERY issue it was asked about, including '
        + 'ones no checker mentioned, so an issue silently dropped from a '
        + 'reply reads as unresolved rather than vanishing from the '
        + 'denominator',
      fn: async () => {
        /**
         Stage over two issues where every reply mentions only the first.
         */
        const result = await runStage({
          client: checkerClient({
            reportFor: () => ({
              checks: [
                {
                  issue: 1,
                  verdict: 'fixed',
                },
              ],
            }),
          },),
          issues: [
            catIssue({ issueId: 'adjudicated/tense', },),
            catIssue({ issueId: 'adjudicated/meaning', },),
          ],
        },);

        expect(Object.keys(result.tallies,).toSorted(),).toStrictEqual([
          'adjudicated/meaning',
          'adjudicated/tense',
        ],);
        expect(result.tallies['adjudicated/meaning']?.resolved,).toBe(false,);
      },
    },),

    it({
      name: 'carries BALLOT irregularities into the findings alongside quorum '
        + 'ones, so a checker numbering an issue that was never on its sheet '
        + 'reaches the scorecard instead of being dropped between the fan-out '
        + 'and the tally',
      fn: async () => {
        /**
         Stage where every checker judged the issue and also named an issue
         number off the sheet.
         */
        const result = await runStage({
          client: checkerClient({
            reportFor: () => ({
              checks: [
                {
                  issue: 1,
                  verdict: 'not-fixed',
                },
                {
                  issue: 9,
                  verdict: 'fixed',
                },
              ],
            }),
          },),
          issues: [catIssue({ issueId: 'adjudicated/tense', },),],
        },);

        expect(result.heardCheckers,).toBe(CHECKERS.length,);
        expect(result.findings.some(function namesOffSheet(finding,): boolean {
          return finding.includes('(9)',);
        },),).toBe(true,);
        expect(result.tallies['adjudicated/tense']?.resolved,).toBe(false,);
      },
    },),

    it({
      name: 'DOES NOT COUNT A REPORT WITH NO USABLE CHECK as a heard checker (ledger L8): XingZ6014 slice 87 '
        + 'closed its round on a heard report that checked nothing, and resolved an issue on the other ballot',
      fn: async () => {
        /**
         Unusable report by checker: empty, off the sheet, and an unknown verdict.
         */
        const unusable: Readonly<Record<string, unknown>> = {
          [CHECKERS[0]]: { checks: [], },
          [CHECKERS[1]]: {
            checks: [{
              issue: 9,
              verdict: 'fixed',
            },],
          },
          [CHECKERS[2]]: {
            checks: [{
              issue: 1,
              verdict: 'mostly',
            },],
          },
        };

        /**
         Stage where no checker cast a usable check.
         */
        const result = await runStage({
          client: checkerClient({
            reportFor: (modelId,) => unusable[modelId],
          },),
          issues: [catIssue({ issueId: 'adjudicated/tense', },),],
        },);

        expect(result.heardCheckers,).toBe(0,);
        expect(result.tallies['adjudicated/tense']?.resolved,).toBe(false,);
      },
    },),

    it({
      name: 'records a WORSE majority as a regression rather than merely as an '
        + 'absent resolution, because selection ranks a regression above total '
        + 'resolution and needs the two told apart',
      fn: async () => {
        /**
         Stage where every checker says the revision damaged the region.
         */
        const result = await runStage({
          client: checkerClient({
            reportFor: () => ({
              checks: [
                {
                  issue: 1,
                  verdict: 'worse',
                },
              ],
            }),
          },),
          issues: [catIssue({ issueId: 'adjudicated/tense', },),],
        },);

        expect(result.tallies['adjudicated/tense']?.resolved,).toBe(false,);
        expect(result.tallies['adjudicated/tense']?.regressed,).toBe(true,);
      },
    },),

    it({
      name: 'returns empty tallies when there was nothing to check, so a slice '
        + 'with no accepted issues costs no checker calls and reports no '
        + 'resolutions',
      fn: async () => {
        /**
         Stage over an empty issue list.
         */
        const result = await runStage({
          client: checkerClient({ reportFor: () => ({ checks: [], }), },),
          issues: [],
        },);

        expect(Object.keys(result.tallies,),).toStrictEqual([],);
      },
    },),

    it({
      name: 'ORDERS THE READINGS AND THE FINDINGS BY ROSTER whichever seats lost their first answer, since '
        + 'the gather hands voices back by the round that heard them and one input must read the same twice',
      fn: async () => {
        /**
         The first checker rules on both issues, the other two on the first
         alone.
         */
        const reports: Readonly<Record<string, unknown>> = {
          [CHECKERS[0]]: {
            checks: [
              {
                issue: 1,
                verdict: 'fixed',
              },
              {
                issue: 2,
                verdict: 'fixed',
              },
            ],
          },
          [CHECKERS[1]]: { checks: [{ issue: 1, verdict: 'fixed', },], },
          [CHECKERS[2]]: { checks: [{ issue: 1, verdict: 'fixed', },], },
        };

        /**
         What each run reads, keyed by the seats whose first answer was lost.
         */
        const runs = await Promise.all([[], [1, 2,], [0, 1,], [0, 2,],]
          .map(async function readRun(lost,): Promise<readonly [string, unknown,]> {
            /**
             Answers each seat has given so far.
             */
            const answered = new Map<string, number>();
            const result = await runStage({
              client: {
                ...checkerClient({ reportFor: (modelId,) => reports[modelId], },),
                chatJson: async function loseFirstAnswer<ValueT,>(
                  request: ChatJsonRequest<ValueT>,
                ): Promise<ChatJsonOutcome<ValueT>> {
                  /**
                   Answers this seat has given before this one.
                   */
                  const before = answered.get(request.modelId,) ?? 0;
                  answered.set(request.modelId, before + 1,);
                  if ((before === 0) && lost.some(function isLost(at,): boolean {
                    return CHECKERS[at] === request.modelId;
                  },))
                    return { kind: 'schema-mismatch', rawText: '', detail: 'scripted voice loss', };
                  return await checkerClient({ reportFor: (modelId,) => reports[modelId], },).chatJson(request,);
                },
              },
              issues: [
                catIssue({ issueId: 'adjudicated/tense', },),
                catIssue({ issueId: 'adjudicated/meaning', },),
              ],
            },);
            return [
              lost.join('+',),
              {
                heard: result.heardCheckers,
                tenseBallots: result.readings['adjudicated/tense']?.ballots.map(function toSeat(ballot,): string {
                  return ballot.modelId;
                },),
                findings: result.findings,
              },
            ];
          },),);
        const read = Object.fromEntries(runs,);

        /**
         What every run must read: all three heard, in roster order.
         */
        const expected = {
          heard: 3,
          tenseBallots: [...CHECKERS,],
          findings: [
            `missing-check (adjudicated/meaning, ${CHECKERS[1]}, unanswered)`,
            `missing-check (adjudicated/meaning, ${CHECKERS[2]}, unanswered)`,
          ],
        };
        expect(read,).toStrictEqual({
          '': expected,
          '1+2': expected,
          '0+1': expected,
          '0+2': expected,
        },);
      },
    },),

    it({
      name: 'NAMES each checker that skipped an issue, so two checkers skipping the second of two issues '
        + 'leave two findings that tell them apart and name the issue by id',
      fn: async () => {
        /**
         The first checker rules on both issues, the other two on the first
         alone.
         */
        const reports: Readonly<Record<string, unknown>> = {
          [CHECKERS[0]]: {
            checks: [
              {
                issue: 1,
                verdict: 'fixed',
              },
              {
                issue: 2,
                verdict: 'fixed',
              },
            ],
          },
          [CHECKERS[1]]: { checks: [{ issue: 1, verdict: 'fixed', },], },
          [CHECKERS[2]]: { checks: [{ issue: 1, verdict: 'fixed', },], },
        };
        const result = await runStage({
          client: checkerClient({ reportFor: (modelId,) => reports[modelId], },),
          issues: [
            catIssue({ issueId: 'adjudicated/tense', },),
            catIssue({ issueId: 'adjudicated/meaning', },),
          ],
        },);

        expect(result.findings,).toEqual([
          `missing-check (adjudicated/meaning, ${CHECKERS[1]}, unanswered)`,
          `missing-check (adjudicated/meaning, ${CHECKERS[2]}, unanswered)`,
        ],);
      },
    },),

    it({
      name: 'NAMES the cause when a checker\'s unknown verdict left an issue unanswered, beside the unknown '
        + 'verdict itself, and the checker that skipped the issue outright as unanswered',
      fn: async () => {
        /**
         The first checker rules on both issues, the second gives the second
         issue a verdict the tally does not know, the third skips it.
         */
        const reports: Readonly<Record<string, unknown>> = {
          [CHECKERS[0]]: {
            checks: [
              {
                issue: 1,
                verdict: 'fixed',
              },
              {
                issue: 2,
                verdict: 'fixed',
              },
            ],
          },
          [CHECKERS[1]]: {
            checks: [
              {
                issue: 1,
                verdict: 'fixed',
              },
              {
                issue: 2,
                verdict: 'mostly',
              },
            ],
          },
          [CHECKERS[2]]: { checks: [{ issue: 1, verdict: 'fixed', },], },
        };
        const result = await runStage({
          client: checkerClient({ reportFor: (modelId,) => reports[modelId], },),
          issues: [
            catIssue({ issueId: 'adjudicated/tense', },),
            catIssue({ issueId: 'adjudicated/meaning', },),
          ],
        },);

        expect(result.findings,).toEqual([
          'unknown-resolution-verdict (mostly)',
          `missing-check (adjudicated/meaning, ${CHECKERS[1]}, unknown-verdict)`,
          `missing-check (adjudicated/meaning, ${CHECKERS[2]}, unanswered)`,
        ],);
      },
    },),

    it({
      name: 'RETURNS the quorum of two the gather closed on for a bench of three with every seat served',
      fn: async () => {
        /**
         Stage where all three checkers rule on the one issue.
         */
        const result = await runStage({
          client: checkerClient({
            reportFor: () => ({
              checks: [
                {
                  issue: 1,
                  verdict: 'fixed',
                },
              ],
            }),
          },),
          issues: [catIssue({ issueId: 'adjudicated/tense', },),],
        },);

        expect(result.quorum,).toBe(2,);
      },
    },),

    it({
      name: 'RETURNS the quorum of three the gather closed on for a bench of five with every seat served',
      fn: async () => {
        /**
         Stage where all five checkers rule on the one issue.
         */
        const result = await runStage({
          client: checkerClient({
            reportFor: () => ({
              checks: [
                {
                  issue: 1,
                  verdict: 'fixed',
                },
              ],
            }),
          },),
          issues: [catIssue({ issueId: 'adjudicated/tense', },),],
          checkerModelIds: FIVE_CHECKERS,
        },);

        expect(result.quorum,).toBe(3,);
      },
    },),

    it({
      name: 'RETURNS the quorum of two the gather closed on for a bench of five whose last three seats no '
        + 'provider serves, beside the short-bench finding that names the same number',
      fn: async () => {
        /**
         Stage whose two reachable checkers rule on the one issue.
         */
        const result = await runStage({
          client: refusingSeats({
            client: checkerClient({
              reportFor: () => ({
                checks: [
                  {
                    issue: 1,
                    verdict: 'fixed',
                  },
                ],
              }),
            },),
            refused: REFUSED_CHECKERS,
          },),
          issues: [catIssue({ issueId: 'adjudicated/tense', },),],
          checkerModelIds: FIVE_CHECKERS,
        },);

        expect(result.quorum,).toBe(2,);
        expect(result.heardCheckers,).toBe(2,);
        expect(result.findings,).toEqual([
          'stage-short-bench (checker reachable 2 of 5, quorum 2)',
          `stage-voice-lost (checker ${CHECKERS[2]})`,
          `stage-voice-lost (checker ${SEAT_HYPER_ONLY})`,
          `stage-voice-lost (checker ${SEAT_OPENROUTER_ONLY_CHECKER})`,
          'stage-roster-incomplete (checker 2/5)',
        ],);
      },
    },),

    it({
      name: 'RETURNS the bench quorum of two when one seat of three is refused, since the two reachable seats '
        + 'meet it and the gather closes on the bench quorum rather than a short one',
      fn: async () => {
        /**
         Stage whose first two checkers are served and rule on the one issue.
         */
        const result = await runStage({
          client: refusingSeats({
            client: checkerClient({
              reportFor: () => ({
                checks: [
                  {
                    issue: 1,
                    verdict: 'fixed',
                  },
                ],
              }),
            },),
            refused: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
          },),
          issues: [catIssue({ issueId: 'adjudicated/tense', },),],
        },);

        expect(result.quorum,).toBe(2,);
        expect(result.findings.some(function isShortBench(finding,): boolean {
          return finding.startsWith('stage-short-bench',);
        },),).toBe(false,);
      },
    },),
  ],
},);
