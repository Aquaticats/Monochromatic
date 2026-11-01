import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { taskPathOf, } from '@monochromatic-dev/translation-repair-run/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Coverage bundle maps
// Ledger T8: which of the build's bundles carry a source map. The coverage
// census places cold code on source lines through the map beside each bundle,
// so a build with no map at all is the normal build, which it cannot read.
//
// A BUNDLE WITHOUT A MAP IS NOT A FAULT BY ITSELF. Rolldown writes no map for a
// chunk holding only import and export statements, since nothing in it maps
// to a source; the package's `index.mjs` became one on 2026-09-29, when the
// last small values rolldown had inlined into it left the package build
// (ledger B31), and the census, which then refused any bundle without a map,
// refused the whole build. The census now needs a map only where it places
// code, and refuses by name where one is missing there.

/**
 The task that builds with maps first and then runs the census, which every refusal about the build points to.
 */
export const COVERAGE_CENSUS_TASK: string = taskPathOf({ command: 'coverage-census', },);

/**
 A build's bundles, split by whether a source map stands beside each.
 */
export type BundleMaps = {
  /**
   Bundles with a map, sorted.
   */
  readonly mapped: readonly string[];

  /**
   Bundles without one, sorted.
   */
  readonly unmapped: readonly string[];
};

/**
 Splits the bundles a build directory holds by whether a source map stands
 beside each.

 The census reads this before the suite runs, and it refuses only a build
 with no map at all. Before 6e631988b, which wrote this, the census refused
 any bundle without a map at the same point, before the suite. Whether such
 a bundle holds code no test ran is known only from the coverage, so that
 refusal (`requireMapFor`) now comes after the suite, and a build it refuses
 has cost one suite run. Unless rolldown leaves the map off a chunk holding
 code, such a bundle can only come from a build writing into the directory
 during the census, since the coverage build cleans the directory before it
 writes (`cleanDir` in `nodeConfig` of `@monochromatic-dev/config-rolldown`,
 which rolldown 1.2.9 documents as cleaning the output directory before
 emitting output).

 @param built - file names the build directory holds

 @param distDirectory - build directory, named by the refusal

 @returns The bundles with a map and those without

 @throws {@link StatedRefusalError} when the directory holds no bundle, which
 no build wrote, or no bundle has a map, which is the normal build and not
 one the census can read

 @example
 ```ts
 const { mapped, unmapped, } = bundleMapsOf({ built: ['nap.mjs', 'nap.mjs.map', 'index.mjs',], distDirectory, },);
 ```
 */
export function bundleMapsOf(
  {
    built,
    distDirectory,
  }: {
    readonly built: readonly string[];
    readonly distDirectory: string;
  },
): BundleMaps {
  /**
   Names the directory holds, for the lookups.
   */
  const names = new Set(built,);
  /**
   Its bundles, sorted.
   */
  const bundles = textsInCodePointOrder({ texts: built
    .filter(function isBundle(name,): boolean {
      return name.endsWith('.mjs',);
    },), },);
  /**
   Bundles with a map beside them.
   */
  const mapped = bundles.filter(function hasMap(bundle,): boolean {
    return names.has(`${bundle}.map`,);
  },);
  if (bundles.length === 0) {
    throw new StatedRefusalError({
      says: `${distDirectory} holds no bundle, so no build has written it; run the census through `
        + `mise run ${COVERAGE_CENSUS_TASK}, which builds with maps first`,
    },);
  }
  if (mapped.length === 0) {
    throw new StatedRefusalError({
      says: `${distDirectory} holds ${String(bundles.length,)} ${
        wordForCount({
          count: bundles.length,
          one: 'bundle',
          many: 'bundles',
        },)
      } and no source map beside any of them, `
        + 'so it is the normal build; run the census through '
        + `mise run ${COVERAGE_CENSUS_TASK}, which builds with maps first`,
    },);
  }
  return {
    mapped,
    unmapped: bundles.filter(function lacksMap(bundle,): boolean {
      return !names.has(`${bundle}.map`,);
    },),
  };
}

/**
 Why the census reads a bundle's map: to place code no test ran on a source
 line, or to say which sources a bundle no test loaded carries.
 */
export type MapNeed = 'cold-code' | 'unloaded-sources';

