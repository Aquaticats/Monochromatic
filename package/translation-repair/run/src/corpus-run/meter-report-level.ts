import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { MeterSample, } from './meter-sample-read.ts';
import { stampText, } from './meter-report-text.ts';

//region Meter report level
// What one provider's meter was reading, at both ends of the record.
//
// SPLIT OUT OF `meter-report.ts` so the entry holds only the wiring.

/**
 One reading that named a level for the provider being reported on.
 */
type LevelReading = {
  /**
   Epoch milliseconds the reading was taken at.
   */
  readonly at: number;

  /**
   That provider's fields on that reading.
   */
  readonly fields: readonly string[];
};

/**
 A reading, or that it named no level for this provider.
 */
type LevelLookup = LevelReading | 'no-level';

/**
 Narrows a lookup to a reading that named something.

 POSITIONAL RATHER THAN DESTRUCTURED, for the reason `isMeterState` in
 `meter-sample-read.ts` is: TypeScript refuses a type predicate naming an
 element of a binding pattern.

 @param lookup - what one reading yielded

 @returns Whether it named a level

 @example
 ```ts
 lookups.filter(namedALevel,);
 ```
 */
function namedALevel(lookup: LevelLookup,): lookup is LevelReading {
  return lookup !== 'no-level';
}

/**
 Renders what one provider's meter was reading, at both ends of the record.

 TWO ENDS RATHER THAN ONE. The last reading answers what the budget is now;
 the first says which way it moved to get there, which is the difference
 between a budget this run drained and one that was empty before it started.

 SAYS SO WHEN NOTHING WAS RECORDED. A run written before the levels were
 added carries states and no numbers, and silence there would read as a
 provider whose meter never said anything.

 @param samples - every reading, from every log

 @param provider - provider to report on

 @returns Lines describing what its meter read

 @example
 ```ts
 for (const line of levelLines({ samples, provider, },)) console.log(line,);
 ```
 */
export function levelLines(
  {
    samples,
    provider,
  }: {
    readonly samples: readonly MeterSample[];
    readonly provider: ProviderName;
  },
): readonly string[] {
  /**
   Readings that named a level for this provider, in time order.

   ATTRIBUTED BY NAME PREFIX, which the record's field names are built to
   carry: neither provider's name is a prefix of the other's.
   */
  const carried = samples
    .map(function forProvider(sample,): LevelLookup {
      /**
       This provider's level fields on that reading.
       */
      const fields = sample
        .levels
        .filter(function mine(field,): boolean {
          return field.startsWith(provider,);
        },);

      if (fields.length === 0)
        return 'no-level';

      return {
        at: sample.at,
        fields,
      };
    },)
    .filter(namedALevel,);

  /**
   Earliest reading that named a level.
   */
  const first = carried.at(0,);

  /**
   Latest reading that named a level.
   */
  const last = carried.at(-1,);

  if ((first === undefined) || (last === undefined))
    return [
      '  level: NOT RECORDED. These readings predate the meter numbers being written down, so a '
        + 'dry one here cannot be told from a threshold that was wrong about a budget that was fine',
    ];

  /**
   Earliest reading's numbers, rendered for the line.
   */
  const firstFields = first
    .fields
    .join(' ',);

  if (carried.length === 1)
    return [`  level ${stampText({ at: first.at, },)}: ${firstFields}`,];

  /**
   Latest reading's numbers, rendered the same way.
   */
  const lastFields = last
    .fields
    .join(' ',);

  return [
    `  level first ${stampText({ at: first.at, },)}: ${firstFields}`,
    `  level last ${stampText({ at: last.at, },)}: ${lastFields}`,
  ];
}

//endregion Meter report level
