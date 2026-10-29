import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  PRODUCTION_PRIOR_ISSUE_DISCLOSURE,
  runIntroducedDefectProbe,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import {
  deriveRefinableEnvelopes,
  parseDocument,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { refineProbeVerdict, } from './refine-probe-verdict.ts';
import { retainsResolvedIssues, } from './refine-recheck.ts';
import { runRefineStage, } from './refine-stage.ts';
import type {
  ChunkRepairOutcome,
  RepairModels,
  RepairSliceSeating,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import {
  checkerBenchAtStage,
  standingSeating,
} from './repair-checker-reseat.ts';
import { collectRefinedAuthors, } from './issue-authors.ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Refine slice settle
// What the naturalness lane does to ONE slice, separated from the loop that
// walks them.
//
// Its own file so the cache in `refine-phase.ts` has a single call to wrap.
// Inline, the resume check would have to sit above four separate exits and the
// persist below each of them, which is how a cache ends up storing three of the
// four answers a stage can give.

/**
 What one slice's refinement settled, as the cache stores it.

 `asked` IS DELIBERATELY NOT IN HERE, and that absence is the point. It says
 whether this RUN reached a rewriter, which decides whether a run overtaken by
 an abort may still call itself finished. A slice resumed from disk asked
 nobody anything, so a stored `asked` would be a previous run's answer to a
 question only the current run can be asked.

 @example
 ```ts
 const settled: RefinedSliceSettlement = { outcome, findings: [], };
 ```
 */
export type RefinedSliceSettlement = {
  /**
   Final outcome for this slice, refined where a rewrite won and survived.
   */
  readonly outcome: ChunkRepairOutcome;

  /**
   Findings this slice contributed, in scorecard-stable wording.
   */
  readonly findings: readonly string[];
};

/**
 Same settlement plus whether this run reached a rewriter.

 @example
 ```ts
 const bought: RefinedSliceOutcome = { outcome, findings: [], asked: true, };
 ```
 */
export type RefinedSliceOutcome = RefinedSliceSettlement & {
  /**
   Whether this slice had anything eligible to rewrite, which is what makes a
   run one that BOUGHT rather than one that only resumed.
   */
  readonly asked: boolean;

  /**
   Models whose rewrite is in the text this returns, empty on every path
   where no rewrite ships: a rewriter that changed nothing, a rewrite the
   recheck rolled back or could not hear a quorum of checkers on, and a
   rewrite the damage probe rolled back or could not hear a quorum of
   probers on.

   NOT STORED, FOR THE REASON `asked` IS NOT. It names what THIS run bought,
   and a slice resumed from disk bought no rewrite. `outcome.authorship`
   already carries who wrote the text for every later reader; this exists so
   an instrument can tell a refiner whose rewrite shipped without a ballot
   from one that never answered, which `authorship` unions away.
   */
  readonly refinedBy: readonly RosterModelId[];

  /**
   Refiners heard with a usable answer on this slice, proposal or not.

   NOT STORED, FOR THE REASON `refinedBy` IS NOT. This exists so an
   instrument can tell a refiner that answered and left the paragraph as it
   stood from one that never answered, which every stored field unions
   away. Empty where no rewriter was asked.
   */
  readonly refinersHeard: readonly RosterModelId[];
};

/**
 @internal

 Runs the naturalness lane over one settled slice.

 @param client - injected model client

 @param outcome - settled accuracy outcome for this slice

 @param sourceText - slice original, which is the faithfulness anchor

 @param incumbentText - archive wording, which a rewrite may land back on

 @param definitions - definitions of the assembled document, so references
 resolve during gating even when their definition lives in another slice

 @param models - role roster

 @param reseat - reads the checker seating as of now, so the recheck and the
 rewrite probe run on the bench a hold that began inside the lane re-seats
 (class one hundred thirteen); the standing seating when absent

 @param refinerModelIds - rewriters, already known non-empty

 @param identityContext - declared names and handles, when any

 @param referenceContext - what the pages the original cites say, with
 their rule, when the original cites any (class forty-one)

 @param declaredNames - same declarations as strings a guard compares

 @param declaredSubjectForms - declared forms among them that name the subject, which may stand in for one another

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - pipeline logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Final outcome, findings, and whether a rewriter was reached

 @param neighbouringSourceText - original of the passages either side, handed
 to the damage probe so a phrase that moved next door is not read as one this
 rewrite deleted

 @param neighbouringIncumbentText - archive English of those same two, which is
 the side a relocation shows

 @example
 ```ts
 const settled = await settleRefinedSlice({ client, outcome, sourceText, incumbentText, definitions, models, refinerModelIds, declaredNames, signal, perCallTimeoutMs, l, stageSettings, },);
 ```
 */
export async function settleRefinedSlice(
  {
    client,
    outcome,
    sourceText,
    incumbentText,
    definitions,
    models,
    reseat = standingSeating,
    refinerModelIds,
    identityContext,
    referenceContext,
    declaredNames,
    declaredSubjectForms,
    neighbouringSourceText,
    neighbouringIncumbentText,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly outcome: ChunkRepairOutcome;
    readonly sourceText: string;
    readonly incumbentText: string;
    readonly definitions: string;
    readonly models: RepairModels;
    readonly reseat?: () => Promise<RepairSliceSeating>;
    readonly refinerModelIds: readonly RosterModelId[];
    readonly identityContext?: string;
    readonly referenceContext?: string;
    readonly declaredNames: readonly string[];
    readonly declaredSubjectForms?: readonly string[];
    readonly neighbouringSourceText?: string;
    readonly neighbouringIncumbentText?: string;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<RefinedSliceOutcome> {
  // NO STOP ON STANDING NON-TRANSLATION VOTES (ledger L15). This returned the
  // slice unrefined on the reading that such a slice shipped deliberately
  // untouched, which stopped being true on 2026-08-16: question 3, answer B
  // keeps the critics as evidence rather than deciding anything and removes
  // every early return they owned, and the repair lane has changed 8 of the
  // 10 such slices over every artifact. The votes still ride on the outcome
  // and in this lane's key.

  /**
   Eligible paragraphs of this slice's repaired text.
   */
  const slice = deriveRefinableEnvelopes({
    document: parseDocument({ text: outcome.repairedText, },),
  },);

  /**
   Whether this slice had anything to rewrite at all.
   */
  const asked = slice.envelopes
    .length
    > 0;

  /**
   What refinement decided for this slice.
   */
  const refined = await runRefineStage({
    client,
    refinerModelIds,
    judgeModelIds: models.judgeModelIds,
    sourceText,
    repairedText: outcome.repairedText,
    envelopes: slice.envelopes,
    definitions,
    ...(identityContext === undefined ? {} : { identityContext, }),
    ...(referenceContext === undefined ? {} : { referenceContext, }),
    declaredNames,
    ...((declaredSubjectForms === undefined) ? {} : { declaredSubjectForms, }),
    mode: { kind: 'comparative', },
    sliceIndex: outcome.sliceIndex,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  },);

  /**
   This slice with the refinement round appended to what the editor stage
   already recorded.

   BUILT BEFORE ANY EXIT OF THIS FUNCTION, because a refinement that lost is exactly the
   round worth reading: it says the panel looked at the repaired text and
   either could not agree or preferred a rewrite the guards then refused.
   Returning the bare outcome on those paths would keep the ballots only when
   they agreed with the result.
   */
  const withRefineRounds: ChunkRepairOutcome = {
    ...outcome,
    rounds: [
      ...outcome.rounds,
      ...refined.rounds,
    ],
  };
  if (!refined.changed)
    return {
      outcome: withRefineRounds,
      findings: [
        ...slice.findings,
        ...refined.findings,
      ],
      asked,
      // Rewriters answered and none of their text is in what ships, so none of
      // them wrote it.
      refinedBy: [],
      refinersHeard: refined.heard,
    };

  /**
   Checkers for this slice's recheck and rewrite probe, on the bench as seated
   now (class one hundred thirteen). Read here, after the rewrite shipped and
   right before the checkers are asked, because the refiners and their judges
   take seconds a hold can begin in; the proof stage re-seats the same way
   (class one hundred nine), and this stage must not keep the bench the chunk
   was seated with.
   */
  const stageCheckers = await checkerBenchAtStage({
    models,
    reseat,
    l,
  },);

  /**
   Whether the recheck reached its quorum of checkers on the round and on
   every issue, every issue they had confirmed is still confirmed in the
   refined text, and no accepted issue the text never fixed regressed on it
   (two or more worse ballots outweighing the rest).
   */
  const retained = await retainsResolvedIssues({
    client,
    checkerModelIds: stageCheckers,
    // BOTH STAGES' AUTHORS. The recheck reads text the editors repaired and the
    // refiners then rewrote, so a checker that had a hand in either is judging
    // its own work and must be discounted for it. The outcome carries the
    // editor's half; the refiners are named here.
    outcome: withRefineRounds,
    refineContributors: refined.contributors,
    sourceText,
    refinedText: refined.refinedText,
    // The rewriters' evidence, which the checkers judge the rewrite by
    // (ledger L14): two worse ballots that outweigh the rest roll the whole
    // slice back.
    ...(identityContext === undefined ? {} : { identityContext, }),
    ...(referenceContext === undefined ? {} : { referenceContext, }),
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  },);
  if (!retained.retained)
    return {
      outcome: {
        ...withRefineRounds,
        // THE ROUND THAT DECIDED THE ROLLBACK. This is the one path where the
        // recheck changes what ships, so dropping its ballots here would lose
        // the most consequential checker round the lane ever buys, and would
        // leave `refine-rolled-back` naming issues with no evidence behind it.
        // A round short of its quorum is kept the same way: the few ballots
        // it did hear, or none, are what `refine-recheck-unheard` rests on.
        recheckReadings: retained.readings,
      },
      findings: [
        ...slice.findings,
        ...refined.findings,
        ...retained.findings,
      ],
      asked,
      // The rewrite was rolled back, so what ships is the editors' text again.
      refinedBy: [],
      refinersHeard: refined.heard,
    };

  /**
   Audit of damage the REWRITE caused, which rolls it back on any claim the
   screen admits (ledger L11) and on a round that heard fewer probers than
   its quorum.

   The accuracy probe already ran, but it compared the original translation
   with the repaired one and finished before this lane started, so it says
   nothing about the text this rewrite produced. Auditing one whole slice
   rather than each rewritten paragraph matches the unit the lane itself
   decides in: `retainsResolvedIssues` rolls back the whole slice too.

   The roster is the checkers, exactly as the accuracy probe uses, and
   `assertCheckerIndependence` in the refine phase has already established that
   no refiner is among them, so nobody audits their own rewrite.
   */
  const refinementDefects = await runIntroducedDefectProbe({
    client,
    proberModelIds: stageCheckers,
    sourceText,
    baselineText: outcome.repairedText,
    regions: [
      {
        envelopeId: `refinement/${String(outcome.sliceIndex,)}`,
        issueIds: outcome.issues
          .map(function toId(issue,) {
            return issue.issueId;
          },),
        before: outcome.repairedText,
        editorAfter: refined.refinedText,
      },
    ],
    issues: outcome.issues,
    editKind: 'naturalness-refinement',
    // The declared names (ledger H8), since an admitted claim rolls the rewrite
    // back, and a prober reading without them would count a declared name kept
    // as written as a defect.
    identityContext: identityContext ?? '',
    // THE SAME WINDOW THE ACCURACY LANE'S PROBE GETS, which is what makes the
    // two lanes' damage telemetry comparable at all. Without it this auditor
    // reasons about a slice in isolation while its counterpart reasons about
    // one in context, so a difference between their findings could be the
    // lanes differing or could be the windows differing, and no reading of the
    // numbers can separate those.
    //
    // It matters more here than for the accuracy probe, not less. This lane
    // rewrites for FLUENCY, and the commonest fluent rewrite of a paragraph
    // that repeats what the paragraph next door already said is to drop the
    // repetition. Judged alone that is a deletion; judged with the neighbour
    // visible it is the redundancy it was.
    ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
    ...((neighbouringIncumbentText === undefined) ? {} : { neighbouringIncumbentText, }),
    // Withheld for the reason the accuracy stage withholds: a prober told what
    // to excuse raised almost nothing, and `introduced-defect-screen.ts`
    // dismisses a claim quoting wording an accepted issue complained about
    // instead. That screen is right on both kinds of slice this lane rewrites:
    // where the patch won, the complaint names wording the repair replaced;
    // where it lost (1,218 of 2,144 refined slices over every run, ledger L11),
    // the defect is still in `T1`, and a rewrite that keeps it introduced
    // nothing.
    disclosure: PRODUCTION_PRIOR_ISSUE_DISCLOSURE,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  },);

  /**
   What the probe's report decides for the rewrite: kept, or rolled back on
   a claim the screen admitted or on a round short of its probers' quorum.
   */
  const probed = refineProbeVerdict({ report: refinementDefects, },);
  if (probed.kind === 'rolled-back') {
    // LEDGER L11, decided for quality under the owner's standing directive
    // (2026-09-28). 175 of 2,144 kept rewrites over every run carried an
    // admitted claim, and a reading of every region with one (the roster
    // calibration of 2026-09-03) found six of ten true, three false and one
    // borderline: rolling back reverts more damaged rewrites than it costs
    // fluent ones, and what comes back is text a checker round or the archive
    // already stood behind. Requiring two claims would have caught 9.
    //
    // A PROBE SHORT OF ITS QUORUM ROLLS BACK THE SAME WAY, as a recheck short
    // of its quorum does: with too few probers heard, no claim is silence
    // rather than a clean bill. A probe outage therefore rolls back rewrites
    // that were sound; the gates between a new text and a standing one fail
    // closed, and the settlement stays out of the cache, so a later run buys
    // the slice again.
    //
    // THE REPORT IS NOT ATTACHED. The lane contest reads `refinementDefects`
    // as damage evidence against the repair candidate, whose text is `T1`
    // again; the counts and the probe stage's own findings go to the findings.
    /**
     The rollback, in scorecard-stable wording.
     */
    const { finding, } = probed;
    l.warn(`slice ${String(outcome.sliceIndex,)}: ${finding}; keeping the text before the rewrite`,);
    return {
      outcome: {
        ...withRefineRounds,
        // The recheck's ballots are kept on the rollback as on every path
        // where it ran, since they read the rewrite a reader will not see.
        recheckReadings: retained.readings,
      },
      findings: [
        ...slice.findings,
        ...refined.findings,
        ...retained.findings,
        ...refinementDefects.findings,
        finding,
      ],
      asked,
      // The rewrite was rolled back, so what ships is the text before it.
      refinedBy: [],
      refinersHeard: refined.heard,
    };
  }

  /**
   Whether the text this slice now returns differs from the archive's, which
   is a different question from whether the rewriter changed anything.
   */
  const changed = refined.refinedText !== incumbentText;
  return {
    outcome: {
      ...withRefineRounds,
      repairedText: refined.refinedText,
      // WHO WROTE THE TEXT THIS RECORD NOW CARRIES, which stopped being the
      // editors alone the moment a refinement shipped. `retainsResolvedIssues`
      // unions both stages for its own recheck, but that union lives in
      // an argument and dies with the call. Left un-stored, this record would
      // credit the editors with words a refiner replaced, and any later reader
      // of it would let that refiner certify its own rewrite at full weight.
      authorship: collectRefinedAuthors({
        editorAuthorship: outcome.authorship,
        refineContributors: refined.contributors,
      },),
      changed,
      // Dropped when the refinement landed back on the archive wording, by the
      // same rule the accuracy stage applies: a resolution credited to text the
      // document does not carry is a repair no reader saw.
      resolvedIssueIds: changed ? outcome.resolvedIssueIds : [],
      // THE DECIDING ROUND'S, carried through unchanged. The `retainsResolvedIssues` recheck is a
      // rollback gate rather than a re-decision: it either keeps the rewrite or
      // discards it whole, and never revises what `resolvedIssueIds` says.
      checkerReadings: outcome.checkerReadings,
      // THE RECHECK'S OWN, kept apart from the deciding round's rather than
      // merged into it. Both rounds rule on the same issue ids, so a reader
      // joining them would present a verdict about the refined text as though
      // it were the one `resolved` rests on.
      recheckReadings: retained.readings,
      // Marks every recorded repair in this slice as pre-refinement text, so a
      // grading sheet can say so instead of presenting an editor replacement as
      // the words that shipped.
      refined: true,
      refinementDefects,
    },
    findings: [
      ...slice.findings,
      ...refined.findings,
      ...retained.findings,
      ...refinementDefects.findings,
    ],
    asked,
    // THE ONLY PATH WHERE A REWRITE SHIPS, so the only one that names anybody.
    refinedBy: refined.contributors,
    refinersHeard: refined.heard,
  };
}

//endregion Refine slice settle
