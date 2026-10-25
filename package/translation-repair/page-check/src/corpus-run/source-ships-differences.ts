import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';
import type { PublishDefect, } from './publish-defects.ts';
import {
  type ReadingDifference,
  readingDifferenceOf,
} from './source-reading-difference.ts';
import {
  namedSlices,
  runBodyOf,
} from './source-ships-runs.ts';
import type { WouldShipSlice, } from './would-ship-text.ts';

//region Source ships differences
// THE DEFECTS FOR A PAGE THAT SHOWS THE ORIGINAL OF A RUN DIFFERENTLY, OR THAT CANNOT BE COMPARED WITH IT. The composition
// writes a run as literal text when the strict grammar refuses it as recorded (`source-ships-compose.ts`), and the
// encoder (`source-ships-encoding.ts`) is built so that a reader of the page sees the text a reader of the source sees.
// Where it does not, the page still ships (a run always ships, the owner, 2026-09-27) and says so: one defect
// `source-reads-differently` names the slices. A run the comparison cannot make is named by the defect
// `source-reading-not-compared`, so that a page which parses and cannot be read against its source is never taken for
// one compared and found equal. A run that ships as recorded is not compared, since what the site compiles is what the
// original is upstream.

/**
 What comparing a run came to when the comparison could not be made.
 */
type NotCompared = {
  readonly kind: 'not-compared';
};

/**
 One run the page carries as literal text, and what comparing it came to.
 */
type ComparedRun = {
  readonly run: readonly number[];
  readonly answer: NotCompared | ReadingDifference;
};

/**
 Reports the runs written as literal text that a reader of the page sees differently from the source, and those that
 could not be compared.

 @param runs - slice indices of every run shipping its original, in page order

 @param encoded - slices the page carries as literal text

 @param slices - every slice's reading, in comparison-row order

 @param entryId - person entry being published

 @param l - logger, tagged by the caller with this entry

 @returns A defect naming the slices of the differing runs and a defect naming those of the runs that could not be
 compared, each only when it has a run to name

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
   Runs the page carries as literal text, each with what comparing it came to.
   */
  const compared = runs
    .filter(function writtenAsText(run,): boolean {
      return run.some(function written(slice,): boolean {
        return encoded.includes(slice,);
      },);
    },)
    .map(function answerFor(run,): ComparedRun {
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
        return {
          run,
          answer,
        };
      }
      catch (error) {
        log.warn(
          `entry ${entryId}: the reading of slices ${namedSlices({ indices: run, },)} could not be compared (${
            Error.isError(error,) ? error.name : typeof error
          })`,
        );
        return {
          run,
          answer: { kind: 'not-compared', },
        };
      }
    },);
  /**
   Runs the page carries as literal text whose reading differs from the source's.
   */
  const differing = compared
    .filter(function differs({ answer, },): boolean {
      return answer.kind === 'differs';
    },)
    .map(function runOf({ run, },): readonly number[] {
      return run;
    },);
  /**
   Runs the comparison could not be made for.
   */
  const notCompared = compared
    .filter(function unread({ answer, },): boolean {
      return answer.kind === 'not-compared';
    },)
    .map(function runOf({ run, },): readonly number[] {
      return run;
    },);
  return [
    ...((differing.length === 0)
      ? []
      : [
        {
          check: 'source-reads-differently' as const,
          message: `entry ${entryId} shows the source text of slices ${
            namedSlices({ indices: differing.flat(), },)
          } differently from the source: a reader of the page sees other text than a reader of the original`,
        },
      ]),
    ...((notCompared.length === 0)
      ? []
      : [
        {
          check: 'source-reading-not-compared' as const,
          message: `entry ${entryId} could not compare the source text of slices ${
            namedSlices({ indices: notCompared.flat(), },)
          } with the page that carries it: a reader of the page may see other text than a reader of the original`,
        },
      ]),
  ];
}

//endregion Source ships differences
