import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { TargetRegion, } from './coverage-foreign-region.ts';
import {
  type CoverageVerdict,
  judgeCoverage,
} from './coverage-verdict.ts';
import {
  buildCoverageMessages,
  type CoverageFollowupEvidence,
  COVERAGE_RESPONSE_FORMAT,
  type CoverageReportWire,
  isCoverageReportWire,
} from './coverage-wire.ts';
import {
  type FanOutMode,
  gatherStageVoices,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type {
  AnchorTarget,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Coverage stage
// Asks a roster whether a translation already carries one passage, and reports
// what they proved rather than what they said.
//
// NOTHING CALLS THIS YET. It exists to answer question 28 with a measurement:
// the four ways out it offered differ in expense rather than in correctness, and
// only one of them can be evaluated before it is chosen. `coverage-probe.ts`
// runs it over the corpus candidates that the two aligners disagree about.

/**
 What one coverage question cost and concluded.

 @example
 ```ts
 const answer: CoverageAnswer = { verdict, findings: [], };
 ```
 */
export type CoverageAnswer = {
  /**
   Verdict over the anchored replies.
   */
  readonly verdict: CoverageVerdict;

  /**
   Roster findings: lost voices and incomplete rosters, which arrive whether
   or not quorum was met, so this being empty means nothing went wrong rather
   than that enough models answered.
   */
  readonly findings: readonly string[];
};

/**
 Asks one roster whether a translation carries one passage.

 @param client - injected model client

 @param modelIds - roster asked

 @param sourcePassage - original-side text whose coverage is in question

 @param translation - whole translation, searched and used to anchor quotes

 @param foreignRegions - target regions the pairing assigned to other source
 slices, which a partial claim cannot draw coverage from (class fifty-one)

 @param followupEvidence - latest unresolved placement evidence

 @param identityContext - declared names preparation holds, so a name the
 English writes by a declared handle is found as the name the passage writes
 (ledger B28)

 @param signal - caller abort honored by every exchange

 @param exchangeTimeoutMs - deadline per exchange

 @param fanOut - seats a round asks: the window of quorum plus one by
 default, or the whole bench a fixture scripting every seat asks for

 @param l - logger of the calling driver

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Verdict plus any roster findings

 @example
 ```ts
 const answer = await runCoverageStage({ client, modelIds, sourcePassage, translation, signal, exchangeTimeoutMs, l, stageSettings, },);
 ```
 */
export async function runCoverageStage(
  {
    client,
    modelIds,
    sourcePassage,
    translation,
    foreignRegions,
    followupEvidence,
    identityContext,
    signal,
    exchangeTimeoutMs,
    l,
    fanOut,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly modelIds: readonly RosterModelId[];
    readonly sourcePassage: string;
    readonly translation: AnchorTarget;
    readonly foreignRegions?: readonly TargetRegion[];
    readonly followupEvidence?: CoverageFollowupEvidence;
    readonly identityContext?: string;
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly fanOut?: FanOutMode;
    readonly stageSettings: StageSettings;
  }>,
): Promise<CoverageAnswer> {
  /**
   Sheet asking about this passage against the whole translation.
   */
  const plan = buildCoverageMessages({
    sourcePassage,
    translationText: translation.text,
    ...((followupEvidence === undefined) ? {} : { followupEvidence, }),
    ...((identityContext === undefined) ? {} : { identityContext, }),
  },);

  /**
   Replies heard from the roster.
   */
  const gather = await gatherStageVoices<CoverageReportWire>({
    client,
    modelIds,
    messages: plan.messages,
    signal,
    exchangeTimeoutMs,
    responseFormat: COVERAGE_RESPONSE_FORMAT,
    validate: isCoverageReportWire,
    stage: 'coverage',
    l,
    // Conditional spread keeps the knob absent instead of undefined.
    ...((fanOut === undefined) ? {} : { fanOut, }),
    graceMs: stageSettings
      .grace
      .roundMs,
  },);
  /**
   Seats asked that a provider could serve. THE REACHABLE SEATS ASKED, NOT
   THE BENCH: since the fan-out window of 2026-09-09 a round asks quorum plus
   one seat first, and a seat the window spared was never silent; since class
   fifty (shi_Yumiaoya4, 2026-09-17) a seat the router refused for want of a
   wet provider is not in the denominator either, since it was never asked
   and withholds no vote. Four of six heard voices found the death passage
   nowhere, two dark seats kept the majority at five of eight, and the
   passage shipped as a recorded gap.
   */
  const reachableAsked = [...gather.asked,]
    .filter(function reached(modelId,): boolean {
      return !gather.unreachable
        .has(modelId,);
    },)
    .length;
  return {
    verdict: judgeCoverage({
      voices: gather.voices,
      document: translation,
      ...((foreignRegions === undefined) ? {} : { foreignRegions, }),
      asked: reachableAsked,
      quorumMet: gather.quorumMet,
      // The quotes were copied out of the translation as the sheet showed it, cut against this passage.
      original: sourcePassage,
    },),
    findings: gather.findings,
  };
}

//endregion Coverage stage
