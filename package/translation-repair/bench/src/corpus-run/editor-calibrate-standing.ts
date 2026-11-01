import { producerModelIds, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  coverageGapLines,
  producerStandings,
  rankStandings,
  readStandingCoverage,
  type SeatAnswers,
  standingLine,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { SelectionRound, } from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Standing report
// ONE SEAT'S STANDING, RENDERED AS LINES rather than printed, so the report can
// be read by a test without capturing the console. `editor-calibrate.ts` is an
// entry module and prints what this returns; it moved here when that file
// reached the line budget.

/**
 Whether a round was judged: at least one ballot was cast over its slate.

 THE ONE PLACE THE EDITOR CALIBRATION DECIDES IT. A slate of one candidate,
 which is what every editor proposing the same wording leaves, needs no vote
 and records a round with no ballot; the standing and the closing paragraph
 once read such a round two ways, the standing counting it as judged and the
 closing as a judged round on its slice, while every row of the standing
 read `UNJUDGED`.

 @param round - one round a seat produced, which every printer of the
 calibration reads through this one predicate so their counts add up

 @returns Whether any ballot was cast on it, which alone makes the round
 evidence about the producers on its slate

 @example
 ```ts
 const judged = roundWasJudged({ round, },);
 ```
 */
export function roundWasJudged({ round, }: { readonly round: SelectionRound; },): boolean {
  return round.ballots
    .length
    > 0;
}

/**
 The rounds that were judged, through `roundWasJudged`.

 @param rounds - rounds of one seat, one slice's or a whole sample's, judged
 or not

 @returns Those with at least one ballot, in the order they were bought

 @example
 ```ts
 const judged = judgedRoundsOf({ rounds, },);
 ```
 */
export function judgedRoundsOf(
  { rounds, }: { readonly rounds: readonly SelectionRound[]; },
): readonly SelectionRound[] {
  return rounds.filter(function judged(round,): boolean {
    return roundWasJudged({ round, },);
  },);
}

/**
 Every model holding a stake in any candidate on one seat's slates, judged or
 not, which is what the coverage reads as having written.

 NAMED FOR EVERY SLATE AUTHOR. Until 2026-10-06 it was `judgedAuthors`, while
 it read every round, a ballot cast on it or not (`roundWasJudged`).

 @internal

 @param perSlice - that seat's rounds, grouped by the slice that bought them

 @returns Model ids, repeats included, in the order the slates carried them

 @example
 ```ts
 const wrote = slatedAuthors({ perSlice, },);
 ```
 */
export function slatedAuthors(
  { perSlice, }: { readonly perSlice: readonly (readonly SelectionRound[])[]; },
): readonly RosterModelId[] {
  return perSlice
    .flat()
    .flatMap(function slateAuthors(round,): readonly RosterModelId[] {
      return round
        .producers
        .flatMap(function stakeholders(producer,): readonly RosterModelId[] {
          return producerModelIds(producer,);
        },);
    },);
}

/**
 Renders one seat's counts per slice, so a reader can bootstrap over slices.

 SLICES ARE THE INDEPENDENT UNIT, NOT BALLOTS. Every ballot in a round was
 cast over one slate and every round on a slice was cut from one passage, so
 the pooled share's ballot-level error understates how far a standing moves
 between runs: a four-slice standing moved one model from 52.2 to 26.7 to
 10.0 percent across identical runs. A bootstrap over whole slices needs each
 slice's counts, and the pooled table throws them away. These lines keep them
 in sample order, so each pairs with its slice progress line by position, and
 they carry votes, ballots and candidates rather than a share, for the reason
 the pooled line carries its denominator. They read the judged rounds alone,
 as the pooled table does, so the two count the same rounds.

 @internal

 @param perSlice - that seat's rounds, grouped by the slice that bought them

 @returns One line per slice that bought a judged round, none for the rest

 @example
 ```ts
 for (const line of sliceStandingLines({ perSlice, },))
   console.log(line,);
 ```
 */
export function sliceStandingLines(
  { perSlice, }: { readonly perSlice: readonly (readonly SelectionRound[])[]; },
): readonly string[] {
  return perSlice.flatMap(function sliceLine(
    bought,
    index,
  ): readonly string[] {
    /**
     The rounds of this slice a ballot was cast on.
     */
    const rounds = judgedRoundsOf({ rounds: bought, },);
    if (rounds.length === 0)
      return [];

    /**
     Per-model counts over this slice alone, best first.
     */
    const cells = rankStandings({ standings: producerStandings({ rounds, },), },)
      .map(function cell(standing,): string {
        return `${standing.modelId} ${String(standing.disinterestedVotes,)}/${
          String(standing.disinterestedBallots,)
        } over ${String(standing.candidates,)}`;
      },);

    return [
      `  slice ${String(index + 1,)}: ${String(rounds.length,)} judged ${
        wordForCount({
          count: rounds.length,
          one: 'round',
          many: 'rounds',
        },)
      }; ${cells.join('; ',)}`,
    ];
  },);
}

/**
 Renders one seat's standing over the rounds it produced, then the same
 counts per slice.

 @internal

 @param seat - what the standing is about, for the heading

 @param roster - seats the run filled, which the coverage is read against

 @param perSlice - that seat's rounds, grouped by the slice that bought them

 @param produced - models known to have written a candidate, judged or not

 @param answered - who the seat heard, or that it does not record that

 @returns Report lines carrying their own indentation, heading first

 @example
 ```ts
 for (const line of standingReportLines({ seat: 'EDITOR', roster, perSlice, produced, answered, },))
   console.log(line,);
 ```
 */
export function standingReportLines(
  {
    seat,
    roster,
    perSlice,
    produced,
    answered,
  }: {
    readonly seat: string;
    readonly roster: readonly RosterModelId[];
    readonly perSlice: readonly (readonly SelectionRound[])[];
    readonly produced: readonly RosterModelId[];
    readonly answered: SeatAnswers;
  },
): readonly string[] {
  /**
   Every round this seat produced, across every slice, judged or not.
   */
  const bought = perSlice.flat();

  /**
   The rounds a ballot was cast on, which are all the standing is over.
   */
  const rounds = judgedRoundsOf({ rounds: bought, },);

  /**
   Slices that produced a judged round.
   */
  const contributed = perSlice.filter(function paidIn(slice,): boolean {
    return judgedRoundsOf({ rounds: slice, },)
      .length
      > 0;
  },);

  /**
   Rounds no ballot was cast on, which the standing is not over.
   */
  const unjudged = bought.length - rounds.length;

  /**
   Clause counting those rounds, empty when there are none, so a heading over
   judged rounds never hides the rounds beside them that drew no ballot.
   */
  const unjudgedClause = (unjudged === 0)
    ? ''
    : `; ${String(unjudged,)} ${
      wordForCount({
        count: unjudged,
        one: 'round',
        many: 'rounds',
      },)
    } drew no ballot`;

  /**
   Heading every report starts with.
   */
  const heading = `\n${seat} standing over ${String(rounds.length,)} judged ${
    wordForCount({
      count: rounds.length,
      one: 'round',
      many: 'rounds',
    },)
  }, `
    + `from ${String(contributed.length,)} of ${String(perSlice.length,)} ${
      wordForCount({
        count: perSlice.length,
        one: 'slice',
        many: 'slices',
      },)
    }${unjudgedClause}`;

  if (bought.length === 0) {
    return [
      heading,
      '  NO ROUNDS. This seat judged nothing across the sample, so it has no standing. '
        + 'For the editor seat that means no slice carried an ACCEPTED issue: critics can '
        + 'raise claims and the panel can adjudicate them and the lane still report '
        + '"nothing to edit", which is what one live slice did. For the refiner seat it '
        + 'means the naturalness lane proposed nothing. Draw more slices.',
    ];
  }

  // ROUNDS WITH NO BALLOT ARE NOT "NO ROUNDS": the seat produced, the heading
  // counts them, and the note says why nothing was voted on rather than that
  // nothing was raised.
  if (rounds.length === 0) {
    return [
      heading,
      '  NO JUDGED ROUNDS. The seat has no standing, since no ballot was cast on any round it produced: a '
        + 'slate of one candidate, which is what every producer proposing the same wording leaves, needs no '
        + 'vote, and a panel whose every judge abstained or failed casts none. Draw more slices.',
    ];
  }

  /**
   Rows of the table, one per model somebody voted on.
   */
  const standings = producerStandings({ rounds, },);

  /**
   Which of the seated models this table actually describes.

   NAMED RATHER THAN OMITTED. `producerStandings` carries a row only for a
   model somebody voted on, so a model whose provider was out of budget
   vanishes, and absence there reads exactly like a model that wrote and
   lost. During a provider outage that is half the roster.
   */
  const coverage = readStandingCoverage({
    roster,
    standings,
    produced,
    answered,
  },);

  return [
    heading,
    ...rankStandings({ standings, },)
      .map(function rendered(standing,): string {
        return `  ${standingLine({ standing, },)}`;
      },),
    ...coverageGapLines({ coverage, },)
      .map(function indented(line,): string {
        return `  ${line}`;
      },),
    ...sliceStandingLines({ perSlice, },),
  ];
}

//endregion Standing report
