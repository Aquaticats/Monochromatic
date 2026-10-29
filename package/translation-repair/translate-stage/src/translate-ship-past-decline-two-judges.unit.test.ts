/**
 Guards the owner's answer of 2026-10-25, "Two judges must name it": a challenge round declined over wording that
 cannot ship (`settleAbsentDecline`, `translate-runoff-tie.ts`) used to ship a candidate by a fixed preference, the
 repair lane's text first, the translate lane's next, then slate order, whatever any judge named. XingZ6021 shipped
 five slices that way after a bench of one reachable judge named one candidate, and the preference chose another.
 One judge alone never decides (the owner's older standing rule), so the round now ships a candidate only when two or
 more judges named it, by the same preference among those; otherwise the archive wording stays, and where there is no
 archive the absence the retry reads is raised as before, which leaves the passage to the existing source-text-with-
 a-note path.

 The owner's rulings of 2026-10-25 are quoted where the rule is stated
 (`doc/design-commitments-votes-repairs-and-run-offs.md`): "floor of 3 - don't set a floor.", "Two ballots decide
 (Recommended)" and "Every weight floor".

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  Candidate,
  CandidateWeight,
  SelectionBallot,
} from '@monochromatic-dev/translation-repair-house-rule';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  laneCandidate,
  TranslateAbsenceError,
  type TranslateCandidateValue,
} from '@monochromatic-dev/translation-repair-translate-floor';
import {
  judgeSlateWithRetry,
  settleAbsentDecline,
  type TranslateStageResult,
} from '../dist/final/node/index.mjs';
import { SEAT_OPENROUTER_DECISIONS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { candidateCarrying, } from '@monochromatic-dev/translation-repair-test-fixture/ts/translate-ballot.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the stage writes through, whose output is not under test.
 */
const l = tagged({ tag: 'translate-ship-past-decline-two-judges-test', },);

/**
 Invented original: the cat dozed on the sill, tail by the radiator.
 */
const SOURCE = '猫猫在窗台上打盹，尾巴垂在暖气片旁边。';

/**
 Archive wording the deterministic rule refused, which stays where no judge pair names a candidate.
 */
const ARCHIVE = 'The cat is doing the sleeping on the windowsill, a tail hanging by the radiator, purring for you.';

/**
 Finding a declined round records where the archive wording stays.
 */
const ARCHIVE_KEPT_FINDING = 'translate-slate-declined-archive-kept';

/**
 Finding a declined round shipped by preference records, before its basis.
 */
const SHIPPED_PAST_DECLINE = 'translate-slate-declined-shipped-by-preference';

/**
 Judges who wrote nothing, so every ballot carries a whole weight.
 */
const JUDGES: readonly RosterModelId[] = [
  'hf:cat/Cat-C',
  'hf:cat/Cat-F',
  'hf:cat/Cat-G',
].map(function toId(id,) {
  return id as unknown as RosterModelId;
},);

/**
 Writer of the two fresh renderings.
 */
const WRITER = 'hf:cat/Cat-A' as unknown as RosterModelId;

/**
 What the writers rendered and what the lanes offer, each carrying a word the scripts ask for.
 */
const TEXTS = {
  dozes: 'The cat dozes on the windowsill, tail draped beside the radiator.',
  naps: 'A cat naps on the sill, its tail hanging near the heater.',
  curls: 'The cat curls up on the ledge, tail tucked by the warmth.',
  yawns: 'The cat yawns on the sill, tail resting against the pipe.',
} as const;

/**
 A fresh rendering written by a model.

 @param text - rendering

 @returns Candidate as the slate carries it

 @example
 ```ts
 const candidate = writtenCandidate({ text: TEXTS.dozes, },);
 ```
 */
function writtenCandidate({ text, }: { readonly text: string; },): Candidate<TranslateCandidateValue> {
  return {
    producer: {
      kind: 'model',
      modelId: WRITER,
    },
    value: {
      text,
      origin: 'fresh',
    },
    rendered: text,
  };
}

/**
 Client whose judges answer as the script says and every other ask fails.

 @param wants - needle each seat backs in every judging, or `reject`

 @returns Client over the candidate ballots only

 @example
 ```ts
 const client = judgeClient({ wants: { 'hf:cat/Cat-C': 'dozes', }, },);
 ```
 */
