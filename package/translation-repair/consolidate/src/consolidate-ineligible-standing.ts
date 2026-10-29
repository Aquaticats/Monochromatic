import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { SliceSyntax, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  CONSOLIDATE_GATE_QUORUM,
  type ConsolidateGateOutcome,
} from './consolidate-gate-stage.ts';
import type {
  ArchiveKeptReason,
  ConsolidationSettlement,
  ConsolidationTerminal,
} from './consolidate-settle.ts';
import { unpolishedBaseline, } from './consolidation-polish-skip.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  IncumbentKind,
  LaneText,
  SliceValidation,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  MIN_SELECTION_BALLOTS,
  type TranslateStageResult,
} from '@monochromatic-dev/translation-repair-translate-stage/ts';

//region Ineligible standing text
// A STANDING TEXT THE DETERMINISTIC GATE HAS ALREADY REFUSED may not ship, and
// may not be offered to the judges as something to keep.
//
// THE OWNER'S DECISION OF 2026-09-04, on the luxuanwen3 pass of that day: the
// archive's front matter broke the identity rule in
// `validateFrontMatterTranslation`, the driver logged "standing text fails
// publication eligibility and remains retryable", the slate judges endorsed
// that standing over two valid proposals (three ballots calling the archive's
// shape "the declared translated identity"), the single attempt shipped it
// "with the finding recorded", and `assertFrontMatterComplete` refused the
// whole entry an hour and 2.61 USD later. Asked whether consolidation should
// refuse to ship a standing the gate had rejected, the owner chose: prefer the
// best valid proposal, else fail the slice at once.
//
// PREFERRING THE BEST VALID PROPOSAL IS DONE BY THE JUDGES, not by a tally of
// ballots cast for the incumbent: the incumbent is simply not on the slate.
// `judgeTranslateSlate` already knows an absent incumbent (a passage the
// archive never carried), and with the standing withheld the judges choose
// among the valid proposals or decline, and a decline is the "else".
//
// THE OWNER'S ADDENDUM OF 2026-09-09, on the sixth Mio pass on Bedrock alone:
// at slice 3 the translate lane's standing failed the gate, the archive's
// paragraph and list for that slice were valid and were never offered, and
// the entry stopped. Asked whether to keep the incumbent for that slice or
// stop, the owner chose to keep it: `readStandingVerdict` reads the gate's
// verdict on the incumbent (`row.incumbentText`, the page text the slice
// replaces) beside the standing's, and where the standing is ineligible and
// the incumbent passes, the settlement runs against the incumbent as its
// standing, with {@link INELIGIBLE_STANDING_REPLACED_FINDING} recorded and no
// contest endorsement, so the single-attempt rule ships it with the
// non-endorsement recorded if the judges keep it. The entry stops only where
// the incumbent is ineligible too, or where the standing IS the incumbent
// (luxuanwen3, where the two were one).
//
// THIS IS NOT THE NO-LOOP DECISION REOPENED. `consolidate-slice-buy.ts` keeps
// its single attempt for a standing that merely lacks contest ENDORSEMENT;
// that standing has passed the deterministic gate and quality machinery may
// not withhold the entry over it. A standing that has NOT passed the gate was
// never going to ship, and the page guard was going to say so after the run
// had been paid for.
//
// THE OWNER'S RULING OF 2026-09-27 REPLACES "ELSE FAIL THE SLICE AT ONCE":
// asked whether an entry keeps stopping where no wording for a slice passes
// the rule, given that a stopped entry leaves the archive's whole page live,
// that slice included, the owner chose "Keep archive, ship". Every exit that
// stopped the entry now settles through {@link keepTheArchive}: the slice
// keeps exactly what the archive has there, nothing where it is silent, the
// page ships with every other repair, and the publish step reports the slice.
// The first half of the 2026-09-04 rule stands, so a valid proposal is still
// preferred wherever one exists.

/**
 Finding recorded on a settlement whose standing was withheld from the slate.
 */
export const INELIGIBLE_STANDING_WITHHELD_FINDING: string = 'ineligible-standing-withheld: the standing text failed the '
  + 'deterministic publication rule, so it was not offered to the slate judges; only valid proposals were';

