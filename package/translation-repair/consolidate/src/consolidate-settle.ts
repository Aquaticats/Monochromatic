import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { SliceSyntax, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { ConsolidateGateOutcome, } from './consolidate-gate-stage.ts';
import { gateAndShip, } from './consolidate-settle-gate.ts';
import {
  floorConsolidateSlate,
  type ProposalValidity,
  type SlateFloor,
} from './consolidate-validity-floor.ts';
import {
  type ProposalVerdict,
  settlementContextOf,
  verdictsOf,
} from './consolidate-settle-context.ts';
import {
  INELIGIBLE_STANDING_WITHHELD_FINDING,
  slateIncumbentFor,
} from './consolidate-ineligible-standing.ts';
import {
  settleWithoutSlate,
  shipWinnerBeforeSlate,
} from './consolidate-settle-unseated.ts';
import { judgeConsolidationSlate, } from './consolidate-settle-judge.ts';
import { settleKeptStanding, } from './consolidate-settle-kept.ts';
import { applyFinalPolish, } from './consolidation-polish-apply.ts';
import type {
  ConsolidationPolish,
  ConsolidationPolishConfig,
} from './consolidation-polish.ts';
import { wrapConsolidationProposals, } from './consolidate-wrap.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  buildTranslateCandidates,
  type DisputedWording,
  type LaneText,
  type TranslateReportWire,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type {
  TranslateDecision,
  TranslateStageResult,
} from '@monochromatic-dev/translation-repair-translate-stage/ts';
import type {
  HeardVoice,
  StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';

//region Consolidate settle
// RUNS THE CONSOLIDATION'S DECIDING HALF, in the order it was specified: floor
// the slate, judge it against the standing text, gate what wins, wrap what
// ships.
//
// The producing half is deliberately NOT here. Producing was split from
// judging so one slate can be judged more than once, and every measurement of
// this stage rests on that split: an arm that rebought its slate would compare
// different candidates. This takes voices already gathered and repaired.
//
// EACH STEP EXISTS BECAUSE A RECORDED RUN NEEDED IT, not because the shape
// looked incomplete:
//
//    -   The floor, because `Zha_Ke#1` finished its repair round with five
//        candidates and zero valid ones, in both runs of the band pair, and
//        shipped a consolidation at both.
//    -   The gate, because the selector's question is a preference while the
//        contest's question measured better on the lane pair, so a preferred
//        consolidation still has to beat the standing text on what the original
//        supports.
//    -   The wrap, because both lanes wrap at their assembly step and this
//        stage had none.
//
// `doc/planning/the-third-rendering.md` records all three.

/**
 How a slice left this stage, as one name rather than as a shape the reader
 has to reconstruct from several optional fields.

 `incumbent-only` IS ITS OWN STATE ON PURPOSE. The calibration bed recorded
 the floor's case as `judged`, which is exactly the misreading the floor
 exists to prevent: nothing was judged, because there was nothing valid to
 judge. A reader counting decisions must be able to tell that apart from a
 panel that considered proposals and preferred the standing text.

 @example
 ```ts
 const terminal: ConsolidationTerminal = 'incumbent-only';
 ```
 */
export type ConsolidationTerminal =
  | 'incumbent-only'
    | 'no-standing-text'
    | 'slate-endorsed-standing'
    | 'slate-unjudged-standing'
    | 'slate-declined-standing'
    | 'gate-kept-standing'
    | 'wrap-erased-difference'
    | 'consolidated';

/**
 Why the archive keeps a slice (owner, 2026-09-27, "Keep archive, ship"), as the artifact and the publish defect say it.

 THREE REASONS BECAUSE THREE THINGS HAPPENED, and one sentence for them was false for two: `no-wording-passed` is the
 only one where no wording passed the deterministic rule; `final-round-unseated` is a final round that could not seat
 two judges with no wording two judges chose to ship (owner, 2026-10-29); `slate-declined` is a slate that seated two
 or more judges and declined every wording that passed.

 @example
 ```ts
 const reason: ArchiveKeptReason = 'final-round-unseated';
 ```
 */
export type ArchiveKeptReason =
  | 'no-wording-passed'
    | 'final-round-unseated'
    | 'slate-declined';

/**
 Why the translate lane's winner ships, as the artifact and the publish defect say it.

 `refused-standing` is the standing the deterministic rule refused, replaced where the final round could not seat two
 judges (owner, 2026-10-29, "Ship the two-ballot text"). `present-tense-standing` is an eligible standing the contest
 did not endorse, replaced where the final round could not seat two judges and the tense detector flags the standing and
 not the winner (owner, 2026-10-29, "Swap only where flagged").

 @example
 ```ts
 const reason: TranslateWinnerReason = 'present-tense-standing';
 ```
 */
export type TranslateWinnerReason =
  | 'refused-standing'
    | 'present-tense-standing';

/**
 Which slate terminal a judged round ends in, per decision.

 THE THREE ANSWER DIFFERENT QUESTIONS ABOUT THE ROSTER, which is why one
 name for them could not be counted. A roster that endorses the archive is
 working. A roster that cannot agree is not. A slate carrying one candidate
 says nothing about the roster at all, because no judge was asked.

 `no-candidate` JOINS THE UNJUDGED rather than the declined, because nothing
 reached a judge on that path either, and `no-voice-heard` joins it for the
 same reason one step earlier: no judge was asked because there was nothing
 to ask about. They are separate decisions and the same terminal, because
 what they tell a reader differs while what they tell the consolidation does
 not. `no-candidate-backed` joins the declined, because candidates existed
 and the judges backed none of them. `unfloored` joins the unjudged, because
 nobody was asked on a slice the floor could compare nothing on; no slate of
 this stage raises it, since only the translate stage (`runTranslateStage`)
 settles a slice before any round (ledger B43).

 A RECORD RATHER THAN A CHAIN, so a decision added to the union fails to
 typecheck here instead of falling quietly into whichever branch is last.
 */
const SLATE_TERMINALS: Record<TranslateDecision, ConsolidationTerminal> = {
  'judged': 'slate-endorsed-standing',
  'sole-candidate': 'slate-unjudged-standing',
  'no-candidate': 'slate-unjudged-standing',
  'no-voice-heard': 'slate-unjudged-standing',
  'unfloored': 'slate-unjudged-standing',
  'declined-indecision': 'slate-declined-standing',
  'declined-rejection': 'slate-declined-standing',
  'no-candidate-backed': 'slate-declined-standing',
};

export type { ProposalVerdict, } from './consolidate-settle-context.ts';

/**
 The slice this stage is deciding about, in the archive's terms.

 @example
 ```ts
 const subject: ConsolidationSubject = { sourceText: '猫', incumbentText: 'A cat.', };
 ```
 */
export type ConsolidationSubject = {
  /**
   Chinese this passage renders, which is the standard both deciders judge
   against.
   */
  readonly sourceText: string;

  /**
   Syntax role requiring dedicated validation and judging.
   */
  readonly syntax?: SliceSyntax;

  /**
   Wording the ARCHIVE carries, which is not the standing text once a lane
   contest has replaced it. The gate is shown both, because a consolidation
   that quietly restores an archive invention is a different fault from one
   that departs from the text now in place.
   */
  readonly incumbentText: string;

  /**
   Front matter identity, absent when the pair declares none.
   */
  readonly identityContext?: string;

  /**
   Whether the original's entry records a death, absent when it records none;
   the slate and the gate show the judges the tense candidates of their texts.
   */
  readonly lifeEnded?: boolean;

  /**
   Names the translation declares for its subject, its display name and its
   aliases, which the tense candidates read as named subjects; absent when it
   declares none.
   */
  readonly subjectNames?: readonly string[];

  /**
   What the pages the original cites say, absent when it cites none.
   */
  readonly referenceContext?: string;

  /**
   Why the archive rendering shown is the repair lane's stand-in, absent on
   an undisputed slice (class one hundred eight, 2026-09-24).
   */
  readonly archiveDisputeNote?: string;

  /**
   Every claim the repair panel accepted for the slice, whether it disputes
   the archive or not, left out of the dispute note when that note already
   carries it; absent where the panel accepted nothing. Read by the final
   polish only: its gate (CuspariaKLSY21 slice 0, 2026-10-21), its writers and
   the judges of their rewrites (ledger B805).
   */
  readonly acceptedRepairClaimsNote?: string;

  /**
   Wordings a disputed slice refuses as proposal, absent on an undisputed
   slice (owner, 2026-09-27, "No eligible standing").
   */
  readonly disputedWordings?: readonly DisputedWording[];

  /**
   What the pictures near this slice were read to say.

   DECLARED HERE FROM 2026-08-22, having been PASSED here earlier. The
   driver builds one subject and hands it to both halves, so the field was
   already arriving; only this type and the judging call were unaware of it,
   which is precisely how the judges came to weigh proposals written against
   evidence they could not see.
   */
  readonly pictureContext?: string;

  /**
   Original of the passages either side, absent for a lone slice.
   */
  readonly neighbouringSourceText?: string;

  /**
   Archive English of those same passages, which is the half that shows a
   relocation.
   */
  readonly neighbouringIncumbentText?: string;
};

/**
 Everything this stage settled, with every round it did not run named absent.

 @example
 ```ts
 const settled: ConsolidationSettlement = { terminal: 'incumbent-only', text: 'A cat.', floor, rewrapped: false, demoted: false, };
 ```
 */
export type ConsolidationSettlement = {
  /**
   How the slice left, which is the field a census should count.
   */
  readonly terminal: ConsolidationTerminal;

  /**
   Wording that ships.
   */
  readonly text: string;

  /**
   What the validity floor made of the slate, kept even when it passed.

   NAMES REFUSALS ONLY WHEN NOTHING SURVIVED. A slate with one survivor and
   five refusals reports the survivor and says nothing about the five, which
   is right for the floor's own question and wrong for a record of the run.
   `verdicts` is what carries the rest.
   */
  readonly floor: SlateFloor;

  /**
   Every voice's verdict, survivors and refusals alike, without their text.
   */
  readonly verdicts: readonly ProposalVerdict[];

  /**
   What the slate judges settled, absent when no slate reached them.
   */
  readonly decided?: TranslateStageResult;

  /**
   What the gate settled, absent when no fresh consolidation won the slate.
   */
  readonly gate?: ConsolidateGateOutcome;

  /**
   Whether the wrap altered what the producer emitted.
   */
  readonly rewrapped: boolean;

  /**
   Whether wrapping left nothing between the consolidation and what stands.
   */
  readonly demoted: boolean;

  /**
   Everything this slice's rounds recorded, as ONE authoritative list.

   READ THIS RATHER THAN DIGGING INTO `decided` OR `gate`. Before it existed,
   the produce half's findings reached a reader only where a judged round or
   a gate round happened to run, so the two terminals that end before either
   one, `no-standing-text` and `incumbent-only`, dropped them on every run.
   Those are exactly the terminals whose findings explain themselves: voice
   loss and transport failure are why a slate had nothing valid on it.

   EACH FINDING APPEARS ONCE. The produce half's findings are threaded into
   the judged round, so a path carrying `decided` already carries them and
   must not add them again.

   PER-PROPOSAL VERDICTS STAY OUT, because `verdicts` already records them
   structurally, and repeating them here would count one refusal twice.
   */
  readonly findings: readonly string[];

  /**
   Final body naturalness decision, absent on exits before final candidate.
   */
  readonly polish?: ConsolidationPolish;

  /**
   Present where no wording for this slice passed the deterministic rule, so
   the archive keeps it (owner, 2026-09-27, "Keep archive, ship"); absent on
   every other settlement.

   `text` IS THEN THE REFUSED STANDING, not what ships: the artifact names
   the archive, which its comparison row carries. Never cached
   (`consolidationWorthResuming`), since a copy resumed without this mark
   would ship the refused text.
   */
  readonly archiveKept?: true;

  /**
   Why the archive keeps the slice, present exactly where {@link ConsolidationSettlement.archiveKept} is: no wording
   passed the rule, the final round could not seat two judges and no two-ballot wording existed, or the slate seated
   two or more judges and none of the wordings drew two ballots. Never cached, as the mark is not.
   */
  readonly archiveReason?: ArchiveKeptReason;

  /**
   Present where the final round could not seat two judges over a standing the rule refused and the translate lane's
   winner shipped instead, since two judges chose it there and it passes the rule (owner, 2026-10-29, "Ship the
   two-ballot text"); `text` is then that winner. Absent on every other settlement.

   NEVER CACHED (`consolidationWorthResuming`): the unseated round is the hour, not the slice, and a copy resumed on a
   healthier bench would ship the winner unasked.
   */
  readonly translateWinnerShipped?: true;

  /**
   Why the translate lane's winner shipped, present exactly where {@link ConsolidationSettlement.translateWinnerShipped}
   is: over a standing the rule refused, or over an unendorsed standing the tense detector flagged (owner, 2026-10-29,
   "Swap only where flagged"). Never cached, as the mark is not.
   */
  readonly translateWinnerReason?: TranslateWinnerReason;

  /**
   Present where the slate bench read fewer reachable judges than the ballots
   a winner needs, so no slate was bought and the standing was kept without a
   call; absent on every other settlement.

   NEVER CACHED (`consolidationWorthResuming`): the shortfall is the hour, not
   the slice, and a copy resumed on a healthier bench would keep the standing
   unasked.
   */
  readonly benchBelowFloor?: true;
};

/**
 Settles one slice's consolidation, from a slate already produced.

 @param client - provider client the rounds borrow

 @param roster - voices that produced the slate, which the candidates are
 attributed to

 @param judgeModelIds - voices seated for the slate's judging round and the
 gate; the producers' roster when not given, and narrower since 2026-09-02
 when the owner unseated GLM-5.3-Flash from every judge seat

 @param subject - slice in the archive's terms

 @param voices - consolidations gathered and repaired, in any order

 @param validity - each voice's structural verdict, keyed by the same model id

 @param producedFindings - what gathering and repairing recorded, which every
 exit reports ahead of its own by `ProducedSlate`'s contract

 @param standingText - wording in place when this stage began, which is what a
 consolidation has to beat and what ships whenever it does not

 @param sliceIndex - prepared position used by naturalness stage records

 @param polishConfig - final body polish roles and document guard facts

 @param standingMayShip - whether unchanged baseline has prior endorsement

 @param standingEligible - whether the standing text passed the deterministic
 publication gate; when it did not, it is withheld from the slate and a
 settlement that would keep it throws instead of shipping it

 @param standingRefusal - why the deterministic gate refused the standing,
 forwarded to the gate judges (class fifty-six, 2026-09-18)

 @param laneTexts - what the repair and translate lanes would ship, offered
 on the slate beside the proposals when the standing is neither endorsed nor
 eligible (`consolidate-lane-offer.ts`, class forty, 2026-09-17)

 @param runoffOverStanding - whether a tied slate over an ELIGIBLE standing is
 run off rather than left to its single round, which the driver sets when a
 strict majority of the usable contest ballots called the archive flawed
 (`archiveFlawedByMajority` in `consolidate-archive-flawed.ts`, class one
 hundred six, 2026-09-24, widened to a majority by the owner on 2026-10-11,
 "Majority flawed yes"); the standing still ships where the run-off ends
 undecided. The same flag forfeits that standing past an undecided gate, so
 the gate ships the slate's choice (owner answers 2026-09-27, "Slate's choice",
 and 2026-10-11, "Majority, like the run-off"). The challenge round no longer
 reads the flag: a proposal that at least two ballots named beats the standing
 in any round now that the weight minimum is gone (the owner's rulings of
 2026-10-25), which is what the flag used to buy for one entry's slice 45
 (2026-10-19)

 @param translateWinnerBallots - ballots that named the translate lane's text in the lane's own round, zero where the
 caller did not count; with two or more, that text on `laneTexts` ships over a refused standing where the final round
 cannot seat two judges (owner, 2026-10-29, "Ship the two-ballot text")

 @param slateReachable - slate judges a wet provider would serve, absent where the caller did not count; fewer than the
 ballots a winner needs settle that shipment without buying a slate

 @param signal - cancellation for the whole settlement

 @param perCallTimeoutMs - bound on any single exchange

 @param l - stage logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns What ships, and every round that decided it

 @throws BlankSelectionError - when the slate judges settle on nothing at all

 @example
 ```ts
 const settled = await settleConsolidation({ client, roster, subject, voices, validity, producedFindings, standingText, standingMayShip, standingEligible, lineStructured, signal, perCallTimeoutMs, l, stageSettings, },);
 ```
 */
export async function settleConsolidation(
  {
    client,
    roster,
    judgeModelIds = roster,
    subject,
    voices,
    validity,
    producedFindings,
    standingText,
    lineStructured,
    sliceIndex = 0,
    polishConfig,
    standingMayShip,
    standingEligible,
    standingRefusal,
    laneTexts = [],
    runoffOverStanding = false,
    translateWinnerBallots = 0,
    slateReachable,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly roster: readonly RosterModelId[];
    readonly judgeModelIds?: readonly RosterModelId[];
    readonly subject: ConsolidationSubject;
    readonly voices: readonly HeardVoice<TranslateReportWire>[];
    readonly validity: readonly ProposalValidity[];
    readonly producedFindings: readonly string[];
    readonly standingText: string;
    readonly lineStructured: boolean;
    readonly sliceIndex?: number;
    readonly polishConfig?: ConsolidationPolishConfig;
    readonly standingMayShip: boolean;
    readonly standingEligible: boolean;
    readonly standingRefusal?: string;
    readonly laneTexts?: readonly LaneText[];
    readonly runoffOverStanding?: boolean;
    readonly translateWinnerBallots?: number;
    readonly slateReachable?: number;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<ConsolidationSettlement> {
  /**
   Logger tagged with this stage.
   */
  const sl = tagged({
    tag: settleConsolidation.name,
    l,
  },);

  /**
   Identity and evidence as the two rounds take them
   (`consolidate-settle-context.ts`).
   */
  const {
    identity,
    evidence,
  } = settlementContextOf({ subject, },);

  /**
   What the structural guard leaves of the slate.
   */
  const floor = floorConsolidateSlate({
    validity,
    l: sl,
  },);

  /**
   Every voice's verdict without its text, which is what a run may record.
   */
  const verdicts = verdictsOf({ validity, },);

  // NO STANDING TEXT IS CHECKED FIRST, and the order is the whole point.
  // A slice with nothing to consolidate against and a slate of refusals is
  // both things at once, and a census counting terminal states has to be told
  // which one it is. The absent standing text is the older and larger fact:
  // the floor would have refused a slate this slice was never going to use.
  //
  // `buildTranslateCandidates` offers a blank incumbent as no candidate at all,
  // which is right for a passage nobody translated and wrong for a contest that
  // declined, so this records the consolidations and stops rather than putting
  // "ship nothing" on the ballot.
  //
  // UNLESS A LANE TEXT IS ON OFFER (class eighty-seven, XingZ623 slice 89,
  // 2026-09-22). A contest that declined both lanes at a passage the archive
  // never carried leaves the empty string standing, and this exit ran ahead of
  // the class forty offer: the translate lane's rendering of the bare
  // `<TextRing/>` tag passed the rule, was never put to a judge, the slice
  // shipped nothing, and the final naturalness check stopped the entry at
  // publish. A lane text the rule admits is the slate the owner's rule of
  // 2026-09-04 asks for (the best valid proposal, else fail at once), so the
  // empty standing is withheld exactly as an ineligible one is and the lane
  // texts go to the judges.
  //
  // AND WITHOUT ONE NO WORDING PASSES THE RULE, so the archive keeps the slice
  // (owner, 2026-09-27, "Keep archive, ship"; ledger E4), as at the
  // incumbent-only exit. This exit used to settle with no polish, and
  // the final naturalness check stopped the entry at persist after every later
  // slice was bought. An empty standing passes the rule only over a blank
  // original, where keeping nothing is the right rendering; the pinned corpus
  // carves no such slice (0 of 1259, 2026-09-27).
  if ((standingText === '') && (laneTexts.length === 0)) {
    /**
     The settlement this exit ends in, the archive kept where nothing passes.
     */
    const settlement = settleWithoutSlate({
      terminal: 'no-standing-text',
      standingEligible,
      standingText,
      floor,
      verdicts,
      producedFindings,
      subject,
      sliceIndex,
      l: sl,
    },);
    if (settlement.archiveKept !== true)
      sl.warn('consolidation: no standing text to judge against, so the slice keeps what it had',);
    return settlement;
  }

  // A SLATE WITH NOTHING VALID ON IT ENDS HERE, before either round is bought.
  // The gate's question is which rendering is more faithful, and that has no
  // meaning when the only proposals are structurally not the page. A slate
  // with lane texts to offer is not that slate: the lanes passed the rule
  // (class forty, 2026-09-17), so the judges are asked.
  if ((floor.kind === 'incumbent-only') && (laneTexts.length === 0)) {
    /**
     The settlement this exit ends in, the archive kept where nothing passes.
     */
    const settlement = settleWithoutSlate({
      terminal: 'incumbent-only',
      standingEligible,
      standingText,
      floor,
      verdicts,
      producedFindings,
      subject,
      sliceIndex,
      l: sl,
    },);
    if (settlement.archiveKept === true)
      return settlement;
    return await applyFinalPolish({
      client,
      settlement,
      subject,
      lineStructured,
      sliceIndex,
      ...((polishConfig === undefined) ? {} : { polishConfig, }),
      eligible: standingMayShip,
      signal,
      perCallTimeoutMs,
      l: sl,
      stageSettings,
    },);
  }

  // A FINAL ROUND THAT CANNOT SEAT TWO JUDGES SHIPS THE TRANSLATE LANE'S TWO-BALLOT TEXT without a slate being bought
  // (owner, 2026-10-29, "Ship the two-ballot text"): fewer slate judges are reachable than the ballots a winner needs,
  // so no slate this settlement could buy would end in anything but one judge alone, and the text two judges chose in
  // the lane and the rule admitted is the wording that ships over a standing the rule refused. XingZ6022 bought 65
  // calls and 10.679 Wh at sixteen slices for a keep-the-archive that this text replaces.
  /**
   Whether the translate lane's winner ships here with no slate bought.
   */
  const beforeSlate = shipWinnerBeforeSlate({
    standingEligible,
    standingMayShip,
    laneTexts,
    translateWinnerBallots,
    ...((slateReachable === undefined) ? {} : { slateReachable, }),
    standingText,
    floor,
    verdicts,
    producedFindings,
    subject,
    sliceIndex,
    l: sl,
  },);
  if (beforeSlate.kind === 'ships')
    return beforeSlate.settlement;

  /**
   Model ids the floor passed, read off the floor so `survivingVoices` takes
   one member step rather than two; none where the floor refused every
   proposal and only lane texts carry the slate on.
   */
  const validModelIds: readonly string[] = (floor.kind === 'incumbent-only') ? [] : floor.validModelIds;

  /**
   Voices the floor passed, which is what the slate may carry.
   */
  const survivingVoices = voices.filter(function survived(voice,): boolean {
    return validModelIds.includes(voice.modelId,);
  },);

  /**
   Those same proposals as they would actually ship.

   WRAPPED BEFORE THE SLATE IS BUILT rather than after the gate has spoken,
   which is the whole of the fix. Wrapping only the winner leaves both
   deciders judging bytes the run then changes, and it did: over the two most
   recent runs of the band pair 15 of the 16 shipped consolidations came back
   from `wrapConsolidation` altered.

   IT ALSO COLLAPSES THE WHITESPACE CASE FOR FREE. A proposal differing from
   an already-wrapped standing text only in where its lines break becomes that
   text exactly, the candidate dedup folds it into the incumbent, and a slate
   left holding the incumbent alone settles unjudged without buying a slate
   round or a gate round.
   */
  const shippableVoices = wrapConsolidationProposals({
    voices: survivingVoices,
    lineStructured,
  },);

  /**
   What the slate offers to keep: the standing text, or nothing when the
   deterministic gate refused it (`consolidate-ineligible-standing.ts`).
   */
  const incumbent = slateIncumbentFor({
    standingEligible,
    standingText,
  },);
  if (!standingEligible)
    sl.warn(`slice ${String(sliceIndex,)}: ${INELIGIBLE_STANDING_WITHHELD_FINDING}`,);
  /**
   Whether a tie is challenged: always over a withheld standing (class
   fifty-five), and over an eligible one only where a strict majority of the
   contest ballots called the archive flawed (class one hundred six, owner
   answer 2026-10-11). An eligible standing that no majority condemns still
   gets the run-off of its tied finalists when it drew no ballot, inside
   `judgeConsolidationSlate`.
   */
  const challenged = (!standingEligible) || runoffOverStanding;
  if (standingEligible && runoffOverStanding) {
    sl.info(
      `slice ${String(sliceIndex,)}: a strict majority of contest ballots called the archive flawed; a tied slate is run off rather than left to the standing (class one hundred six)`,
    );
  }

  if (laneTexts.length > 0) {
    /**
     Lane names, for the log.
     */
    const laneNames = laneTexts.map(function laneOf(laneText,): string {
      return laneText.lane;
    },);
    sl.info(
      `slice ${String(sliceIndex,)}: lane texts offered on the slate beside the proposals: ${
        laneNames.join(', ',)
      }`,
    );
  }

  /**
   Distinct proposals the judges will see, incumbent among them when it may
   ship, and the lane texts when the standing is neither endorsed nor
   eligible (class forty, 2026-09-17).
   */
  const built = buildTranslateCandidates({
    voices: shippableVoices,
    translatorModelIds: roster,
    incumbentText: incumbent.incumbentText,
    lineStructured,
    laneTexts,
  },);

  /**
   What the slate judges decided (`consolidate-settle-judge.ts`), a decision
   over a withheld standing too (ledger B51).
   */
  const decided = await judgeConsolidationSlate({
    client,
    judgeModelIds,
    subject,
    built,
    survivors: survivingVoices.length,
    producedFindings,
    standingEligible,
    incumbent,
    identity,
    evidence,
    lineStructured,
    challenged,
    signal,
    perCallTimeoutMs,
    l: sl,
    stageSettings,
  },);

  // THE JUDGES CHOOSING THE INCUMBENT ENDS IT. There is no consolidation to
  // gate, and asking the gate anyway would buy ballots about the text that is
  // already in place.
  if (decided.origin !== 'fresh') {
    return await settleKeptStanding({
      client,
      decided,
      terminal: SLATE_TERMINALS[decided.decision],
      standingText,
      floor,
      verdicts,
      standingEligible,
      standingMayShip,
      laneTexts,
      translateWinnerBallots,
      subject,
      lineStructured,
      sliceIndex,
      ...((polishConfig === undefined) ? {} : { polishConfig, }),
      signal,
      perCallTimeoutMs,
      l: sl,
      stageSettings,
    },);
  }

  // GATE WHAT WON, WRAP WHAT SHIPS (`consolidate-settle-gate.ts`).
  return await gateAndShip({
    client,
    judgeModelIds,
    subject,
    decided,
    standingText,
    lineStructured,
    floor,
    verdicts,
    sliceIndex,
    ...((polishConfig === undefined) ? {} : { polishConfig, }),
    standingMayShip,
    standingEligible,
    ...((standingRefusal === undefined) ? {} : { standingRefusal, }),
    standingFlawedByMajority: runoffOverStanding,
    identity,
    signal,
    perCallTimeoutMs,
    l: sl,
    stageSettings,
  },);
}

//endregion Consolidate settle