function judgeClient(
  { wants, }: { readonly wants: Readonly<Record<string, string>>; },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the translate lane',);
    },
    quotas: async () => {
      throw new Error('quotas unused by the translate lane',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Sheet as the judge reads it.
       */
      const content = request.messages
        .map(function textOf(message,): string {
          return messageText({ message, },);
        },)
        .join('\n',);
      /**
       What this seat backs, a rejection when the script says nothing.
       */
      const wanted = wants[request.modelId] ?? 'reject';
      /**
       Ballot for this ask.
       */
      const ballot: unknown = {
        best: (wanted === 'reject') ? 0 : candidateCarrying({
          content,
          needle: wanted,
        },),
        assessment: `fixture reason of ${request.modelId}`,
      };
      if (!request.validate(ballot,))
        throw new Error('the fixture ballot failed the wire guard',);
      return {
        kind: 'ok',
        value: ballot as ValueT,
        rawText: JSON.stringify(ballot,),
      };
    },
  };
}

/**
 Judges a withheld passage's slate through the retry: the archive wording exists and cannot ship, so it is not on the
 slate, and the slate offers two fresh renderings and both lane texts.

 @param wants - needle each seat backs in every judging

 @param withheldStanding - whether the archive wording exists and was refused, or the passage has none

 @returns What the retry settled on

 @example
 ```ts
 const result = await judgedWith({ wants: {}, withheldStanding: true, },);
 ```
 */
async function judgedWith(
  {
    wants,
    withheldStanding,
  }: {
    readonly wants: Readonly<Record<string, string>>;
    readonly withheldStanding: boolean;
  },
): Promise<TranslateStageResult> {
  return await judgeSlateWithRetry({
    judging: {
      client: judgeClient({ wants, },),
      produced: {
        candidates: [
          writtenCandidate({ text: TEXTS.dozes, },),
          writtenCandidate({ text: TEXTS.naps, },),
          laneCandidate({
            lane: 'repair',
            text: TEXTS.curls,
          },),
          laneCandidate({
            lane: 'translate',
            text: TEXTS.yawns,
          },),
        ],
        heardTranslators: 2,
        findings: [],
      },
      judgeModelIds: JUDGES,
      sourceText: SOURCE,
      incumbentText: withheldStanding ? ARCHIVE : '',
      incumbentKind: 'absent',
      lineStructured: false,
      withheldStanding,
      fanOut: 'whole-bench',
      signal: AbortSignal.timeout(HANG_STOP_MS,),
      perCallTimeoutMs: HANG_STOP_MS,
      l,
      stageSettings: SCRIPTED_STAGE_SETTINGS,
    },
  },);
}

/**
 One ballot as a declined round records it.

 @param best - one-based candidate the judge named, zero for none

 @param reason - judge's stated reason

 @param modelId - judge, first of the written judges by default

 @returns Ballot at a whole weight

 @example
 ```ts
 const ballot = ballotFor({ best: 1, reason: 'reads best', },);
 ```
 */
function ballotFor(
  {
    best,
    reason,
    modelId = JUDGES[0] ?? WRITER,
  }: {
    readonly best: number;
    readonly reason: string;
    readonly modelId?: RosterModelId;
  },
): SelectionBallot {
  return {
    modelId,
    best,
    reason,
    weight: 1,
    selfVote: false,
  };
}

/**
 What a candidate drew.

 @param index - one-based position
 @param ballots - judges that named it
 @param weight - summed weight

 @returns Row as the round records it

 @example
 ```ts
 const row = drew({ index: 1, ballots: 2, weight: 2, },);
 ```
 */
function drew(
  {
    index,
    ballots,
    weight,
  }: {
    readonly index: number;
    readonly ballots: number;
    readonly weight: number;
  },
): CandidateWeight {
  return {
    index,
    ballots,
    fullVotes: ballots,
    selfVotes: 0,
    weight,
  };
}

/**
 Settles a declined challenge round over a fixed slate of three candidates, the writer's rendering first, the repair
 lane's second and the translate lane's third, with the ballots and rows given.

 @param perCandidate - what the three drew

 @param ballots - ballots cast

 @param shipPastDecline - whether the caller ships past a decline

 @param challenged - whether this is the challenge round

 @returns Stage result the decline comes to

 @example
 ```ts
 const result = settledWith({ perCandidate: [], ballots: [], shipPastDecline: true, challenged: true, },);
 ```
 */