/**
 Finding recorded on a settlement whose ineligible standing was replaced
 by the slice's incumbent, which passed the gate (owner, 2026-09-09).
 */
export const INELIGIBLE_STANDING_REPLACED_FINDING: string = 'ineligible-standing-replaced-by-incumbent: the '
  + 'standing text failed the deterministic publication rule and the incumbent passed it, so the incumbent stands '
  + 'in as the wording the slate judges may keep';

/**
 Finding recorded on a settlement whose gate settled on neither rendering
 while the standing was ineligible, so the proposal the slate chose shipped
 (class fifty-four, XingZ604 slice 13, 2026-09-18).
 */
export const UNDECIDED_GATE_SHIPS_PROPOSAL_FINDING: string = 'undecided-gate-ships-proposal: the gate settled '
  + 'on neither rendering and the standing text failed the deterministic publication rule, so the proposal the '
  + 'slate judges chose ships as the best valid text';

/**
 Finding recorded on a settlement whose gate settled on neither rendering
 over an eligible standing a strict majority of contest ballots called
 flawed, so the proposal the slate chose shipped (class one hundred
 seventy-seven, TianqiChen66610 slice 13, owner answer 2026-09-27: "Slate's
 choice"; every ballot until the owner's answer of 2026-10-11, "Majority, like
 the run-off").
 */
export const FLAWED_STANDING_GATE_SHIPS_PROPOSAL_FINDING: string = 'undecided-gate-ships-proposal (standing flawed '
  + 'by a majority of contest ballots): the gate settled on neither rendering over a standing a majority of '
  + 'contest ballots called flawed, so the proposal the slate judges chose ships';

/**
 Finding recorded on a settlement whose gate preferred a standing the
 deterministic rule refused, so the proposal the slate chose shipped (class
 one hundred eighty-five, TianqiChen66619 slice 9, 2026-09-27).
 */
export const GATE_PREFERRED_INELIGIBLE_STANDING_FINDING: string = 'gate-preferred-ineligible-standing: the gate '
  + 'preferred a standing text that failed the deterministic publication rule, which cannot ship, so the proposal '
  + 'the slate judges chose ships as the best valid text; the gate ballots name what they held against it';

/**
 Finding recorded on a settlement whose gate heard fewer than two answering
 judges over a standing that could not be kept by default, so no proposal
 shipped past it (the owner's rulings of 2026-10-25: one judge alone never
 decides, two ballots decide).
 */
export const UNHEARD_GATE_KEEPS_STANDING_FINDING: string = 'unheard-gate-keeps-standing: the gate heard fewer than two '
  + 'answering judges, so it settled nothing and the proposal the slate judges chose does not ship past it';

/**
 Finding recorded on a settlement whose gate heard fewer than two answering
 judges over a standing the deterministic rule refused or a contest majority
 called flawed, so the proposal two or more slate ballots chose shipped rather
 than that standing (the owner's answers of 2026-10-25, "Ship the slate's
 wording" and "Ship the slate's pick (Recommended)").
 */
export const UNHEARD_GATE_SHIPS_SLATE_PICK_FINDING: string = 'unheard-gate-ships-slate-pick: the gate heard fewer than '
  + 'two answering judges over a standing that failed the deterministic publication rule or that a majority of the '
  + 'contest ballots called flawed, so the proposal two or more slate ballots chose ships rather than that standing';

/**
 Finding recorded on a settlement where no wording passed the deterministic
 rule, so the archive keeps the slice (owner, 2026-09-27, "Keep archive,
 ship").
 */
export const NO_VALID_WORDING_FINDING: string = 'no-valid-wording: no wording for this slice passed the '
  + 'deterministic publication rule, so the archive keeps it and the page ships with it reported';

/**
 Finding recorded on a settlement whose final round could not seat two judges and held no wording two judges chose, so
 the archive keeps the slice (owner, 2026-09-27, "Keep archive, ship"; 2026-10-25, "one judge alone never decides").
 */
export const FINAL_ROUND_UNSEATED_FINDING: string = 'no-valid-wording: the final round could not seat two judges and '
  + 'no wording two judges chose passed the deterministic publication rule, so the archive keeps this slice and the '
  + 'page ships with it reported';

