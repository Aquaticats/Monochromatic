import pLimit from 'p-limit';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { BenchSlice, } from './bench-sample.ts';
import {
  type SliceRounds,
  sliceProgressLine,
} from './editor-calibrate-slice.ts';

//region Editor calibrate drive
// RUNS THE SAMPLE'S SLICES, at most `overlap` of them at once, and prints one
// progress line as each finishes.

/**
 Runs every slice of the sample, admitting at most `overlap` at a time.

 WHAT EVERY SLICE PRODUCED, IN SAMPLE ORDER rather than completion order.

 ORDER MATTERS HERE AND NOWHERE ELSE IN THE RUN. Every standing the command
 reports is computed off this array, so a report that depended on which slice
 happened to finish first would not be comparable between two runs of one
 sample, which is exactly what the overlap dial measures. `allInInputOrder`
 keeps input order whatever order the work completes in, and reports the
 failure of the first slice in sample order when several slices fail.

 WHAT THE SEQUENTIAL DRIVER'S NOTE SAID, kept because it is the claim under
 test: slices ran one at a time because one slice already fans eight models
 across several stages, and stacking whole lanes was expected to queue
 behind the per-model concurrency the client enforces anyway. That was
 written before the multi-provider routing existed, and the run the
 timing work measured spends 87.2% of its round time waiting after quorum, at a mean of
 2.56 calls in flight.

 @param sample - slices every model edits, in the order their lines are numbered

 @param overlap - slices that may be in flight at once

 @param runSlice - runs one slice through the lane; handed the client by the
 caller so this stays about order and admission

 @returns Rounds of every slice, in sample order

 @example
 ```ts
 const perSlice = await driveEditorCalibrate({ sample, overlap: 4, runSlice, },);
 ```
 */
export async function driveEditorCalibrate(
  {
    sample,
    overlap,
    runSlice,
  }: {
    readonly sample: readonly BenchSlice[];
    readonly overlap: number;
    readonly runSlice: (input: { readonly slice: BenchSlice; }) => Promise<SliceRounds>;
  },
): Promise<readonly SliceRounds[]> {
  /**
   Admits at most `overlap` slices at a time.
   */
  const inFlight = pLimit(overlap,);

  return await allInInputOrder({
    members: sample.map(function admit(
      slice,
      position,
    ): Promise<SliceRounds> {
      return inFlight(async function admitted(): Promise<SliceRounds> {
        /**
         Rounds this slice produced, both seats.
         */
        const rounds = await runSlice({ slice, },);

        // PER SLICE RATHER THAN ONLY AT THE END. A whole lane per slice makes
        // this a long run, and a report that arrives only on completion is
        // unreadable while it matters most: a reader watching an outage needs
        // to know whether rounds are accumulating at all.
        console.log(sliceProgressLine({
          position,
          total: sample.length,
          slice,
          rounds,
        },),);

        return rounds;
      },);
    },),
  },);
}

//endregion Editor calibrate drive
