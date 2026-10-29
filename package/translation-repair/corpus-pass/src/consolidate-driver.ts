import type { ArchiveDispute, } from '@monochromatic-dev/translation-repair-archive/ts';
import type { DeclaredNamePair, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  archiveFlawedByMajority,
  archiveStandInFor,
  type ArtifactConsolidateSlice,
  type ConsolidateRoster,
  consolidateSliceKey,
  type ConsolidateSliceSeating,
  laneTextsForSlate,
  type ProjectedLanes,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { PreparedSheetFacts, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  buyConsolidationSlice,
  ConsolidationLedgerGapError,
  type ConsolidationPolishConfig,
  type ConsolidationSettlement,
  persistConsolidationSettlement,
  readStandingVerdict,
  standingTextFor,
} from '@monochromatic-dev/translation-repair-consolidate/ts';
import type { ArtifactContestSlice, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import {
  contestedCountLine,
  contestedRows,
  contestedSliceName,
  indexConsolidationInputs,
  rosterRunShapeOf,
} from './consolidate-driver-index.ts';
import {
  type BoughtConsolidation,
  laneChoiceOf,
  recordedConsolidateSlice,
} from './consolidate-driver-records.ts';
import { acquireConsolidation, } from './consolidate-driver-acquire.ts';
import { consolidationSubjectOf, } from './consolidate-driver-subject.ts';
import type { SliceNeighbourContext, } from '@monochromatic-dev/translation-repair-repair/ts';
import {
  armSliceCost,
  type SliceCache,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { inSliceLogContext, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  mapOverlapped,
  type StageSettings,
  type TwinMemo,
} from '@monochromatic-dev/translation-repair-stage/ts';

//region Consolidate driver
// THE CONSOLIDATION OVER ONE DOCUMENT: which slices get a third rendering
// asked for, and what the roster settled at each.
//
// RUNS ONLY WHERE THE CONTEST RAN. A slice the two lanes worded identically has
// nothing to consolidate: the lanes agree, so a third rendering would be
// competing against their agreement rather than resolving a difference. The
// contest's own eligibility already selected those slices, so this drives off
// its records rather than re-deriving them.
//
// WHY A THIRD RENDERING AT ALL is argued in `consolidate-wire.ts` and measured
// in `doc/planning/the-third-rendering.md`: the lanes fail in opposite
// directions, and at least one slice was found where each lane was better than
// the other in a DIFFERENT PLACE of the same passage. No selection can produce
// that slice's best text, because that text is neither candidate.
//
// IDENTICAL QUESTIONS SHARE ONE CACHE-ELIGIBLE PURCHASE. The position-free key
// names everything the stages see, while `ConsolidationSettlement` carries no
// slice index needing restamp. An unsettled panel is neither persisted nor
// memoized, so its twin asks again exactly as a warm run would.

/**
 Asks the roster for a third rendering at every slice the contest was asked
 about, settled or not; a slice the contest left with no standing text is
 settled as `no-standing-text` without a producer being asked.

 @param client - synthetic chat client

 @param projected - both ledgers as version 2 rows, beside their comparison

 @param contests - one record per contested slice, as the contest wrote them
 for the artifact

 @param modelIds - roster to ask for consolidations

 @param judgeModelIds - roster that judges each slate and gates its winner;
 `modelIds` when not given

 @param archiveDisputes - slices whose archive rendering the repair lane's
 adjudicators disputed, whose repair text stands in for the archive

 @param acceptedRepairClaimNotes - the claims that panel accepted per slice,
 which the final polish gate reads and the key names

 @param translateWinnerBallots - ballots that named the translate lane's text in the lane's own round, per slice; a
 slice absent from it drew none (owner, 2026-10-29, "Ship the two-ballot text")

 @param sheetFacts - what the page tells every sheet and key: the declared names,
 whether the original's entry records a death, the names the translation
 declares for its subject, and what the pages the original cites say; each
 left out where the page lacks it

 @param polishConfig - final body naturalness roles and document guard facts

 @param frontMatterSlices - syntax-bearing metadata slice indexes

 @param lineStructuredSlices - chunk indices whose original is verse or
 otherwise line-structured, which decides whether a producer is shown the rule
 against merging lines

 @param pictureContextBySlice - what the pictures near each slice were read to
 say, keyed by chunk index and already windowed by the caller, since the window
 is positional in the prepared slices and this driver holds none of them

 @param neighbourContextBySlice - passages either side of each slice, keyed the
 same way and computed by the same caller for the same reason

 @param cache - per-entry store of settlements already bought

 @param signal - abort shared with the rest of the entry

 @param perCallTimeoutMs - per-call ceiling

 @param overlap - most contested slices in flight; one reproduces former loop

 @param l - logger to tag

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns One record per consolidated slice, in comparison-row order

 @throws Error - when a contested slice has no row in the repair ledger, which
 means the comparison and the ledger disagree about which slices exist

 @example
 ```ts
 const slices = await consolidateDocument({ client, projected, contests, modelIds, frontMatterSlices, lineStructuredSlices, pictureContextBySlice, neighbourContextBySlice, cache, signal, perCallTimeoutMs, l, stageSettings, },);
 ```
 */
export async function consolidateDocument(
  {
    client,
    projected,
    contests,
    modelIds,
    judgeModelIds = modelIds,
    sheetFacts,
    polishConfig,
    frontMatterSlices,
    lineStructuredSlices,
    pictureContextBySlice,
    neighbourContextBySlice,
    cache,
    signal,
    perCallTimeoutMs,
    overlap = 1,
    beforeSlice,
    archiveDisputes,
    acceptedRepairClaimNotes,
    translateWinnerBallots,
    declaredNamePairs = [],
    l,
    stageSettings,
  }: {
    readonly client: ModelClient;
    readonly projected: ProjectedLanes;

    /**
     Slices whose archive rendering the repair lane's adjudicators disputed;
     the repair text stands in for the archive here too, as the standing
     where the contest chose neither lane (class one hundred seven).
     */
    readonly archiveDisputes?: ReadonlyMap<number, ArchiveDispute>;

    /**
     The claims the repair lane's panel accepted per slice, as the sheet
     block `acceptedRepairClaimsNote` writes them, for the final polish (its
     writers, the judges of their rewrites and its gate) and the key; a slice
     the panel accepted nothing on has no entry.
     */
    readonly acceptedRepairClaimNotes?: ReadonlyMap<number, string>;

    /**
     Ballots that named the translate lane's text in the lane's own round, per slice; a slice absent from the map drew
     none. Two or more, with the final round unable to seat two judges, ship that text over a refused standing (owner,
     2026-10-29, "Ship the two-ballot text").
     */
    readonly translateWinnerBallots?: ReadonlyMap<number, number>;
    /**
     Name pairs the front matter declares, which the publication rule reads
     for a linked title naming a declared person (class one hundred
     fourteen); none leaves that floor silent.
     */
    readonly declaredNamePairs?: readonly DeclaredNamePair[];
    readonly contests: readonly ArtifactContestSlice[];
    readonly modelIds: readonly RosterModelId[];
    readonly judgeModelIds?: readonly RosterModelId[];
    readonly sheetFacts?: PreparedSheetFacts;
    readonly polishConfig?: ConsolidationPolishConfig;
    readonly frontMatterSlices: ReadonlySet<number>;
    readonly lineStructuredSlices: ReadonlySet<number>;
    readonly pictureContextBySlice: ReadonlyMap<number, string>;
    readonly neighbourContextBySlice: ReadonlyMap<number, SliceNeighbourContext>;
    readonly cache: SliceCache<ConsolidationSettlement>;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly overlap?: number;

    /**
     Awaited before each slice starts, so a caller can hold the slice back
     while a named provider hold keeps the bench from quorum.
     */
    readonly beforeSlice?: () => Promise<ConsolidateSliceSeating>;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  },
): Promise<readonly ArtifactConsolidateSlice[]> {
  /**
   Logger naming this driver.
   */
  const dl = tagged({
    l,
    tag: consolidateDocument.name,
  },);

  /**
   Original of each slice (the repair ledger alone carries it) and the
   contest record of each slice the contest answered.
   */
  const {
    sourceTexts,
    contestBySlice,
  } = indexConsolidationInputs({
    projected,
    contests,
  },);

  /**
   What a run on one roster asks, folded into every key it settles, so a
   slice seated on a re-read roster is keyed by the roster that settled it.

   @param roster - writers, judges and naturalness roles the slice runs on

   @returns Run shape for that roster
   */
  function shapeFor({ roster, }: { readonly roster: ConsolidateRoster; },): string {
    return rosterRunShapeOf({
      roster,
      ...((sheetFacts === undefined) ? {} : { sheetFacts, }),
      declaredNamePairs,
    },);
  }

  /**
   Roster the consolidation started on, which every slice runs on until a
   hold re-seats one.
   */
  const startingRoster: ConsolidateRoster = {
    modelIds,
    judgeModelIds,
    ...((polishConfig === undefined) ? {} : { polishConfig, }),
  };

  /**
   What this run asks on its starting roster, folded into every key.
   */
  const runShape = shapeFor({ roster: startingRoster, },);

  /**
   Comparison rows beside contests that selected them, in document order.
   */
  const eligibleRows = contestedRows({
    projected,
    contestBySlice,
  },);

  /**
   Cache-eligible purchases in this document, shared by identical questions.
   */
  const twins: TwinMemo<ConsolidationSettlement> = new Map();

  dl.info(contestedCountLine({ count: contests.length, },),);
  return await mapOverlapped({
    items: eligibleRows,
    overlap,
    signal,
    l: dl,
    nameOf: contestedSliceName,
    oneItem: async function consolidateOne({
      item: {
        row,
        contest,
      },
    },): Promise<ArtifactConsolidateSlice> {
      return await inSliceLogContext({
        lane: 'consolidation',
        sliceIndex: row.sliceIndex,
        run: async function consolidateInContext(): Promise<ArtifactConsolidateSlice> {
          // A SLICE RUNS ON THE ROSTER ITS HOOK HANDS OVER (ledger H5,
          // 2026-09-28), as both lanes' slices do: the hook only waited, so a
          // dry-out inside the consolidation left every later slice on the
          // writers and judges read before it.
          /**
           Seating the hook hands this slice: a roster read under a hold, or
           none, which keeps the roster the consolidation started on.
           */
          const seating: ConsolidateSliceSeating = (beforeSlice === undefined) ? {} : await beforeSlice();
          /**
           Writers, judges and naturalness roles this slice runs on.
           */
          const sliceRoster = seating.roster ?? startingRoster;
          /**
           Original of this slice, which every ledger row carries.
           */
          const sourceText = sourceTexts.get(row.sliceIndex,);
          if (sourceText === undefined)
            throw new ConsolidationLedgerGapError({ sliceIndex: row.sliceIndex, },);

          /**
           Time bracket making this slice visible before and after settlement.
           */
          using cost = armSliceCost({
            l: dl,
            lane: 'consolidation',
            sliceIndex: row.sliceIndex,
            sourceChars: sourceText.length,
            signal,
          },);
          cost.left({ exit: 'failed', },);

          /**
           Lane contest selected, or refusal of both.
           */
          const choice = laneChoiceOf({ verdict: contest.verdict, },);
          /**
           Archive wording as this settlement takes it, the repair lane's text on
           a disputed slice (class one hundred seven), and whether a kept standing
           is that stand-in.
           */
          const {
            incumbentText,
            standInShips,
            disputeNote,
            disputedWordings,
          } = archiveStandInFor({
            ...((archiveDisputes === undefined) ? {} : { archiveDisputes, }),
            sliceIndex: row.sliceIndex,
            incumbentText: row.incumbentText,
            choice,
            l: dl,
          },);
          /**
           Wording the contest left standing, which ships without this stage
           where the gate passes it.
           */
          const laneStanding = standingTextFor({
            choice,
            repairText: row.repairText,
            translateText: row.translateText,
            incumbentText,
          },);
          /**
           Whether the line-structure rule governs this slice.

           READ ONCE, because four places need this same answer: the sheet
           the producers are shown, the guard that reads their proposals, the key
           the settlement resumes under, and the wrap. Asking the set four times
           is how four answers drift into three.
           */
          const lineStructured = lineStructuredSlices.has(row.sliceIndex,);
          /**
           Syntax role shared by every consolidation phase and cache key.
           */
          const syntax = frontMatterSlices.has(row.sliceIndex,)
            ? 'front-matter' as const
            : undefined;

          /**
           Deterministic eligibility and contest endorsement of the standing text,
           and the wording the settlement runs against: the lane's standing, or
           the incumbent where the standing failed the gate and the incumbent
           passes it (owner, 2026-09-09).
           */
          const {
            standingValid,
            standingMayShip,
            settlementText: standingText,
            findings: standingFindings,
            incumbentStandsIn,
            standingRefusal,
          } = readStandingVerdict({
            sourceText,
            standingText: laneStanding,
            incumbentText,
            ...((syntax === undefined) ? {} : { syntax, }),
            lineStructured,
            choice,
            contestVerdict: contest.verdict,
            sliceIndex: row.sliceIndex,
            l: dl,
            declared: declaredNamePairs,
            disputedWordings,
          },);

          /**
           Lane texts the slate offers beside the proposals: each lane's rendering
           that passes the deterministic rule and is not the standing, whenever
           the standing is neither contest-endorsed nor eligible (class forty,
           2026-09-17, `consolidate-lane-offer.ts`).
           */
          const laneTexts = laneTextsForSlate({
            sourceText,
            incumbentText,
            repairText: row.repairText,
            translateText: row.translateText,
            standingText,
            standingMayShip,
            standingEligible: standingValid,
            ...((syntax === undefined) ? {} : { syntax, }),
            lineStructured,
            declared: declaredNamePairs,
            disputedWordings,
          },);

          /**
           What the pictures near this slice were read to say, empty where none
           were.

           MISSING AND EMPTY ARE ONE STATE, folded here on purpose. A slice near no
           readable picture gets an empty block from the windowing, and a slice the
           map never mentions is a slice in exactly that position, so distinguishing
           them would only let the sheet and the key disagree about which spelling
           the caller happened to use.
           */
          const pictureContext = pictureContextBySlice.get(row.sliceIndex,) ?? '';

          /**
           Passages either side of this slice, folded the same way and for the same
           reason: a lone slice has an empty window and a slice the map never
           mentions is a slice in exactly that position.
           */
          const neighbours = neighbourContextBySlice.get(row.sliceIndex,)
            ?? {
              sourceText: '',
              incumbentText: '',
            };

          /**
           What the repair panel accepted about this slice, absent when nothing.
           */
          const acceptedRepairClaimsNote = acceptedRepairClaimNotes?.get(row.sliceIndex,);
          /**
           Slice as both halves take it (`consolidate-driver-subject.ts`).
           */
          const subject = consolidationSubjectOf({
            sourceText,
            incumbentText,
            repairText: row.repairText,
            translateText: row.translateText,
            ballots: contest.ballots,
            ...((syntax === undefined) ? {} : { syntax, }),
            lineStructured,
            declared: declaredNamePairs,
            ...((sheetFacts === undefined) ? {} : { sheetFacts, }),
            ...((disputeNote === undefined) ? {} : { disputeNote, }),
            ...((acceptedRepairClaimsNote === undefined) ? {} : { acceptedRepairClaimsNote, }),
            disputedWordings,
            pictureContext,
            neighbours,
          },);

          /**
           Key this settlement resumes under.
           */
          const key = consolidateSliceKey({
            runShape: (seating.roster === undefined) ? runShape : shapeFor({ roster: seating.roster, },),
            sourceText,
            incumbentText,
            ...((syntax === undefined) ? {} : { syntax, }),
            repairText: row.repairText,
            translateText: row.translateText,
            standingText,
            ballots: contest.ballots,
            lineStructured,
            pictureContext,
            neighbouringSourceText: neighbours.sourceText,
            neighbouringIncumbentText: neighbours.incumbentText,
            laneTexts,
            ...((disputeNote === undefined) ? {} : { archiveDisputeNote: disputeNote, }),
            ...((acceptedRepairClaimsNote === undefined) ? {} : { acceptedRepairClaimsNote, }),
          },);

          /**
           What the roster settled here, bought, resumed, or reused from a twin.
           */
          const acquired = await acquireConsolidation({
            key,
            cache,
            twins,
            buy: async function buyThisRow(): Promise<BoughtConsolidation> {
              /**
               Settlement bought for this question.
               */
              const bought = await buyConsolidationSlice({
                client,
                roster: sliceRoster.modelIds,
                judgeModelIds: sliceRoster.judgeModelIds,
                subject,
                standingText,
                lineStructured,
                sliceIndex: row.sliceIndex,
                ...((sliceRoster.polishConfig === undefined) ? {} : { polishConfig: sliceRoster.polishConfig, }),
                standingMayShip,
                standingEligible: standingValid,
                ...((standingRefusal === undefined) ? {} : { standingRefusal, }),
                standingFindings,
                laneTexts,
                // The hook's slate reach is the floor check: fewer judges than ballots buys no slate.
                ...((seating.slateReachable === undefined) ? {} : { slateReachable: seating.slateReachable, }),
                // Over a refused standing, the translate lane's text two judges chose ships where the slate cannot seat two.
                translateWinnerBallots: translateWinnerBallots?.get(row.sliceIndex,) ?? 0,
                // The run-off over this eligible standing and its forfeit past an
                // undecided gate both read a strict majority of the contest ballots
                // (owner answers 2026-10-11, "Majority flawed yes" and "Majority,
                // like the run-off"), from the ballots the key already carries.
                runoffOverStanding: archiveFlawedByMajority({
                  verdict: contest.verdict,
                  ballots: contest.ballots,
                },),
                signal,
                perCallTimeoutMs,
                // Tagged with the slice so every producer, slate, gate and
                // settlement line names the passage it bought (class one hundred
                // eighty-five: parallel slices wrote their ballots unlabelled).
                l: tagged({
                  l: dl,
                  tag: `slice ${String(row.sliceIndex,)}`,
                },),
                stageSettings,
              },);

              /**
               Whether purchase became reusable evidence.
               */
              const persisted = await persistConsolidationSettlement({
                key,
                settlement: bought,
                cache,
                standingMayShip,
                signal,
              },);
              return {
                settlement: bought,
                persisted,
              };
            },
            l: dl,
            signal,
          },);
          /**
           What the slice settled, polish included, before artifact projection.
           */
          const { settlement, } = acquired;
          cost.left({ exit: acquired.exit, },);
          // THE TENSE FINDING IS RECORDED HERE, from the wording the slice shipped,
          // so a settlement resumed from the cache records the candidates of the
          // text that shipped as a fresh one does, and no verdict reads them.
          return recordedConsolidateSlice({
            sliceIndex: row.sliceIndex,
            settlement,
            incumbentStandsIn: incumbentStandsIn || standInShips,
            ...((sheetFacts === undefined) ? {} : { sheetFacts, }),
          },);
        },
      },);
    },
  },);
}

//endregion Consolidate driver