/**
 Finding recorded on a settlement whose slate seated two or more judges and declined every wording, none of which
 drew two ballots, so the archive keeps the slice (owner, 2026-10-25, "Two judges must name it").
 */
export const SLATE_DECLINED_FINDING: string = 'no-valid-wording: the slate judges declined every wording that passed '
  + 'the deterministic publication rule and none drew two ballots, so the archive keeps this slice and the page ships '
  + 'with it reported';

/**
 Finding recorded on a settlement whose final round could not seat two judges over a standing the rule refused, so the
 translate lane's winner shipped (owner, 2026-10-29, "Ship the two-ballot text (Recommended)": "When the final round
 can't seat two judges, the translate lane's winner ships if two judges chose it and it passes the publication rule.").
 */
export const TRANSLATE_WINNER_SHIPS_FINDING: string = 'translate-winner-ships: the final round could not seat two '
  + 'judges over a standing the deterministic publication rule refused, so the translate lane winner ships, since two '
  + 'judges chose it in the lane and it passes the rule';

/**
 The finding that says why the archive keeps a slice.

 @param reason - why the archive keeps it

 @returns Finding recorded on the settlement

 @example
 ```ts
 const finding = archiveKeptFinding({ reason: 'slate-declined', },);
 ```
 */
export function archiveKeptFinding(
  { reason, }: { readonly reason: ArchiveKeptReason; },
): string {
  if (reason === 'no-wording-passed')
    return NO_VALID_WORDING_FINDING;
  if (reason === 'final-round-unseated')
    return FINAL_ROUND_UNSEATED_FINDING;
  return SLATE_DECLINED_FINDING;
}

/**
 Whether a round heard fewer judges than the ballots a winner needs, which is what a final round that could not seat two
 judges is (owner, 2026-10-25, "one judge alone never decides").

 @param heard - judges the round heard or would reach

 @returns Whether that is fewer than the ballots a winner needs

 @example
 ```ts
 const unseated = unseatedFinalRound({ heard: 1, },);
 ```
 */
export function unseatedFinalRound(
  { heard, }: { readonly heard: number; },
): boolean {
  return heard < MIN_SELECTION_BALLOTS;
}

/**
 The translate lane's text when two or more judges chose it in the lane and the lane offer admitted it.

 A TEXT ON `laneTexts` HAS PASSED THE RULE, since the lane offer drops one that does not
 (`laneTextsForSlate` in the artifact package), so the two conditions the owner's answer of 2026-10-29 names are the
 text on the offer and the ballots of the lane that chose it. Fewer than two ballots is one judge alone, which never
 decides.

 @param laneTexts - lane texts the slate offers

 @param ballots - ballots that named the translate lane's text in the lane's own round

 @returns Translate lane's text as an offered winner, or a named absence where it is not offered or drew fewer than
 two ballots

 @example
 ```ts
 const winner = translateWinnerOf({ laneTexts, ballots: 2, },);
 ```
 */
export function translateWinnerOf(
  {
    laneTexts,
    ballots,
  }: {
    readonly laneTexts: readonly LaneText[];
    readonly ballots: number;
  },
): TranslateWinner {
  if (ballots < MIN_SELECTION_BALLOTS)
    return { kind: 'none', };
  /**
   The translate lane's text on the offer, where it is there.
   */
  const offered = laneTexts.find(function isTranslate(laneText,): boolean {
    return laneText.lane === 'translate';
  },);
  return (offered === undefined)
    ? { kind: 'none', }
    : {
      kind: 'offered',
      laneText: offered,
    };
}

/**
 Whether the translate lane holds a text that ships where the final round cannot seat two judges.

 @example
 ```ts
 const winner: TranslateWinner = { kind: 'none', };
 ```
 */
export type TranslateWinner =
  | {
    /**
     The lane offer admitted the translate lane's text and two or more judges chose it in the lane.
     */
    readonly kind: 'offered';

    /**
     The text, as the lane would ship it.
     */
    readonly laneText: LaneText;
  }
    | {
    /**
     The translate lane holds no such text: it was not offered, or it drew fewer than two ballots.
     */
      readonly kind: 'none';
    };

