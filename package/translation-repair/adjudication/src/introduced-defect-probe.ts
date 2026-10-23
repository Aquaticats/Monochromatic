import type { Logger, } from '@monochromatic-dev/module-logger/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type {
  AdjudicatedIssue,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  PRODUCTION_PRIOR_ISSUE_DISCLOSURE,
  buildIntroducedDefectMessages,
  type IntroducedDefectCheckWire,
  isIntroducedDefectReportWire,
  type PriorIssueDisclosure,
  type ProbedEditKind,
} from './introduced-defect-wire.ts';
import { INTRODUCED_DEFECT_RESPONSE_FORMAT, } from './introduced-defect-format.ts';
import {
  type RegionDefectTally,
  resolveProberChecks,
  screenIntroducedDefects,
} from './introduced-defect-screen.ts';
import { proberOriginalOf, } from './introduced-defect-shown.ts';
import type { RepairRegion, } from './repair-region.ts';
import {
  gatherStageVoices,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';

//region Introduced-defect probe stage
// Asks whether the repair broke anything nobody had raised.
//
// OVER AN ACCURACY PATCH IT DECIDES NOTHING DIRECTLY: the report reaches the
// outcome and the artifacts, and the lane contest weighs its admitted claims
// as evidence (`repair-damage-evidence.ts`). A claim there would discard edits
// fixing defects a panel confirmed, including ones in other envelopes, on a
// signal a reading of every flagged region put at six of ten true.
//
// OVER A NATURALNESS REWRITE AN ADMITTED CLAIM ROLLS THE REWRITE BACK (ledger
// L11, decided for quality 2026-09-28): what a rollback loses there is
// fluency, and what comes back is text a checker round or the archive already
// stood behind (`refine-slice-settle.ts`). So does a round that heard fewer
// probers than its quorum, read off the `stage-quorum-unmet` finding this
// stage's gather returns (`refine-probe-verdict.ts`).

/**
 Everything the probe stage produced for one chunk.

 @example
 ```ts
 const { regions, } = await runIntroducedDefectProbe({ ... },);
 ```
 */
export type IntroducedDefectReport = {
  /**
   Screened tally per replaced region, in region order.
   */
  readonly regions: readonly RegionDefectTally[];

  /**
   Probers whose reply arrived and validated.
   */
  readonly heardProbers: number;

  /**
   Probers asked. Kept beside the heard count because a claim confirmed by
   two of three heard probers and one confirmed by two of six configured are
   different evidence, and only the pair distinguishes them.
   */
  readonly configuredProbers: number;

  /**
   Wire irregularities across probers in scorecard-stable wording: the
   gather's own, then a `<model id>: duplicate-check (<region>)` for each
   later check a prober cast on a region it had answered.
   */
  readonly findings: readonly string[];
};

/**
 Screened claim counts summed across every probed region.

 Named so the fold that builds it states its own type. Left to the seed
 literal every count is writable, and the accumulator parameter then reports
 as mutable in a fold that only ever reads it.
 */
type ClaimTotals = Readonly<{
  /**
   Claims of added damage the differential bore out: the quote is in the
   AFTER text and absent from BEFORE. One prober's claim suffices; no second
   prober is asked to agree.
   */
  corroborated: number;

  /**
   Claims of dropped content the differential bore out: the quote is in the
   BEFORE text and absent from AFTER.
   */
  removalCorroborated: number;

  /**
   Claims the baseline text refuted.
   */
  contradicted: number;

  /**
   Claims quoting text neither side carries.
   */
  unanchored: number;

  /**
   Claims restating a defect an accepted issue already named.
   */
  preExisting: number;

  /**
   Probers that read the region and raised nothing.
   */
  noneFound: number;

  /**
   Probers that declined to answer.
   */
  uncertain: number;
}>;

/**
 Report of a probe that never ran, for chunks with nothing replaced.
 */
export const EMPTY_INTRODUCED_DEFECT_REPORT: IntroducedDefectReport = {
  regions: [],
  heardProbers: 0,
  configuredProbers: 0,
  findings: [],
};

/**
 Asks whether each replaced region introduced a defect the baseline lacked.

 @param client - injected model client

 @param proberModelIds - roster asked, writer-disjoint like the checkers

 @param sourceText - original chunk text

 @param baselineText - translation before any replacement

 @param regions - regions the accuracy stage replaced

 @param issues - accepted issues, shown so probers can discount them

 @param identityContext - declared names and handles, or the empty string on a page
 declaring none; REQUIRED so no caller omits it by accident (ledger H8, the
 H2 lesson)

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - pipeline logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Screened tallies plus roster accounting

 @example
 ```ts
 const probe = await runIntroducedDefectProbe({ ... },);
 ```
 */
export async function runIntroducedDefectProbe(
  {
    client,
    proberModelIds,
    sourceText,
    baselineText,
    regions,
    issues,
    editKind = 'accuracy-repair',
    disclosure = PRODUCTION_PRIOR_ISSUE_DISCLOSURE,
    neighbouringIncumbentText,
    neighbouringSourceText,
    identityContext,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly proberModelIds: readonly RosterModelId[];
    readonly sourceText: string;
    readonly baselineText: string;
    readonly regions: readonly RepairRegion[];
    readonly issues: readonly AdjudicatedIssue[];
    readonly editKind?: ProbedEditKind;
    readonly disclosure?: PriorIssueDisclosure;
    readonly neighbouringIncumbentText?: string;
    readonly neighbouringSourceText?: string;
    readonly identityContext: string;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<IntroducedDefectReport> {
  if (regions.length === 0)
    return EMPTY_INTRODUCED_DEFECT_REPORT;

  /**
   Prober sheet plus the region numbering order.
   */
  const plan = buildIntroducedDefectMessages({
    sourceText,
    baselineText,
    regions,
    issues,
    editKind,
    disclosure,
    ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
    ...((neighbouringIncumbentText === undefined) ? {} : { neighbouringIncumbentText, }),
    ...((identityContext === '') ? {} : { identityContext, }),
  },);

  /**
   Heard probers after retry-to-quorum.
   */
  const gather = await gatherStageVoices({
    client,
    modelIds: proberModelIds,
    messages: plan.messages,
    signal,
    exchangeTimeoutMs: perCallTimeoutMs,
    responseFormat: INTRODUCED_DEFECT_RESPONSE_FORMAT,
    validate: isIntroducedDefectReportWire,
    stage: 'introduced-defect-probe',
    l,
    graceMs: stageSettings
      .grace
      .roundMs,
  },);

  /**
   Checks per prober, keyed by model id.
   */
  const ballots: Record<string, readonly IntroducedDefectCheckWire[]> = Object.fromEntries(
    gather.voices
      .map(function toEntry(voice,): readonly [
        string,
        readonly IntroducedDefectCheckWire[],
      ] {
        return [
          voice.modelId,
          voice.value
            .checks,
        ];
      },),
  );

  /**
   Screened tally per region.
   */
  const screened = screenIntroducedDefects({
    regions,
    ballots,
    issues,
    // The regions are screened as the sheet showed them, against the same originals it was cut against.
    original: proberOriginalOf({
      sourceText,
      ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
    },),
  },);

  /**
   Claims summed across regions, for one readable log line.

   The accumulator carries an explicit type rather than taking one from the
   seed literal. An inferred seed makes every count writable, and the rule
   then reports the fold's own parameter while naming the enclosing function
   as the origin, which is the signature line rather than anything that
   produced a value.
   */
  const totals: ClaimTotals = screened.reduce(
    function addRegion(
      running,
      tally,
    ): ClaimTotals {
      return {
        corroborated: running.corroborated + tally.corroborated,
        removalCorroborated: running.removalCorroborated + tally.removalCorroborated,
        contradicted: running.contradicted + tally.contradicted,
        unanchored: running.unanchored + tally.unanchored,
        preExisting: running.preExisting + tally.preExisting,
        noneFound: running.noneFound + tally.noneFound,
        uncertain: running.uncertain + tally.uncertain,
      };
    },
    {
      corroborated: 0,
      removalCorroborated: 0,
      contradicted: 0,
      unanchored: 0,
      preExisting: 0,
      noneFound: 0,
      uncertain: 0,
    } satisfies ClaimTotals,
  );

  // The negative counts belong in the line as much as the positive ones. All
  // claim counts at zero is equally consistent with every prober finding
  // nothing, every prober declining, and every ballot being dropped as a wire
  // fault, and those are three very different states to read a quiet run as.
  l.info(
    `introduced-defect probe: ${String(Object.keys(ballots,)
      .length,)}/${String(proberModelIds.length,)} heard over ${
      String(regions.length,)
    } ${
      wordForCount({
        count: regions.length,
        one: 'region',
        many: 'regions',
      },)
    }, ${String(totals.corroborated,)} added-damage corroborated, ${
      String(totals.removalCorroborated,)
    } dropped-content corroborated, ${
      String(totals.contradicted,)
    } contradicted by the baseline, ${String(totals.unanchored,)} unanchored, ${
      String(totals.preExisting,)
    } restating an accepted issue, ${
      String(totals.noneFound,)
    } found nothing, ${String(totals.uncertain,)} declined`,
  );

  return {
    regions: screened,
    heardProbers: Object.keys(ballots,)
      .length,
    configuredProbers: proberModelIds.length,
    findings: [
      ...gather.findings,
      ...resolveProberChecks({ ballots, },)
        .findings,
    ],
  };
}

//endregion Introduced-defect probe stage
