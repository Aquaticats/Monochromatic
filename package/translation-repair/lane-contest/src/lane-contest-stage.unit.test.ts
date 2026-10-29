/**
 Tests for the lane contest: what the roster has to agree on before one
 candidate is called the winner, and what a decline means.

 WHY DECLINING IS COUNTED RATHER THAN DISCARDED. A judge answering `neither`
 has said something: that the two candidates differ only in wording. Treating
 that as a lost voice would make an undecidable slice indistinguishable from
 an unanswered one, and those need opposite handling.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { setTimeout as abortableWait, } from 'node:timers/promises';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  reachableQuorum,
  shortBenchStageFinding,
} from '@monochromatic-dev/translation-repair-stage';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  createSyntheticClient,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  contestLaneSlice,
  LANE_CONTEST_QUORUM,
  settleArchiveBallots,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { ballot, } from '@monochromatic-dev/translation-repair-test-fixture/ts/structural-ballot.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import {
  SCRIPTED_STAGE_SETTINGS,
  scriptedStageSettingsWaiting,
} from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 One contested slice, standing in for a corpus passage.
 */
const SUBJECT = {
  lineStructured: false,
  sourceText: '猫睡了。',
  incumbentText: 'The cat slept all afternoon in the sun.',
  repairText: 'The cat slept all afternoon.',
  translateText: 'The cat slept.',
};

/**
 Roster of three, the smallest that can produce a two-to-one split.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 Roster with two delayed eligible voices after two fast inadmissible ones.
 */
const ELIGIBILITY_ROSTER = [
  ...ROSTER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
] as const;

/**
 Bench of six, whose quorum of three the router can leave out of reach while
 the contest's own quorum stays in it.
 */
const SIX_SEAT_ROSTER = [
  ...ELIGIBILITY_ROSTER,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_VISION,
] as const;

/**
 Logger for the stage under test.
 */
const l = tagged({ tag: 'lane-contest-stage-test', },);

/**
 Delay of a seat that never answers inside a round: past the exchange
 bound, the shared hang stop, and cut when the round abandons the call, so no
 timer outlives it.
 */
const NEVER_ANSWERS_MS = HANG_STOP_MS * 2;

/**
 Builds a client whose models reply in roster order.

 @param replyByModel - reply body per model

 @param delayByModel - response delay per model position

 @returns Client over a canned transport

 @example
 ```ts
 const client = cannedClient({ replyByModel: [ballot({ choice: 'repair', },),], },);
 ```
 */
function cannedClient(
  {
    replyByModel,
    delayByModel = [],
  }: {
    readonly replyByModel: readonly string[];
    readonly delayByModel?: readonly number[];
  },
) {
  /**
   Calls served so far, so each model gets its own reply.
   */
  const served: string[] = [];
  return createSyntheticClient({
    apiKey: 'test-key',
    transport: async function cannedTransport(exchange,) {
      /**
       Which reply this call receives.
       */
      const at = served.length;
      served.push(exchange.label,);

      /**
       This model's reply text.
       */
      const content = replyByModel[at] ?? replyByModel[0] ?? '';
      // ABORTABLE, so a seat the round abandons stops waiting when the round
      // cuts it; `wait` from module-async-time takes no signal.
      await abortableWait(
        delayByModel[at] ?? 0,
        undefined,
        { signal: exchange.signal, },
      );
      return {
        status: 200,
        bodyText: `data: ${
          JSON.stringify({
            choices: [
              {
                index: 0,
                delta: { content, },
              },
            ],
          },)
        }\n\ndata: [DONE]\n\n`,
      };
    },
  },);
}

/**
 Builds a client the router refuses some seats on, every other seat naming
 one candidate.

 @param choice - candidate every seat that answers names

 @param refused - seats the router refuses for want of a wet provider

 @returns Scripted contest client

 @example
 ```ts
 const client = refusingClient({ choice: 'repair', refused: [], },);
 ```
 */