/**
 What the slate offers as its incumbent: the standing text when it may
 ship, nothing when the gate has refused it.

 @param standingEligible - whether the standing passed the deterministic gate

 @param standingText - wording in place when the stage began

 @returns Incumbent text and kind as the slate builder and the judges take them

 @example
 ```ts
 const incumbent = slateIncumbentFor({ standingEligible: false, standingText, },);
 ```
 */
export function slateIncumbentFor(
  {
    standingEligible,
    standingText,
  }: {
    readonly standingEligible: boolean;
    readonly standingText: string;
  },
): {
  readonly incumbentText: string;
  readonly incumbentKind: IncumbentKind;
} {
  if (standingEligible) {
    return {
      incumbentText: standingText,
      incumbentKind: 'present',
    };
  }
  return {
    incumbentText: '',
    incumbentKind: 'absent',
  };
}

/**
 Whether a settlement about to end this way leaves no wording the rule
 passed, so the archive keeps the slice.

 ASKED AT EVERY EXIT THAT KEEPS THE STANDING: the empty floor, the missing
 standing, the judges' absent slate, and the gate's refusal of the
 consolidation they chose. A `consolidated` terminal ships fresh wording the
 floor passed, which is the one outcome that needs no archive.

 @param standingEligible - whether the standing passed the deterministic gate

 @param terminal - how the settlement is about to end

 @returns Whether nothing the settlement holds may ship

 @example
 ```ts
 const keep = nothingValidShips({ standingEligible, terminal: 'incumbent-only', },);
 ```
 */
export function nothingValidShips(
  {
    standingEligible,
    terminal,
  }: {
    readonly standingEligible: boolean;
    readonly terminal: ConsolidationTerminal;
  },
): boolean {
  return (!standingEligible) && (terminal !== 'consolidated');
}

/**
 Settles a slice whose every wording the deterministic rule refused by
 keeping the archive's (owner, 2026-09-27, "Keep archive, ship").

 THE SETTLEMENT KEEPS ITS TERMINAL, which still says which round ended it;
 the mark and the finding say what ships. Its text stays the refused
 standing, since the archive this slice keeps is the comparison row's, which
 the artifact reads, and on a disputed slice the incumbent here is the
 repair lane's stand-in rather than the archive.

 @param settlement - what the stage settled, before any polish

 @param subject - slice, whose syntax role decides the not-run reason the
 final naturalness check requires; taken whole so no exit has to thread the
 role across on its own

 @param sliceIndex - prepared position of the slice, for the log

 @param reason - why the archive keeps the slice, which the finding, the log line and the artifact say

 @param l - stage logger, told that the archive keeps the slice

 @returns Settlement marked archive-kept, with its reason, its finding and a not-run polish

 @example
 ```ts
 return keepTheArchive({ settlement, subject, sliceIndex, reason: 'no-wording-passed', l, },);
 ```
 */
export function keepTheArchive(
  {
    settlement,
    subject,
    sliceIndex,
    reason,
    l,
  }: {
    readonly settlement: ConsolidationSettlement;
    readonly subject: { readonly syntax?: SliceSyntax; };
    readonly sliceIndex: number;
    readonly reason: ArchiveKeptReason;
    readonly l: Logger;
  },
): ConsolidationSettlement {
  /**
   What happened at this slice, in the words of the log line.
   */
  const happened = (reason === 'no-wording-passed')
    ? 'no wording passed the deterministic publication rule'
    : ((reason === 'final-round-unseated')
      ? 'the final round could not seat two judges and no wording two judges chose passed the deterministic '
        + 'publication rule'
      : 'the slate judges declined every wording that passed the deterministic publication rule and none drew two '
        + 'ballots');
  l.warn(
    `slice ${String(sliceIndex,)}: ${happened} (${settlement.terminal}); `
      + 'the archive keeps this slice and the page ships with it reported',
  );
  return {
    ...settlement,
    archiveKept: true,
    archiveReason: reason,
    findings: [
      ...settlement.findings,
      archiveKeptFinding({ reason, },),
    ],
    polish: unpolishedBaseline(subject,),
  };
}

