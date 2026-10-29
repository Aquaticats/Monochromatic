import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { DeclaredNamePair, } from '@monochromatic-dev/translation-repair-house-rule/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  inSliceLogContext,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  applyLaneContestEligibility,
  type ArtifactContestSlice,
  type BoughtLaneContest,
  contestEligibleIndexes,
  contestLaneSlice,
  describeContestSlice,
  describeInadmissibleLanes,
  describeSoleLaneSlice,
  frontMatterContestEligibility,
  ineligibleCandidatesOf,
  laneContestChoiceVerdict,
  type LaneContestOutcome,
  laneContestRunShape,
  laneContestSliceKey,
  persistLaneContestOutcome,
  soleLaneOf,
  storedContestOf,
} from '@monochromatic-dev/translation-repair-lane-contest/ts';
import {
  type FanOutMode,
  mapOverlapped,
  reuseTwinOrBuy,
  type StageSettings,
  type TwinMemo,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { ProjectedLanes, } from '@monochromatic-dev/translation-repair-artifact/ts';
import type {
  BenchSeating,
  SliceCache,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { PreparedSheetFacts, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Lane contest driver
// The contest over one document: which slices are worth asking about, and what
// the roster said at each.
//
// ASKS NOTHING WHERE THE TWO LANES AGREE. A contest between two identical
// candidates has no question to put and no answer worth buying, and the slices
// where they agree are the majority of most documents.
//
// AN EMPTY CONTEST IS STILL A CONTEST. A document whose lanes never differed
// returns no slices, and the caller records that as a contest that ran and
// found nothing rather than as a question nobody asked. Those are different
// facts and the artifact keeps them apart.
//
// IDENTICAL QUESTIONS SHARE ONE CACHE-ELIGIBLE PURCHASE. Contest keys omit
// slice position because position changes nothing the roster sees, and contest
// outcomes carry no slice index that needs restamping. The promise memo keeps
// overlapped twins from buying contradictory ballots or racing to overwrite one
// cache file. An unheard outcome is neither persisted nor memoized, so its twin
// asks again exactly as a warm run would.

/**
 Names a compared slice on the lines the overlapped map writes about a failure
 it leaves unthrown.

 @param item - comparison row of the slice

 @returns Slice by its index

 @example
 ```ts
 comparedSliceName({ item: { sliceIndex: 3, }, },); // 'slice 3'
 ```
 */
function comparedSliceName({ item, }: { readonly item: { readonly sliceIndex: number; }; },): string {
  return `slice ${String(item.sliceIndex,)}`;
}

/**
 Asks the roster which lane should ship, at every slice the two lanes worded
 differently and both had a passage to work on. A slice only one lane applies
 to is settled by rule as a `sole-lane` verdict, with no roster asked (ledger
 L6).

 @param client - synthetic chat client

 @param projected - both ledgers as version 2 rows, beside their comparison

 @param modelIds - roster to ask

 @param sheetFacts - what the page tells every sheet and key: the declared names,
 whether the original's entry records a death, the names the translation
 declares for its subject, and what the pages the original cites say; each
 left out where the page lacks it

 @param frontMatterSlices - syntax-bearing metadata slice indexes

 @param lineStructuredSlices - slice indexes the line-structure rule governs,
 which the publication verdict reads so a winner merging a governed slice's
 lines is neither persisted nor twin-memoized (ledger H2)

 @param cache - per-entry store of ballots already bought

 @param signal - abort shared with the rest of the entry

 @param perCallTimeoutMs - per-call ceiling

 @param overlap - most contested slices in flight; one reproduces former loop

 @param l - logger to tag

 @param fanOut - seats a contest round asks: the window of quorum plus one
 by default, or the whole bench a fixture scripting every seat asks for

 @param beforeSlice - awaited before each slice starts, handing it the judges
 it runs on; none keeps the given ones

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns One record per contested slice, in comparison-row order

 @example
 ```ts
 const slices = await contestDocumentLanes({ client, projected, modelIds, frontMatterSlices, lineStructuredSlices, cache, signal, perCallTimeoutMs, l, stageSettings, },);
 ```
 */
export async function contestDocumentLanes(
  {
    client,
    projected,
    modelIds,
    sheetFacts,
    frontMatterSlices,
    lineStructuredSlices,
    cache,
    damageClaimsBySlice = new Map<number, readonly string[]>(),
    disputeNotesBySlice = new Map<number, string>(),
    declaredNamePairs = [],
    signal,
    perCallTimeoutMs,
    overlap = 1,
    l,
    fanOut,
    beforeSlice,
    stageSettings,
  }: {
    readonly client: ModelClient;
    readonly projected: ProjectedLanes;
    readonly modelIds: readonly RosterModelId[];
    readonly sheetFacts?: PreparedSheetFacts;
    readonly frontMatterSlices: ReadonlySet<number>;
    readonly lineStructuredSlices: ReadonlySet<number>;
    readonly cache: SliceCache<LaneContestOutcome>;
    /**
     Corroborated added-damage claims against the repair lane, per slice,
     shown to the judges as evidence (`repair-damage-evidence.ts`).
     */
    readonly damageClaimsBySlice?: ReadonlyMap<number, readonly string[]>;
    /**
     Dispute notes against the archive rendering, per slice, shown to the
     judges as evidence (class one hundred eight).
     */
    readonly disputeNotesBySlice?: ReadonlyMap<number, string>;
    /**
     Name pairs the front matter declares, which the publication rule reads
     for a linked title naming a declared person (class one hundred
     fourteen); none leaves that floor silent.
     */
    readonly declaredNamePairs?: readonly DeclaredNamePair[];
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly overlap?: number;
    readonly l: Logger;
    readonly fanOut?: FanOutMode;
    readonly beforeSlice?: () => Promise<BenchSeating>;
    readonly stageSettings: StageSettings;
  },
): Promise<readonly ArtifactContestSlice[]> {
  /**
   Logger naming this driver.
   */
  const dl = tagged({
    l,
    tag: contestDocumentLanes.name,
  },);

  /**
   Original of each slice, which the comparison does not carry and the ledger
   does.
   */
  const sourceTexts = new Map(projected.delivery
    .repair
    .map(function nameSource(row,): readonly [
      number,
      string,
    ] {
      return [
        row.sliceIndex,
        row.sourceText,
      ];
    },),);

  /**
   Slices worth asking about.
   */
  const eligible = new Set(contestEligibleIndexes({ comparison: projected.comparison, },),);

  /**
   What a run on one bench asks, folded into every key it settles, so a
   slice seated on re-read judges is keyed by the judges that voted on it.

   @param judges - bench the slice runs on

   @returns Run shape for that bench
   */
  function shapeFor({ judges, }: { readonly judges: readonly RosterModelId[]; },): string {
    return laneContestRunShape({
      modelIds: judges,
      ...sheetFacts,
    },);
  }

  /**
   What this run asks on its starting bench, folded into every key.
   */
  const runShape = shapeFor({ judges: modelIds, },);

  /**
   Comparison rows whose lane wordings differ, in document order.
   */
  const eligibleRows = projected.comparison
    .filter(function isEligible(row,): boolean {
      return eligible.has(row.sliceIndex,);
    },);

  /**
   Slices the two lanes covered at all, which the eligible count is read
   against.
   */
  const compared = projected.comparison
    .length;
  dl.info(`lane contest: slices that differ: ${String(eligible.size,)} of ${String(compared,)}`,);

  /**
   Cache-eligible purchases in this document, shared by every contested row.
   */
  const twins: TwinMemo<LaneContestOutcome> = new Map();

  return await mapOverlapped({
    items: eligibleRows,
    overlap,
    signal,
    l: dl,
    nameOf: comparedSliceName,
    oneItem: async function contestOne({ item: row, }): Promise<ArtifactContestSlice> {
      return await inSliceLogContext({
        lane: 'contest',
        sliceIndex: row.sliceIndex,
        run: async function contestInContext(): Promise<ArtifactContestSlice> {
          // ONE APPLICABLE LANE NEEDS NO ROSTER (ledger L6). Checked before the
          // bench is read or the cache is asked, so a one-lane slice costs no
          // call, no seating and no cache write. A syntax-bearing slice stays on
          // the roster, since its candidates are admitted one by one by the
          // source-backed rule and the record must carry that admission.
          /**
           Lane that applies alone at this slice, absent where both lanes did.
           */
          const sole = soleLaneOf({ row, },);
          /**
           Whether this slice is syntax-bearing, which keeps it on the roster.
           */
          const syntaxBearing = frontMatterSlices.has(row.sliceIndex,);
          if ((sole.kind === 'one-lane') && (!syntaxBearing)) {
            dl.info(
              `slice ${String(row.sliceIndex,)}: the ${sole.lane} lane is the only one with a passage here; the roster is not asked`,
            );
            return describeSoleLaneSlice({
              sliceIndex: row.sliceIndex,
              lane: sole.lane,
            },);
          }
          // A SLICE RUNS ON THE JUDGES ITS HOOK HANDS OVER (ledger X12,
          // 2026-09-28), as the lanes' and the consolidation's slices do: the
          // contest read its judges once, so a dry-out inside it left every
          // later slice on the judges read before it.
          /**
           Seating the hook hands this slice: judges read under a hold, or none,
           which keeps those the contest started on.
           */
          const seating: BenchSeating = (beforeSlice === undefined) ? {} : await beforeSlice();
          /**
           Judges this slice runs on.
           */
          const sliceModelIds = seating.modelIds ?? modelIds;
          /**
           Original of this slice, which every ledger row carries. The
           comparison is built from the repair ledger row by row
           (`compareLanes`), so every compared slice has one there.
           */
          const sourceText = nonNullishOrThrow(sourceTexts.get(row.sliceIndex,),);

          /**
           Syntax role shared by prompt and cache key.
           */
          const syntax = frontMatterSlices.has(row.sliceIndex,)
            ? 'front-matter' as const
            : undefined;
          /**
           Source-backed candidate admission for syntax-bearing contest.
           */
          const eligibility = (syntax === 'front-matter')
            ? frontMatterContestEligibility({
              sourceText,
              incumbentText: row.incumbentText,
              repairText: row.repairText,
              translateText: row.translateText,
            },)
            : undefined;
          /**
           Candidate names prompt marks unavailable before panel votes.
           */
          const ineligibleCandidates = (eligibility === undefined)
            ? []
            : ineligibleCandidatesOf({ eligibility, },);
          /**
           Probe claims against this slice's repair text, empty for most slices.
           */
          const repairDamageClaims = damageClaimsBySlice.get(row.sliceIndex,) ?? [];
          // Logged so a run's log witnesses whether the judges were shown any claim,
          // which the ballots alone do not say; the count takes its own word
          // (ledger B66).
          if (repairDamageClaims.length > 0) {
            dl.info(
              `slice ${String(row.sliceIndex,)}: ${String(repairDamageClaims.length,)} corroborated damage ${
                wordForCount({
                  count: repairDamageClaims.length,
                  one: 'claim',
                  many: 'claims',
                },)
              } shown to the judges`,
            );
          }
          /**
           Those claims as the optional subject and key field, absent when none.
           */
          const damageFragment = (repairDamageClaims.length === 0)
            ? {}
            : { repairDamageClaims, };
          /**
           Accepted additions against this slice's archive rendering, absent for
           most slices (class one hundred eight).
           */
          const archiveDisputeNote = disputeNotesBySlice.get(row.sliceIndex,);
          /**
           That note as the optional subject and key field, absent when none.
           */
          const disputeFragment = (archiveDisputeNote === undefined)
            ? {}
            : { archiveDisputeNote, };
          /**
           Whether the line rule governs this slice: the judges read it, the key
           names it, and the winner is validated under it (ledger S15, H2).
           */
          const lineStructured = lineStructuredSlices.has(row.sliceIndex,);
          /**
           Key these ballots resume under.
           */
          const key = laneContestSliceKey({
            runShape: (seating.modelIds === undefined) ? runShape : shapeFor({ judges: seating.modelIds, },),
            sourceText,
            incumbentText: row.incumbentText,
            incumbentKind: row.incumbentKind,
            ...((syntax === undefined) ? {} : { syntax, }),
            repairText: row.repairText,
            translateText: row.translateText,
            ...damageFragment,
            ...disputeFragment,
            lineStructured,
          },);

          /**
           Ballots an earlier run already bought for this slice, if any.
           */
          const resumed = cache
            .resumed
            .get(key,);

          /**
           What the roster settled here, bought, resumed, or reused from a twin.
           */
          const outcome = await (async function resumeOrBuy(): Promise<LaneContestOutcome> {
            if (resumed !== undefined)
              return resumed;
            /**
             Twin's persisted ballots or this row's fresh purchase.
             */
            const asked = await reuseTwinOrBuy({
              key,
              memo: twins,
              buy: async function buyThisRow(): Promise<BoughtLaneContest> {
                /**
                 Ballots bought for this question.
                 */
                const rawBought = await contestLaneSlice({
                  client,
                  modelIds: sliceModelIds,
                  subject: {
                    sourceText,
                    incumbentText: row.incumbentText,
                    repairText: row.repairText,
                    translateText: row.translateText,
                    ...((syntax === undefined) ? {} : { syntax, }),
                    ...((ineligibleCandidates.length === 0) ? {} : { ineligibleCandidates, }),
                    ...sheetFacts,
                    ...damageFragment,
                    ...disputeFragment,
                    lineStructured,
                  },
                  signal,
                  exchangeTimeoutMs: perCallTimeoutMs,
                  // Tagged with the slice so each ballot line the stage writes
                  // says which passage it judged.
                  l: tagged({
                    l: dl,
                    tag: `slice ${String(row.sliceIndex,)}`,
                  },),
                  // Conditional spread keeps the knob absent instead of undefined.
                  ...((fanOut === undefined) ? {} : { fanOut, }),
                  stageSettings,
                  // The hook's reach is the floor check: fewer seats than voices opens no round.
                  ...((seating.reachable === undefined) ? {} : { reachable: seating.reachable, }),
                },);
                /**
                 Effective result after inadmissible raw choices are excluded.
                 */
                const bought = applyLaneContestEligibility({
                  outcome: rawBought,
                  ...((eligibility === undefined) ? {} : { eligibility, }),
                },);
                // Gather rounds degrade torn-down calls to silence. A quorum that
                // arrived before the abort must not make the abandoned entry look
                // done or become warm-run evidence.
                /**
                 Verdict on whether selected lane can cross final publication
                 boundary, with the findings for the log.
                 */
                const choiceVerdict = laneContestChoiceVerdict({
                  outcome: bought,
                  sourceText,
                  incumbentText: row.incumbentText,
                  repairText: row.repairText,
                  translateText: row.translateText,
                  ...((syntax === undefined) ? {} : { syntax, }),
                  lineStructured,
                  declared: declaredNamePairs,
                },);
                /**
                 Whether selected lane can cross final publication boundary.
                 */
                const choiceMayShip = choiceVerdict.mayShip;
                if (!choiceMayShip) {
                  /**
                   Findings joined for the log line.
                   */
                  const findings = choiceVerdict.findings
                    .join(' ',);

                  /**
                   Why each excluded lane was inadmissible, which the floor
                   finding alone does not say; empty off the metadata slice.
                   */
                  const inadmissible = (syntax === 'front-matter')
                    ? describeInadmissibleLanes({
                      sourceText,
                      incumbentText: row.incumbentText,
                      repairText: row.repairText,
                      translateText: row.translateText,
                    },)
                    : [];

                  /**
                   Verdict findings and the lanes' reasons as one line.
                   */
                  const reported = [
                    findings,
                    ...inadmissible,
                  ].join(' ',);
                  dl.warn(
                    `slice ${String(row.sliceIndex,)}: contest winner fails publication invariants and remains retryable: ${reported}`,
                  );
                }
                /**
                 Whether this purchase became reusable evidence.
                 */
                const persisted = await persistLaneContestOutcome({
                  key,
                  outcome: bought,
                  cache,
                  choiceMayShip,
                  signal,
                },);
                return {
                  outcome: bought,
                  persisted,
                };
              },
              persistedOf: storedContestOf,
              l: dl,
              signal,
            },);
            if (asked.kind === 'reused')
              return asked.twin;
            return asked.bought
              .outcome;
          })();
          return describeContestSlice({
            sliceIndex: row.sliceIndex,
            outcome,
            ...((eligibility === undefined) ? {} : { eligibility, }),
          },);
        },
      },);
    },
  },);
}

//endregion Lane contest driver
