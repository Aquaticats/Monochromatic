import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  countStates,
  drySpans,
  dutyCycle,
  longestDrySpan,
  seriesFor,
} from './meter-dry-span.ts';
import { levelLines, } from './meter-report-level.ts';
import { outageLines, } from './meter-report-text.ts';
import type { MeterSample, } from './meter-sample-read.ts';

//region Meter report provider
// One provider's availability across the whole record: how its readings
// fell, what share of those that answered found budget, its longest outage
// and what its meter read.
//
// SPLIT OUT OF `meter-report.ts` so the entry holds only the wiring.

/**
 Reports one provider's availability across the whole record.

 @param samples - every reading, from every log

 @param provider - provider to report on

 @example
 ```ts
 reportProvider({ samples, provider: 'hyper', },);
 ```
 */
export function reportProvider(
  {
    samples,
    provider,
  }: {
    readonly samples: readonly MeterSample[];
    readonly provider: ProviderName;
  },
): void {
  /**
   That provider's readings in time order.
   */
  const series = seriesFor({
    samples,
    provider,
  },);

  /**
   How many readings fell in each state.
   */
  const counts = countStates({ series, },);

  /**
   Fraction of answering readings that found budget, absent where none did.
   */
  const wetFraction = dutyCycle({ counts, },);

  /**
   Readings whose meter answered, which is the fraction's denominator.
   */
  const answered = counts.wet + counts.dry;

  console.log(
    `\n${provider}: wet=${String(counts.wet,)} dry=${String(counts.dry,)} `
      + `unreadable=${String(counts.unreadable,)}`,
  );
  console.log(
    (wetFraction === 'none-answered')
      ? '  spendable on NO MEASURABLE FRACTION: no reading in this record answered'
      : `  spendable on ${(wetFraction * 100).toFixed(1,)}% of readings that answered `
        + `(${String(counts.wet,)} of ${String(answered,)})`,
  );

  for (const line of outageLines({ span: longestDrySpan({ spans: drySpans({ series, },), },), },)) {
    console.log(line,);
  }

  for (const line of levelLines({
    samples,
    provider,
  },)) {
    console.log(line,);
  }
}

//endregion Meter report provider
