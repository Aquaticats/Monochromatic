import type { RepairAttemptRecord, } from './repair-benchmark.ts';
import type { SeedDetectionVerdict, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Repair scorecard
// Aggregation of graded repair attempts, split out of the benchmark runner
// when that file reached its line budget. The runner dispatches and records;
// this decides what the records MEAN, which is a separate job and the one
// every milestone verdict quotes.
/**
 Milestone-two scorecard.

 @example
 ```ts
 const scorecard = computeRepairScorecard({ records, },);
 ```
 */
export type RepairScorecard = {
  /**
   Entries dispatched (not skipped).
   */
  readonly dispatchedEntries: number;

  /**
   Fraction of entries dispatched before the budget cut.
   */
  readonly coverage: number;

  /**
   Planted seeds across dispatched entries, detection's denominator.
   */
  readonly plantedSeeds: number;

  /**
   Planted seeds with an accepted issue anchored at their region.
   */
  readonly detectedSeeds: number;

  /**
   Detection rate: detected over planted;
   the gap between this and the repair rate is the editor's share of
   every miss.
   */
  readonly seedDetectionRate: number;

  /**
   Planted seeds the panel saw and declined on protective grounds.

   Not detection failures. The house policy instructs the pipeline never to
   restore a suicide method or a drug name and dosage even when the original
   states it plainly, while this benchmark plants seeds by deleting published
   English and scores restoration. Where the two meet, declining is the
   pipeline obeying its own rules, and counting it as a miss would score
   correct behavior as failure.
   */
  readonly policyDeclinedSeeds: number;

  /**
   Detection rate over seeds the pipeline was actually free to repair.

   Published BESIDE {@link RepairScorecard.seedDetectionRate} rather than
   replacing it: both are true, and a verdict has to say which one it cites.
   They differ only when `policyDeclinedSeeds` is above zero.
   */
  readonly seedDetectionRateExcludingPolicy: number;

  /**
   Planted seeds a judge quorum ruled NOT derivable from the Chinese.

   Also not detection failures, and for a sharper reason than the policy
   declines: there is nothing in the source to notice missing. This benchmark
   plants seeds by deleting published English, and published English is free
   to carry a translator's addition the original never stated. Deleting one
   creates a hole no reader of the Chinese could find, so counting it against
   detection scores the pipeline on a question it was never asked.

   Only `not-derivable` counts here. `partially-derivable` leaves something in
   the source to notice, and an unjudged seed defaults to `derivable`, so an
   unheard probe can never excuse a miss.
   */
  readonly nonDerivableSeeds: number;

  /**
   Detection rate over seeds that were both repairable and derivable.

   Published BESIDE the other two rather than replacing them, on the same
   reasoning: all three are true and a verdict has to say which it cites.
   */
  readonly seedDetectionRateExcludingUnfair: number;

  /**
   Seeds the bilingual judge ensemble reached a quorum verdict on,
   the zh-anchored rate's denominator.
   */
  readonly judgedSeeds: number;

  /**
   Judged seeds ruled fully restored.
   */
  readonly restoredSeeds: number;

  /**
   Judged seeds ruled partially restored.
   */
  readonly partialSeeds: number;

  /**
   THE go/no-go number: judge-restored over judged seeds,
   anchored on the Chinese source.
   */
  readonly seededRepairRate: number;

  /**
   Judge-restored-or-partial over judged seeds;
   the lenient companion to the strict rate.
   */
  readonly seededRepairRateLenient: number;

  /**
   Lexical-overlap grade's measurable-seed denominator, for comparison.
   */
  readonly lexicalUniverse: number;

  /**
   Lexical-overlap seeds graded restored.
   */
  readonly lexicalRestoredSeeds: number;

  /**
   Lexical-overlap repair rate, the retired grader kept for comparison.
   */
  readonly lexicalRepairRate: number;

  /**
   Runs per completion status.
   */
  readonly statusCounts: Readonly<Record<string, number>>;
};

/**
 Aggregates graded repair attempts into the milestone-two scorecard.

 @param records - graded attempts in run order

 @returns Scorecard over dispatched attempts with honest coverage

 @example
 ```ts
 const scorecard = computeRepairScorecard({ records, },);
 ```
 */
export function computeRepairScorecard(
  { records, }: { readonly records: readonly RepairAttemptRecord[]; },
): RepairScorecard {
  /**
   Attempts that actually dispatched.
   */
  const dispatched = records.filter(function isDispatched(record,) {
    return record.outcomeKind !== 'skipped';
  },);

  /**
   Measurable seed grades across dispatched attempts.
   */
  const grades = dispatched.flatMap(function toGrades(record,) {
    return Object
      .values(record.seedGrades,)
      .filter(function isMeasurable(grade,) {
        return grade.measurable;
      },);
  },);

  /**
   Grades that reached the restoration threshold.
   */
  const restored = grades.filter(function isRestored(grade,) {
    return grade.restored;
  },);

  /**
   Detection verdicts across dispatched attempts.
   */
  const detections = dispatched.flatMap(function toDetections(record,) {
    return Object.values(record.seedDetection,);
  },);

  /**
   Seeds whose region carried an accepted issue.
   */
  const detected = detections.filter(function isDetected(verdict,) {
    return verdict === 'accepted';
  },);

  /**
   Seeds the panel saw and declined on protective grounds, which the house
   policy makes correct behavior rather than a miss.
   */
  const policyDeclined = detections.filter(function isProtective(verdict,) {
    return verdict === 'declined-protective';
  },);

  /**
   Seeds the pipeline was free to repair at all.
   */
  const repairableSeeds = detections.length - policyDeclined.length;

  /**
   Seeds a judge quorum ruled the Chinese does not license at all.

   `judged` is required as well as the verdict. An unjudged seed already
   defaults to `derivable`, so this is belt and braces, and it keeps the
   count meaning what its name says: seeds someone actually ruled on.
   */
  const nonDerivable = dispatched.flatMap(function toDerivability(record,) {
    return Object.values(record.seedDerivability,);
  },)
    .filter(function isUnrecoverable(seed,) {
      return seed.judged && (seed.verdict === 'not-derivable');
    },);

  /**
   Seeds it is fair to score detection on: repairable AND derivable.

   JOINED PER SEED rather than subtracting both counts from the total. A seed
   can be policy-declined AND non-derivable at once, and subtracting would
   remove it twice, shrinking the denominator and making the rate too
   generous by the size of the overlap. The two records are keyed by the same
   seed id, so the join costs nothing and the shortcut has no excuse.
   */
  const fairSeeds = dispatched.reduce(
    function countFair(
      total: number,
      record,
    ): number {
      /**
       Seeds of this record that neither exclusion removes.
       */
      const fair = Object
        .entries(record.seedDetection,)
        .filter(function isFair([seedId, verdict,]: ReadonlyEntry<string, SeedDetectionVerdict>,): boolean {
          if (verdict === 'declined-protective')
            return false;

          /**
           This seed's derivability, absent when the probe ran over no
           references, which counts as derivable rather than as unfair.
           */
          const seed = record.seedDerivability[seedId];
          return (seed === undefined)
            || (!seed.judged)
            || (seed.verdict !== 'not-derivable');
        },);

      return total + fair.length;
    },
    0,
  );

  /**
   Judge verdicts with a quorum ruling across dispatched attempts.
   */
  const judgments = dispatched.flatMap(function toJudgments(record,) {
    return Object
      .values(record.seedJudgments,)
      .filter(function isJudged(judgment,) {
        return judgment.judged;
      },);
  },);

  /**
   Judged seeds ruled fully restored.
   */
  const judgeRestored = judgments.filter(function isRestored(judgment,) {
    return judgment.verdict === 'restored';
  },);

  /**
   Judged seeds ruled restored or partial.
   */
  const judgeLenient = judgments.filter(function isLenient(judgment,) {
    return (judgment.verdict === 'restored') || (judgment.verdict === 'partial');
  },);

  /**
   Runs per completion status; a map until handed back, as every record
   filled by a key is (ledger B77).
   */
  const statusCounts = new Map<string, number>();
  for (const record of dispatched) {
    if (record.status === undefined)
      continue;
    statusCounts.set(
      record.status,
      (statusCounts.get(record.status,) ?? 0) + 1,
    );
  }

  return {
    dispatchedEntries: dispatched.length,
    coverage: records.length === 0 ? 1 : dispatched.length / records.length,
    plantedSeeds: detections.length,
    detectedSeeds: detected.length,
    seedDetectionRate: detections.length === 0 ? 0 : detected.length / detections.length,
    policyDeclinedSeeds: policyDeclined.length,
    seedDetectionRateExcludingPolicy: repairableSeeds === 0
      ? 0
      : detected.length / repairableSeeds,
    nonDerivableSeeds: nonDerivable.length,
    seedDetectionRateExcludingUnfair: fairSeeds === 0
      ? 0
      : detected.length / fairSeeds,
    judgedSeeds: judgments.length,
    restoredSeeds: judgeRestored.length,
    partialSeeds: judgeLenient.length - judgeRestored.length,
    seededRepairRate: judgments.length === 0 ? 0 : judgeRestored.length / judgments.length,
    seededRepairRateLenient: judgments.length === 0 ? 0 : judgeLenient.length / judgments.length,
    lexicalUniverse: grades.length,
    lexicalRestoredSeeds: restored.length,
    lexicalRepairRate: grades.length === 0 ? 0 : restored.length / grades.length,
    statusCounts: Object.fromEntries(statusCounts,),
  };
}

//endregion Repair scorecard
