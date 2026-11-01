import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  buildCriticMessages,
  type ClaimAttribution,
  type ClaimEmission,
  collectClaimAttributions,
  CRITIC_RESPONSE_FORMAT,
  criticOriginalsOf,
  isCriticReportWire,
  resolveCriticIssue,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import {
  type AnchorTarget,
  computeIssueClaimId,
  type DocumentSide,
  type IssueClaim,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type FanOutMode,
  gatherStageVoices,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';

//region Critic and panel stages
// The two claim-producing stages of one chunk's repair: critics fan out and
// their validated claims come back, then a provenance-blind panel judges the
// aggregated clusters. A lost voice never fails the stage; the ensemble is
// built from individually unreliable models.

/**
 Everything the critic fan-out produced for one chunk.

 @example
 ```ts
 const { claims, nonTranslationVotes, } = await runCriticStage({ ... },);
 ```
 */
export type CriticStageResult = {
  /**
   Validated claims across every heard critic, in critic then report order.
   */
  readonly claims: readonly IssueClaim[];

  /**
   Critics reporting a critical non-translation at wire level;
   anchoring is best-effort for such pairs, so this count is taken
   before anchor resolution.
   */
  readonly nonTranslationVotes: number;

  /**
   Critics whose reply arrived and validated.
   */
  readonly heardCritics: number;

  /**
   WHICH critics answered, sorted by model id.

   This is the denominator attribution needs and cannot supply. A critic that
   was heard and raised nothing produces no attribution entry, and so does a
   critic that was never heard at all; without the roster those two are
   indistinguishable, so hits can be counted but rates cannot. That is the
   same silence-reads-as-clean failure the per-voice probe
   rates exposed, and those rates are answerable only because its telemetry carries the denominator.
   */
  readonly heardCriticIds: readonly RosterModelId[];

  /**
   Which critics raised each claim, keyed by deterministic claim id.
   Built here because `aggregateClaims` collapses structurally identical
   claims later, and after that collapse a second emitter is unrecoverable.
   Calibration only; adjudication never sees it.

   BEFORE SCREENING: collected at resolution time, so this includes claims
   `screenNonTranslationVotes` may still drop. Callers must filter with
   `retainAttributions` against the surviving claim ids, or a critic ends up
   credited with a hit the pipeline threw away.
   */
  readonly claimAttributions: readonly ClaimAttribution[];

  /**
   Resolution failures in scorecard-stable wording.
   */
  readonly findings: readonly string[];
};

/**
 Runs the critic fan-out for one chunk pair.

 @param client - injected model client

 @param criticModelIds - critics to fan out to

 @param sourceText - original chunk text

 @param targetText - translation chunk text

 @param documents - parsed chunk pair claims anchor against

 @param identityContext - declared names from both sides' front matter,
 carried whole-document because chunk text never contains front matter

 @param referenceContext - what the original's cited pages say (class
 thirty-five), carried whole-document likewise

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - pipeline logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Validated claims plus wire-level non-translation votes

 @example
 ```ts
 const critic = await runCriticStage({ ... },);
 ```
 */
export async function runCriticStage(
  {
    client,
    criticModelIds,
    sourceText,
    targetText,
    documents,
    identityContext,
    referenceContext,
    neighbouringIncumbentText,
    neighbouringSourceText,
    signal,
    perCallTimeoutMs,
    l,
    fanOut,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly criticModelIds: readonly RosterModelId[];
    readonly sourceText: string;
    readonly targetText: string;
    readonly documents: Readonly<Record<DocumentSide, AnchorTarget>>;
    readonly identityContext?: string;
    readonly referenceContext?: string;
    readonly neighbouringIncumbentText?: string;
    readonly neighbouringSourceText?: string;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly fanOut?: FanOutMode;
    readonly stageSettings: StageSettings;
  }>,
): Promise<CriticStageResult> {
  /**
   Shared critic prompt for this chunk.
   */
  const messages = buildCriticMessages({
    sourceText,
    targetText,
    ...(identityContext === undefined ? {} : { identityContext, }),
    ...((referenceContext === undefined) ? {} : { referenceContext, }),
    ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
    ...((neighbouringIncumbentText === undefined) ? {} : { neighbouringIncumbentText, }),
  },);

  /**
   The texts whose comments are the original's own on this sheet, which the
   quotes of the reports are resolved against as well.
   */
  const original = criticOriginalsOf({
    sourceText,
    ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
  },);

  /**
   Heard critics after retry-to-quorum. Full-roster retries were tried
   and reverted (user decision 2026-07-23): live probing showed critics
   answer 7/7 on the first round nearly always, paragraph slicing plus
   panel-judged merging carry the thoroughness burden, and waiting on a
   complete roster stalls runs when a voice genuinely wedges.
   */
  const gather = await gatherStageVoices({
    client,
    modelIds: criticModelIds,
    messages,
    signal,
    exchangeTimeoutMs: perCallTimeoutMs,
    responseFormat: CRITIC_RESPONSE_FORMAT,
    validate: isCriticReportWire,
    stage: 'critic',
    l,
    // Conditional spread keeps the knob absent instead of undefined.
    ...((fanOut === undefined) ? {} : { fanOut, }),
    graceMs: stageSettings
      .grace
      .roundMs,
  },);

  /**
   Reports that actually arrived.
   */
  const reports = gather.voices
    .map(function toReport(voice,) {
      return voice.value;
    },);

  /**
   Critics reporting a critical non-translation before anchoring;
   degenerate pairs defeat anchoring, so the wire level is the honest one.
   */
  const nonTranslationVotes = reports.filter(function votesNonTranslation(report,) {
    return report.issues
      .some(function isCriticalNonTranslation(issue,) {
        return (issue.category === 'accuracy/non-translation')
          && (issue.severity === 'critical');
      },);
  },)
    .length;

  /**
   Findings accumulated across resolutions, seeded with quorum
   degradation findings from the gather.
   */
  const findings: string[] = [...gather.findings,];

  /**
   One entry per resolved claim per critic, before deduplication collapses
   identical claims and takes the second emitter with it.
   */
  const emissions: ClaimEmission[] = [];

  /**
   Validated claims across every report.
   Iterates VOICES rather than reports so each claim keeps the speaker that
   `HeardVoice` carries; the traversal order is the same, so claim order is
   unchanged.
   */
  const claims = gather.voices
    .flatMap(function resolveVoice(voice,) {
    /**
     Report this critic returned.
     */
      const report = voice.value;
      return report.issues
        .flatMap(function resolveOne(wire,): readonly IssueClaim[] {
      /**
       Resolution of this wire issue.
       */
          const resolution = resolveCriticIssue({
            wire,
            documents,
        // The quotes are read in the translation as the sheet showed it, cut against the same originals.
            original,
          },);
          if (!resolution.resolved) {
            findings.push(resolution.reason,);
            return [];
          }
          emissions.push({
            claimId: computeIssueClaimId({ claim: resolution.claim, },),
            modelId: voice.modelId,
          },);
          return [resolution.claim,];
        },);
    },);

  l.info(
    `critic stage: ${String(reports.length,)}/${String(criticModelIds.length,)} heard, ${
      String(claims.length,)
    } ${
      wordForCount({
        count: claims.length,
        one: 'claim',
        many: 'claims',
      },)
    }, ${String(nonTranslationVotes,)} non-translation ${
      wordForCount({
        count: nonTranslationVotes,
        one: 'vote',
        many: 'votes',
      },)
    }`,
  );

  return {
    claims,
    nonTranslationVotes,
    heardCritics: reports.length,
    heardCriticIds: textsInCodePointOrder({ texts: gather.voices
      .map(function toModelId(voice,) {
        return voice.modelId;
      },), },),
    claimAttributions: collectClaimAttributions({ emissions, },),
    findings,
  };
}


//endregion Critic and panel stages
