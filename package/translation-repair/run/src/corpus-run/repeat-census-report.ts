import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { failedControlsOf, } from './repeat-census-control.ts';
import {
  packageControlOf,
  packageGroupsOfPage,
} from './repeat-census-package.ts';
import { censusOfPage, } from './repeat-census-page.ts';
import {
  type EntryCensus,
  lineControlOf,
  summarise,
} from './repeat-census-summary.ts';

//region Repeat census report
// What the repeat-census command prints, and when it declines. The command
// (`repeat-census.ts`) reads the corpus and hands the pages here, so the part
// that decides the exit is tested without a corpus.

/**
 Runs both readings of the census over one page.

 @param id - entry id, the directory name under `people/`

 @param text - page text

 @returns The entry's line census and the package's own groups for it

 @example
 ```ts
 const entry = censusEntryOf({ id: 'TianqiChen666', text, },);
 ```
 */
export function censusEntryOf(
  {
    id,
    text,
  }: {
    readonly id: string;
    readonly text: string;
  },
): EntryCensus {
  return {
    id,
    census: censusOfPage({ text, },),
    groups: packageGroupsOfPage({ text, },),
  };
}

/**
 Prints the census summary whole, then declines when a positive control read
 false or could not be read, so the command exits non-zero naming each failed
 control and a reader still has every count to look at.

 @param entries - each entry's two readings

 @throws {@link StatedRefusalError} when any positive control of either reading read false or the control entry is absent

 @example
 ```ts
 reportRepeatCensus({ entries, },);
 ```
 */
export function reportRepeatCensus({ entries, }: { readonly entries: readonly EntryCensus[]; },): void {
  console.log(JSON.stringify(
    summarise({ entries, },),
    null,
    1,
  ),);

  /**
   Names of the controls that read false or went unread, in both readings.
   */
  const failed = failedControlsOf({
    readings: {
      lines: lineControlOf({ entries, },),
      package: packageControlOf({ entries, },),
    },
  },);
  if (failed.length > 0) {
    throw new StatedRefusalError({
      says: `the positive controls of the census did not hold: ${failed.join('; ',)}. `
        + 'The report was printed in full, and its counts are not to be trusted until every control reads true.',
    },);
  }
}

//endregion Repeat census report
