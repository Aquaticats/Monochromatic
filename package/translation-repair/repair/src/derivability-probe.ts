import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  buildDerivabilityMessages,
  DERIVABILITY_RESPONSE_FORMAT,
  type DerivabilityVerdict,
  resolveDerivabilityJudgment,
} from './derivability-wire.ts';
import {
  isRestorationJudgeWire,
  type JudgeReference,
} from './restoration-judge-wire.ts';
import {
  gatherStageVoices,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Derivability probe stage
// Ensemble audit of whether deleted sentences are fully derivable from the
// Chinese source. No single judge decides (settled architecture): judges fan
// out with retry-to-quorum, and each seed's verdict is the UPPER median of
// the cast verdicts, opposite of the restoration judge's lower median. The
// asymmetry is deliberate: this probe can only EXCUSE a partial restoration
// (an underivable sentence caps at partial through no fault of the editor),
// so a split must round toward MORE derivable, keeping the burden of proof
// on the excuse rather than on the pipeline.

/**
 One seed's ensemble derivability.

 @example
 ```ts
 const probed: SeedDerivability = { verdict: 'derivable', judged: true, votes: 3, };
 ```
 */
export type SeedDerivability = {
  /**
   Upper-median verdict across judges;
   `derivable` when no judge ruled and the seed stays unjudged, so an
   unheard probe never excuses a partial restoration.
   */
  readonly verdict: DerivabilityVerdict;

  /**
   Whether a quorum of judges ruled on this seed.
   */
  readonly judged: boolean;

  /**
   Judges that cast a verdict on this seed.
   */
  readonly votes: number;
};

/**
 Ordinal rank of a verdict, least to most derivable.

 @param verdict - verdict to rank

 @returns Rank from zero (not-derivable) to two (derivable)

 @example
 ```ts
 derivabilityRank({ verdict: 'derivable', },);
 ```
 */
function derivabilityRank(
  { verdict, }: { readonly verdict: DerivabilityVerdict; },
): number {
  if (verdict === 'not-derivable')
    return 0;
  if (verdict === 'partially-derivable')
    return 1;
  return 2;
}

/**
 Upper-median verdict over cast verdicts.
 Sorting by rank and taking the upper of the two middle elements means an
 even split rounds toward the more-derivable verdict, because wrongly
 excusing an editor failure is the costlier error for this probe.

 @param cast - verdicts cast on one seed, at least one

 @returns Upper-median verdict

 @example
 ```ts
 upperMedianVerdict({ cast: ['derivable', 'not-derivable',], },);
 ```
 */
function upperMedianVerdict(
  { cast, }: { readonly cast: readonly DerivabilityVerdict[]; },
): DerivabilityVerdict {
  /**
   Cast verdicts sorted least to most derivable.
   */
  const sorted = [...cast,].toSorted(function byRank(
    left,
    right,
  ) {
    return derivabilityRank({ verdict: left, },) - derivabilityRank({ verdict: right, },);
  },);

  /**
   Upper-median index: for even counts the upper of the two middles.
   */
  const index = Math.ceil((sorted.length - 1) / 2,);
  return nonNullishOrThrow(sorted[index],);
}

/**
 Audits whether each deleted sentence is fully derivable from the Chinese
 source. Judges fan out over the whole probe sheet with retry-to-quorum; a
 seed counts as judged only when quorum was met and at least one judge
 ruled on it, and its verdict is the upper median.

 @param client - injected model client

 @param judgeModelIds - bilingual judge roster

 @param sourceText - original Chinese document

 @param references - deleted sentences with their seed ids

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - benchmark logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Derivability per seed id

 @example
 ```ts
 const derivability = await runDerivabilityProbe({ ... },);
 ```
 */
export async function runDerivabilityProbe(
  {
    client,
    judgeModelIds,
    sourceText,
    references,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly judgeModelIds: readonly RosterModelId[];
    readonly sourceText: string;
    readonly references: readonly JudgeReference[];
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<Readonly<Record<string, SeedDerivability>>> {
  if (references.length === 0)
    return {};

  /**
   Probe sheet plus the seed numbering order.
   */
  const plan = buildDerivabilityMessages({
    sourceText,
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
    responseFormat: DERIVABILITY_RESPONSE_FORMAT,
    validate: isRestorationJudgeWire,
    stage: 'derivability-probe',
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
      return resolveDerivabilityJudgment({
        wire: voice.value,
        seedIds: plan.seedIds,
      },)
        .verdicts;
    },);

  l.info(
    `derivability probe: ${String(gather.voices
      .length,)}/${
      String(judgeModelIds.length,)
    } heard, quorum ${gather.quorumMet ? 'met' : 'unmet'}`,
  );

  return Object.fromEntries(plan.seedIds
    .map(function toEntry(seedId,): readonly [
      string,
      SeedDerivability,
    ] {
    /**
     Verdicts cast on this seed across heard judges.
     */
      const cast = ballots.flatMap(function toVerdict(ballot,): readonly DerivabilityVerdict[] {
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
            verdict: 'derivable',
            judged: false,
            votes: cast.length,
          },
        ];
      }
      return [
        seedId,
        {
          verdict: upperMedianVerdict({ cast, },),
          judged: true,
          votes: cast.length,
        },
      ];
    },),);
}

//endregion Derivability probe stage