/**
 Refuses a bundle the census must read a map for when none stands beside it.

 @param bundle - bundle the census reads

 @param need - what the census reads its map for

 @param unmapped - bundles the build holds with no map

 @param distDirectory - build directory, named by the refusal

 @throws {@link StatedRefusalError} when the bundle has no map: the coverage
 build writes one for every bundle that holds code, so the bundle came from
 another build

 @example
 ```ts
 requireMapFor({ bundle: 'nap.mjs', need: 'cold-code', unmapped: ['index.mjs',], distDirectory, },);
 ```
 */
export function requireMapFor(
  {
    bundle,
    need,
    unmapped,
    distDirectory,
  }: {
    readonly bundle: string;
    readonly need: MapNeed;
    readonly unmapped: readonly string[];
    readonly distDirectory: string;
  },
): void {
  if (!unmapped.includes(bundle,))
    return;
  /**
   What the census cannot do without the map.
   */
  const loss = (need === 'cold-code')
    ? `coverage finds code no test ran in ${bundle}, which ${distDirectory} holds with no source map beside it, `
      + 'so the census cannot place that code on a source line'
    : `no test loaded ${bundle}, which ${distDirectory} holds with no source map beside it, so the census `
      + 'cannot say which sources it carries';
  throw new StatedRefusalError({
    says: `${loss}; the coverage build writes a map for every bundle that holds code, so this one came from `
      + `another build: run the census through mise run ${COVERAGE_CENSUS_TASK}, `
      + 'which builds with maps first',
  },);
}

// A MINIFIED BUILD READS FOLDED GUARDS AS RUN (ledger M79). Compression
// writes `if (x) continue;` as `!x&&(…)` and a chain of guard returns as one
// `||` expression, and V8 gives the untaken side of either no range, so a
// guard no test reaches has nothing to report it; it also swaps branches,
// stretching a cold arm over lines that ran. The coverage build keeps the
// code as written (`rolldown.coverage.config.ts`), and rolldown then opens
// each module's code in a chunk with a region comment, which minification
// strips: the compressed normal build of 2026-09-30 held none in any of its
// chunks, the unminified coverage build held one in 174 of its 175 (all but
// the index, which only re-exports).

/**
 Comment rolldown opens each module's code with in a chunk it has not
 minified, followed by the module's path.
 */
const MODULE_REGION_MARK = '//#region ';

/**
 Whether a bundle's text opens some module's code with rolldown's region
 comment.

 @param text - bundle text

 @returns Whether some line starts with the mark

 @example
 ```ts
 carriesModuleRegion('//#region src/nap.ts\nfunction nap() {}\n'); // true
 ```
 */
function carriesModuleRegion(text: string,): boolean {
  return text
    .split('\n',)
    .some(function opensRegion(line,): boolean {
      return line.startsWith(MODULE_REGION_MARK,);
    },);
}

/**
 Refuses a build whose mapped bundles were minified, before the suite runs
 on it.

 @param texts - text of every bundle with a map beside it

 @param distDirectory - build directory, named by the refusal

 @throws {@link StatedRefusalError} when no bundle opens a module's code with
 rolldown's region comment, which a minified build strips

 @example
 ```ts
 requireUnminifiedBuild({ texts: ['//#region src/nap.ts\nfunction nap() {}\n',], distDirectory, },);
 ```
 */
export function requireUnminifiedBuild(
  {
    texts,
    distDirectory,
  }: {
    readonly texts: readonly string[];
    readonly distDirectory: string;
  },
): void {
  if (texts.some(carriesModuleRegion,))
    return;
  throw new StatedRefusalError({
    says: `${distDirectory} holds ${String(texts.length,)} ${
      wordForCount({
        count: texts.length,
        one: 'bundle',
        many: 'bundles',
      },)
    } with a source map and no module region `
      + 'comment in any of them, so the build was minified, and minification folds guards into expressions '
      + 'the coverage gives no range, which would read as run (ledger M79): run the census through '
      + `mise run ${COVERAGE_CENSUS_TASK}, whose build keeps the code as written, `
      + 'or restore minify: false in rolldown.coverage.config.ts if it was changed',
  },);
}

//endregion Coverage bundle maps
