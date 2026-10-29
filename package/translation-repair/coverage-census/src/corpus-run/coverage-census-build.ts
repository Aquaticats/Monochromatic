import { join, } from 'node:path';

import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  namesIn,
  StatedRefusalError,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  type BundleMaps,
  bundleMapsOf,
  COVERAGE_CENSUS_TASK,
  requireUnminifiedBuild,
} from './coverage-bundle-maps.ts';
import type { ReadText, } from './coverage-census-read-text.ts';

//region Coverage census build
// Ledger T8: what the census checks of the build directory before it spends a
// suite on it. A build with no bundle, no map or a minified one costs no
// suite run (ledger M79), and a build directory that cannot be listed is the
// same mistake as one holding no bundle: the census was not run through the
// task that builds with maps first, or not from the package directory.

/**
 Lists the build directory's files, refusing a directory that cannot be
 listed as the operator's mistake it is.

 @param distDirectory - build directory

 @returns Names of its files

 @throws StatedRefusalError where the directory cannot be listed, naming the
 filesystem code and never the system's own message

 @example
 ```ts
 const built = await builtFilesOf({ distDirectory, },);
 ```
 */
async function builtFilesOf({ distDirectory, }: { readonly distDirectory: string; },): Promise<readonly string[]> {
  /**
   The directory's files, or why it could not be listed.
   */
  const listing = await namesIn({
    dir: distDirectory,
    kind: 'file',
  },);
  if (listing.kind === 'unreadable')
    throw new StatedRefusalError({
      says: `${distDirectory} could not be listed (${listing.reason}), so no build has written it there; run the `
        + `census from the package directory through mise run ${COVERAGE_CENSUS_TASK}, `
        + 'which builds with maps first',
    },);
  return listing.names;
}

/**
 Reads the build directory's bundles and refuses a build the census cannot
 read, before any suite runs.

 @param distDirectory - build directory the tests import

 @param l - logger the one progress line goes to

 @param readText - reads one bundle's text, passed in so a case scripts which of two bundles' reads ends first

 @returns Build's bundles, split by whether a map stands beside each

 @throws StatedRefusalError where the directory cannot be listed, holds no
 bundle, holds none with a map beside it, or holds a minified build

 @example
 ```ts
 const bundleMaps = await requireCoverageBuild({ distDirectory, l, readText: readUtf8Text, },);
 ```
 */
export async function requireCoverageBuild(
  {
    distDirectory,
    l,
    readText,
  }: {
    readonly distDirectory: string;
    readonly l: Logger;
    readonly readText: ReadText;
  },
): Promise<BundleMaps> {
  /**
   The build's bundles, split by whether a map stands beside each.
   */
  const bundleMaps = bundleMapsOf({
    built: await builtFilesOf({ distDirectory, },),
    distDirectory,
  },);
  /**
   Bundles without a map, named so a census that reads none of them says so.
   */
  const {
    mapped,
    unmapped,
  } = bundleMaps;
  // Before the suite, so a minified build costs no suite run (ledger M79).
  requireUnminifiedBuild({
    texts: await allInInputOrder({
      members: mapped.map(function textOf(bundle,): Promise<string> {
        return readText({
          path: join(
            distDirectory,
            bundle,
          ),
        },);
      },),
    },),
    distDirectory,
  },);
  l.info(`bundles with no source map, read only where the census must place code in them: ${(unmapped.length === 0) ? 'none' : unmapped.join(', ',)}`,);
  return bundleMaps;
}

//endregion Coverage census build