function settledWith(
  {
    perCandidate,
    ballots,
    shipPastDecline,
    challenged,
  }: {
    readonly perCandidate: readonly CandidateWeight[];
    readonly ballots: readonly SelectionBallot[];
    readonly shipPastDecline: boolean;
    readonly challenged: boolean;
  },
): TranslateStageResult {
  return settleAbsentDecline({
    challenged,
    outcome: {
      kind: 'declined',
      reason: 'judges tied',
      disposition: 'indecision',
      tally: {
        judgesAvailable: 3,
        ballots: ballots.length,
        abstentions: 0,
        selfVotes: 0,
      },
      findings: [],
      ballots,
      perCandidate,
    },
    rotated: [
      writtenCandidate({ text: TEXTS.dozes, },),
      laneCandidate({
        lane: 'repair',
        text: TEXTS.curls,
      },),
      laneCandidate({
        lane: 'translate',
        text: TEXTS.yawns,
      },),
    ],
    keepIncumbent: {
      text: ARCHIVE,
      origin: 'incumbent',
      producer: {
        kind: 'incumbent',
        matched: [],
      },
      voteWeight: 0,
      tally: {
        judgesAvailable: 0,
        ballots: 0,
        abstentions: 0,
        selfVotes: 0,
      },
      ballots: [],
      heardTranslators: 2,
      candidateCount: 3,
      slate: [],
      selectedIndex: 0,
      shippedIndex: 0,
      perCandidate: [],
    },
    declineFindings: ['translate-declined (indecision)',],
    declined: 'declined-indecision',
    shipPastDecline,
    l,
  },);
}

