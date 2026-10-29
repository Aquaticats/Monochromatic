import type {
  DestinationStamp,
  FileStamp,
  GlobStamp,
  StalenessEntry,
} from './staleness-types.ts';

/**
 Returns unique file stamps by path.
 
 @param entries - Manifest entries to collect from.
 
 @param selectStamps - Stamp selector for each entry.
 
 @returns Sorted unique stamps by path.
 
 @example
 ```ts
 const sources = uniqueStampsByPath({ entries, selectStamps: entry => entry.sourceFiles });
 ```
 */
export function uniqueStampsByPath<TStamp extends FileStamp,>(
  {
    entries,
    selectStamps,
  }: {
    readonly entries: readonly StalenessEntry[];
    readonly selectStamps: (entry: StalenessEntry) => readonly TStamp[];
  },
): readonly TStamp[] {
  /**
   Latest stamp by path.
   */
  const stampsByPath = new Map<string, TStamp>();
  entries.forEach(function collectEntryStamps(entry,): void {
    selectStamps(entry,)
      .forEach(function collectStamp(stamp,): void {
        stampsByPath.set(
          stamp.path,
          stamp,
        );
      },);
  },);
  return [...stampsByPath.values(),]
    .toSorted(function compareStampPaths(
      leftStamp,
      rightStamp,
    ): number {
      return leftStamp
        .path
        .localeCompare(rightStamp.path,);
    },);
}

/**
 Returns unique glob stamps by pattern.
 
 @param entries - Manifest entries to collect from.
 
 @returns Sorted unique glob stamps.
 
 @example
 ```ts
 const globs = uniqueGlobStamps(entries);
 ```
 */
export function uniqueGlobStamps(entries: readonly StalenessEntry[],): readonly GlobStamp[] {
  /**
   Latest glob stamp by pattern.
   */
  const globsByPattern = new Map<string, GlobStamp>();
  entries.forEach(function collectEntryGlobs(entry,): void {
    entry
      .sourceGlobs
      .forEach(function collectGlob(glob,): void {
        globsByPattern.set(
          glob.pattern,
          glob,
        );
      },);
  },);
  return [...globsByPattern.values(),]
    .toSorted(function compareGlobPatterns(
      leftGlob,
      rightGlob,
    ): number {
      return leftGlob
        .pattern
        .localeCompare(rightGlob.pattern,);
    },);
}

/**
 Selects source stamps from a manifest entry.
 
 @param entry - Manifest entry to inspect.
 
 @returns Source file stamps for the entry.
 
 @example
 ```ts
 const sourceFiles = selectSources(entry);
 ```
 */
export function selectSources(entry: StalenessEntry,): readonly FileStamp[] {
  return entry.sourceFiles;
}

/**
 Selects destination stamps from a manifest entry.
 
 @param entry - Manifest entry to inspect.
 
 @returns Destination file stamps for the entry.
 
 @example
 ```ts
 const destinationFiles = selectDestinations(entry);
 ```
 */
export function selectDestinations(entry: StalenessEntry,): readonly DestinationStamp[] {
  return entry.destinationFiles;
}
