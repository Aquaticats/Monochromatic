import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  baselineReportLines,
} from './coverage-census-print.ts';
import {
  type BaselineCensus,
  baselineStatusesOf,
  coldSinceOf,
  editedClaimsOf,
  emptyClaimsOf,
} from './coverage-census-baseline.ts';
import type { sourcesEditedSince, } from './coverage-census-commit.ts';
import type { CensusStretch, } from './coverage-census-report.ts';

//region Coverage census baseline lines
// Ledger T8: the reading of this run against each earlier census the command
// line named, one set of report lines per baseline. Split from the report
// (`coverage-census-reading.ts`) so the git question it asks, which sources
// were edited since the baseline's commit, is handed in and scripted.

/**
 An earlier census named on the command line, read.

 @example
 ```ts
 const baseline: Baseline = { path: 'census.json', census, };
 ```
 */
export type Baseline = {
  /**
   File named.
   */
  readonly path: string;

  /**
   What it recorded.
   */
  readonly census: BaselineCensus;
};

/**
 Reads this run against each baseline, a source edited since the baseline's
 commit read apart.

 @param baselines - earlier census files named, already read

 @param packageDirectory - package directory the edits are asked about

 @param claimed - sources the batch claims, empty for every source

 @param stretches - this run's cold stretches

 @param loadedSources - sources this run's loaded bundles carry

 @param editedSince - asks git which files of the work tree changed since a
 commit, passed in because it runs git

 @returns Each baseline's report lines, one list per baseline, in the order
 the baselines were named

 @throws Whatever `editedSince` throws

 @example
 ```ts
 const readings = await baselineReadingsOf({ baselines, packageDirectory, claimed, stretches, loadedSources, editedSince: sourcesEditedSince, },);
 ```
 */
export async function baselineReadingsOf(
  {
    baselines,
    packageDirectory,
    claimed,
    stretches,
    loadedSources,
    editedSince,
  }: {
    readonly baselines: readonly Baseline[];
    readonly packageDirectory: string;
    readonly claimed: ReadonlySet<string>;
    readonly stretches: readonly CensusStretch[];
    readonly loadedSources: ReadonlySet<string>;
    readonly editedSince: typeof sourcesEditedSince;
  },
): Promise<readonly (readonly string[])[]> {
  return await allInInputOrder({
    members: baselines.map(async function baselineLines({
      path,
      census: baseline,
    },): Promise<readonly string[]> {
      /**
       Files of the work tree changed since the baseline's commit, named as
       the census names sources, whose baseline lines name other code now.
       */
      const edited = await editedSince({
        packageDirectory,
        head: baseline.head,
      },);
      return baselineReportLines({
        path,
        head: baseline.head,
        statuses: baselineStatusesOf({
          baseline: baseline.stretches,
          current: stretches,
          loadedSources,
          sources: claimed,
          edited,
        },),
        coldSince: coldSinceOf({
          baseline,
          current: stretches,
          sources: claimed,
          edited,
        },),
        emptyClaims: emptyClaimsOf({
          baseline,
          edited,
          current: stretches,
          loadedSources,
          sources: claimed,
        },),
        editedClaims: editedClaimsOf({
          baseline,
          edited,
          current: stretches,
          loadedSources,
          sources: claimed,
        },),
      },);
    },),
  },);
}

//endregion Coverage census baseline lines