await describe({
  name: 'a declined challenge round ships a candidate only when two judges named it (owner, 2026-10-25)',
  children: [
    describe({
      name: judgeSlateWithRetry.name,
      children: [
        it({
          name: 'KEEPS THE ARCHIVE WORDING where one judge alone named a candidate, though the repair lane\'s text '
            + 'stands first by preference: the fixed order ships nothing no judge pair backed',
          fn: async () => {
            const result = await judgedWith({
              wants: { [JUDGES[0] ?? '']: 'dozes', },
              withheldStanding: true,
            },);
            expect({
              text: result.text,
              origin: result.origin,
              shippedPastDecline: result.shippedPastDecline,
              kept: result.findings.includes(ARCHIVE_KEPT_FINDING,),
              shipped: result.findings.some(function byPreference(finding,): boolean {
                return finding.startsWith(SHIPPED_PAST_DECLINE,);
              },),
            },).toEqual({
              text: ARCHIVE,
              origin: 'incumbent',
              shippedPastDecline: undefined,
              kept: true,
              shipped: false,
            },);
          },
        },),

        it({
          name: 'KEEPS THE ARCHIVE WORDING where every judge rejected every candidate',
          fn: async () => {
            const result = await judgedWith({
              wants: {},
              withheldStanding: true,
            },);
            expect({
              text: result.text,
              origin: result.origin,
              kept: result.findings.includes(ARCHIVE_KEPT_FINDING,),
            },).toEqual({
              text: ARCHIVE,
              origin: 'incumbent',
              kept: true,
            },);
          },
        },),

        it({
          name: 'STILL RAISES the absence where the passage has no archive wording at all, which leaves it to the '
            + 'path that ships the source text under the untranslated note (the positive control: it passes on '
            + 'the preference rule as well)',
          fn: async () => {
            /**
             What the judging raised, if anything.
             */
            const raised = await (async function attempt(): Promise<unknown> {
              try {
                await judgedWith({
                  wants: {},
                  withheldStanding: false,
                },);
                return 'settled';
              }
              catch (error) {
                return error;
              }
            })();
            expect(raised instanceof TranslateAbsenceError,).toBe(true,);
          },
        },),

        it({
          name: 'STILL SELECTS a candidate two judges named, over a withheld archive (the positive control: the '
            + 'decision never reaches the decline path)',
          fn: async () => {
            const result = await judgedWith({
              wants: {
                [JUDGES[0] ?? '']: 'naps',
                [JUDGES[1] ?? '']: 'naps',
              },
              withheldStanding: true,
            },);
            expect({
              text: result.text,
              decision: result.decision,
            },).toEqual({
              text: TEXTS.naps,
              decision: 'judged',
            },);
          },
        },),

        it({
          name: 'KEEPS THE ARCHIVE WORDING where a tie across every finalist has one ballot each (class one hundred '
            + 'seventy-four, settled by the owner\'s "Two judges must name it"): the fixed order no longer ships the '
            + 'repair lane\'s text over candidates one judge named apiece',
          fn: async () => {
            const result = await judgedWith({
              wants: {
                [JUDGES[0] ?? '']: 'naps',
                [JUDGES[1] ?? '']: 'curls',
              },
              withheldStanding: true,
            },);
            expect({
              text: result.text,
              origin: result.origin,
              kept: result.findings.includes(ARCHIVE_KEPT_FINDING,),
              tieBroken: result.findings.some(function isTieBroken(finding,): boolean {
                return finding.startsWith('translate-runoff-tie-broken',);
              },),
            },).toEqual({
              text: ARCHIVE,
              origin: 'incumbent',
              kept: true,
              tieBroken: false,
            },);
          },
        },),
      ],
    },),

    describe({
      name: settleAbsentDecline.name,
      children: [
        it({
          name: 'SHIPS THE ONE CANDIDATE TWO JUDGES NAMED, not the repair lane\'s text that the fixed order puts first '
            + 'over a candidate one judge named',
          fn: async () => {
            const result = settledWith({
              perCandidate: [
                drew({ index: 1, ballots: 2, weight: 1, },),
                drew({ index: 2, ballots: 1, weight: 1, },),
                drew({ index: 3, ballots: 0, weight: 0, },),
              ],
              ballots: [
                ballotFor({ best: 1, reason: 'first reads best', },),
                ballotFor({ best: 1, reason: 'first reads best again', },),
                ballotFor({ best: 2, reason: 'the ledge reads best', },),
              ],
              shipPastDecline: true,
              challenged: true,
            },);
            expect({
              text: result.text,
              selectedIndex: result.selectedIndex,
              basis: result.shippedPastDecline?.basis,
              objections: result.shippedPastDecline?.objections,
            },).toEqual({
              text: TEXTS.dozes,
              selectedIndex: 1,
              basis: 'slate order',
              objections: ['the ledge reads best',],
            },);
          },
        },),

        it({
          name: 'RECORDS NO OBJECTION from a ballot whose reason is invisible characters alone, which trim() keeps '
            + 'and no refiner could read (ledger B40), nor from a decision seat\'s ballot, whose reason is a '
            + 'distribution and objects to nothing in words (ledger B125), beside a written objection',
          fn: async () => {
            const result = settledWith({
              perCandidate: [
                drew({ index: 1, ballots: 2, weight: 1, },),
                drew({ index: 2, ballots: 1, weight: 1, },),
                drew({ index: 3, ballots: 0, weight: 0, },),
              ],
              ballots: [
                ballotFor({ best: 2, reason: '\u{200B}\u{3164}', },),
                ballotFor({
                  best: 2,
                  reason: 'a distribution over the candidates',
                  modelId: SEAT_OPENROUTER_DECISIONS,
                },),
                ballotFor({ best: 2, reason: 'the ledge reads best', },),
              ],
              shipPastDecline: true,
              challenged: true,
            },);
            expect({
              text: result.text,
              objections: result.shippedPastDecline?.objections,
            },).toEqual({
              text: TEXTS.dozes,
              objections: ['the ledge reads best',],
            },);
          },
        },),

        it({
          name: 'SHIPS THE PREFERRED OF THE CANDIDATES TWO JUDGES NAMED when several qualify: the translate lane\'s '
            + 'text over the writer\'s, and never the repair lane\'s that only one judge named',
          fn: async () => {
            const result = settledWith({
              perCandidate: [
                drew({ index: 1, ballots: 2, weight: 1, },),
                drew({ index: 2, ballots: 1, weight: 0.5, },),
                drew({ index: 3, ballots: 2, weight: 1, },),
              ],
              ballots: [],
              shipPastDecline: true,
              challenged: true,
            },);
            expect({
              text: result.text,
              basis: result.shippedPastDecline?.basis,
            },).toEqual({
              text: TEXTS.yawns,
              basis: 'translate lane',
            },);
          },
        },),

        it({
          name: 'KEEPS THE ARCHIVE WORDING, naming the decline and why, where no candidate drew two ballots',
          fn: async () => {
            const result = settledWith({
              perCandidate: [
                drew({ index: 1, ballots: 1, weight: 1, },),
                drew({ index: 2, ballots: 1, weight: 1, },),
                drew({ index: 3, ballots: 0, weight: 0, },),
              ],
              ballots: [
                ballotFor({ best: 1, reason: 'first reads best', },),
                ballotFor({ best: 2, reason: 'the ledge reads best', },),
              ],
              shipPastDecline: true,
              challenged: true,
            },);
            expect({
              text: result.text,
              origin: result.origin,
              decision: result.decision,
              findings: result.findings,
              shippedPastDecline: result.shippedPastDecline,
              ballots: result.ballots.length,
            },).toEqual({
              text: ARCHIVE,
              origin: 'incumbent',
              decision: 'declined-indecision',
              findings: ['translate-declined (indecision)', ARCHIVE_KEPT_FINDING,],
              shippedPastDecline: undefined,
              ballots: 2,
            },);
          },
        },),

        it({
          name: 'KEEPS THE ARCHIVE WORDING where every candidate tied at the top with one ballot each (class one hundred '
            + 'seventy-four, settled by "Two judges must name it"), naming the decline and why, and breaks no tie',
          fn: async () => {
            const result = settledWith({
              perCandidate: [
                drew({ index: 1, ballots: 1, weight: 1, },),
                drew({ index: 2, ballots: 1, weight: 1, },),
                drew({ index: 3, ballots: 1, weight: 1, },),
              ],
              ballots: [
                ballotFor({ best: 1, reason: 'first reads best', },),
                ballotFor({ best: 2, reason: 'the ledge reads best', },),
                ballotFor({ best: 3, reason: 'the pipe reads best', },),
              ],
              shipPastDecline: true,
              challenged: true,
            },);
            expect({
              text: result.text,
              origin: result.origin,
              findings: result.findings,
            },).toEqual({
              text: ARCHIVE,
              origin: 'incumbent',
              findings: ['translate-declined (indecision)', ARCHIVE_KEPT_FINDING,],
            },);
          },
        },),

        it({
          name: 'STILL BREAKS A TIE ACROSS EVERY CANDIDATE BY PREFERENCE where each drew two ballots (class one '
            + 'hundred seventy-four, the positive control: it passes on the old rule as well)',
          fn: async () => {
            const result = settledWith({
              perCandidate: [
                drew({ index: 1, ballots: 2, weight: 1.5, },),
                drew({ index: 2, ballots: 2, weight: 1.5, },),
                drew({ index: 3, ballots: 2, weight: 1.5, },),
              ],
              ballots: [],
              shipPastDecline: true,
              challenged: true,
            },);
            expect({
              text: result.text,
              findings: result.findings,
            },).toEqual({
              text: TEXTS.curls,
              findings: ['translate-declined (indecision)', 'translate-runoff-tie-broken (repair lane)',],
            },);
          },
        },),

        it({
          name: 'BREAKS A TIE ACROSS EVERY CANDIDATE BY PREFERENCE AMONG THOSE TWO JUDGES NAMED only: the translate '
            + 'lane\'s text that drew two ballots, not the repair lane\'s text that stands first and drew one',
          fn: async () => {
            const result = settledWith({
              perCandidate: [
                drew({ index: 1, ballots: 1, weight: 1, },),
                drew({ index: 2, ballots: 1, weight: 1, },),
                drew({ index: 3, ballots: 2, weight: 1, },),
              ],
              ballots: [],
              shipPastDecline: true,
              challenged: true,
            },);
            expect({
              text: result.text,
              findings: result.findings,
            },).toEqual({
              text: TEXTS.yawns,
              findings: ['translate-declined (indecision)', 'translate-runoff-tie-broken (translate lane)',],
            },);
          },
        },),

        it({
          name: 'RAISES the absence where a tie across every candidate has one ballot each and the caller does not '
            + 'ship past a decline, since no judge pair backed a candidate and no wording stands to keep',
          fn: async () => {
            expect(function singleBallotTie() {
              settledWith({
                perCandidate: [
                  drew({ index: 1, ballots: 1, weight: 1, },),
                  drew({ index: 2, ballots: 1, weight: 1, },),
                  drew({ index: 3, ballots: 1, weight: 1, },),
                ],
                ballots: [],
                shipPastDecline: false,
                challenged: true,
              },);
            },).toThrow(TranslateAbsenceError,);
          },
        },),

        it({
          name: 'RAISES the absence where the caller does not ship past a decline, as before (the positive '
            + 'control: it passes on the preference rule as well)',
          fn: async () => {
            expect(function noShipping() {
              settledWith({
                perCandidate: [
                  drew({ index: 1, ballots: 2, weight: 1, },),
                  drew({ index: 2, ballots: 1, weight: 1, },),
                  drew({ index: 3, ballots: 0, weight: 0, },),
                ],
                ballots: [],
                shipPastDecline: false,
                challenged: true,
              },);
            },).toThrow(TranslateAbsenceError,);
          },
        },),

        it({
          name: 'RAISES the absence on a first round, which defers to the challenge round (the positive control: '
            + 'it passes on the preference rule as well)',
          fn: async () => {
            expect(function firstRound() {
              settledWith({
                perCandidate: [
                  drew({ index: 1, ballots: 2, weight: 1, },),
                  drew({ index: 2, ballots: 1, weight: 1, },),
                  drew({ index: 3, ballots: 0, weight: 0, },),
                ],
                ballots: [],
                shipPastDecline: true,
                challenged: false,
              },);
            },).toThrow(TranslateAbsenceError,);
          },
        },),
      ],
    },),
  ],
},);
