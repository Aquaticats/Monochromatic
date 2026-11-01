import type {
  AdjudicatedIssue,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type IssueAuthorship,
  MIN_SELECTION_BALLOTS,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import type {
  ClaimAttribution,
  IntroducedDefectReport,
  IssueCheckerReading,
  RepairRegion,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import type { RepairJudgedRound, } from './repair-round-record.ts';

//region Repair contract
// Role roster and chunk outcome types shared by the chunk runner, the
// document driver, and the benchmark; they live beside neither so the
// runner stays within its line budget.

/**
 Model roster for one repair run, by role.

 @example
 ```ts
 const models: RepairModels = {
   criticModelIds: wholeRoster,
   panelModelIds: wholeRoster,
   editorModelIds: twoEditors,
   judgeModelIds: wholeRoster,
   checkerModelIds: rosterMinusEditors,
 };
 ```
 */
export type RepairModels = {
  /**
   Critic fan-out electorate.
   */
  readonly criticModelIds: readonly RosterModelId[];

  /**
   Fixed adjudication panel.
   */
  readonly panelModelIds: readonly RosterModelId[];

  /**
   Editors that each independently propose a repaired candidate;
   judges drawn from {@link RepairModels.judgeModelIds} choose what ships,
   so no one model decides the repaired text.
   */
  readonly editorModelIds: readonly RosterModelId[];

  /**
   Whole roster candidate selection draws judges from, editors included.

   Producers used to be removed per round, which is why this once required a
   model that never edits. Since the ruling of 2026-08-15 they are seated and
   a ballot for their own work is discounted instead, so the only requirement
   left is that a candidate could draw the ballots a winner needs: see
   {@link assertJudgeableEditorRoster}.
   */
  readonly judgeModelIds: readonly RosterModelId[];

  /**
   Extra rule line appended to the editor system prompt, for prompt
   calibration experiments; absent means the baseline prompt.
   */
  readonly editorRuleAddendum?: string;

  /**
   Rewriters proposing naturalness refinements over the repaired text.

   Absent means the lane is off, which is a supported configuration: the
   accuracy pipeline is complete without it.
   */
  readonly refinerModelIds?: readonly RosterModelId[];

  /**
   Resolution checkers proving the repair. Excludes every editor and every
   refiner unless {@link RepairModels.checkerSelfCertificationPermitted} says
   otherwise, since a model certifying text it wrote is not proof.
   */
  readonly checkerModelIds: readonly RosterModelId[];

  /**
   Permits a checker to also write, so the whole roster can check.

   MEASUREMENT SWITCH, NOT A SETTING, and it exists because two arms had to be
   runnable at once. The owner's ruling of 2026-08-23 sent checker width to
   measurement rather than opinion: run the same entries at the disjoint three
   and at all six, compare per-issue resolution, ship the winner and delete
   the loser. Whichever way that lands, this field goes with it.

   SAFE ONLY BECAUSE THE DISCOUNT ALREADY EXISTS. `tallyResolutionChecks`
   picks each checker's weight PER ISSUE through `wroteTextForIssue`, so an
   overlapping checker is halved on issues whose shipped text it helped write
   and keeps full weight everywhere else. Permitting overlap without that
   would let three writers certify themselves at full strength.

   NEVER RELAXES THE OTHER TWO REFUSALS. A repeated checker id stays a fault
   whatever this says, and so does a roster too small to reach a verdict: see
   {@link assertCheckerQuorumReachable}.
   */
  readonly checkerSelfCertificationPermitted?: boolean;
};

/**
 What a caller hands the repair driver before one slice starts: the roster
 to seat that slice with, or nothing, when the roster the driver was given
 still stands.

 A SEATING RATHER THAN A ROSTER OR NOTHING because the repair lane's bench
 is read once at the lanes boundary and a provider can run dry inside the
 lane (class one hundred three, zheermao7, 2026-09-23: the checker bench ran
 on one voice for ten of twelve rounds while the substitute a dry reading
 seats sat idle); the per-chunk hook re-reads the seats under a hold and
 hands them over here.

 @example
 ```ts
 const seating: RepairSliceSeating = { repairModels: reseated, };
 const unchanged: RepairSliceSeating = {};
 ```
 */
export type RepairSliceSeating = {
  /**
   Roster read since the lane started, absent while the given one stands.
   */
  readonly repairModels?: RepairModels;
};

/**
 Thrown when a roster could not decide a round however its judges voted.

 NAMED FOR THE ROLE IT ACTUALLY GUARDS. It was `EditorRosterError` while the
 editor ensemble was the only stage that produced candidates; the translate
 lane and the naturalness lane throw it too, and both were passing their own
 producers into a field called `editorModelIds`. The `role` parameter has
 always said which stage the message names, so the type was the last place
 still claiming editors.

 @example
 ```ts
 throw new ProducerRosterError({ producerModelIds, judgeModelIds, fault, },);
 ```
 */
export class ProducerRosterError extends Error {
  /**
   Declares this message safe to forward: it names roster roles, model ids and counts.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the report from the two colliding rosters.

   @param producerModelIds - models that propose candidates in this stage

   @param judgeModelIds - roster judges are drawn from

   @param role - what the producers do, so the message names the real stage
   rather than always saying editor; defaults to `editor`

   @param fault - what exactly is wrong, since the guard refuses for several
   unrelated reasons and a message covering all of them sends whoever reads it
   looking in the wrong place
   */
  constructor(
    {
      producerModelIds,
      judgeModelIds,
      role = 'editor',
      fault,
    }: {
      readonly producerModelIds: readonly RosterModelId[];
      readonly judgeModelIds: readonly RosterModelId[];
      readonly role?: string;
      readonly fault: string;
    },
  ) {
    super(
      `this ${role} roster cannot select anything: ${fault}. ${role}s [${
        producerModelIds.join(', ',)
      }], judges [${judgeModelIds.join(', ',)}]`,
    );
    this.name = 'ProducerRosterError';
  }
}

/**
 Refuses an editor roster that could not decide a round.

 Names the editor role and defers everything else to
 `assertJudgeableProducerRoster`, which holds the rule and the reasoning.

 @param editorModelIds - editors that propose candidates

 @param judgeModelIds - roster judges are drawn from

 @throws {@link ProducerRosterError} when either side repeats, editors are
 empty, or fewer judges are seated than a winner needs ballots

 @example
 ```ts
 assertJudgeableEditorRoster({ editorModelIds, judgeModelIds, },);
 ```
 */
export function assertJudgeableEditorRoster(
  {
    editorModelIds,
    judgeModelIds,
  }: {
    readonly editorModelIds: readonly RosterModelId[];
    readonly judgeModelIds: readonly RosterModelId[];
  },
): void {
  assertJudgeableProducerRoster({
    producerModelIds: editorModelIds,
    judgeModelIds,
    role: 'editor',
  },);
}

/**
 Refuses a roster that could not decide a round however it voted.

 WHAT THIS NO LONGER REQUIRES, by the user ruling of 2026-08-15: judges
 outside the producer roster. Self-judging is allowed and carries reduced
 weight instead, which is `SELF_VOTE_WEIGHT`, so a model grading its own work
 is a discounted opinion rather than a forbidden one. Refusing here would have
 made that ruling unreachable, since a roster where every model produces has
 no disinterested judge at all.

 WHAT REMAINS STRUCTURAL: a judge contributes at most one ballot, so a
 roster with fewer seats than the ballots a winner needs cannot select
 anything, and every round would decline into the fallback. That reads as a
 working pipeline in logs and in tests, which is why it is refused at stage
 entry rather than left to be inferred from a corpus of unchanged documents.

 NO WEIGHT IS COMPARED. By the owner's rulings of 2026-10-25, "floor of 3 -
 don't set a floor." and "Every weight floor", a candidate wins with two
 answering judges whatever they weigh, and a producer's vote for its own text
 is one answering judge at half weight. This guard used to measure the
 heaviest candidate the roster could write against a summed weight of two,
 which refused two authors judging only each other and one author judged by
 itself and one other model; either can draw two ballots now.

 Repeats are refused on both sides: a repeated id is one model counted twice,
 inflating an ensemble without adding an independent voice, and a repeated
 judge would count as two answering judges by itself.

 @param producerModelIds - models that generate candidates

 @param judgeModelIds - roster judges are drawn from

 @throws {@link ProducerRosterError} when either side repeats, producers are
 empty, or fewer judges are seated than a winner needs ballots

 @example
 ```ts
 assertJudgeableProducerRoster({ producerModelIds, judgeModelIds, role: 'editor', },);
 ```
 */
export function assertJudgeableProducerRoster(
  {
    producerModelIds,
    judgeModelIds,
    role,
  }: {
    readonly producerModelIds: readonly RosterModelId[];
    readonly judgeModelIds: readonly RosterModelId[];
    readonly role: string;
  },
): void {
  /**
   Producers keyed for membership tests, also revealing repeats by size.
   */
  const producers = new Set(producerModelIds,);
  if (producers.size === 0)
    throw new ProducerRosterError({
      producerModelIds,
      judgeModelIds,
      role,
      fault: `no ${role} was seated`,
    },);
  if (producers.size !== producerModelIds.length)
    throw new ProducerRosterError({
      producerModelIds,
      judgeModelIds,
      role,
      fault: `a ${role} is listed more than once, which is one voice pretending to be two`,
    },);

  /**
   Judges keyed the same way, since a repeated judge is one opinion counted
   twice and would count as two answering judges on its own.
   */
  const judges = new Set(judgeModelIds,);
  if (judges.size !== judgeModelIds.length)
    throw new ProducerRosterError({
      producerModelIds,
      judgeModelIds,
      role,
      fault: 'a judge is listed more than once, which would let one model count as two answering judges',
    },);
  if (judges.size >= MIN_SELECTION_BALLOTS)
    return;
  throw new ProducerRosterError({
    producerModelIds,
    judgeModelIds,
    role,
    fault: `judges seated: ${String(judges.size,)}; ballots a winner needs: ${
      String(MIN_SELECTION_BALLOTS,)
    }; so nothing it proposes could ever be selected`,
  },);
}

/**
 Thrown when a checker would certify text it helped write.

 @example
 ```ts
 throw new CheckerIndependenceError({ overlapping, },);
 ```
 */
export class CheckerIndependenceError extends Error {
  /**
   Declares this message safe to forward: it names model ids and never anything they wrote.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the report from whichever fault was found.

   @param overlapping - models that both write and check; empty when the
   roster fault is a repeat rather than an overlap

   @param duplicated - checker ids listed more than once
   */
  constructor(
    {
      overlapping = [],
      duplicated = [],
    }: {
      readonly overlapping?: readonly RosterModelId[];
      readonly duplicated?: readonly RosterModelId[];
    },
  ) {
    super(
      overlapping.length === 0
        ? `these checker ids are listed more than once: [${duplicated.join(', ',)}]; a repeated id `
          + `is one model counted twice, which meets quorum on fewer independent voices than the `
          + `roster size promises`
        : `these models would check text they wrote themselves: [${overlapping.join(', ',)}]; `
          + `checkerModelIds must exclude every editor and every refiner`,
    );
    this.name = 'CheckerIndependenceError';
  }
}

/**
 Refuses a roster where an editor also checks.

 The checker stage is the proof that an accepted issue is actually gone. A
 model grading its own rewrite is not proof, and unlike the judge roster this
 one is not filtered at runtime, so nothing else would catch the overlap.

 THE REPEAT REFUSAL IS UNCONDITIONAL and the overlap refusal is not, which is
 the asymmetry to read carefully. Overlap is a question about evidence quality
 that `tallyResolutionChecks` can answer by discounting, so the owner sent it
 to measurement and `selfCertificationPermitted` is how the second arm runs. A
 repeated id is not that kind of question: it makes the quorum count disagree
 with the ballot count, so no weighting can rescue it.

 @param editorModelIds - editors that propose candidates

 @param refinerModelIds - naturalness rewriters; absent means the lane is off

 @param checkerModelIds - checkers proving the shipped repair

 @param selfCertificationPermitted - allows overlap, leaving self-votes to be
 discounted per issue rather than excluded; see
 {@link RepairModels.checkerSelfCertificationPermitted}

 @throws {@link CheckerIndependenceError} when a checker id repeats, or when
 a model holds both roles and overlap was not permitted

 @example
 ```ts
 assertCheckerIndependence({ editorModelIds, checkerModelIds, },);
 ```
 */
export function assertCheckerIndependence(
  {
    editorModelIds,
    refinerModelIds = [],
    checkerModelIds,
    selfCertificationPermitted = false,
  }: {
    readonly editorModelIds: readonly RosterModelId[];
    readonly refinerModelIds?: readonly RosterModelId[];
    readonly checkerModelIds: readonly RosterModelId[];
    readonly selfCertificationPermitted?: boolean;
  },
): void {
  /**
   Checker ids keyed for membership tests, also revealing repeats by size.

   Refused for the same reason `assertJudgeableProducerRoster` refuses
   repeated producers, and the consequence here is quieter: `gatherStageVoices`
   counts a repeated id's replies separately toward quorum, while
   `runCheckerStage` keys its ballots by model id and so collapses them into
   one. A roster of three with a repeat could therefore report quorum on two
   voices that are one model.
   */
  const distinctCheckers = new Set(checkerModelIds,);
  if (distinctCheckers.size !== checkerModelIds.length) {
    throw new CheckerIndependenceError({
      duplicated: [...distinctCheckers,].filter(function repeats(modelId,) {
        return checkerModelIds.filter(function isSame(candidate,) {
          return candidate === modelId;
        },)
          .length
          > 1;
      },),
    },);
  }

  if (selfCertificationPermitted)
    return;

  /**
   Every model that writes shipped text, keyed for membership tests.

   Refiners belong here as much as editors do: the recheck that follows a
   refinement asks the checkers whether the accepted issues survived it, and
   a refiner among them would be answering for its own rewrite.
   */
  const writers = new Set([
    ...editorModelIds,
    ...refinerModelIds,
  ],);

  /**
   Models holding both roles.
   */
  const overlapping = checkerModelIds.filter(function alsoWrites(modelId,) {
    return writers.has(modelId,);
  },);
  if (overlapping.length === 0)
    return;
  throw new CheckerIndependenceError({ overlapping, },);
}

/**
 Fewest checkers a verdict can be read from: two, since one judge alone never
 decides.

 THE OWNER'S ANSWER OF 2026-10-25, "Allow two checkers", moved this from
 three. `tallyResolutionChecks` resolves an issue on
 `fixed > (notFixed + worse)` among at least two answering checkers, so a
 roster of two decides a unanimous two-nil and a split one against one
 resolves nothing: it returns no verdict, never a wrong one. Three was the
 smallest roster where a split still read as a majority; the owner chose to
 run on two and take the split as undecided. One checker is still refused,
 because a single answer is no ballot count.

 SCALE-FREE UNDER THE DISCOUNT, which is why one number covers both arms.
 Halving every weight on an issue halves both sides of that comparison, so a
 roster whose members all wrote the text resolves the same issues.
 */
const MINIMUM_CHECKER_COUNT = 2;

/**
 Thrown when a checker roster is too small to return a verdict.

 @example
 ```ts
 throw new CheckerQuorumError({ checkerModelIds, },);
 ```
 */
export class CheckerQuorumError extends Error {
  /**
   Declares this message safe to forward: it names model ids and counts them.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the report from the roster that cannot decide.

   @param checkerModelIds - checkers configured for this run
   */
  constructor({ checkerModelIds, }: { readonly checkerModelIds: readonly RosterModelId[]; },) {
    super(
      `this run configures ${String(checkerModelIds.length,)} checker(s), [${
        checkerModelIds.join(', ',)
      }], against a floor of ${String(MINIMUM_CHECKER_COUNT,)}; below that no verdict can be read, `
        + `because resolution needs two answering checkers and one judge alone never decides, so `
        + `checking would run and decide nothing`,
    );
    this.name = 'CheckerQuorumError';
  }
}

/**
 Refuses a checker roster too small to return a verdict: fewer than two
 checkers.

 WHY THIS EXISTS SEPARATELY FROM THE EMPTY-ROLE FLOOR. `assertRostersConfigured`
 refuses a role with nobody in it and nothing more, so a roster of one passed
 every guard while being unable to produce a verdict at all. Narrowing
 checking is exactly what widening the producing roles does to a fixed-size
 roster, so the guard belongs next to the one that permits it. The floor was
 three until the owner's answer of 2026-10-25, "Allow two checkers".

 @param checkerModelIds - checkers proving the shipped repair

 @throws {@link CheckerQuorumError} when fewer checkers are configured than a
 verdict needs

 @example
 ```ts
 assertCheckerQuorumReachable({ checkerModelIds, },);
 ```
 */
export function assertCheckerQuorumReachable(
  { checkerModelIds, }: { readonly checkerModelIds: readonly RosterModelId[]; },
): void {
  if (checkerModelIds.length >= MINIMUM_CHECKER_COUNT)
    return;
  throw new CheckerQuorumError({ checkerModelIds, },);
}

/**
 Everything one chunk's repair decided.

 @example
 ```ts
 const outcome = await repairChunk({ ... },);
 if (outcome.changed) splice(outcome.repairedText,);
 ```
 */
export type ChunkRepairOutcome = {
  /**
   Chunk position within the document.
   */
  readonly sliceIndex: number;

  /**
   Winning chunk text; equals the input when unchanged won.
   */
  readonly repairedText: string;

  /**
   Whether the repaired candidate demonstrably beat unchanged.
   */
  readonly changed: boolean;

  /**
   Adjudicated issues of this chunk.
   */
  readonly issues: readonly AdjudicatedIssue[];

  /**
   Which critics raised each surviving claim, keyed by deterministic claim id.

   Chunk-level rather than attached to each issue, deliberately. Every
   `AdjudicatedIssue` already carries its member claims with their ids, so a
   consumer joins on the ids an issue actually holds. That sidesteps the
   transitive-cluster hazard by construction: clustering can chain A to B to C
   without A and C overlapping, so unioning a cluster's proposers onto each
   issue it produced would credit critics for claims their issue does not
   represent. Nothing is unioned here, so nothing can be miscredited.

   Calibration only. Adjudication never sees it, and it must never enter a
   judging prompt: a real defect can arrive with exactly one proposer.
   */
  readonly claimAttributions: readonly ClaimAttribution[];

  /**
   WHICH critics answered on this chunk, sorted by model id.

   The denominator {@link ChunkRepairOutcome.claimAttributions} cannot supply.
   A critic heard that raised nothing and a critic never heard both leave no
   attribution entry, so hits are countable without this and rates are not.
   Its presence also marks the chunk as attributable at all, which is what
   separates a critic that stayed silent from an outcome written before any of
   this existed.
   */
  readonly heardCriticIds: readonly RosterModelId[];

  /**
   Accepted issues the checkers confirmed fixed in the winning text;
   empty when unchanged won.
   */
  readonly resolvedIssueIds: readonly string[];

  /**
   What the checkers said about each issue, keyed by issue id.

   BESIDE {@link ChunkRepairOutcome.resolvedIssueIds} RATHER THAN INSTEAD OF
   IT, because that list is a set of majority verdicts and a majority is not
   evidence of itself. Without the ballots a settled run cannot say whether a
   resolution was unanimous or one vote wide, cannot name the dissenter, and
   cannot be re-read at a roster width this run did not use.

   EMPTY WHERE NO CHECKER RAN, which is a real state rather than a missing
   one: a chunk with no accepted issue, or one the lane declined, buys no
   checker round at all. Required rather than optional so every construction
   site has to say which it is.
   */
  readonly checkerReadings: Readonly<Record<string, IssueCheckerReading>>;

  /**
   What the checkers said when asked AGAIN about the refined text.

   A SECOND ROUND ABOUT THE SAME ISSUE IDS, and the reason it is a separate
   field rather than an update to {@link ChunkRepairOutcome.checkerReadings}.
   The naturalness lane rewrites text the editors already repaired, then asks
   the checkers whether every confirmed issue survived the rewrite. That round
   is a rollback gate: it keeps the rewrite or discards the whole slice, and
   never revises {@link ChunkRepairOutcome.resolvedIssueIds}. Merging the two
   would present a verdict about the refined text as the one `resolved` rests
   on, and would hide a checker that changed its mind between them.

   EMPTY ON EVERY SLICE THE LANE DID NOT REWRITE, which is most of them: a
   slice with no confirmed issue buys no recheck, and neither does one whose
   refinement changed nothing.
   */
  readonly recheckReadings: Readonly<Record<string, IssueCheckerReading>>;

  /**
   Every accepted issue the checkers confirmed fixed IN THE PATCHED CANDIDATE,
   whether or not an applied operation served it and whether or not that
   candidate won.

   Kept because {@link ChunkRepairOutcome.resolvedIssueIds} discards two
   different things and both are worth auditing: verdicts on issues no
   operation served, which must not earn selection credit but are still what
   the checkers said, and every verdict at all when the unchanged text won, so
   that a rejected candidate leaves no trace of how it was judged. This field
   decides nothing; it exists so a later analysis can ask why.
   */
  readonly candidateResolvedIssueIds: readonly string[];

  /**
   Regions the accuracy stage replaced, with the accepted issues each served.

   Recorded whether or not the patched candidate won, because "no repair was
   attempted" and "a repair was attempted and lost to unchanged" are different
   facts and only the second one indicts the stage.
   */
  readonly repairRegions: readonly RepairRegion[];

  /**
   Shadow-mode audit of defects the edit itself introduced, absent on chunks
   where no region was replaced and so nothing was probed.

   Nothing reads this to decide what ships. It exists because
   `regressedKnownIssues` can only see issues a critic already raised, so a
   patch that fixes its target and mangles the clause beside it currently
   scores as clean and the pipeline had no way to notice. Whether these
   verdicts deserve to gate is a question this round's human repair grades
   answer, not one the probe's existence settles.
   */
  readonly introducedDefects?: IntroducedDefectReport;

  /**
   Whether the patched candidate beat unchanged in the ACCURACY stage's own
   selection.

   Separate from {@link ChunkRepairOutcome.changed} because the two diverge
   downstream: the naturalness lane sets `changed` on a refinement-only
   rewrite, so after that phase `changed` no longer answers whether any
   accuracy repair was selected. Reading `changed` for that would report a
   rejected repair as a shipped one whenever refinement happened to touch the
   same slice.
   */
  readonly accuracyPatchSelected: boolean;

  /**
   Whether the naturalness lane rewrote this slice after the accuracy pass
   settled, which makes every {@link RepairRegion.editorAfter} in
   {@link ChunkRepairOutcome.repairRegions} pre-refinement text rather than
   what shipped. A grading sheet has to disclose that, and disclosing it only
   where it happened keeps the caveat meaningful.
   */
  readonly refined: boolean;

  /**
   Every judged round this slice went through, editor and refinement alike,
   with the ballots that decided each one.

   ONE ARRAY RATHER THAN ONE FIELD PER STAGE, because the interesting question
   is asked across stages: which panel preferred which wording, and the stage
   is recorded on the round itself. Rounds appear in the order they ran, so a
   refinement that undid an accuracy repair reads as the later entry.

   Empty on a slice nothing judged, which is an ordinary outcome: an envelope
   only one editor proposed for is adopted without a vote, and a chunk every
   editor agreed on ships unjudged.
   */
  readonly rounds: readonly RepairJudgedRound[];

  /**
   Who wrote {@link ChunkRepairOutcome.repairedText}, so a later stage can
   discount a checker judging text it wrote itself.

   STORED RATHER THAN DERIVED FROM {@link ChunkRepairOutcome.rounds}. The
   naturalness lane rechecks this text and may run from cache, long after the
   stage that knew the answer returned. Rounds cannot supply it: one exit
   ships a real editor's repair after the judges declined to rank anything,
   and a declined round records ballots without a winner.
   */
  readonly authorship: IssueAuthorship;

  /**
   Declared names a winning patch would have dropped, empty on every slice
   that dropped none.

   Required on every outcome, so a reader never has to tell "dropped
   nothing" from "field not written". Its translate-lane counterpart on
   `TranslateSliceRecord` is carried only by a declared-name refusal, which
   the record's union says and the translate cache reader checks.
   */
  readonly droppedDeclaredNames: readonly string[];

  /**
   Shadow-mode audit of damage the NATURALNESS REWRITE caused, present only on
   a slice the lane actually rewrote.

   Separate from {@link ChunkRepairOutcome.introducedDefects} because the two
   audit different edits against different baselines, and merging them would
   produce a rate about neither: the accuracy probe compares the original
   translation with the repaired one, this compares the repaired one with the
   refined one.

   It exists because the lane was the one stage that could change shipped text
   with nothing asking whether it broke anything. `retainsResolvedIssues`
   guards the opposite direction, that a rewrite did not UNDO a confirmed
   repair, and a rewrite can leave every confirmed repair standing while
   damaging the wording around them.
   */
  readonly refinementDefects?: IntroducedDefectReport;

  /**
   Critics reporting critical non-translation at wire level.
   */
  readonly nonTranslationVotes: number;

  /**
   Whether deterministic evidence (enough validated content-critique
   claims anchored into target text) contradicted the votes;
   callers must never block on contradicted votes.
   */
  readonly nonTranslationContradicted: boolean;

  /**
   Whether votes met the block threshold uncontradicted. Evidence only since
   question 3, answer B (2026-08-16): the slice was repaired like any other,
   and the caller weighs standing slices by character share for the
   document-level dominance finding, which reports and blocks nothing.
   */
  readonly nonTranslationStanding: boolean;

  /**
   Critics heard, for the caller's degradation accounting.
   */
  readonly heardCritics: number;

  /**
   Stage findings in scorecard-stable wording.
   */
  readonly findings: readonly string[];

  /**
   The stages that heard fewer voices than their quorum and whose shortfall a finding could not carry, as labels such
   as `panel 0/6 (cluster/abc)`; empty when no such stage was silent.

   DATA, NOT A FINDING, so that a located or reworded finding cannot hide it (XingZ6018 slice 85: five panel packets
   heard nobody, their findings began `panel-packet (cluster): `, and the settlement was cached as examined). Required
   on every outcome, so a reader never has to tell "no silent stage" from "field not written": a record without the
   field was written by a build that could not say, and is not resumed (`slice-cache-store.ts`). Read through
   `silentStagesOfRecord`, which adds the stages that left a plain quorum-unmet finding.
   */
  readonly silentStages: readonly string[];
};

//endregion Repair contract
