import type { ProducerStanding, } from './producer-standing.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Producer silence
// WHO IS MISSING FROM A STANDING TABLE, and which of the reasons it is.
//
// `producerStandings` counts ballots, so it carries a row only for a model that
// wrote a candidate somebody voted on. Every other seated model vanishes, and an
// absent row reads exactly like a model that wrote and lost. Three times that
// has been the wrong reading: a provider out of credit takes half the roster
// off the table; a slate where every producer proposed the same wording ships
// without a ballot being cast over it; and a rewriter that answers every ask
// and leaves the paragraph as it stands never reaches a slate at all, which
// was once reported as provider silence beside a SEAT line saying the seat
// had answered 31 of 31.
//
// THE REASONS CALL FOR DIFFERENT ACTIONS, which is why they are separated
// rather than reported as one absence. A model that answered nothing is
// evidence nobody has bought yet, and re-running those seats buys it. A model
// that answered and was never slated has been paid for and will answer the
// same way again; only slices with something to rewrite would seat it. A model
// that wrote text nobody voted on has already been paid for, and only more
// slices separate it from a model that lost.
//
// WHO ANSWERED IS KNOWN ONLY WHERE THE STAGE CARRIES IT OUT. The refine stage
// does; the editor and translate stages carry only a count out. A
// seat that cannot say who answered says so in its line rather than folding
// the unknown into "wrote nothing", because that fold is exactly the misreport.
//
// SEATED ROSTER IN, so this file names no roster of its own. Both calibrations
// pass the seats they filled, and a table describing seats a run never filled is
// refused rather than reported.

/**
 What a seat knows about which models answered it at all.

 `recorded` carries every model heard with a usable answer at the seat,
 whether or not the answer proposed anything; `unrecorded` says the seat does
 not carry that list out, so silence there is indistinguishable from an
 answer dropped before judging.

 @example
 ```ts
 const answered: SeatAnswers = { kind: 'recorded', modelIds: heard, };
 ```
 */
export type SeatAnswers =
  | {
    readonly kind: 'recorded';

    /**
     Models heard with a usable answer, repeats allowed.
     */
    readonly modelIds: readonly RosterModelId[];
  }
    | {
      readonly kind: 'unrecorded';
    };

/**
 Roster models a standing table does and does not describe.

 THE FOUR STATES ARE DISJOINT AND COVER THE SEATED ROSTER, so a reader can
 add the lengths and check the total against the seats a run filled.
 `answeredUnslated` is empty whenever answers were not recorded, and then
 `neverWrote` means only that no candidate reached a slate.

 @example
 ```ts
 const { judged, wroteUnjudged, answeredUnslated, neverWrote, } = readStandingCoverage({
   roster,
   standings,
   produced,
   answered,
 },);
 ```
 */
export type StandingCoverage = {
  /**
   Models the standing table carries a row for.
   */
  readonly judged: readonly RosterModelId[];

  /**
   Models that produced text no disinterested judge ever voted on.
   */
  readonly wroteUnjudged: readonly RosterModelId[];

  /**
   Models heard with a usable answer, none of which became a candidate.
   */
  readonly answeredUnslated: readonly RosterModelId[];

  /**
   Models no candidate of which reached any slate and, where answers were
   recorded, no usable answer of which was heard.
   */
  readonly neverWrote: readonly RosterModelId[];

  /**
   Whether the seat recorded who answered, which decides what `neverWrote`
   can be read as.
   */
  readonly answersRecorded: boolean;
};

/**
 Raised when a standing, a slate or an answer names a model the run never
 seated.

 @example
 ```ts
 throw new UnseatedStandingError({ modelIds: ['minimax-m3',], },);
 ```
 */
export class UnseatedStandingError extends Error {
  /**
   Declares this message safe to forward: it names model ids.
   */
  readonly messageNamesOnly: true = true;

  /**
   @param modelIds - models named by the evidence and absent from the roster
   */
  public constructor(
    { modelIds, }: { readonly modelIds: readonly RosterModelId[]; },
  ) {
    super(
      `evidence names ${modelIds.join(', ',)}, which this run never seated, so the table `
        + 'describes a roster other than the one measured and its shares compare nothing',
    );
    this.name = 'UnseatedStandingError';
  }
}

/**
 Splits a seated roster by what the standing table can say about each model.

 @param roster - seats this run filled

 @param standings - what the ballot tally produced

 @param produced - models known to have written a candidate, judged or not;
 a caller that cannot tell passes what it can see

 @param answered - models heard with a usable answer at this seat, or a
 statement that the seat does not record them

 @returns Seated roster split four ways

 @throws {@link UnseatedStandingError} when a standing, a produced model or an
 answering model was never seated, since coverage of one roster cannot be
 read off another

 @example
 ```ts
 const coverage = readStandingCoverage({ roster: RUN_ROSTER, standings, produced, answered, },);
 ```
 */
