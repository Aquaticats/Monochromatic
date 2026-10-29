import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type CapCensus,
  type CapCensusRow,
  type CapFlag,
  capFlagsOf,
  type ProviderCapReading,
} from './cap-census-rule.ts';

//region Cap census print
// HOW THE CAP CENSUS READS ON A TERMINAL (ledger P10): ids and numbers only,
// since a run log holds unlicensed corpus wording and nothing here reads past
// the two line kinds the census needs.

/**
 Multiplier turning a fraction into a percentage.
 */
const PERCENT = 100;

/**
 What the walk and the reads counted before the rule was applied.

 @example
 ```ts
 const tally: CapCensusTally = { logs: 3, passRunLogs: 1, samples: 40, unreadable: 0, danglingLinks: 0, unstampedLines: 2, };
 ```
 */
export type CapCensusTally = {
  /**
   Log files found under the named paths.
   */
  readonly logs: number;

  /**
   Of those, the pass-run logs read.
   */
  readonly passRunLogs: number;

  /**
   Completed calls across them.
   */
  readonly samples: number;

  /**
   Paths the walk could not read.
   */
  readonly unreadable: number;

  /**
   Dangling links the walk passed over for a name no log has, which hold nothing to read.
   */
  readonly danglingLinks: number;

  /**
   Lines left out for a stamp the logger did not write.
   */
  readonly unstampedLines: number;
};

/**
 Renders one provider's two readings.

 @param reading - provider's calls of one seat

 @returns Line for the report

 @example
 ```ts
 console.log(capCensusProviderLine({ reading, },),);
 ```
 */
export function capCensusProviderLine({ reading, }: { readonly reading: ProviderCapReading; },): string {
  /**
   Share of capped calls that ran to the cap.
   */
  const share = (reading.sinceCaps === 0)
    ? 'n/a'
    : `${((reading.atCap / reading.sinceCaps) * PERCENT).toFixed(2,)}%`;
  return `  ${reading.provider}: ${String(reading.calls,)} ${
    wordForCount({
      count: reading.calls,
      one: 'call',
      many: 'calls',
    },)
  }, p99 ${String(reading.p99,)}; since the caps `
    + `${String(reading.sinceCaps,)} ${
      wordForCount({
        count: reading.sinceCaps,
        one: 'call',
        many: 'calls',
      },)
    }, ${String(reading.atCap,)} at the cap (${share}): `
    + `${String(reading.atCapWithContent,)} with content, ${String(reading.atCapNoContent,)} with none, `
    + `${String(reading.atCapUnpaired,)} unpaired`;
}

/**
 What each flag says, in the words the report prints.
 */
const FLAG_TEXT: Readonly<Record<CapFlag, (row: CapCensusRow,) => string>> = {
  'placeholder-with-calls': function placeholderText(): string {
    return 'PLACEHOLDER WITH A DISTRIBUTION: the card names the pooled 99th and its calls now read by the rule';
  },
  'rule-off-card': function offCardText(row,): string {
    return `RULE READS ${String(row.ruleCap,)} AGAINST THE CARD'S ${String(row.cardCap,)}`;
  },
  'cuts-over-share': function cutsText(): string {
    return 'CUTS OVER ONE PERCENT on at least one provider with enough calls to say';
  },
};

/**
 Prints the census: what was read, then each seat's rows and flags.

 @param census - rule applied per seat

 @param tally - what the walk and the reads counted

 @example
 ```ts
 printCapCensus({ census, tally, },);
 ```
 */
export function printCapCensus(
  {
    census,
    tally,
  }: {
    readonly census: CapCensus;
    readonly tally: CapCensusTally;
  },
): void {
  console.log(
    `cap-census: ${String(tally.logs,)} ${
      wordForCount({
        count: tally.logs,
        one: 'log',
        many: 'logs',
      },)
    }, ${String(tally.passRunLogs,)} pass-run ${
      wordForCount({
        count: tally.passRunLogs,
        one: 'log',
        many: 'logs',
      },)
    }, `
      + `${String(tally.samples,)} completed ${
        wordForCount({
          count: tally.samples,
          one: 'call',
          many: 'calls',
        },)
      }, ${String(census.offRoster,)} on ids no card names, `
      + `${String(tally.unreadable,)} ${
        wordForCount({
          count: tally.unreadable,
          one: 'path',
          many: 'paths',
        },)
      } unreadable, `
      + `${String(tally.danglingLinks,)} dangling ${
        wordForCount({
          count: tally.danglingLinks,
          one: 'link',
          many: 'links',
        },)
      } with no log name passed over; `
      + `lines left out for a stamp the logger did not write: ${String(tally.unstampedLines,)}`,
  );
  for (const row of census.rows) {
    console.log(
      `${row.modelId}: card cap ${String(row.cardCap,)}${row.placeholder ? ' (pooled placeholder)' : ''}, `
        + `rule reads ${(row.ruleCap === 'too-few-calls') ? 'nothing (too few calls)' : String(row.ruleCap,)}`,
    );
    for (const reading of row.providers)
      console.log(capCensusProviderLine({ reading, },),);
    for (const flag of capFlagsOf({ row, },))
      console.log(`  ${FLAG_TEXT[flag](row,)}`,);
  }
  console.log(
    'A rule reading can differ from a card for reasons that are not the model: a narrower log scope than the '
      + '2026-09-09 table, calls from before the caps, or calls the cap itself cut. Read each provider\'s cut '
      + 'columns before moving a cap; completion-cap.ts records the 2026-09-28 reading.',
  );
}

//endregion Cap census print
