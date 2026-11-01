import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { BenchmarkEntry, } from '@monochromatic-dev/translation-repair-archive/ts';
import type { RepairScorecard, } from '@monochromatic-dev/translation-repair-repair/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type { RecallChoice, } from './recall-benchmark-choose.ts';

//region Recall benchmark report
// WHAT THE RECALL BENCHMARK PRINTS, and the one refusal that replaces a
// scorecard no rate of which measures anything.

/**
 Decimal places rates are reported to.
 */
const RATE_DECIMALS = 3;

/**
 Opening line: the tip, how many entries and seeds were chosen, how they fall
 across the bands and how long the run may take.

 @param tip - pipeline commit recorded into the scorecard

 @param choice - entries chosen and each band's count

 @param budgetMs - wall budget of the whole run

 @returns The line, without a newline

 @example
 ```ts
 console.log(recallStartLine({ tip, choice, budgetMs: 43_200_000, },),);
 ```
 */
export function recallStartLine(
  {
    tip,
    choice,
    budgetMs,
  }: {
    readonly tip: string;
    readonly choice: RecallChoice;
    readonly budgetMs: number;
  },
): string {
  /**
   Entries the run will seed.
   */
  const { chosen, } = choice;

  /**
   Total seeds this run will plant, the detection denominator.
   */
  const plannedSeeds = chosen
    .reduce(
    function addSeeds(
      sum: number,
      entry: BenchmarkEntry,
    ): number {
        /**
         Seeds this entry plants.
         */
      const { seeds, } = entry;
      return sum + seeds.length;
    },
    0,
    );

  return `START tip=${tip} entries=${String(chosen.length,)} seeds=${String(plannedSeeds,)} `
    + `perBand=${JSON.stringify(choice.perBand,)} budget=${String(budgetMs,)}ms`;
}

/**
 Line a setup check ends on when it found the benchmark ready to run.

 @param tip - pipeline commit recorded into the scorecard

 @param chosen - entries the run would seed

 @returns The line, without a newline

 @example
 ```ts
 console.log(recallPlanLine({ tip, chosen, },),);
 ```
 */
export function recallPlanLine(
  {
    tip,
    chosen,
  }: {
    readonly tip: string;
    readonly chosen: readonly BenchmarkEntry[];
  },
): string {
  return `PLAN ok tip=${tip} client=constructed entries=${
    chosen
      .map(function toId(entry,): string {
        return entry.entryId;
      },)
      .join(',',)
  }`;
}

/**
 Refuses a scorecard whose denominators are zero, since none of its rates then
 measures anything.

 A ZERO DENOMINATOR IS NOT A ZERO RATE. The scorecard prints 0 for a recall
 over no seeds and a coverage over no attempts, which reads like a measured
 zero to anyone who does not check the counts first; the record is already
 kept and the run refuses here rather than print such a line.

 @param scorecard - aggregate of the run

 @param keptAt - where the scorecard was kept, so the refusal can say so

 @throws {@link StatedRefusalError} when no entry was dispatched or no seed
 was planted

 @example
 ```ts
 refuseUnmeasuredScorecard({ scorecard, keptAt, },);
 ```
 */
export function refuseUnmeasuredScorecard(
  {
    scorecard,
    keptAt,
  }: {
    readonly scorecard: RepairScorecard;
    readonly keptAt: string;
  },
): void {
  if ((scorecard.dispatchedEntries === 0) || (scorecard.plantedSeeds === 0))
    throw new StatedRefusalError({
      says: `the bench dispatched ${String(scorecard.dispatchedEntries,)} ${
        wordForCount({
          count: scorecard.dispatchedEntries,
          one: 'entry',
          many: 'entries',
        },)
      } and planted ${
        String(scorecard.plantedSeeds,)
      } ${
        wordForCount({
          count: scorecard.plantedSeeds,
          one: 'seed',
          many: 'seeds',
        },)
      }, so none of its rates measures anything; the scorecard is kept at ${keptAt}`,
    },);
}

/**
 Lines reporting a measured scorecard.

 @param scorecard - aggregate of the run

 @param keptAt - where the scorecard was kept

 @returns One line per entry of the report, in print order

 @example
 ```ts
 for (const line of recallScorecardLines({ scorecard, keptAt, },))
   console.log(line,);
 ```
 */
export function recallScorecardLines(
  {
    scorecard,
    keptAt,
  }: {
    readonly scorecard: RepairScorecard;
    readonly keptAt: string;
  },
): readonly string[] {
  /**
   The lines that print whatever the counts are.
   */
  const lines: string[] = [
    `SCORECARD dispatched=${String(scorecard.dispatchedEntries,)} coverage=${
      scorecard.coverage
        .toFixed(RATE_DECIMALS,)
    } planted=${String(scorecard.plantedSeeds,)} detected=${
      String(scorecard.detectedSeeds,)
    } detectionRate=${
      scorecard.seedDetectionRate
        .toFixed(RATE_DECIMALS,)
    } policyDeclined=${String(scorecard.policyDeclinedSeeds,)} detectionRateExcludingPolicy=${
      scorecard.seedDetectionRateExcludingPolicy
        .toFixed(RATE_DECIMALS,)
    }`,
    `SCORECARD kept at ${keptAt}`,
  ];

  // Printed beside the raw rate rather than left in the JSON. Attributing a
  // miss to the house policy instead of to the critics is the whole reason
  // both numbers are computed, and a driver that prints only the raw one hands
  // the reader a recall figure with no way to see whether the pipeline failed
  // to detect a seed or correctly refused to treat it as a defect.
  if (scorecard.policyDeclinedSeeds > 0)
    lines.push(
      'NOTE policyDeclined counts seeds the panel ruled a source defect at the '
        + 'seed region rather than a translation error. Those are the policy '
        + 'working, not recall failing, which is why the excluding-policy rate '
        + 'sits beside the raw one instead of replacing it.',
    );

  // The same reasoning one step further out, and printed ALWAYS rather than
  // only when nonzero. A zero here is a real reading: it says the probe ran and
  // found every deleted sentence recoverable from the Chinese. Printing it only
  // when nonzero would make "the probe never ran" and "nothing was unfair" look
  // identical, which is the failure this whole run keeps rediscovering.
  lines.push(
    `DERIVABILITY nonDerivable=${String(scorecard.nonDerivableSeeds,)} `
      + `detectionExcludingUnfair=${
      scorecard.seedDetectionRateExcludingUnfair
        .toFixed(RATE_DECIMALS,)
    }`,
    `REPAIR judged=${String(scorecard.judgedSeeds,)} restored=${
      String(scorecard.restoredSeeds,)
    } partial=${String(scorecard.partialSeeds,)} strict=${
      scorecard.seededRepairRate
        .toFixed(RATE_DECIMALS,)
    } lenient=${
      scorecard.seededRepairRateLenient
        .toFixed(RATE_DECIMALS,)
    }`,
  );
  return lines;
}

//endregion Recall benchmark report