/**
 Ships the translate lane's winner over a standing the rule refused, in a final round that could not seat two judges
 (owner, 2026-10-29, "Ship the two-ballot text (Recommended)").

 THE SETTLEMENT KEEPS ITS TERMINAL, as {@link keepTheArchive} does, and the mark and the finding say what ships. Its
 text is the winner's, which the lane offer admitted because it passes the rule and which two judges chose in the lane;
 no polish runs over it, since the wording shipped as the lane left it and no judge here heard it.

 @param settlement - what the stage settled, before any polish

 @param winner - translate lane's text, which two or more judges chose in the lane

 @param subject - slice, whose syntax role decides the not-run reason the final naturalness check requires

 @param sliceIndex - prepared position of the slice, for the log

 @param l - stage logger, told that the winner ships

 @returns Settlement marked translate-winner-shipped, carrying the winner as its text, its finding and a not-run polish

 @example
 ```ts
 return shipTranslateWinner({ settlement, winner, subject, sliceIndex, l, },);
 ```
 */
export function shipTranslateWinner(
  {
    settlement,
    winner,
    subject,
    sliceIndex,
    l,
  }: {
    readonly settlement: ConsolidationSettlement;
    readonly winner: LaneText;
    readonly subject: { readonly syntax?: SliceSyntax; };
    readonly sliceIndex: number;
    readonly l: Logger;
  },
): ConsolidationSettlement {
  l.warn(
    `slice ${String(sliceIndex,)}: the final round could not seat two judges over a standing the deterministic `
      + 'publication rule refused; the translate lane winner, which two or more ballots chose in the lane and which '
      + 'passes the rule, ships',
  );
  return {
    ...settlement,
    text: winner.text,
    translateWinnerShipped: true,
    translateWinnerReason: 'refused-standing',
    findings: [
      ...settlement.findings,
      TRANSLATE_WINNER_SHIPS_FINDING,
    ],
    polish: unpolishedBaseline(subject,),
  };
}

/**
 Ballots of the slate round that named the proposal it chose, which is how
 many judges stand behind the pick the gate is then asked about.

 ZERO WHERE THE SLATE RECORDS NO ROW FOR THE SHIPPED POSITION, which a pick
 that never drew a ballot is (a tie broken by the fixed order across candidates
 nobody named). Zero is below the two a pick needs, so such a pick is never
 shipped past an unheard gate.

 @param decided - what the slate judges settled, a fresh consolidation

 @returns Ballots that named the shipped position, self-votes included

 @example
 ```ts
 const slatePickBallots = slatePickBallotsOf({ decided, },);
 ```
 */
export function slatePickBallotsOf(
  { decided, }: { readonly decided: Pick<TranslateStageResult, 'perCandidate' | 'shippedIndex'>; },
): number {
  /**
   The slate round's row for the shipped position, absent where no ballot named it.
   */
  const shippedRow = decided.perCandidate
    .find(function atShipped(row,): boolean {
      return row.index === decided.shippedIndex;
    },);
  return shippedRow?.ballots ?? 0;
}

