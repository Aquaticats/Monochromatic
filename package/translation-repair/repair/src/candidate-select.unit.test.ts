/**
 Tests for candidate selection: producer exclusion, the vote thresholds that
 stop one judge deciding, and the two decline dispositions that tell an
 unranked field apart from a rejected one.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { readdir, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { PatchOutcome, } from '@monochromatic-dev/translation-repair-page-pass';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { selectBestCandidate, } from '@monochromatic-dev/translation-repair-translate-stage';
import {
  type FanOutMode,
  rosterQuorumSize,
} from '@monochromatic-dev/translation-repair-stage';
import { ProducerRosterError, } from '@monochromatic-dev/translation-repair-assembly';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import {
  type Candidate,
  FULL_VOTE_WEIGHT,
  MIN_SELECTION_BALLOTS,
  SELF_VOTE_WEIGHT,
} from '@monochromatic-dev/translation-repair-house-rule';
import { CANDIDATE_NONE, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  chunkCandidateOf,
  selectChunkPatch,
  selectPerEnvelope,
} from '../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  candidateFor,
  ENVELOPE,
  TARGET_TEXT,
} from './editor-candidate-envelope.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { runsDirPointedAt, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/runs-dir-pointed.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import { windowDialUnreadable, } from '@monochromatic-dev/translation-repair-test-fixture/ts/window-dial-unreadable.test-fixture.ts';

/**
 Logger for the stages under test.
 */
const l = tagged({ tag: 'candidate-select-test', },);

/**
 Original the judges compare against.
 */
const SOURCE_TEXT = '猫猫喜欢追蝴蝶。';

/**
 Ballots a scripted judge casts, keyed by model id.
 */
type BallotScript = Readonly<Record<string, number>>;

/**
 Client answering every selection ballot from a script, and counting calls so
 a test can prove a round never reached the judges.

 @param ballots - one-based candidate index per judge, zero to decline

 @param counter - mutable call tally the caller inspects afterwards

 @returns Client usable by the selection stage

 @example
 ```ts
 const client = scriptedJudges({ ballots, counter, },);
 ```
 */
function scriptedJudges(
  {
    ballots,
    counter,
  }: {
    readonly ballots: BallotScript;
    readonly counter: { calls: number; };
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by candidate selection',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      counter.calls += 1;

      /**
       Scripted ballot for the judge that was asked.
       */
      const scripted: unknown = {
        best: ballots[request.modelId] ?? 0,
        assessment: 'scripted',
      };
      if (!request.validate(scripted,))
        throw new Error('stub script failed the ballot guard',);
      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by candidate selection',);
    },
  };
}

/**
 Two competing string candidates from two named models.
 */
const STRING_CANDIDATES: readonly Candidate<string>[] = [
  {
    producer: {
      kind: 'model',
      modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
    },
    value: 'first',
    rendered: 'The cat chases butterflies.',
  },
  {
    producer: {
      kind: 'model',
      modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    },
    value: 'second',
    rendered: 'The cat loves chasing butterflies.',
  },
];

/**
 A fresh proposal against the archive's own wording, which a decline keeps:
 the shape of a consolidation slate over a standing text.
 */
const PROPOSAL_AGAINST_ARCHIVE: readonly Candidate<string>[] = [
  {
    producer: {
      kind: 'model',
      modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
    },
    value: 'proposal',
    rendered: 'The cat chases butterflies.',
  },
  {
    producer: {
      kind: 'incumbent',
      // A model that reproduced the archive exactly was collapsed into it and
      // is discounted when it judges it.
      matched: [SEAT_SYNTHETIC_VISION_WITHHELD,],
    },
    value: 'archive',
    rendered: 'The cat loves chasing butterflies.',
  },
];

/**
 Builds a judged-selection candidate carrying a synthetic chunk replacement,
 keeping the string candidate's producer and rendering so the judge sheet
 still reads as a slate over the same two proposals.

 @param candidate - string candidate this test's slate holds

 @param index - position of the candidate on the slate, varying its
 replacement text so the two candidates stay distinct

 @returns Candidate ready for `selectChunkPatch`

 @example
 ```ts
 const candidates = STRING_CANDIDATES.map(function toPatchCandidate(
   candidate,
   index,
 ): Candidate<PatchOutcome> {
   return patchCandidateFor({ candidate, index, },);
 },);
 ```
 */
function patchCandidateFor(
  {
    candidate,
    index,
  }: {
    readonly candidate: Candidate<string>;
    readonly index: number;
  },
): Candidate<PatchOutcome> {
  return {
    producer: candidate.producer,
    value: candidateFor({
      modelId: candidate.producer.kind === 'model'
        ? candidate.producer.modelId
        : SEAT_HYPER_OPENROUTER_VISION_EDITOR,
      newText: `Replacement ${String(index + 1,)}.`,
    },).patch,
    rendered: candidate.rendered,
  };
}

