import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  isInsertionChunk,
  parseDocument,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  readUntranslatedTail,
  runCoverageStage,
  type TargetRegion,
} from '@monochromatic-dev/translation-repair-archive/ts';
import type {
  InsertionAdmission,
  PreparedDocumentPair,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  type FanOutMode,
  mapOverlapped,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import { TranslationRepairInterruptedError, } from '@monochromatic-dev/translation-repair-translate-stage/ts';
import { droppedDestinations, } from '@monochromatic-dev/translation-repair-page-check/ts';
import {
  admitContainerDeficit,
  admitContainerHalves,
  admitReferencedDefinitions,
  classifyInsertionCoverage,
  type InsertionCandidate,
  type InsertionCoverageRow,
} from '@monochromatic-dev/translation-repair-insertion/ts';
import type { BenchSeating, } from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Pass insertion admission
// Production proof for writing source-only passages into a memorial page.
// Coverage is semantic verdict; page shortfall or missing destination is
// independent corroboration. SINGLE ROUND BY DESIGN: a passage the round
// leaves unresolved is not admitted and not proven carried; it is recorded
// as findings and the page ships without it, because inserting on
// uncorroborated evidence duplicates carried content while a recorded gap
// stays visible to the publisher's destination report and the reading
// (doc/planning/translation-repair-no-loop-design.md). Only an unheard
// roster still throws, as infrastructure failure.

/**
 Names a source-only candidate on the lines the overlapped map writes about a
 failure it leaves unthrown.

 @param item - candidate, named by its slice

 @returns Slice by its index

 @example
 ```ts
 candidateSliceName({ item: candidate, },); // 'slice 3'
 ```
 */
function candidateSliceName({ item, }: { readonly item: { readonly sliceIndex: number; }; },): string {
  return `slice ${String(item.sliceIndex,)}`;
}

/**
 Decides which source-only slices a corpus pass may ask translators to fill.

 @param client - provider client shared with pass stages

 @param prepared - one source/target preparation carrying insertion slices

 @param modelIds - measured production coverage roster

 @param overlap - most coverage questions in flight

 @param signal - entry deadline and caller abort

 @param perCallTimeoutMs - deadline per coverage exchange

 @param l - entry logger

 @param beforeCandidate - awaited before each candidate's coverage question,
 handing it the roster it is asked of; none keeps the given one (ledger X12)

 @param fanOut - seats a round asks: the window of quorum plus one by
 default, or the whole bench a fixture scripting every seat asks for, since
 the window is rotated by the sheet text and a sheet edit elsewhere moves
 which seats it holds

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Admitted positions and count-only evidence for every candidate

 @throws {@link TranslationRepairInterruptedError}
 when no coverage voice was heard for some passage

 @example
 ```ts
 const admission = await decidePassInsertionAdmission({ client, prepared, modelIds, overlap: 4, signal, perCallTimeoutMs, l, stageSettings, },);
 ```
 */
export async function decidePassInsertionAdmission(
  {
    client,
    prepared,
    modelIds,
    overlap,
    signal,
    perCallTimeoutMs,
    l,
    beforeCandidate,
    fanOut,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly prepared: PreparedDocumentPair;
    readonly modelIds: readonly RosterModelId[];
    readonly overlap: number;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly beforeCandidate?: () => Promise<BenchSeating>;
    readonly fanOut?: FanOutMode;
    readonly stageSettings: StageSettings;
  }>,
): Promise<InsertionAdmission> {
  /**
   Source-only slices, in document order.
   */
  const candidates: readonly InsertionCandidate[] = prepared
    .slices
    .flatMap(function toCandidate(
      slice,
      position,
    ): readonly InsertionCandidate[] {
      /**
       Placement on target side.
       */
      const { target, } = slice;
      if (!isInsertionChunk(target,))
        return [];
      /**
       Source passage paired with that placement.
       */
      const { source, } = slice;
      return [{
        position,
        sliceIndex: target.sliceIndex,
        sourceText: source.text,
        frontMatter: slice.syntax === 'front-matter',
      },];
    },);
  if (candidates.length === 0) {
    return {
      positions: new Set(),
      findings: [],
    };
  }

  /**
   Metadata insertion positions admitted by explicit syntax role.
   */
  const frontMatterPositions = new Set(candidates
    .filter(function isFrontMatter(candidate,): boolean {
      return candidate.frontMatter;
    },)
    .map(function positionOf(candidate,): number {
      return candidate.position;
    },),);
  /**
   Prose candidates requiring semantic coverage proof.
   */
  const semanticCandidates = candidates.filter(function isProse(candidate,): boolean {
    return !candidate.frontMatter;
  },);

  /**
   Target parsed once so every coverage quote anchors against one document.
   */
  const target = parseDocument({ text: prepared.targetText, },);

  /**
   Target regions the pairing assigned to some source slice. Every one of
   them is foreign to a source-only candidate, since that English renders a
   different original; a partial claim quoting one is no evidence for the
   candidate (class fifty-one, shi_Yumiaoya4).
   */
  const foreignRegions: readonly TargetRegion[] = prepared
    .slices
    .flatMap(function pairedRegion(slice,): readonly TargetRegion[] {
      /**
       Placement on target side.
       */
      const { target: placement, } = slice;
      if (isInsertionChunk(placement,))
        return [];
      return [{
        startOffset: placement.startOffset,
        endOffset: placement.endOffset,
      },];
    },);

  /**
   Declared names preparation built, which every coverage call reads.
   */
  const { identityContext, } = prepared;

  /**
   Logger marking admission as one stage beneath entry.
   */
  const al = tagged({
    tag: decidePassInsertionAdmission.name,
    l,
  },);

  /**
   Semantic and destination evidence per source-only slice.
   */
  const initialRows = await mapOverlapped({
    items: semanticCandidates,
    overlap,
    signal,
    l: al,
    nameOf: candidateSliceName,
    oneItem: async function readCandidate({ item: candidate, },): Promise<InsertionCoverageRow> {
      /**
       Candidate fields used by both evidence readers.
       */
      const {
        sliceIndex,
        sourceText,
      } = candidate;

      /**
       Whether this passage carries a destination absent from whole target.
       */
      const destinations = droppedDestinations({
        sourceText,
        pageText: prepared.targetText,
      },);
      /**
       Missing source destinations counted without exposing their values.
       */
      const missingDestinationCount = destinations
        .dropped
        .length;

      // A CANDIDATE IS ASKED OF THE ROSTER ITS HOOK HANDS OVER (ledger X12,
      // 2026-09-28): the admission read its roster once, with the lanes', so
      // a dry-out inside it left every later candidate on the roster read
      // before it.
      /**
       Seating the hook hands this candidate: a roster read under a hold, or
       none, which keeps the one the admission started on.
       */
      const seating: BenchSeating = (beforeCandidate === undefined) ? {} : await beforeCandidate();
      /**
       Roster verdict independent of pairing and shortfall.
       */
      const answer = await runCoverageStage({
        client,
        modelIds: seating.modelIds ?? modelIds,
        sourcePassage: sourceText,
        translation: target,
        foreignRegions,
        // The names the passage writes, as the page renders them (ledger B28).
        ...((identityContext === undefined) ? {} : { identityContext, }),
        signal,
        exchangeTimeoutMs: perCallTimeoutMs,
        l: al,
        // Conditional spread keeps the knob absent instead of undefined.
        ...((fanOut === undefined) ? {} : { fanOut, }),
        stageSettings,
      },);
      /**
       Coverage result fields persisted as counts and findings.
       */
      const {
        verdict,
        findings: stageFindings,
      } = answer;
      al.info(
        `slice ${String(sliceIndex,)}: coverage=${verdict.kind}, `
          + `missingDestinations=${String(missingDestinationCount,)}`,
      );
      return {
        ...candidate,
        verdictKind: verdict.kind,
        anchoredFull: verdict.anchoredFull,
        anchoredPartial: verdict.anchoredPartial,
        absentCount: verdict.absent,
        heard: verdict.heard,
        asked: verdict.asked,
        missingDestinationCount,
        coverageFinding: `insertion-coverage (slice ${String(sliceIndex,)}, verdict ${verdict.kind}, `
          + `full ${String(verdict.anchoredFull,)}, partial ${String(verdict.anchoredPartial,)}, `
          + `absent ${String(verdict.absent,)}, heard ${String(verdict.heard,)} of ${String(verdict.asked,)})`,
        stageFindings,
        coverageEvidence: verdict.evidence,
        destinationFindings: destinations.findings,
      };
    },
  },);

  // An unheard roster is infrastructure failure, never a quality refusal;
  // masking it as an unresolved recording would freeze an outage into the page.
  for (const row of initialRows) {
    if (row.heard === 0) {
      throw new TranslationRepairInterruptedError({
        reason: 'provider-unavailable',
        findings: [row.coverageFinding,],
      },);
    }
  }
  /**
   Untranslated tail read off the pairing, budgeted on its own (owner,
   2026-09-19; XingZ608).
   */
  const tail = readUntranslatedTail({ slices: prepared.slices, },);
  /**
   Slices after the last agreed pair.
   */
  const tailCount = tail.positions
    .size;
  /**
   Expected English of the tail, rounded for the log.
   */
  const tailExpected = Math.round(tail.expected,);
  /**
   The page's own expansion, two decimals for the log.
   */
  const tailExpansion = tail.expansion
    .toFixed(2,);
  al.info(
    `untranslated tail: ${String(tailCount,)} slice(s) after the last agreed pair, ${
      String(tail.sourceCodePoints,)
    } source code points at expansion ${tailExpansion}, expected ${String(tailExpected,)}, `
      + `last pair renders ${String(tail.lastPairTargetCodePoints,)}: ${
        tail.exceedsLastPair ? 'admitted on that bound' : 'left with the whole-page budget'
      }`,
  );
  /**
   Single-round resolution of every candidate.
   */
  const classification = classifyInsertionCoverage({
    candidates,
    rows: initialRows,
    frontMatterPositions,
    sourceText: prepared.sourceText,
    targetText: prepared.targetText,
    tail,
  },);
  /**
   Classification with every absent passage inside a container the archive
   carries short of blocks admitted on that deficit (class sixty-one,
   XingZ611, 2026-09-19).
   */
  const deficit = admitContainerDeficit({
    slices: prepared.slices,
    positions: classification.positions,
    unresolvedRows: classification.unresolvedRows,
  },);
  for (const finding of deficit.findings)
    al.info(finding,);
  /**
   Classification with each container's halves admitted together (class
   fifty-seven, XingZ607, 2026-09-18).
   */
  const halves = admitContainerHalves({
    slices: prepared.slices,
    positions: deficit.positions,
    unresolvedRows: deficit.unresolvedRows,
  },);
  for (const finding of halves.findings)
    al.info(finding,);
  /**
   Admission with every definition a shipping marker references admitted
   beside it (class fifty-nine, XingZ608, 2026-09-19).
   */
  const definitions = admitReferencedDefinitions({
    slices: prepared.slices,
    positions: halves.positions,
    unresolvedRows: halves.unresolvedRows,
    targetText: prepared.targetText,
  },);
  for (const finding of definitions.findings)
    al.info(finding,);
  for (const row of definitions.unresolvedRows) {
    al.info(
      `slice ${String(row.sliceIndex,)}: placement unresolved after the single round `
        + `(verdict ${row.verdictKind}); not admitted, recorded as findings`,
    );
  }
  return {
    positions: definitions.positions,
    carried: classification.carried,
    findings: [
      ...classification.findings,
      ...deficit.findings,
      ...halves.findings,
      ...definitions.findings,
      ...definitions.unresolvedRows
        .map(function unresolvedFinding(row,): string {
          return `insertion-unresolved-after-single-round (slice ${String(row.sliceIndex,)}, `
            + `verdict ${row.verdictKind}); passage not admitted`;
        },),
    ],
  };
}
//endregion Pass insertion admission