function refusingClient(
  {
    choice,
    refused,
  }: {
    readonly choice: 'repair' | 'translate' | 'neither';
    readonly refused: readonly RosterModelId[];
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the lane contest',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      if (refused.includes(request.modelId,)) {
        throw new NoProviderForModelError({
          modelId: request.modelId,
          reason: 'every provider serving this cat is out of budget',
        },);
      }
      /**
       This seat's ballot.
       */
      const value: unknown = JSON.parse(ballot({ choice, },),);
      if (!request.validate(value,))
        throw new Error('scripted lane contest ballot failed validation',);
      return {
        kind: 'ok',
        value,
        rawText: JSON.stringify(value,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by the lane contest',);
    },
  };
}

/**
 Runs one contest over a canned roster.

 @param replyByModel - reply body per model

 @returns What the roster settled on

 @example
 ```ts
 const outcome = await contest({ replyByModel: [], },);
 ```
 */
async function contest(
  { replyByModel, }: { readonly replyByModel: readonly string[]; },
) {
  return await contestLaneSlice({
    // Whole bench: this case scripts every seat and reads over the bench it wrote.
    fanOut: 'whole-bench',
    client: cannedClient({ replyByModel, },),
    modelIds: ROSTER,
    subject: SUBJECT,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    exchangeTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

/**
 Builds one ballot body that also answers the archive question.

 @param choice - candidate this judge names

 @param archive - what this judge makes of the archive rendering

 @returns Reply body a judge would return

 @example
 ```ts
 const body = archiveBallot({ choice: 'neither', archive: 'flawed', },);
 ```
 */
function archiveBallot(
  {
    choice,
    archive,
  }: {
    readonly choice: string;
    readonly archive: string;
  },
): string {
  return JSON.stringify({
    assessment: 'the original supports it',
    dropped: [],
    unsupported: [],
    verdict: choice,
    verdictOnArchive: archive,
  },);
}

/**
 Runs one contest over a slice whose archive rendering is absent.

 @param replyByModel - reply body per model

 @returns What the roster settled on

 @example
 ```ts
 const outcome = await contestWithNoArchive({ replyByModel: [], },);
 ```
 */
async function contestWithNoArchive(
  { replyByModel, }: { readonly replyByModel: readonly string[]; },
) {
  return await contestLaneSlice({
    // Whole bench: this case scripts every seat and reads over the bench it wrote.
    fanOut: 'whole-bench',
    client: cannedClient({ replyByModel, },),
    modelIds: ROSTER,
    subject: {
      ...SUBJECT,
      incumbentText: '',
    },
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    exchangeTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: contestLaneSlice.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHIPS the candidate two judges named against one for the other',
          fn: async () => {
            const outcome = await contest({
              replyByModel: [
                ballot({ choice: 'repair', },),
                ballot({ choice: 'repair', },),
                ballot({ choice: 'translate', },),
              ],
            },);
            expect(outcome.choice,).toBe('repair',);
            expect(outcome.usable,).toBe(3,);
            expect(outcome.findings,).toEqual([],);
          },
        },),
        it({
          name:
            'NAMES THE JUDGE on every ballot (hulicaijia26, 2026-09-26: four of five contest ballots '
              + 'called the archive\'s wordplay note an addition and no record said which models cast them)',
          fn: async () => {
            const outcome = await contest({
              replyByModel: [
                ballot({ choice: 'repair', },),
                ballot({ choice: 'repair', },),
                ballot({ choice: 'translate', },),
              ],
            },);
            expect(outcome.ballots.map(function judgeOf(cast,) {
              return cast.modelId;
            },),).toEqual([...ROSTER,],);
          },
        },),
        it({
          name: 'RECORDS RAW HALF-QUORUM BALLOTS without waiting for delayed seats excluded downstream',
          fn: async () => {
            const outcome = await contestLaneSlice({
              // Whole bench: this case scripts every seat and reads over the bench it wrote.
              fanOut: 'whole-bench',
              client: cannedClient({
                replyByModel: [
                  ballot({ choice: 'repair', },),
                  ballot({ choice: 'repair', },),
                  ballot({ choice: 'translate', },),
                  ballot({ choice: 'translate', },),
                ],
                // NEVER, NOT 100 MS (ledger T5): at 0.2 CPU the two prompt seats
                // took longer than 100 ms and the delayed ones arrived too, 2 of 5
                // runs; seats that answer only past the exchange bound are
                // abandoned at quorum whatever the machine's speed.
                delayByModel: [
                  0,
                  0,
                  NEVER_ANSWERS_MS,
                  NEVER_ANSWERS_MS,
                ],
              },),
              modelIds: ELIGIBILITY_ROSTER,
              subject: {
                ...SUBJECT,
                ineligibleCandidates: ['repair',],
              },
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              exchangeTimeoutMs: HANG_STOP_MS,
              stageSettings: scriptedStageSettingsWaiting({ graceMs: 0, },),
              l,
            },);
            expect(outcome.usable,).toBe(2,);
            expect(outcome.ballots.filter(function choseTranslate(ballotValue,): boolean {
              return ballotValue.choice === 'translate';
            },),).toHaveLength(0,);
            // RAW STAGE RETAINS BALLOTS UNCHANGED. `lane-contest-driver` applies
            // deterministic eligibility, settles this as neither, and refuses
            // persistence, pinned by its unsafe front-matter winner test.
            expect(outcome.choice,).toBe('repair',);
          },
        },),

        it({
          name: 'KEEPS DELAYED ELIGIBLE VOICES that arrive inside bounded grace',
          fn: async () => {
            const outcome = await contestLaneSlice({
              // Whole bench: this case scripts every seat and reads over the bench it wrote.
              fanOut: 'whole-bench',
              client: cannedClient({
                replyByModel: [
                  ballot({ choice: 'repair', },),
                  ballot({ choice: 'repair', },),
                  ballot({ choice: 'translate', },),
                  ballot({ choice: 'translate', },),
                ],
                delayByModel: [
                  0,
                  0,
                  30,
                  30,
                ],
              },),
              modelIds: ELIGIBILITY_ROSTER,
              subject: {
                ...SUBJECT,
                ineligibleCandidates: ['repair',],
              },
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              exchangeTimeoutMs: HANG_STOP_MS,
              // FAR LONGER THAN THE DELAYS, since the round ends when every voice
              // has answered and so never waits this long. A window a loaded
              // machine can outrun would fail the case on a slow timer alone.
              stageSettings: SCRIPTED_STAGE_SETTINGS,
              l,
            },);
            expect(outcome.usable,).toBe(4,);
            expect(outcome.ballots.filter(function choseTranslate(ballotValue,): boolean {
              return ballotValue.choice === 'translate';
            },),).toHaveLength(2,);
          },
        },),

        it({
          name: 'REFUSES to ship on a tie, rather than picking by list order',
          fn: async () => {
            // A CANDIDATE THAT TIES HAS NOT BEEN CHOSEN. Shipping either here would
            // be deciding what a reader sees on a memorial page by which lane the
            // code happens to name first.
            const outcome = await contest({
              replyByModel: [
                ballot({ choice: 'repair', },),
                ballot({ choice: 'translate', },),
                ballot({ choice: 'neither', },),
              ],
            },);
            expect(outcome.choice,).toBe('neither',);
            expect(outcome.usable,).toBe(3,);
          },
        },),
        it({
          name: 'REFUSES a candidate only ONE judge named, however the rest split',
          fn: async () => {
            const outcome = await contest({
              replyByModel: [
                ballot({ choice: 'repair', },),
                ballot({ choice: 'neither', },),
                ballot({ choice: 'neither', },),
              ],
            },);
            expect(outcome.choice,).toBe('neither',);
          },
        },),
        it({
          name: 'READS a unanimous decline as a settled verdict, with NO finding',
          fn: async () => {
            // THE DISTINCTION THE FINDING EXISTS FOR: three judges saying the
            // candidates are equally faithful is a decided slice. Three judges
            // never answering is not, and only the second is a finding.
            const outcome = await contest({
              replyByModel: [
                ballot({ choice: 'neither', },),
                ballot({ choice: 'neither', },),
                ballot({ choice: 'neither', },),
              ],
            },);
            expect(outcome.choice,).toBe('neither',);
            expect(outcome.usable,).toBe(3,);
            expect(outcome.findings,).toEqual([],);
          },
        },),
        it({
          name: 'REPORTS a finding when too few ballots survived to settle anything',
          fn: async () => {
            const outcome = await contest({
              replyByModel: [
                ballot({ choice: 'repair', },),
                'not json at all',
                'also not json',
              ],
            },);
            expect(outcome.choice,).toBe('neither',);
            expect(outcome.usable,).toBe(1,);
            expect(outcome.findings.length,).toBe(1,);
          },
        },),
        it({
          name: 'SAYS THE BENCH WAS SHORT when the router refuses seats past the bench quorum, and still settles on '
            + 'the ballots it heard (ledger X8)',
          fn: async () => {
            /**
             Seats the router refuses, leaving the contest's quorum in reach and
             the bench's out of it.
             */
            const refused = SIX_SEAT_ROSTER.slice(LANE_CONTEST_QUORUM,);

            /**
             The bench's quorum once those seats are out of reach.
             */
            const quorum = reachableQuorum({
              benchSize: SIX_SEAT_ROSTER.length,
              unreachable: refused.length,
            },);
            expect(quorum.short,).toBe(true,);

            /**
             What the two reachable seats settled.
             */
            const outcome = await contestLaneSlice({
              // Whole bench: this case scripts every seat and reads over the bench it wrote.
              fanOut: 'whole-bench',
              client: refusingClient({
                choice: 'repair',
                refused,
              },),
              modelIds: SIX_SEAT_ROSTER,
              subject: SUBJECT,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(outcome.choice,).toBe('repair',);
            expect(outcome.usable,).toBe(SIX_SEAT_ROSTER.length - refused.length,);
            expect(outcome.findings,).toStrictEqual([
              shortBenchStageFinding({
                stage: 'lane-contest',
                quorum,
                benchSize: SIX_SEAT_ROSTER.length,
              },),
            ],);
          },
        },),
        it({
          name: 'KEEPS the findings each judge reported, for the audit trail',
          fn: async () => {
            const outcome = await contest({
              replyByModel: [
                JSON.stringify({
                  assessment: 'the repair candidate adds an afternoon the original never mentions',
                  dropped: [],
                  unsupported: [ 'repair', ],
                  verdict: 'translate',
                },),
                JSON.stringify({
                  assessment: 'same',
                  dropped: [],
                  unsupported: [ 'repair', ],
                  verdict: 'translate',
                },),
                ballot({ choice: 'neither', },),
              ],
            },);
            expect(outcome.choice,).toBe('translate',);
            expect(outcome.ballots.at(0,)?.unsupported,).toEqual([ 'repair', ],);
          },
        },),
      ],
    },),

    describe({
      name: 'archive answers on a slice with no archive',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS THE ANSWERS where an archive rendering exists, which is the positive control: '
            + 'without it, a stripped result would look the same as a roster that never answered',
          fn: async () => {
            const outcome = await contest({
              replyByModel: [
                archiveBallot({ choice: 'neither', archive: 'flawed', },),
                archiveBallot({ choice: 'neither', archive: 'flawed', },),
                archiveBallot({ choice: 'neither', archive: 'flawed', },),
              ],
            },);

            expect(settleArchiveBallots({ ballots: outcome.ballots, },),).toBe('declined',);
          },
        },),

        it({
          name: 'DROPS THE ANSWERS where the archive rendering is absent, because the sheet then shows '
            + 'judges an empty block while the schema still asks whether it is publishable, so the '
            + 'answers that come back are about nothing at all',
          fn: async () => {
            const outcome = await contestWithNoArchive({
              replyByModel: [
                archiveBallot({ choice: 'neither', archive: 'flawed', },),
                archiveBallot({ choice: 'neither', archive: 'flawed', },),
                archiveBallot({ choice: 'neither', archive: 'flawed', },),
              ],
            },);

            expect(settleArchiveBallots({ ballots: outcome.ballots, },),).toBe('unjudged',);
            expect(outcome.usable,).toBe(3,);
            expect(outcome.choice,).toBe('neither',);
          },
        },),
      ],
    },),
  ],
},);
