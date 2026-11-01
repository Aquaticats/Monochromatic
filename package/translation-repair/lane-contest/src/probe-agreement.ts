import { judgeRegionProbe, } from './probe-telemetry.ts';
import type { TelemetryProbeReading, } from './probe-attribution.ts';
import type { RepairVerdict, } from '@monochromatic-dev/translation-repair-adjudication/ts';

//region Probe agreement
// Comparing what the introduced-defect probe said about a repair with what the
// human said about the same repair, which is the only thing that can decide
// whether the probe should ever be allowed to block one.
//
// The comparison is made at ISSUE level rather than region level, because that
// is the level the two instruments share. A region can serve several accepted
// issues, and the human grades issues; asking "what did the human say about
// this region" has no answer when a merged envelope's issues were graded
// differently.
//
// ONE CELL OF THIS TABLE IS CLEAN AND THE REST ARE NOT, and saying so is the
// point. A repair grade of Y means "fully fixes this defect AND breaks nothing
// nearby", so Y beside a probe finding is a direct human refutation: the human
// looked at the same wording and said nothing nearby broke. N is ambiguous by
// construction, since it fires both for a repair that did not fix its target
// and for one that damaged something, and nothing in the sheet separates those.
// So `refutedByHuman` is evidence and `sharedWithHuman` is only suggestive.

/**
 What the two instruments jointly said about one round's repairs.

 @example
 ```ts
 const agreement: ProbeAgreement = scoreProbeAgainstGrades({ items, },);
 ```
 */
export type ProbeAgreement = {
  /**
   Graded issues that could be joined to a probe reading at all.
   */
  readonly joined: number;

  /**
   Joined issues where a majority of the configured roster corroborated
   introduced damage in some region serving them.
   */
  readonly probeFlagged: number;

  /**
   Flagged issues the human graded `fixes`, meaning they read the same wording
   and said it breaks nothing nearby.

   These are the probe's demonstrable false positives, and the count a gate
   proposal has to answer for: each one is a correct repair the gate would
   have discarded.
   */
  readonly refutedByHuman: number;

  /**
   Flagged issues the human graded `does-not-fix`.

   SUGGESTIVE, NOT CONFIRMING. The sheet's N fires both for a repair that
   failed to fix its target and for one that broke something, so this does not
   establish that the human saw the damage the probe claimed.
   */
  readonly sharedWithHuman: number;

  /**
   Flagged issues the human left unscored, which prove nothing either way.
   */
  readonly flaggedUnscored: number;

  /**
   Issues the probe did NOT flag that the human graded `does-not-fix`, an
   upper bound on what the probe missed, inflated by the same ambiguity in N.
   */
  readonly unflaggedFailures: number;
};

/**
 One graded issue paired with the probe reading of its chunk.

 @example
 ```ts
 const item: ProbeAgreementItem = { verdict: 'fixes', reading, };
 ```
 */
export type ProbeAgreementItem = {
  /**
   What the human said about this issue's repair.
   */
  readonly verdict: RepairVerdict;

  /**
   Probe reading for the regions serving it, absent where the chunk was never
   probed.
   */
  readonly reading?: TelemetryProbeReading;
};

/**
 Whether the probe flagged damage in any region serving one issue.

 A single majority-flagged region is enough, because a gate would have
 rejected the candidate on that one region's verdict.

 @param reading - probe reading for this issue

 @returns Whether a gate would have blocked on this issue's regions

 @example
 ```ts
 const flagged = probeFlaggedIssue({ reading, },);
 ```
 */
export function probeFlaggedIssue(
  { reading, }: { readonly reading: TelemetryProbeReading; },
): boolean {
  return reading.regions
    .some(function isFlagged(tally,) {
      return judgeRegionProbe({
        tally,
        configuredProbers: reading.configuredProbers,
      },)
        === 'majority-introduced';
    },);
}

/**
 One graded item reduced to the two facts the agreement table joins on.

 Named rather than inferred, because an inferred object literal carries
 writable properties and every cell count reading it then takes a mutable
 parameter it never mutates.
 */
type JoinedReading = Readonly<{
  /**
   Human verdict on the repair.
   */
  verdict: RepairVerdict;

  /**
   Whether the probe flagged this item.
   */
  flagged: boolean;
}>;

/**
 Scores the probe against the human repair grades.

 @param items - graded issues paired with their probe readings

 @returns Joint counts, with the clean cell separated from the ambiguous ones

 @example
 ```ts
 const agreement = scoreProbeAgainstGrades({ items, },);
 ```
 */
export function scoreProbeAgainstGrades(
  { items, }: { readonly items: readonly ProbeAgreementItem[]; },
): ProbeAgreement {
  /**
   Items carrying both a probe reading and a place in the table.
   */
  const joined: readonly JoinedReading[] = items.flatMap(function toJoined(item,) {
    /**
     Probe reading of this item, when its chunk was probed.
     */
    const { reading, } = item;
    if (reading === undefined)
      return [];
    return [
      {
        verdict: item.verdict,
        flagged: probeFlaggedIssue({ reading, },),
      },
    ];
  },);

  /**
   Counts joined items matching a flag state and verdict.

   @param flagged - whether the probe flagged the issue

   @param verdict - human verdict to match

   @returns Items in that cell

   @example
   ```ts
   countCell({ flagged: true, verdict: 'fixes', },);
   ```
   */
  function countCell(
    {
      flagged,
      verdict,
    }: {
      readonly flagged: boolean;
      readonly verdict: RepairVerdict;
    },
  ): number {
    return joined.filter(function matches(entry,) {
      return (entry.flagged === flagged) && (entry.verdict === verdict);
    },)
      .length;
  }

  return {
    joined: joined.length,
    probeFlagged: joined.filter(function wasFlagged(entry,) {
      return entry.flagged;
    },)
      .length,
    refutedByHuman: countCell({
      flagged: true,
      verdict: 'fixes',
    },),
    sharedWithHuman: countCell({
      flagged: true,
      verdict: 'does-not-fix',
    },),
    flaggedUnscored: countCell({
      flagged: true,
      verdict: 'unscored',
    },),
    unflaggedFailures: countCell({
      flagged: false,
      verdict: 'does-not-fix',
    },),
  };
}

//endregion Probe agreement