/**
 Ships the proposal the slate chose past a gate that settled on neither
 rendering, when the standing it would otherwise keep is ineligible.

 CLASS FIFTY-FOUR (XingZ604 slice 13, 2026-09-18): the archive's paragraph
 and the contest winner both carried the original's neutral pronoun
 untranslated, the standing was withheld from the slate, the slate judges
 chose a valid proposal 3 of 4, and the gate went 5 of 7 usable with neither
 rendering at quorum. The gate's rule that indecision keeps the standing
 text is a conservative default, and with an ineligible standing there is
 nothing conservative to keep: the entry stopped at 4h53m over a text the
 judges had already endorsed. The owner's 2026-09-04 rule prefers the best
 valid proposal.

 CLASS ONE HUNDRED SEVENTY-SEVEN (TianqiChen66610 slice 13, owner answer
 2026-09-27: "Slate's choice"). The standing was eligible, but every contest
 ballot had called it flawed; the class one hundred six run-off ran, the slate
 chose a valid proposal, and the gate tied 2 to 2, so the condemned archive
 shipped. A standing the whole contest condemned is no conservative default
 either, so the same indecision ships the slate's choice there too. The owner
 widened this to a strict majority of the contest ballots on 2026-10-11
 ("Majority, like the run-off"), after TianqiChen66622 slice 4 kept an
 archive four of five ballots called flawed.

 CLASS ONE HUNDRED EIGHTY-FIVE (TianqiChen66619 slice 9, 2026-09-27). The
 gate preferred an ineligible standing 2 of 4 over the slate's valid choice,
 and the entry stopped. The class fifty-four note had kept that stop ("a
 gate that refuses the consolidation by quorum still keeps the standing"),
 which read the owner's "else fail the slice at once" as covering a gate's
 preference; the rule fails the slice only where no valid proposal exists,
 and a standing the deterministic rule refused cannot ship whatever the gate
 thinks of it. So a gate that prefers an ineligible standing ships the
 slate's choice too, with the finding naming the preference; an eligible
 standing the gate prefers still ships as before.

 A GATE THAT HEARD FEWER THAN TWO ANSWERING JUDGES SHIPS NOTHING PAST ITS
 NEITHER ON THE GATE'S OWN AUTHORITY (XingZ6021, 2026-10-25): its neither is
 the want of a quorum, not an indecision among judges, and one judge alone
 never decides. What the slate decided stands on its own, though: where two or
 more slate ballots chose the proposal and the standing is the archive the
 deterministic rule refused, or an eligible standing a strict majority of the
 contest ballots called flawed, the proposal ships, with
 {@link UNHEARD_GATE_SHIPS_SLATE_PICK_FINDING}, because the slate's choice is
 itself a two-ballot decision (the owner's answers of 2026-10-25, "Ship the
 slate's wording" for the refused archive and "Ship the slate's pick
 (Recommended)" for the flawed one). Over an eligible standing no contest
 majority called flawed the outcome comes back untouched before this rule is
 read, and over a pick fewer than two slate ballots chose, it comes back as it stands with
 {@link UNHEARD_GATE_KEEPS_STANDING_FINDING}.

 @param outcome - what the gate settled

 @param standingEligible - whether the standing passed the deterministic gate

 @param standingFlawedByMajority - whether a strict majority of the contest
 ballots called the standing flawed (`archiveFlawedByMajority` in
 `consolidate-archive-flawed.ts`)

 @param slatePickBallots - ballots of the slate round that named the proposal
 the gate was asked about (`slatePickBallotsOf`), whose ships-anyway rule needs
 two; required so that a caller cannot omit what a one-ballot gate turns on

 @param l - stage logger, told when the rule applies

 @returns Outcome as settled, or one shipping the consolidation with the
 finding recorded

 @example
 ```ts
 const gated = shipPastForfeitStanding({ outcome, standingEligible, standingFlawedByMajority, slatePickBallots, l, },);
 ```
 */
