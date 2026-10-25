import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';
import type { PublishDefect, } from './publish-defects.ts';
import { readingDifferenceOf, } from './source-reading-difference.ts';
import {
  namedSlices,
  runBodyOf,
} from './source-ships-runs.ts';
import type { WouldShipSlice, } from './would-ship-text.ts';

//region Source ships differences
// THE DEFECT FOR A PAGE THAT SHOWS THE ORIGINAL OF A RUN DIFFERENTLY. The composition writes a run as literal text when
// the strict grammar refuses it as recorded (`source-ships-compose.ts`), and the encoder (`source-ships-encoding.ts`)
// is built so that a reader of the page sees the text a reader of the source sees. Where it does not, the page still
// ships (a run always ships, the owner, 2026-09-27) and says so: one defect `source-reads-differently` names the slices.
// A run that ships as recorded is not compared, since what the site compiles is what the original is upstream.

/**
 Reports the runs written as literal text that a reader of the page sees differently from the source.

 @param runs - slice indices of every run shipping its original, in page order

 @param encoded - slices the page carries as literal text

 @param slices - every slice's reading, in comparison-row order

 @param entryId - person entry being published

 @param l - logger, tagged by the caller with this entry

 @returns One defect naming the slices of the differing runs, none when every run reads as its source does or cannot be
 compared

 @example
 ```ts
 const defects = differenceDefectsOf({ runs, encoded: [88, 89,], slices, entryId: 'BookshopCat', l, },);
 ```
 */
export function differenceDefectsOf(
  {
    runs,
    encoded,
    slices,
    entryId,
    l,
  }: {
    readonly runs: readonly (readonly number[])[];
    readonly encoded: readonly number[];
    readonly slices: readonly WouldShipSlice[];
    readonly entryId: string;
    readonly l: Logger;
  },
): readonly PublishDefect[] {
  /**
   Logger naming this function.
   */
  const log = tagged({
    l,
    tag: differenceDefectsOf.name,
  },);
  /**
   Runs the page carries as literal text whose reading differs from the source's.
   */
  const differing = runs.filter(function differs(run,): boolean {
    if (!run.some(function written(slice,): boolean {
      return encoded.includes(slice,);
    },))
      return false;
    try {
      /**
       What the comparison says of the run.
       */
      const answer = readingDifferenceOf({
        source: runBodyOf({
          run,
          slices,
          encoding: 'as-written',
        },),
        carried: runBodyOf({
          run,
          slices,
          encoding: 'literal',
        },),
      },);
      if (answer.kind === 'undecided') {
        log.info(`entry ${entryId}: the reading of slices ${namedSlices({ indices: run, },)} is not compared: ${answer.reason}`,);
      }
      return answer.kind === 'differs';
    }
    catch (error) {
      log.warn(
        `entry ${entryId}: the reading of slices ${namedSlices({ indices: run, },)} could not be compared (${
          Error.isError(error,) ? error.name : typeof error
        })`,
      );
      return false;
    }
  },);
  if (differing.length === 0)
    return [];
  return [
    {
      check: 'source-reads-differently',
      message: `entry ${entryId} shows the source text of slices ${
        namedSlices({ indices: differing.flat(), },)
      } differently from the source: a reader of the page sees other text than a reader of the original`,
    },
  ];
}

//endregion Source ships differences
