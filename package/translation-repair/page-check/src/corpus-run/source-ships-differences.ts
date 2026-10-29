import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { PublishDefect, } from './publish-defects.ts';
import {
  type ReadingComparison,
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
// `source-reads-differently` names the slices. A run holding a block the comparison cannot decide (a footnote in a block
// quote or a list item that the two readings end at different places, where the ruling takes the page's reading) is named
// by `source-reading-undecided`, and a run the comparison cannot make at all by `source-reading-not-compared`, so that
// no page is taken for one compared and found equal that was not. A run that ships as recorded is not compared, since
// what the site compiles is what the original is upstream.

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

 @param compareReadings - what compares the page of a run with its source, the comparison of the page check by default; a
 caller hands in another to reach the state of a page that parses and whose comparison fails

 @returns Defect naming the slices of the differing runs, one naming those of the runs with a block the comparison
 could not decide, and one naming those of the runs that could not be compared, each only when it has a run to name

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
    compareReadings = readingDifferenceOf,
  }: {
    readonly runs: readonly (readonly number[])[];
    readonly encoded: readonly number[];
    readonly slices: readonly WouldShipSlice[];
    readonly entryId: string;
    readonly l: Logger;
    readonly compareReadings?: ReadingComparison;
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
        const answer = compareReadings({
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
   Runs with a block the comparison could not decide, whether or not another block of the run differs.
   */
  const undecided = compared
    .filter(function unsure({ answer, },): boolean {
      return (answer.kind === 'undecided') || (('undecidedBlocks' in answer) && ((answer.undecidedBlocks ?? 0) > 0));
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
    ...((undecided.length === 0)
      ? []
      : [
        {
          check: 'source-reading-undecided' as const,
          message: `entry ${entryId} could not decide whether the page shows the source text of slices ${
            namedSlices({ indices: undecided.flat(), },)
          } as the source does: a footnote in a block quote or a list item ends at different places in the two readings, `
            + 'and the page keeps the reading the two-readings ruling takes, which a reader of plain Markdown cannot give',
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