/**
 Client whose listed seats are refused by the router for want of a wet
 provider, the way `judgeSeatsFor` leaves a dry provider's seats on the
 bench, and whose other seats vote from the script.

 @param ballots - one-based candidate index per judge, zero to decline

 @param unreachable - seats no provider serves

 @returns Client usable by the selection stage

 @example
 ```ts
 const client = dryBenchJudges({ ballots, unreachable: [SEAT_HYPER_VISION,], },);
 ```
 */
function dryBenchJudges(
  {
    ballots,
    unreachable,
  }: {
    readonly ballots: BallotScript;
    readonly unreachable: readonly RosterModelId[];
  },
): ModelClient {
  /**
   Scripted client answering the reachable seats.
   */
  const scripted = scriptedJudges({
    ballots,
    counter: { calls: 0, },
  },);
  return {
    ...scripted,
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      if (unreachable.includes(request.modelId,)) {
        throw new NoProviderForModelError({
          modelId: request.modelId,
          reason: 'every provider serving this model is out of budget',
        },);
      }
      return scripted.chatJson(request,);
    },
  };
}

/**
 Whole roster selection draws judges from.
 */
const JUDGES: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
];

/**
 The first three seats of the fixture roster, the bench a short day leaves.
 */
const THREE_JUDGES: readonly RosterModelId[] = JUDGES.slice(0, 3,);

/**
 Runs one selection round over the fixture candidates.

 @param ballots - one-based candidate index per judge

 @param judgeModelIds - roster to seat; defaults to the fixture roster

 @param candidates - slate to judge; defaults to the two fixture proposals

 @returns Outcome plus how many judge calls it took

 @example
 ```ts
 const { outcome, } = await runSelection({ ballots, },);
 ```
 */
async function runSelection(
  {
    ballots,
    judgeModelIds = JUDGES,
    fanOut = 'whole-bench',
    candidates = STRING_CANDIDATES,
    logger = l,
  }: {
    readonly ballots: BallotScript;
    readonly judgeModelIds?: readonly RosterModelId[];
    readonly fanOut?: FanOutMode;
    readonly candidates?: readonly Candidate<string>[];
    readonly logger?: Logger;
  },
) {
  /**
   Judge calls the round made.
   */
  const counter = { calls: 0, };

  /**
   Verdict over the fixture candidates.
   */
  const outcome = await selectBestCandidate({
    client: scriptedJudges({
      ballots,
      counter,
    },),
    candidates,
    judgeModelIds,
    // THE WHOLE BENCH BY DEFAULT HERE, since these cases script every seat's
    // ballot and read the tally over the bench they wrote; production asks the
    // window of quorum plus one (`stage-fanout-window.ts`), pinned by the
    // "ASKS a window of quorum plus one judges in production" case.
    fanOut,
    task: 'Pick one.',
    criteria: ['Faithful.',],
    evidence: [
      {
        label: 'ORIGINAL',
        text: SOURCE_TEXT,
      },
    ],
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l: logger,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    outcome,
    calls: counter.calls,
  };
}

/**
 Runs one round over a COLLAPSED candidate and its only rival.

 The collapsed candidate stands for several models that returned identical
 text, which `buildTranslateCandidates` merges into one entry carrying every
 contributor. Every contributor then votes for it, and every other judge
 abstains, which isolates the question these cases ask: what self votes alone
 can carry.

 @param contributors - models the collapsed candidate is credited to

 @returns Outcome of that round

 @example
 ```ts
 const { outcome, } = await runCollapsedSelection({ contributors, },);
 ```
 */
async function runCollapsedSelection(
  { contributors, }: { readonly contributors: readonly RosterModelId[]; },
) {
  /**
   Judge calls the round made.
   */
  const counter = { calls: 0, };

  /**
   Slate: one merged proposal, one rival nobody votes for.
   */
  const candidates: readonly Candidate<string>[] = [
    {
      producer: {
        kind: 'composite',
        contributors,
      },
      value: 'collapsed',
      rendered: 'The cat naps in the sun.',
    },
    {
      producer: {
        kind: 'model',
        modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
      },
      value: 'rival',
      rendered: 'The cat is napping in the sunshine.',
    },
  ];

  /**
   Contributors back their own merged text; everyone else declines.
   */
  const ballots: BallotScript = Object.fromEntries(
    JUDGES.map(function toBallot(modelId,): readonly [string, number,] {
      return [
        modelId,
        contributors.includes(modelId,) ? 1 : CANDIDATE_NONE,
      ];
    },),
  );

  return {
    outcome: await selectBestCandidate({
      client: scriptedJudges({
        ballots,
        counter,
      },),
      candidates,
      judgeModelIds: JUDGES,
      fanOut: 'whole-bench',
      task: 'Pick one.',
      criteria: ['Faithful.',],
      evidence: [
        {
          label: 'ORIGINAL',
          text: SOURCE_TEXT,
        },
      ],
      signal: new AbortController().signal,
      perCallTimeoutMs: HANG_STOP_MS,
      l,
      stageSettings: SCRIPTED_STAGE_SETTINGS,
    },),
    calls: counter.calls,
  };
}

/**
 Seated wide bench of eight, of which a dry day leaves three reachable:
 the shape of Bedrock alone on 2026-09-09.
 */
const WIDE_BENCH: readonly RosterModelId[] = [
  ...JUDGES,
  SEAT_HYPER_VISION,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_ONLY,
];

/**
 Seats of that bench no provider serves on the dry day.
 */
const DRY_SEATS: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_VISION,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_ONLY,
];

