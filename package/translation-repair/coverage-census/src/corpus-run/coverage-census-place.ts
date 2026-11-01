import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  type BundleMaps,
  requireMapFor,
} from './coverage-bundle-maps.ts';
import type { InvariantThrowStretch, } from './coverage-census-invariant.ts';
import type { UnloadedSource, } from './coverage-census-print.ts';
import { readUtf8Text, } from './coverage-census-read-text.ts';
import {
  type CensusStretch,
  censusStretchesOf,
  requirePlacedFunctions,
} from './coverage-census-report.ts';
import {
  readBundle,
  unloadedSourcesOf,
} from './coverage-census-steps.ts';
import {
  type StretchReading,
  stretchReadingOf,
} from './coverage-invariant-throw.ts';
import {
  mapFunction,
  type MappedFunction,
} from './coverage-lines.ts';
import { mapStretch, } from './coverage-pieces.ts';
import type {
  ColdStretch,
  CoverageTally,
  UncalledFunction,
} from './coverage-tally.ts';

//region Coverage census placement
// Ledger T8: the census's reading of a painted tally against the build, split
// from the entry (`coverage-census.ts`) at its line budget: each cold stretch
// and uncalled function placed on its source lines through the bundle maps,
// the sources the loaded bundles carry, and the bundles no test loaded with
// the sources only they carry. A bundle with no map is read only where code
// must be placed in it, and refused by name there (`coverage-bundle-maps.ts`).
//
// A stretch that is nothing but invariant throws is placed apart from the
// cold stretches, read from its own text in the bundle
// (`coverage-invariant-throw.ts`), so nothing built from the cold stretches,
// a row, a total or a baseline reading, counts a guard no honest input
// reaches as code a test should run (`coverage-census-invariant.ts`).

/**
 A painted tally placed on source lines.
 */
export type PlacedTally = {
  /**
   Cold stretches as the census records them, one per piece, those that are
   nothing but invariant throws left out.
   */
  readonly stretches: readonly CensusStretch[];

  /**
   Stretches no test ran that are nothing but invariant throws, one record
   per piece, each with what the stretch throws.
   */
  readonly invariantThrows: readonly InvariantThrowStretch[];

  /**
   Uncalled functions with their lines.
   */
  readonly uncalled: readonly MappedFunction[];

  /**
   Sources the loaded bundles carry.
   */
  readonly loadedSources: ReadonlySet<string>;

  /**
   Bundles no process loaded, sorted.
   */
  readonly unloadedBundles: readonly string[];

  /**
   Sources only those bundles carry, with their physical lines.
   */
  readonly unloadedSources: readonly UnloadedSource[];
};

/**
 One stretch no process ran, placed: its census records, one per piece, and
 what its text reads as.
 */
type PlacedStretch = {
  readonly records: readonly CensusStretch[];
  readonly reading: StretchReading;
};

/**
 Reads one cold stretch as invariant throws or as cold code.

 A function no process called is code, whatever holds it: a callback a
 throw's message is built with keeps the stretch holding its first
 character cold, which is also where `requirePlacedFunctions` looks for it.

 @param stretch - stretch the tally found cold

 @param text - text of the bundle holding it

 @param uncalled - every function no process called

 @returns What its text between its offsets reads as, cold where a function
 no process called starts inside it

 @example
 ```ts
 const reading = standingOf({ stretch, text, uncalled: tally.uncalledFunctions(), },);
 ```
 */
function standingOf(
  {
    stretch,
    text,
    uncalled,
  }: {
    readonly stretch: ColdStretch;
    readonly text: string;
    readonly uncalled: readonly UncalledFunction[];
  },
): StretchReading {
  /**
   What the stretch's own text reads as.
   */
  const reading = stretchReadingOf({
    text: text.slice(
      stretch.start,
      stretch.end,
    ),
  },);
  if (reading.kind === 'cold')
    return reading;
  /**
   Whether a function no process called starts inside the stretch.
   */
  const holdsFunction = uncalled.some(function startsInside(fn,): boolean {
    return (fn.bundle === stretch.bundle)
      && (fn.start >= stretch.start)
      && (fn.start < stretch.end);
  },);
  return holdsFunction ? { kind: 'cold', } : reading;
}

/**
 Reads every mapped bundle side by side, keyed by name.

 @param mapped - bundles with a map beside each, in the order a failure must be reported in

 @param readOne - reads one bundle, passed in so a case scripts which of two reads ends first

 @returns Each bundle's reading by name, in the order given

 @throws Whatever the first bundle in the order given, whose read failed, was refused with

 @example
 ```ts
 const read = await readMappedBundles({ mapped: ['index.mjs',], readOne: ({ bundle, },) => readBundle({ distDirectory, packageDirectory, bundle, },), },);
 ```
 */
export async function readMappedBundles<const Reading,>(
  {
    mapped,
    readOne,
  }: {
    readonly mapped: readonly string[];
    readonly readOne: (input: { readonly bundle: string; },) => Promise<Reading>;
  },
): Promise<ReadonlyMap<string, Reading>> {
  return new Map(await allInInputOrder({
    members: mapped.map(async function readNamed(bundle,) {
      return [
        bundle,
        await readOne({ bundle, },),
      ] as const;
    },),
  },),);
}

