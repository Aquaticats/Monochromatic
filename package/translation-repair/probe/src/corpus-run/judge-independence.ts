import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Judge independence
// Who may re-examine an accepted issue, given who proposed it.
//
// WHAT THIS CAN AND CANNOT ESTABLISH, stated here because the number it feeds
// will be read by someone who did not run it. `RUN_MODELS` seats ONE SET as
// critics, as adjudication panel, and as judges: six models from one provider
// when this was written on 2026-08-13 (`model-catalog` confirmed that provider
// served no seventh), ten across two providers since 2026-08-24, and the
// roster less what `RUN_WIDE_SEATS` drops since 2026-09-01 (eight of nine,
// then seven, then six on 2026-09-03). The count moved;
// the shape did not, and the shape is what matters here: every judge sits on
// the panel, so there is no seat outside the panel to bar an adjudicator with.
//
// So a judge can be independent of a claim's AUTHORSHIP and cannot be
// independent of its ADJUDICATION. Every candidate judge already voted on the
// issue. A crosscheck built on this measures whether a verdict survives being
// re-asked with the author removed. It does not measure precision, and calling
// it precision would be the confident wrong number this work exists to avoid.
//
// The one thing it must never do is quietly shrink. A claim nobody may judge is
// reported as such, because dropping it takes a row out of the denominator and
// lifts every rate above it while looking entirely ordinary. That exact defect
// was already found and fixed once in the attribution reader.

/**
 Who may judge one claim, and who was barred from it.

 @example
 ```ts
 const seat = seatJudges({ proposers, roster, },);
 ```
 */
export type JudgeSeating = {
  /**
   Models that may re-examine the claim, in roster order.
   */
  readonly judges: readonly RosterModelId[];

  /**
   Models barred because they proposed the claim, in roster order.

   Reported rather than merely subtracted, so a reading can tell a claim
   judged by five from one judged by two.
   */
  readonly barred: readonly RosterModelId[];

  /**
   Whether anyone at all may judge it.

   False means every seated model proposed the claim. Such a claim is
   reported in its own category, never dropped and never counted as judged.
   */
  readonly judgeable: boolean;
};

/**
 Works out who may re-examine a claim its authors must not judge.

 Proposers are plain strings, NOT `RosterModelId`. They are read from an
 artifact some earlier run wrote, and typing them as the current union would
 assert a guarantee the data does not carry: two ids were retired on
 2026-08-05 and artifacts naming them still exist. A retired proposer simply
 bars nobody, which is correct, and the alternative is a decoder that throws
 on history.

 @param proposers - models that proposed this claim, from attribution

 @param roster - models available to judge

 @returns Seating, with the barred models named

 @example
 ```ts
 const { judges, judgeable, } = seatJudges({ proposers, roster: RUN_ROSTER, },);
 ```
 */
export function seatJudges(
  {
    proposers,
    roster,
  }: {
    readonly proposers: readonly string[];
    readonly roster: readonly RosterModelId[];
  },
): JudgeSeating {
  /**
   Authors of the claim, as a lookup.
   */
  const authors = new Set(proposers,);

  /**
   Roster models that did not propose it.
   */
  const judges = roster.filter(function isDisinterested(modelId,): boolean {
    return !authors.has(modelId,);
  },);

  return {
    judges,
    barred: roster.filter(function isAuthor(modelId,): boolean {
      return authors.has(modelId,);
    },),
    judgeable: judges.length > 0,
  };
}

/**
 Smallest eligible population a rate may be reported over.

 Below this the scorer renders `n/a` rather than a number, matching the
 discipline already in `score-attribution.ts`. A rate over a handful of claims
 looks exactly like a rate over a thousand and invites the same weight.

 PROVISIONAL, and stated so on purpose. This floor is a round number chosen to
 refuse obviously thin populations, not a calibrated threshold.
 `LOSS_FRACTION_LIMIT` in `preservation-check.ts` reads with the same
 authority and was fitted on 50 human-graded repairs; this one has nothing
 behind it yet. Cite it as a guard, never as a measured minimum, and refit it
 once the crosscheck has produced enough judged claims to say where a rate
 stops moving.
 */
export const MIN_JUDGED_CLAIMS = 30;

//endregion Judge independence
