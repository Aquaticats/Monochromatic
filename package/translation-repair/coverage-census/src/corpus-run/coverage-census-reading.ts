import { join, } from 'node:path';

import { runnerEntrySources, } from '../family-build-entries.ts';
import { writeFileAtomic, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type { BundleMaps, } from './coverage-bundle-maps.ts';
import {
  type Baseline,
  baselineReadingsOf,
} from './coverage-census-baseline-lines.ts';
import type { sourcesEditedSince, } from './coverage-census-commit.ts';
import {
  type CensusArguments,
  censusFileText,
} from './coverage-census-input.ts';
import { invariantThrowRowsOf, } from './coverage-census-invariant.ts';
import type { placeTally, } from './coverage-census-place.ts';
import { censusReportLines, } from './coverage-census-print.ts';
import {
  kindTotalsOf,
  sourceRowsOf,
} from './coverage-census-report.ts';
import type { CoverageTally, } from './coverage-tally.ts';

//region Coverage census reading
// Ledger T8: what the census does with a painted tally once the suite has
// run: places it on source lines, writes the census file, and prints the
// report, then the reading against each baseline. The two steps that touch
// the build and git are handed in, so a case scripts them.

/**
 Maps the tally to source lines, writes the census file and prints the
 report, then the baseline reading when one was asked for.

 @param asked - command line as read

 @param packageDirectory - package directory

 @param distDirectory - build directory

 @param bundleMaps - bundles the build holds, split by whether a map stands
 beside each

 @param head - commit built from

 @param clean - whether the files matched it

 @param passes - passing markers the suite printed

 @param tally - painted tally

 @param reportDirectory - where the census file goes

 @param baselines - earlier census files named, already read

 @param place - places the tally on source lines through the build's maps,
 passed in because it reads the build

 @param editedSince - asks git which files changed since a commit, passed in
 because it runs git

 @throws StatedRefusalError where the coverage names a bundle the build
 does not hold, finds code no test ran in a bundle with no map, or finds
 a bundle no test loaded that has no map to say which sources it carries,
 or where the census places an uncalled function outside its own source's
 stretches

 @example
 ```ts
 await reportCensus({ asked, packageDirectory, distDirectory, bundleMaps, head, clean, passes, tally, reportDirectory, baselines, place: placeTally, editedSince: sourcesEditedSince, },);
 ```
 */
export async function reportCensus(
  {
    asked,
    packageDirectory,
    distDirectory,
    bundleMaps,
    head,
    clean,
    passes,
    tally,
    reportDirectory,
    baselines,
    place,
    editedSince,
  }: {
    readonly asked: CensusArguments;
    readonly packageDirectory: string;
    readonly distDirectory: string;
    readonly bundleMaps: BundleMaps;
    readonly head: string;
    readonly clean: boolean;
    readonly passes: number;
    readonly tally: CoverageTally;
    readonly reportDirectory: string;
    readonly baselines: readonly Baseline[];
    readonly place: typeof placeTally;
    readonly editedSince: typeof sourcesEditedSince;
  },
): Promise<void> {
  /**
   Runner entry sources.
   */
  const entryFiles = runnerEntrySources();
  /**
   The tally placed on source lines.
   */
  const {
    stretches,
    invariantThrows,
    uncalled,
    loadedSources,
    unloadedBundles,
    unloadedSources,
  } = await place({
    packageDirectory,
    distDirectory,
    bundleMaps,
    tally,
    entryFiles,
  },);
  /**
   One row per source holding cold code.
   */
  const rows = sourceRowsOf({
    stretches,
    uncalled,
    entryFiles,
  },);
  /**
   Where the census goes.
   */
  const censusPath = join(
    reportDirectory,
    'census.json',
  );
  // ATOMIC, since a later census reads this file back as its baseline: a plain
  // write a full disk refuses part way leaves text cut short at the path.
  await writeFileAtomic({
    path: censusPath,
    text: censusFileText({
      head,
      clean,
      testFiles: asked.testFiles,
      passes,
      stretches,
      invariantThrows,
      uncalled,
      loadedSources,
      unloadedBundles,
      unloadedSources,
    },),
  },);
  /**
   Each baseline reading, sources edited since its commit read apart.
   */
  const baselineReadings = await baselineReadingsOf({
    baselines,
    packageDirectory,
    claimed: new Set(asked.sources,),
    stretches,
    loadedSources,
    editedSince,
  },);
  /**
   Every line of the report, then each baseline reading.
   */
  const lines = [
    ...censusReportLines({
      census: {
        head,
        clean,
        testFiles: asked.testFiles,
        passes,
        totals: kindTotalsOf({ rows, },),
        rows,
        invariantThrows: invariantThrowRowsOf({
          invariantThrows,
          entryFiles,
        },),
        uncalled,
        unloadedBundles,
        unloadedSources,
        censusPath,
      },
    },),
    ...baselineReadings.flat(),
  ];
  for (const line of lines)
    console.log(line,);
}

//endregion Coverage census reading