export function shipPastForfeitStanding(
  {
    outcome,
    standingEligible,
    standingFlawedByMajority = false,
    slatePickBallots,
    l,
  }: {
    readonly outcome: ConsolidateGateOutcome;
    readonly standingEligible: boolean;
    readonly standingFlawedByMajority?: boolean;
    readonly slatePickBallots: number;
    readonly l: Logger;
  },
): ConsolidateGateOutcome {
  if ((outcome.choice === 'standing') && (!standingEligible)) {
    l.warn(
      `consolidate gate: preferred an ineligible standing, which cannot ship, so the proposal the slate chose ships (${
        String(outcome.usable,)
      } usable ${
        wordForCount({
          count: outcome.usable,
          one: 'ballot',
          many: 'ballots',
        },)
      })`,
    );
    return {
      ...outcome,
      ships: 'consolidated',
      findings: [
        ...outcome.findings,
        GATE_PREFERRED_INELIGIBLE_STANDING_FINDING,
      ],
    };
  }
  if (outcome.choice !== 'neither')
    return outcome;
  /**
   Whether the standing is no conservative default to keep: ineligible, or
   condemned by a strict majority of the contest ballots.
   */
  const standingForfeit = (!standingEligible) || standingFlawedByMajority;
  if (!standingForfeit)
    return outcome;
  // A GATE THAT HEARD FEWER THAN TWO ANSWERING JUDGES HAS NO VERDICT, undecided or otherwise, to ship a proposal past.
  // XingZ6021 shipped five consolidations (slices 13, 14, 33, 44 and 84) on a gate with one usable ballot, which
  // broke the owner's standing rule that one judge alone never decides, restated on 2026-10-25 ("Two ballots
  // decide"). The standing stays unless the slate's own pick was chosen by two or more slate ballots, over an
  // archive the rule refused (owner, 2026-10-25, "Ship the slate's wording") or an eligible standing a majority of the
  // contest ballots called flawed (owner, 2026-10-25, "Ship the slate's pick (Recommended)"), where the pick ships;
  // otherwise an ineligible standing leaves the archive kept through `nothingValidShips` (owner, 2026-09-27, "Keep
  // archive, ship"), and a majority-flawed one ships as it stands.
  if (outcome.usable < CONSOLIDATE_GATE_QUORUM) {
    if (slatePickBallots >= MIN_SELECTION_BALLOTS) {
      l.warn(
        `consolidate gate: heard ${String(outcome.usable,)} usable ${
          wordForCount({
            count: outcome.usable,
            one: 'ballot',
            many: 'ballots',
          },)
        }, below the two judges a verdict needs, over ${
          standingEligible ? 'a standing a majority of contest ballots called flawed' : 'an ineligible standing'
        }; the proposal ${
          String(slatePickBallots,)
        } ${
          wordForCount({
            count: slatePickBallots,
            one: 'slate ballot',
            many: 'slate ballots',
          },)
        } chose ships, since the slate's pick is a two-ballot decision of its own`,
      );
      return {
        ...outcome,
        ships: 'consolidated',
        findings: [
          ...outcome.findings,
          UNHEARD_GATE_SHIPS_SLATE_PICK_FINDING,
        ],
      };
    }
    l.warn(
      `consolidate gate: heard ${String(outcome.usable,)} usable ${
        wordForCount({
          count: outcome.usable,
          one: 'ballot',
          many: 'ballots',
        },)
      }, below the two judges a verdict needs, so the proposal the slate chose does not ship past it`,
    );
    return {
      ...outcome,
      findings: [
        ...outcome.findings,
        UNHEARD_GATE_KEEPS_STANDING_FINDING,
      ],
    };
  }
  /**
   Why the standing is not kept, for the log and the finding.
   */
  const why = standingEligible ? 'a standing a majority of contest ballots called flawed' : 'an ineligible standing';
  l.warn(
    `consolidate gate: settled on neither over ${why}, so the proposal the slate chose ships (${
      String(outcome.usable,)
    } usable ${
      wordForCount({
        count: outcome.usable,
        one: 'ballot',
        many: 'ballots',
      },)
    })`,
  );
  return {
    ...outcome,
    ships: 'consolidated',
    findings: [
      ...outcome.findings,
      standingEligible ? FLAWED_STANDING_GATE_SHIPS_PROPOSAL_FINDING : UNDECIDED_GATE_SHIPS_PROPOSAL_FINDING,
    ],
  };
}

/**
 One line saying why the deterministic gate refused a standing text, for
 the run log.

 WRITTEN FOR THE READING, not the judges: on 2026-09-04 the luxuanwen3 log
 said only that a standing "fails publication eligibility", and learning
 that the cause was a link destination the archive had rewritten took
 opening the slice records. A refusal the log names is a defect class the
 next reading finds in one grep.

 @param validation - deterministic verdict on the standing text

 @returns Findings joined into one line, the reason no comparison was
 possible, or a word for a pass

 @example
 ```ts
 dl.warn(`slice 1: ${describeStandingVerdict({ validation, },)}`,);
 ```
 */
export function describeStandingVerdict(
  { validation, }: { readonly validation: SliceValidation; },
): string {
  if (validation.kind === 'valid')
    return 'passes the deterministic publication rule';
  if (validation.kind === 'invalid')
    return validation.findings
      .join(' ',);
  return `no comparison was possible: ${validation.detail}`;
}

//endregion Ineligible standing text
