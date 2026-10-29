import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  buildRestorationJudgeMessages,
  isRestorationJudgeWire,
  type JudgeReference,
  RESTORATION_JUDGE_RESPONSE_FORMAT,
  resolveRestorationJudgment,
  type RestorationVerdict,
} from './restoration-judge-wire.ts';
import {
  gatherStageVoices,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Restoration judge stage
// Ensemble bilingual grading of one entry's restored seeds. No single judge
// decides (settled architecture): judges fan out with retry-to-quorum, and
// each seed's verdict is the conservative lower median of the cast verdicts,
// so a split lands toward the less-credited outcome rather than over-crediting
// a doubtful restoration.

/**
 One seed's ensemble verdict.

 @example
 ```ts
 const graded: SeedJudgment = { verdict: 'restored', judged: true, votes: 3, };
 ```
 */
export type SeedJudgment = {
  /**
   Conservative lower-median verdict across judges;
   `absent` when no judge ruled and the seed stays unjudged.
   */
  readonly verdict: RestorationVerdict;

  /**
   Whether a quorum of judges ruled on this seed,
   so the scorecard can exclude unjudged seeds honestly.
   */
  readonly judged: boolean;

  /**
   Judges that cast a verdict on this seed.
   */
  readonly votes: number;
};

/**
 Ordinal rank of a verdict, least to most credited.

 @param verdict - verdict to rank

 @returns Rank from zero (absent) to two (restored)

 @example
 ```ts
 verdictRank({ verdict: 'restored', },);
 ```
 */
function verdictRank(
  { verdict, }: { readonly verdict: RestorationVerdict; },
): number {
  if (verdict === 'absent')
    return 0;
  if (verdict === 'partial')
    return 1;
  return 2;
}

/**
 Conservative lower-median verdict over cast verdicts.
 Sorting by rank and taking the lower of the two middle elements means an
 even split rounds toward the less-credited verdict, because a wrongly
 credited restoration is the costlier error for a quality metric.

 @param cast - verdicts cast on one seed, at least one

 @returns Lower-median verdict

 @example
 ```ts
 lowerMedianVerdict({ cast: ['restored', 'partial', 'absent',], },);
 ```
 */
function lowerMedianVerdict(
  { cast, }: { readonly cast: readonly RestorationVerdict[]; },
): RestorationVerdict {
  /**
   Cast verdicts sorted least to most credited.
   */
  const sorted = [...cast,].toSorted(function byRank(
    left,
    right,
  ) {
    return verdictRank({ verdict: left, },) - verdictRank({ verdict: right, },);
  },);

  /**
   Lower-median index: for even counts the lower of the two middles.
   */
  const index = Math.floor((sorted.length - 1) / 2,);
  return nonNullishOrThrow(sorted[index],);
}

/**
 Grades one entry's restored seeds against the Chinese source.
 Judges fan out over the whole entry sheet with retry-to-quorum; a seed
 counts as judged only when quorum was met and at least one judge ruled on
 it, and its verdict is the conservative lower median.

 @param client - injected model client

 @param judgeModelIds - bilingual judge roster

 @param sourceText - original Chinese document

 @param repairedText - repaired translation under grading

 @param references - deleted sentences with their seed ids

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - benchmark logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Verdict per seed id

 @example
 ```ts
 const judgments = await runRestorationJudge({ ... },);
 ```
 */
export async function runRestorationJudge(
  {
    client,
    judgeModelIds,
    sourceText,
    repairedText,
    references,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly judgeModelIds: readonly RosterModelId[];
    readonly sourceText: string;
    readonly repairedText: string;
    readonly references: readonly JudgeReference[];
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<Readonly<Record<string, SeedJudgment>>> {
  if (references.length === 0)
    return {};

  /**
   Judge sheet plus the seed numbering order.
   */
  const plan = buildRestorationJudgeMessages({
    sourceText,
    repairedText,
    references,
  },);

  /**
   Heard judge voices after retry-to-quorum.
   */
  const gather = await gatherStageVoices({
    client,
    modelIds: judgeModelIds,
    messages: plan.messages,
    signal,
    exchangeTimeoutMs: perCallTimeoutMs,
    responseFormat: RESTORATION_JUDGE_RESPONSE_FORMAT,
    validate: isRestorationJudgeWire,
    stage: 'restoration-judge',
    l,
    graceMs: stageSettings
      .grace
      .roundMs,
  },);

  /**
   Seed-keyed verdicts per heard judge.
   */
  const ballots = gather.voices
    .map(function toBallot(voice,) {
      return resolveRestorationJudgment({
        wire: voice.value,
        seedIds: plan.seedIds,
      },)
        .verdicts;
    },);

  l.info(
    `restoration judge: ${String(gather.voices
      .length,)}/${
      String(judgeModelIds.length,)
    } heard, quorum ${gather.quorumMet ? 'met' : 'unmet'}`,
  );

  return Object.fromEntries(plan.seedIds
    .map(function toEntry(seedId,): readonly [
      string,
      SeedJudgment,
    ] {
    /**
     Verdicts cast on this seed across heard judges.
     */
      const cast = ballots.flatMap(function toVerdict(ballot,): readonly RestorationVerdict[] {
      /**
       This judge's verdict on the seed, when cast.
       */
        const verdict = ballot[seedId];
        return verdict === undefined ? [] : [verdict,];
      },);
      if ((cast.length === 0) || (!gather.quorumMet)) {
        return [
          seedId,
          {
            verdict: 'absent',
            judged: false,
            votes: cast.length,
          },
        ];
      }
      return [
        seedId,
        {
          verdict: lowerMedianVerdict({ cast, },),
          judged: true,
          votes: cast.length,
        },
      ];
    },),);
}

//endregion Restoration judge stage