export function readStandingCoverage(
  {
    roster,
    standings,
    produced,
    answered,
  }: {
    readonly roster: readonly RosterModelId[];
    readonly standings: readonly ProducerStanding[];
    readonly produced: readonly RosterModelId[];
    readonly answered: SeatAnswers;
  },
): StandingCoverage {
  /**
   Seats this run filled, for membership tests.
   */
  const seated = new Set(roster,);

  /**
   Models the standing table carries a row for.
   */
  const judged = new Set(standings.map(function toModelId(standing,): RosterModelId {
    return standing.modelId;
  },),);

  /**
   Models known to have written something, whether or not it was judged.
   */
  const wrote = new Set(produced,);

  /**
   Models heard with a usable answer, empty where the seat records none.
   */
  const heard = new Set((answered.kind === 'recorded') ? answered.modelIds : [],);

  /**
   Models the evidence names that no seat was filled with.
   */
  const unseated = [
    ...judged,
    ...wrote,
    ...heard,
  ].filter(function wasNotSeated(modelId,): boolean {
    return !seated.has(modelId,);
  },);

  // EVERY INPUT IS CHECKED AGAINST THE SEATS, because any one naming an
  // unseated model means the table and the roster came from different runs, and
  // reporting coverage across those would compare shares nothing shares.
  if (unseated.length > 0)
    throw new UnseatedStandingError({ modelIds: unseated, },);

  return {
    judged: roster.filter(function hasRow(modelId,): boolean {
      return judged.has(modelId,);
    },),
    wroteUnjudged: roster.filter(function wroteOnly(modelId,): boolean {
      return wrote.has(modelId,) && (!judged.has(modelId,));
    },),
    answeredUnslated: roster.filter(function answeredOnly(modelId,): boolean {
      return heard.has(modelId,)
        && (!wrote.has(modelId,))
        && (!judged.has(modelId,));
    },),
    neverWrote: roster.filter(function stayedSilent(modelId,): boolean {
      return (!heard.has(modelId,))
        && (!wrote.has(modelId,))
        && (!judged.has(modelId,));
    },),
    answersRecorded: answered.kind === 'recorded',
  };
}

/**
 Words for the silent seats, chosen by whether the seat knows who answered.

 @param silent - silent models, rendered for the line

 @param described - seats the table describes

 @param seats - seats the coverage was read over

 @param answersRecorded - whether silence here means no usable answer

 @returns One report line

 @example
 ```ts
 const line = silentSeatLine({ silent, described: 1, seats: 4, answersRecorded: true, },);
 ```
 */
function silentSeatLine(
  {
    silent,
    described,
    seats,
    answersRecorded,
  }: {
    readonly silent: string;
    readonly described: number;
    readonly seats: number;
    readonly answersRecorded: boolean;
  },
): string {
  /**
   Fraction of the roster the table describes, shared by both wordings.
   */
  const covers = `covers ${String(described,)} of ${String(seats,)} ${
    wordForCount({
      count: seats,
      one: 'seat',
      many: 'seats',
    },)
  }`;

  /**
   Where the counts are, shared by both wordings and pinned by a test.
   */
  const pointer = 'the SEAT lines at the end of this command say how often each seat was asked '
    + 'and how many answers were usable, and the run log names the failure. Re-run these '
    + 'seats before reading the table as a comparison of the roster.';

  if (answersRecorded) {
    return `ANSWERED NOTHING USABLE: ${silent}. No answer of theirs was heard at this seat, so `
      + `the table ${covers}. A provider out of budget, a refused sheet and a call that timed `
      + `out all look like this from here; ${pointer}`;
  }
  return `NO CANDIDATE OF THEIRS REACHED ANY SLATE: ${silent}, so the table ${covers}. This `
    + 'seat does not record who answered, so a model that answered and was dropped before '
    + `judging and a provider that failed every call look alike from here; ${pointer}`;
}

/**
 Renders what the standing table leaves out, one line per reason.

 PRINTS NOTHING WHERE THERE IS NOTHING TO SAY, so a run with the whole roster
 on the table carries no note claiming completeness it would then have to
 keep true.

 @param coverage - seated roster split four ways

 @returns Report lines, empty where every seated model was judged

 @example
 ```ts
 for (const line of coverageGapLines({ coverage, },))
   console.log(`  ${line}`,);
 ```
 */
export function coverageGapLines(
  { coverage, }: { readonly coverage: StandingCoverage; },
): readonly string[] {
  /**
   Models that wrote and drew no ballot, rendered for the line.
   */
  const unvoted = coverage
    .wroteUnjudged
    .join(', ',);

  /**
   Models heard and never slated, rendered the same way.
   */
  const unslated = coverage
    .answeredUnslated
    .join(', ',);

  /**
   Models that answered nothing, rendered the same way.
   */
  const silent = coverage
    .neverWrote
    .join(', ',);

  /**
   Seats the table describes, which is the numerator the silent line needs.
   */
  const described = coverage
    .judged
    .length;

  /**
   Seats the coverage was read over, summed back from its four parts.
   */
  const seats = described
    + coverage
      .wroteUnjudged
      .length
    + coverage
      .answeredUnslated
      .length
    + coverage
      .neverWrote
      .length;

  return [
    ...((unvoted === '')
      ? []
      : [
        `WROTE AND WAS NEVER VOTED ON: ${unvoted}. Their text reached a slate and no `
          + 'disinterested ballot was cast over it, which is what a slice where every producer '
          + 'proposed the same wording does: it ships unjudged. The table says nothing about them '
          + 'either way, and more slices are what would.',
      ]),
    ...((unslated === '')
      ? []
      : [
        `ANSWERED AND WAS NEVER SLATED: ${unslated}. At least one usable answer of theirs was `
          + 'heard and none became a candidate a judge saw: a rewriter that leaves a paragraph as '
          + 'it stands, or whose rewrite is dropped before judging, looks like this. The table '
          + 'says nothing about them, their SEAT lines carry the answers, and re-running them buys '
          + 'the same again; slices with something to rewrite are what would seat them.',
      ]),
    ...((silent === '')
      ? []
      : [
        silentSeatLine({
          silent,
          described,
          seats,
          answersRecorded: coverage.answersRecorded,
        },),
      ]),
  ];
}

//endregion Producer silence