/**
 Places a painted tally on source lines through the build's maps.

 @param packageDirectory - package directory the sources are named from

 @param distDirectory - build directory

 @param bundleMaps - bundles the build holds, split by whether a map stands
 beside each

 @param tally - painted tally

 @param entryFiles - sources the build names as runner entries

 @returns The cold stretches, the invariant throws apart from them, the
 uncalled functions and the sources, placed

 @throws StatedRefusalError where the coverage names a bundle the build
 does not hold, finds code no test ran in a bundle with no map, or finds a
 bundle no test loaded that has no map to say which sources it carries, or
 where the census places an uncalled function outside its own source's
 stretches

 @example
 ```ts
 const placed = await placeTally({ packageDirectory, distDirectory, bundleMaps, tally, entryFiles, },);
 ```
 */
export async function placeTally(
  {
    packageDirectory,
    distDirectory,
    bundleMaps,
    tally,
    entryFiles,
  }: {
    readonly packageDirectory: string;
    readonly distDirectory: string;
    readonly bundleMaps: BundleMaps;
    readonly tally: CoverageTally;
    readonly entryFiles: ReadonlySet<string>;
  },
): Promise<PlacedTally> {
  /**
   Bundles with a map, which the census reads, and those without.
   */
  const {
    mapped,
    unmapped,
  } = bundleMaps;
  /**
   Every mapped bundle's text, lines and sources, by name.
   */
  const read = await readMappedBundles({
    mapped,
    readOne: function readOne({ bundle, },) {
      return readBundle({
        distDirectory,
        packageDirectory,
        bundle,
      },);
    },
  },);
  /**
   Bundles some process loaded.
   */
  const loaded = new Set(tally.loadedBundles(),);
  /**
   Reading of a loaded bundle.

   @param bundle - bundle the coverage names

   @returns Its text, positions and map
   */
  function readingOf(bundle: string,): Awaited<ReturnType<typeof readBundle>> {
    /**
     Its reading.
     */
    const reading = read.get(bundle,);
    requireMapFor({
      bundle,
      need: 'cold-code',
      unmapped,
      distDirectory,
    },);
    if (reading === undefined)
      throw new StatedRefusalError({ says: `coverage names ${bundle}, which ${distDirectory} does not hold; rebuild and run again`, },);
    return reading;
  }
  /**
   Every function no process called, as the tally reports them.
   */
  const uncalledSites = tally.uncalledFunctions();
  /**
   Every stretch no process ran, as the census records it, one record per
   piece, with whether it reads as invariant throws.
   */
  const placedStretches = tally.coldStretches()
    .map(function placedStretch(stretch,): PlacedStretch {
      /**
       Text, positions and map of its bundle.
       */
      const {
        text,
        lines,
      } = readingOf(stretch.bundle,);
      return {
        records: censusStretchesOf({
          stretch: mapStretch({
            lines,
            stretch,
          },),
        },),
        reading: standingOf({
          stretch,
          text,
          uncalled: uncalledSites,
        },),
      };
    },);
  /**
   Cold stretches as the census records them, one per piece.
   */
  const stretches = placedStretches.flatMap(function coldRecords({
    records,
    reading,
  },): readonly CensusStretch[] {
    return (reading.kind === 'cold') ? records : [];
  },);
  /**
   Stretches that are nothing but invariant throws, each record with what
   its stretch throws.
   */
  const invariantThrows = placedStretches.flatMap(function apartRecords({
    records,
    reading,
  },): readonly InvariantThrowStretch[] {
    if (reading.kind === 'cold')
      return [];
    return records.map(function withThrown(record,): InvariantThrowStretch {
      return {
        ...record,
        thrown: reading.thrown,
      };
    },);
  },);
  /**
   Uncalled functions with their lines.
   */
  const uncalled = uncalledSites.map(function placed(fn,) {
    return mapFunction({
      lines: readingOf(fn.bundle,)
        .lines,
      uncalled: fn,
    },);
  },);
  requirePlacedFunctions({
    stretches,
    uncalled,
  },);
  /**
   Sources the loaded bundles carry.
   */
  const loadedSources = new Set([...read,].flatMap(function carried([bundle, reading,],): readonly string[] {
    return loaded.has(bundle,) ? reading.sources : [];
  },),);
  /**
   Bundles no process loaded.
   */
  const unloadedBundles = textsInCodePointOrder({ texts: [
    ...mapped,
    ...unmapped,
  ]
    .filter(function unloaded(bundle,): boolean {
      return !loaded.has(bundle,);
    },), },);
  for (const bundle of unloadedBundles) {
    requireMapFor({
      bundle,
      need: 'unloaded-sources',
      unmapped,
      distDirectory,
    },);
  }
  return {
    stretches,
    invariantThrows,
    uncalled,
    loadedSources,
    unloadedBundles,
    unloadedSources: await unloadedSourcesOf({
      packageDirectory,
      // Every unloaded bundle has a map by now, so the mapped readings hold
      // each one's sources.
      carried: [...read,].flatMap(function carried([bundle, reading,],): readonly string[] {
        return loaded.has(bundle,) ? [] : reading.sources;
      },),
      loadedSources,
      entryFiles,
      readText: readUtf8Text,
    },),
  };
}

//endregion Coverage census placement
