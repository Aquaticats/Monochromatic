import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type { BenchSlice, } from './bench-sample.ts';
import type { CommandLineOf, } from '@monochromatic-dev/translation-repair-run/ts';
import type { WidthDraw, } from './editor-width-model.ts';

//region Editor width probe draw
// WHICH HALF OF THE SAMPLE a width probe spends, named on its command line,
// and the slices that half holds.

/**
 Half of the sample spent when the caller names no draw.
 */
const DEFAULT_DRAW: WidthDraw = 'a';

/**
 Position the draw name is read from among the arguments after the flags.

 Second, after the slice count (`command-lines.ts`).
 */
const DRAW_POSITION = 1;

/**
 Position within the sample each draw takes.

 Alternate positions rather than a front and back half, so both draws stay as
 evenly spread across the corpus as the whole sample was.
 */
const DRAW_POSITIONS: Readonly<Record<WidthDraw, number>> = {
  a: 0,
  b: 1,
};

/**
 Reads which half of the sample the run spends.

 @param line - the probe's command line, read whole by `reportingRefusals`

 @returns The named draw, or draw A when none was named

 @throws {@link StatedRefusalError} when the named draw is neither half,
 rather than quietly spending draw A and reporting it under whatever was
 asked for

 @example
 ```ts
 const draw = readWidthDraw({ line, },);
 ```
 */
export function readWidthDraw({ line, }: { readonly line: CommandLineOf<'editor-width-probe'>; },): WidthDraw {
  /**
   Half of the sample this run spends, named on the command line.
   */
  const asked = line.positionals[DRAW_POSITION] ?? DEFAULT_DRAW;

  if ((asked !== 'a') && (asked !== 'b'))
    throw new StatedRefusalError({
      says: `editor width probe refused: draw must be 'a' or 'b', not '${asked}'; spending draw A `
        + 'under another name would burn the held-back half of the sample without saying so',
    },);

  return asked;
}

/**
 Takes the slices one draw spends out of the whole sample.

 SPLIT RATHER THAN REDRAWN, so the other half exists already if this one
 lands near its own null band. Taking alternate positions out of one spread
 sample keeps both halves as evenly spread as the whole.

 @param sample - whole sample, spread across the corpus

 @param draw - half to take

 @returns Slices this run spends, leaving the other half untouched

 @example
 ```ts
 const drawn = halfOfSample({ sample, draw: 'b', },);
 ```
 */
export function halfOfSample(
  {
    sample,
    draw,
  }: {
    readonly sample: readonly BenchSlice[];
    readonly draw: WidthDraw;
  },
): readonly BenchSlice[] {
  /**
   Positions this draw takes out of the sample.
   */
  const wantedPosition = DRAW_POSITIONS[draw];

  return sample.filter(function inThisDraw(
    _slice,
    at,
  ): boolean {
    return (at % 2) === wantedPosition;
  },);
}

//endregion Editor width probe draw