/**
 Runs one selection round over the wide bench with the dry seats refused.

 @param ballots - one-based candidate index per reachable judge

 @param unreachable - seats no provider serves; defaults to the dry day

 @returns Outcome of that round

 @example
 ```ts
 const outcome = await runShortBench({ ballots, },);
 ```
 */
async function runShortBench(
  {
    ballots,
    unreachable = DRY_SEATS,
  }: {
    readonly ballots: BallotScript;
    readonly unreachable?: readonly RosterModelId[];
  },
) {
  return selectBestCandidate({
    client: dryBenchJudges({
      ballots,
      unreachable,
    },),
    candidates: STRING_CANDIDATES,
    judgeModelIds: WIDE_BENCH,
    fanOut: 'whole-bench',
    task: 'Pick one.',
    criteria: ['Faithful.',],
    evidence: [
      {
        label: 'ORIGINAL',
        text: SOURCE_TEXT,
      },
    ],
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: `${selectBestCandidate.name} beside the process's own variables`,
      // ONE AT A TIME: the case points the process's round window and runs
      // directory variables elsewhere.
      concurrency: 1,
      children: [
        it({
          name: 'JUDGES UNDER THE ROUND WINDOW ITS SETTINGS NAME AND RECORDS THE CONTEST UNDER THE RUNS DIRECTORY '
            + 'THEY NAME, while the process holds a round window no dial can read and names another runs directory',
          fn: async () => {
            /**
             Runs directory the settings name.
             */
            await using kept = await scratchDir({ prefix: 'select-kept-', },);
            /**
             Runs directory the process's variable names, which must stay empty.
             */
            await using elsewhere = await scratchDir({ prefix: 'select-elsewhere-', },);
            using pointed = runsDirPointedAt({ path: elsewhere.path, },);
            using round = windowDialUnreadable({ variable: 'TRANSLATION_REPAIR_STRAGGLER_GRACE_MS', },);

            /**
             Verdict of judges who all back the second candidate.
             */
            const outcome = await selectBestCandidate({
              client: scriptedJudges({
                ballots: Object.fromEntries(JUDGES.map(function backsSecond(modelId,): readonly [string, number,] {
                  return [
                    modelId,
                    2,
                  ];
                },),),
                counter: { calls: 0, },
              },),
              candidates: STRING_CANDIDATES,
              judgeModelIds: JUDGES,
              fanOut: 'whole-bench',
              task: 'Pick one.',
              criteria: ['Faithful.',],
              evidence: [
                {
                  label: 'ORIGINAL',
                  text: SOURCE_TEXT,
                },
              ],
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: {
                ...SCRIPTED_STAGE_SETTINGS,
                contestLedger: {
                  kind: 'kept',
                  runsDir: kept.path,
                },
              },
            },);

            expect({
              chosen: (outcome.kind === 'selected') ? outcome.value : outcome.kind,
              kept: (await readdir(join(
                kept.path,
                'ledger',
              ),)).length,
              elsewhere: await readdir(elsewhere.path,),
            },).toEqual({
              chosen: 'second',
              kept: 1,
              elsewhere: [],
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'selection on a bench short of quorum',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SEATS A WINNER TWO JUDGES NAMED where the router refused most of the bench: three of eight '
            + 'reachable against a quorum of four, a producer and one disinterested judge agreeing ship, '
            + 'and the finding names the bench (the owner\'s decision of 2026-09-09 scaled a weight '
            + 'minimum here, which the owner removed on 2026-10-25)',
          fn: async () => {
            /** Quorum over the seated bench. */
            const quorum = rosterQuorumSize({ rosterSize: WIDE_BENCH.length, },);
            /** Seats the dry day leaves reachable. */
            const reachable = WIDE_BENCH.length - DRY_SEATS.length;
            /** GLM-5.3-Flash backs its own text at half weight, Qwen backs it at full, gpt-oss declines. */
            const outcome = await runShortBench({
              ballots: {
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 1,
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 1,
                [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: 0,
              },
            },);
            expect(outcome.kind,).toBe('selected',);
            if (outcome.kind !== 'selected')
              throw new Error('unreachable',);
            expect(outcome.voteWeight,).toBe(SELF_VOTE_WEIGHT + FULL_VOTE_WEIGHT,);
            expect(outcome.tally.judgesAvailable,).toBe(WIDE_BENCH.length,);
            expect(outcome.tally.ballots,).toBe(reachable,);
            expect(outcome.findings,).toContain(
              `select-short-bench (reachable ${String(reachable,)} of ${String(WIDE_BENCH.length,)}, quorum ${
                String(quorum,)
              })`,
            );
            // Every dry seat is reported lost, as before; the `select-short-bench`
            // finding is what says the losses were the bench and not the weather.
            for (const seat of DRY_SEATS)
              expect(outcome.findings,).toContain(`stage-voice-lost (select ${seat})`,);
          },
        },),

        it({
          name: 'KEEPS TWO BALLOTS AS THE FLOOR on a short bench, so one judge never '
            + 'decides: two of eight reachable, and a lone full '
            + 'ballot is declined as one judge alone rather than seated',
          fn: async () => {
            /** Quorum over the seated bench. */
            const quorum = rosterQuorumSize({ rosterSize: WIDE_BENCH.length, },);
            /** The dry day's seats and the GLM seat. */
            const unreachable = [
              ...DRY_SEATS,
              SEAT_HYPER_OPENROUTER_VISION_EDITOR,
            ];
            /** Seats left reachable. */
            const reachable = WIDE_BENCH.length - unreachable.length;
            /** Only Qwen and gpt-oss reachable; Qwen names candidate 1 at full weight, gpt-oss declines. */
            const outcome = await runShortBench({
              ballots: {
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 1,
                [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: 0,
              },
              unreachable,
            },);
            // Only the two-ballot floor declines it.
            expect(outcome.kind,).toBe('declined',);
            if (outcome.kind !== 'declined')
              throw new Error('unreachable',);
            expect(outcome.reason,).toBe('winner named by one judge alone',);
            expect(outcome.disposition,).toBe('indecision',);
            expect(outcome.findings,).toContain(
              `select-short-bench (reachable ${String(reachable,)} of ${String(WIDE_BENCH.length,)}, quorum ${
                String(quorum,)
              })`,
            );
          },
        },),

        it({
          name: 'DECIDES THE SAME AGREEMENT ON A BENCH AT QUORUM: with the dry seats served again, the same '
            + 'producer-plus-one agreement at three halves ships and no short-bench finding is made, '
            + 'where it declined against the absolute weight minimum of 2 until 2026-10-25',
          fn: async () => {
            /** The same ballots on a bench every provider serves. */
            const outcome = await runShortBench({
              ballots: {
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 1,
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 1,
              },
              unreachable: [],
            },);
            expect(outcome.kind,).toBe('selected',);
            if (outcome.kind !== 'selected')
              throw new Error('unreachable',);
            expect(outcome.findings.some(function isShortBench(finding,): boolean {
              return finding.startsWith('select-short-bench',);
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: selectBestCandidate.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SEATS a model that produced a candidate and counts its ballot for '
            + 'its own work at half weight, which is the whole trade the user '
            + 'chose on 2026-08-15: these models have different blind spots, so a '
            + 'producer reading its own text is a weaker instrument than a '
            + 'disinterested one rather than a worthless one',
          fn: async () => {
            // Both producers land on their own candidate here because that is the
            // case under test, not because it is what usually happens: the sheet
            // is anonymized, so a judge cannot see which candidate is its own and
            // a self-vote is a mild tilt rather than a decision to back itself.
            // GLM-5.2 and Kimi take candidate 1, Qwen and the two remaining judges
            // take candidate 2.
            const { outcome, calls, } = await runSelection({
              ballots: {
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 1,
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 1,
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 2,
                [SEAT_HYPER_OPENROUTER_UNMEASURED]: 2,
                [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: 2,
              },
            },);
            // Every judge on the roster was asked, producers included.
            expect(calls,).toBe(5,);
            expect(outcome.tally.judgesAvailable,).toBe(5,);
            expect(outcome.kind,).toBe('selected',);
            expect(outcome.kind === 'selected' ? outcome.value : '',).toBe('second',);
            // THE DISCOUNT IS THE ASSERTION. Three ballots named candidate 2, so a
            // count would read 3; one of the three was its own author's, so the
            // weight reads 2.5. Candidate 1 sits at 1.5 rather than 2 for the same
            // reason, which is the margin the discount exists to create.
            expect(outcome.kind === 'selected' ? outcome.voteWeight : 0,)
              .toBe((2 * FULL_VOTE_WEIGHT) + SELF_VOTE_WEIGHT,);
            expect(outcome.tally.abstentions,).toBe(0,);
            // The self-vote is recorded rather than assumed away, so its rate is
            // readable from artifacts instead of argued about.
            expect(outcome.tally.selfVotes,).toBe(2,);
            expect(outcome.findings,).toContain('select-self-vote (hf:zai-org/GLM-5.3-Flash)',);
            expect(outcome.findings,).toContain('select-self-vote (hf:Qwen/Qwen3.8-27B)',);
          },
        },),

        it({
          name: 'SHIPS a collapsed candidate backed only by the models that wrote '
            + 'it, once two of them wrote it. Four models returning '
            + 'byte-identical text merge into one candidate, so four ballots for '
            + 'it are four SELF votes, and they are four answering judges with no '
            + 'disinterested judge involved. That is deliberate: independent '
            + 'models agreeing to the byte IS the corroboration, and it is the one '
            + 'case where the ballots do not require an outside voice',
          fn: async () => {
            /** Models that wrote the one collapsed candidate. */
            const contributors = [
              SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              SEAT_SYNTHETIC_VISION_WITHHELD,
              SEAT_HYPER_OPENROUTER_UNMEASURED,
            ];
            const { outcome, } = await runCollapsedSelection({ contributors, },);
            // Enough self votes to count as the ballots a winner needs, which this case needs to mean anything.
            expect(contributors.length,).toBeGreaterThanOrEqual(MIN_SELECTION_BALLOTS,);
            expect(outcome.kind,).toBe('selected',);
            expect(outcome.kind === 'selected' ? outcome.value : '',).toBe('collapsed',);
            expect(outcome.kind === 'selected' ? outcome.voteWeight : 0,).toBe(contributors.length * SELF_VOTE_WEIGHT,);
            expect(outcome.tally.selfVotes,).toBe(contributors.length,);
          },
        },),

        it({
          name: 'SHIPS the same shape with three contributors, which three halves left unshipped until 2026-10-25: '
            + 'three models wrote it and each backed it, which is three answering judges',
          fn: async () => {
            const { outcome, } = await runCollapsedSelection({
              contributors: [
                SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                SEAT_SYNTHETIC_VISION_WITHHELD,
              ],
            },);
            expect(outcome.kind,).toBe('selected',);
            expect(outcome.tally.selfVotes,).toBe(3,);
          },
        },),

        it({
          name: 'DECLINES the same shape with one contributor, which is what keeps the collapsed case from being '
            + 'a hole: a candidate one model wrote and nobody else endorsed is one judge alone',
          fn: async () => {
            const { outcome, } = await runCollapsedSelection({
              contributors: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
            },);
            expect(outcome.kind,).toBe('declined',);
            expect(outcome.kind === 'declined' ? outcome.reason : '',).toBe('winner named by one judge alone',);
            expect(outcome.tally.selfVotes,).toBe(1,);
          },
        },),

        it({
          name: 'ASKS a window of quorum plus one judges in production, so five healthy judges cost '
            + 'four calls and the fifth is never asked while quorum stands',
          fn: async () => {
            const { outcome, calls, } = await runSelection({
              ballots: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 1,
                [SEAT_HYPER_OPENROUTER_UNMEASURED]: 1,
                [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: 1,
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 1,
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 1,
              },
              fanOut: 'window',
            },);
            expect(outcome.kind,).toBe('selected',);
            expect(calls,).toBe(4,);
          },
        },),

        it({
          name: 'declines when the leader draws a single vote, so a lone judge '
            + 'cannot decide the stage by itself',
          fn: async () => {
            // One judge names a candidate and the other four answer that NO
            // candidate is acceptable, which is what a zero ballot means and what
            // the judge sheet asks for by name. A plurality of one is one model in
            // control, which is the thing the ensemble exists to prevent.
            const { outcome, } = await runSelection({
              ballots: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 1,
                [SEAT_HYPER_OPENROUTER_UNMEASURED]: 0,
                [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: 0,
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 0,
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 0,
              },
            },);
            expect(outcome.kind,).toBe('declined',);
            expect(outcome.kind === 'declined' ? outcome.reason : '',).toBe('winner named by one judge alone',);
            expect(outcome.kind === 'declined' ? outcome.disposition : '',).toBe('indecision',);
            expect(outcome.tally.abstentions,).toBe(4,);
          },
        },),

        it({
          name: 'reads a tie as indecision, not as a verdict against the candidates',
          fn: async () => {
            const { outcome, } = await runSelection({
              ballots: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 1,
                [SEAT_HYPER_OPENROUTER_UNMEASURED]: 2,
                [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: 0,
              },
            },);
            expect(outcome.kind,).toBe('declined',);
            expect(outcome.kind === 'declined' ? outcome.reason : '',).toBe('judges tied',);
            expect(outcome.kind === 'declined' ? outcome.disposition : '',).toBe('indecision',);
          },
        },),

        it({
          name: 'reads unanimous refusal as a rejection, a substantive verdict '
            + 'rather than a failure to rank',
          fn: async () => {
            const { outcome, } = await runSelection({
              ballots: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 0,
                [SEAT_HYPER_OPENROUTER_UNMEASURED]: 0,
                [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: 0,
              },
            },);
            expect(outcome.kind,).toBe('declined',);
            expect(outcome.kind === 'declined' ? outcome.reason : '',).toBe('every judge declined',);
            expect(outcome.kind === 'declined' ? outcome.disposition : '',).toBe('rejection',);
          },
        },),

        it({
          name: 'counts a ballot naming a candidate that does not exist as an '
            + 'abstention rather than discarding it',
          fn: async () => {
            const { outcome, } = await runSelection({
              ballots: {
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 2,
                [SEAT_HYPER_OPENROUTER_UNMEASURED]: 2,
                [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: 9,
              },
            },);
            expect(outcome.kind,).toBe('selected',);
            // The out-of-range ballot, plus the two producers left unscripted.
            expect(outcome.tally.abstentions,).toBe(3,);
            expect(outcome.tally.ballots,).toBe(5,);
          },
        },),

        it({
          name: 'REFUSES a roster naming one judge twice, before spending a single '
            + 'call. Two exchanges to one model are two ballots from one opinion, '
            + 'which counts as two answering judges alone, and the stage guard cannot '
            + 'catch it because selectPerEnvelope and selectChunkPatch are '
            + 'reachable without one',
          fn: async () => {
            /**
             Calls a refused round is allowed to make.
             */
            const counter = { calls: 0, };

            /**
             Round over a roster naming one model twice.
             */
            const refused = selectBestCandidate({
              client: scriptedJudges({
                ballots: { [SEAT_SYNTHETIC_VISION_WITHHELD]: 1, },
                counter,
              },),
              candidates: STRING_CANDIDATES,
              judgeModelIds: [
                SEAT_SYNTHETIC_VISION_WITHHELD,
                SEAT_SYNTHETIC_VISION_WITHHELD,
              ],
              task: 'Pick one.',
              criteria: ['Faithful.',],
              evidence: [
                {
                  label: 'ORIGINAL',
                  text: SOURCE_TEXT,
                },
              ],
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            await expect(refused,).rejects.toBeInstanceOf(ProducerRosterError,);
            expect(counter.calls,).toBe(0,);
          },
        },),

        it({
          name: 'still seats a producer that judges, which is the arrangement the '
            + 'repeat check must not break: the same model appearing once as an '
            + 'author and once as a judge is one voice with a stake, not two voices',
          fn: async () => {
            const { outcome, } = await runSelection({
              ballots: {
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 1,
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 1,
                [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: 1,
              },
            },);
            expect(outcome.kind,).toBe('selected',);
            expect(outcome.tally.selfVotes,).toBe(1,);
          },
        },),
      ],
    },),

    describe({
      name: selectPerEnvelope.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'adopts a sole proposal without spending a judge call, since there '
            + 'is nothing to compare it against',
          fn: async () => {
            /** Judge calls the pass made. */
            const counter = { calls: 0, };

            /** Composite over one editor's single proposal. */
            const selection = await selectPerEnvelope({
              client: scriptedJudges({
                ballots: {},
                counter,
              },),
              candidates: [
                candidateFor({
                  modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                  newText: 'The cat chases butterflies.',
                },),
              ],
              envelopes: [ENVELOPE,],
              judgeModelIds: JUDGES,
              sourceText: SOURCE_TEXT,
              targetText: TARGET_TEXT,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(counter.calls,).toBe(0,);
            expect(selection.soleCount,).toBe(1,);
            expect(selection.judgedCount,).toBe(0,);
            expect(selection.operations.length,).toBe(1,);
            expect([...selection.contributors,],).toEqual([SEAT_HYPER_OPENROUTER_VISION_EDITOR,],);
            // Recorded as a round of its own kind, so the author survives into
            // the attribution instead of vanishing with the vote that never was.
            expect(selection.rounds.length,).toBe(1,);
            expect(selection.rounds[0]?.kind,).toBe('adopted',);
            expect(selection.rounds[0]?.envelopeId,).toBe(ENVELOPE.envelopeId,);
            expect(selection.rounds[0]?.slate.length,).toBe(1,);
            expect(selection.rounds[0]?.slate[0]?.producer,).toEqual({
              kind: 'model',
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
            },);
          },
        },),

        it({
          name: 'leaves an envelope unedited when judges decline it, and credits '
            + 'no contributor for it',
          fn: async () => {
            /** Judge calls the pass made. */
            const counter = { calls: 0, };

            /** Composite over two competing proposals every judge refuses. */
            const selection = await selectPerEnvelope({
              client: scriptedJudges({
                ballots: {},
                counter,
              },),
              candidates: [
                candidateFor({
                  modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                  newText: 'The cat chases butterflies.',
                },),
                candidateFor({
                  modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                  newText: 'The cat loves chasing butterflies.',
                },),
              ],
              envelopes: [ENVELOPE,],
              judgeModelIds: JUDGES,
              sourceText: SOURCE_TEXT,
              targetText: TARGET_TEXT,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(counter.calls,).toBeGreaterThan(0,);
            expect(selection.declinedCount,).toBe(1,);
            expect(selection.operations.length,).toBe(0,);
            expect(selection.contributors.length,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: selectChunkPatch.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ships the strongest repair when judges fail to rank, because a '
            + 'disagreement about wording is not a verdict against repairing',
          fn: async () => {
            /** Repair kept when judges cannot converge, named for its author. */
            const indecisionFallback: Candidate<PatchOutcome> = chunkCandidateOf(candidateFor({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              newText: 'The cat chases butterflies.',
            },),);

            /** Untouched chunk, reserved for an outright rejection. */
            const rejectionFallback: PatchOutcome = {
              patchedText: TARGET_TEXT,
              applied: [],
              rejected: [],
            };

            /** Judge calls the pass made. */
            const counter = { calls: 0, };

            /** Verdict over two candidates the judges tie on. */
            const { patch, shippedProducer, } = await selectChunkPatch({
              client: scriptedJudges({
                ballots: {
                  [SEAT_SYNTHETIC_VISION_WITHHELD]: 1,
                  [SEAT_HYPER_OPENROUTER_UNMEASURED]: 2,
                  [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: 0,
                },
                counter,
              },),
              candidates: STRING_CANDIDATES.map(function toPatchCandidate(
                candidate,
                index,
              ): Candidate<PatchOutcome> {
                return patchCandidateFor({ candidate, index, },);
              },),
              judgeModelIds: JUDGES,
              sourceText: SOURCE_TEXT,
              indecisionFallback,
              rejectionFallback,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(patch.patchedText,).toContain('The cat chases butterflies.',);
            expect(patch.applied.length,).toBe(1,);
            // WHO shipped it, not just what shipped. A declined round records
            // ballots and no winner, so this is the one path where authorship
            // cannot be read back off the round, and leaving it unnamed lets the
            // editor that wrote this text certify its own work at full weight.
            expect(shippedProducer,).toEqual({
              kind: 'model',
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
            },);
          },
        },),

        it({
          name: 'ships no repair when every judge refuses, rather than overruling '
            + 'a verdict that none of the candidates is good enough',
          fn: async () => {
            /** Repair that must NOT ship over an outright rejection. */
            const indecisionFallback: Candidate<PatchOutcome> = chunkCandidateOf(candidateFor({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              newText: 'The cat chases butterflies.',
            },),);

            /** Untouched chunk. */
            const rejectionFallback: PatchOutcome = {
              patchedText: TARGET_TEXT,
              applied: [],
              rejected: [],
            };

            /** Judge calls the pass made. */
            const counter = { calls: 0, };

            /** Verdict over candidates every judge refuses. */
            const { patch, } = await selectChunkPatch({
              client: scriptedJudges({
                ballots: {},
                counter,
              },),
              candidates: STRING_CANDIDATES.map(function toPatchCandidate(
                candidate,
                index,
              ): Candidate<PatchOutcome> {
                return patchCandidateFor({ candidate, index, },);
              },),
              judgeModelIds: JUDGES,
              sourceText: SOURCE_TEXT,
              indecisionFallback,
              rejectionFallback,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(patch.patchedText,).toBe(TARGET_TEXT,);
            expect(patch.applied.length,).toBe(0,);
          },
        },),

        it({
          name: 'ships a sole surviving candidate unjudged, since deduplication '
            + 'means every editor and the composite wrote the same text',
          fn: async () => {
            /** Judge calls the pass made. */
            const counter = { calls: 0, };

            /** Text every proposal agreed on. */
            const agreed = candidateFor({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              newText: 'The cat chases butterflies.',
            },).patch;

            /**
             The one distinct candidate, which is also what would ship if judges
             declined, so both roles are filled by the same object here.
             */
            const sole: Candidate<PatchOutcome> = {
              producer: {
                kind: 'composite',
                contributors: [
                  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                ],
              },
              value: agreed,
              rendered: agreed.patchedText,
            };

            /** Verdict over the one distinct candidate. */
            const { patch, } = await selectChunkPatch({
              client: scriptedJudges({
                ballots: {},
                counter,
              },),
              candidates: [sole,],
              judgeModelIds: JUDGES,
              sourceText: SOURCE_TEXT,
              indecisionFallback: sole,
              rejectionFallback: {
                patchedText: TARGET_TEXT,
                applied: [],
                rejected: [],
              },
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(counter.calls,).toBe(0,);
            expect(patch.patchedText,).toContain('The cat chases butterflies.',);
          },
        },),
      ],
    },),

    describe({
      name: 'a plurality of two ballots over valid finalists (class seventy-three, mikaela4, 2026-09-19)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SEATS the leader named by two ballots in the first round, since abstentions answer neither; '
            + 'it needed a run-off to do so while the weight minimum stood (removed 2026-10-25)',
          fn: async () => {
            // GLM-5.3-Flash wrote candidate 1 and names it at half weight, one
            // disinterested judge names it at full weight, one names candidate 2,
            // two abstain: 1.5 against 1 with two ballots behind the leader.
            // That is mikaela4's slice 28 run-off.
            /** Ballots the round heard. */
            const ballots = {
              [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 1,
              [SEAT_SYNTHETIC_VISION_WITHHELD]: 1,
              [SEAT_HYPER_OPENROUTER_UNMEASURED]: 2,
              [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: 0,
              [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 0,
            };
            const { outcome, } = await runSelection({ ballots, },);
            expect(outcome.kind,).toBe('selected',);
            expect(outcome.kind === 'selected' ? outcome.value : '',).toBe('first',);
            expect(outcome.kind === 'selected' ? outcome.voteWeight : 0,).toBe(SELF_VOTE_WEIGHT + FULL_VOTE_WEIGHT,);
            expect(outcome.tally.abstentions,).toBe(2,);
          },
        },),
        it({
          name: 'LOGS an abstaining ballot with its reason beside the choosing ballots, since a run-off lost to two '
            + 'abstentions whose grounds were readable only off the cached replies (mikaela4 slice 28)',
          fn: async () => {
            /** Every line the round logged. */
            const messages: string[] = [];
            await runSelection({
              ballots: {
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 1,
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 1,
                [SEAT_HYPER_OPENROUTER_UNMEASURED]: 0,
                [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: 0,
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 9,
              },
              logger: capturingLogger({ messages, },),
            },);
            expect(messages.some(function namesTheAbstention(message,): boolean {
              return message.includes(`${SEAT_HYPER_OPENROUTER_UNMEASURED} declined every candidate: scripted`,);
            },),).toBe(true,);
            expect(messages.some(function namesTheStray(message,): boolean {
              return message.includes(`${SEAT_SYNTHETIC_VISION_NO_OPENROUTER} named candidate 9, which is not on the slate of 2: scripted`,);
            },),).toBe(true,);
          },
        },),
        it({
          name: 'still DECLINES a round whose leader one judge alone named, so the ballot floor holds',
          fn: async () => {
            const { outcome, } = await runSelection({
              ballots: {
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 1,
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 0,
                [SEAT_HYPER_OPENROUTER_UNMEASURED]: 0,
                [SEAT_SYNTHETIC_TEXT_EVERYWHERE]: 0,
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 0,
              },
            },);
            expect(outcome.kind,).toBe('declined',);
            expect(outcome.kind === 'declined' ? outcome.disposition : '',).toBe('indecision',);
          },
        },),
      ],
    },),

    describe({
      name: 'a fallback the contest majority called flawed (one entry\'s slice 45, 2026-10-19), which no longer '
        + 'changes how a round is decided since the owner removed the weight minimum on 2026-10-25',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SEATS the leader two ballots named, though every ballot fell on one candidate and so no run-off '
            + 'could narrow it, whether or not the fallback was called flawed',
          fn: async () => {
            // Three judges are heard. The model that wrote candidate 1 names it at half weight, a
            // disinterested one names it at full weight and the third declines everything: 1.5
            // against 0 with two ballots behind the leader. It was declined against the absolute
            // weight minimum of 2 until 2026-10-25, which a flag for a condemned fallback got around.
            /** Ballots the round heard, in the shape of the live slice. */
            const ballots = {
              [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 1,
              [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 1,
              [SEAT_SYNTHETIC_VISION_WITHHELD]: 0,
            };
            const { outcome, } = await runSelection({
              ballots,
              judgeModelIds: THREE_JUDGES,
              candidates: PROPOSAL_AGAINST_ARCHIVE,
            },);
            expect(outcome.kind,).toBe('selected',);
            expect(outcome.kind === 'selected' ? outcome.value : '',).toBe('proposal',);
            expect(outcome.kind === 'selected' ? outcome.voteWeight : 0,).toBe(SELF_VOTE_WEIGHT + FULL_VOTE_WEIGHT,);
            expect(outcome.tally.abstentions,).toBe(1,);
            expect(outcome.findings.includes('select-condemned-fallback-under-minimum',),).toBe(false,);
          },
        },),
        it({
          name: 'still DECLINES a leader one judge alone named, so one model never decides (owner, 2026-09-09)',
          fn: async () => {
            const { outcome, } = await runSelection({
              ballots: {
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 0,
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 1,
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 0,
              },
              judgeModelIds: THREE_JUDGES,
              candidates: PROPOSAL_AGAINST_ARCHIVE,
            },);
            expect(outcome.kind,).toBe('declined',);
            expect(outcome.kind === 'declined' ? outcome.disposition : '',).toBe('indecision',);
            expect(outcome.kind === 'declined' ? outcome.reason : '',).toBe('winner named by one judge alone',);
          },
        },),
        it({
          name: 'SEATS the standing text when two ballots lead with it, as it was at a weight of 2: the ballots '
            + 'decide the round and a fallback with two judges behind it is a judged choice',
          fn: async () => {
            const { outcome, } = await runSelection({
              ballots: {
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 0,
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 2,
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 2,
              },
              judgeModelIds: THREE_JUDGES,
              candidates: PROPOSAL_AGAINST_ARCHIVE,
            },);
            expect(outcome.kind,).toBe('selected',);
            expect(outcome.kind === 'selected' ? outcome.value : '',).toBe('archive',);
          },
        },),
        it({
          name: 'still RUNS OFF a tie and declines a round every judge refused',
          fn: async () => {
            // Each writer names its own text at half weight: 0.5 against 0.5.
            const tied = await runSelection({
              ballots: {
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 1,
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 0,
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 2,
              },
              judgeModelIds: THREE_JUDGES,
              candidates: PROPOSAL_AGAINST_ARCHIVE,
            },);
            expect(tied.outcome.kind,).toBe('declined',);
            expect(tied.outcome.kind === 'declined' ? tied.outcome.reason : '',).toBe('judges tied',);

            const refused = await runSelection({
              ballots: {
                [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: 0,
                [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: 0,
                [SEAT_SYNTHETIC_VISION_WITHHELD]: 0,
              },
              judgeModelIds: THREE_JUDGES,
              candidates: PROPOSAL_AGAINST_ARCHIVE,
            },);
            expect(refused.outcome.kind,).toBe('declined',);
            expect(refused.outcome.kind === 'declined' ? refused.outcome.disposition : '',).toBe('rejection',);
          },
        },),
      ],
    },),
  ],
},);
