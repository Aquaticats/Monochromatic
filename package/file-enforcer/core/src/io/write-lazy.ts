import {
  captureTrackedSources,
} from '../tracker.ts';
import type { GlobResults, } from './cat.ts';
import {
  destinationsForFiles,
  writeGlobDestinations,
} from './write-each-destinations.ts';
import {
  freshStalenessEntryExists,
  rememberFreshStalenessEntry,
  stalenessKeyForDest,
  stalenessKeyForDestGlob,
} from './staleness.ts';

/**
 Lazy content builder accepted by {@link overwrite} for staleness-cache skips.
 */
export type ContentBuilder = () => string | Promise<string>;

/**
 Content accepted by `overwrite()`.
 */
export type OverwriteContent = string | ContentBuilder;

/**
 Lazy glob-results builder accepted by `overwriteEach()`.
 */
export type GlobResultsBuilder = () => GlobResults | Promise<GlobResults>;

/**
 Files accepted by `overwriteEach()`.
 */
export type OverwriteEachFiles = GlobResults | GlobResultsBuilder;

/**
 Shared write function supplied by `write.ts`.
 */
export type WriteIfChanged = (
  args: {
    readonly dest: string;
    readonly content: string;
    readonly manifestPath: string;
    readonly sourcePath?: string;
    readonly recordStaleness?: boolean;
  },
) => Promise<void>;

/**
 Returns true when overwrite content is a lazy builder.
 
 @param content - Content option passed to `overwrite()`.
 
 @returns Whether content must be invoked lazily.
 
 @example
 ```ts
 const lazy = isContentBuilder(async function build() { return 'x'; });
 ```
 */
export function isContentBuilder(content: OverwriteContent,): content is ContentBuilder {
  return (typeof content) === 'function';
}

/**
 Returns true when overwriteEach files are produced by a lazy builder.
 
 @param files - Files option passed to `overwriteEach()`.
 
 @returns Whether files must be invoked lazily.
 
 @example
 ```ts
 const lazy = isGlobResultsBuilder(async function readFiles() { return await cat('./*.md'); });
 ```
 */
export function isGlobResultsBuilder(files: OverwriteEachFiles,): files is GlobResultsBuilder {
  return (typeof files) === 'function';
}

/**
 Writes a single lazy overwrite, skipping the builder when the manifest is fresh.
 
 @param dest - Destination path.
 
 @param content - Lazy content builder.
 
 @param sourcePath - Optional source path for mirror-style logs.
 
 @param manifestPath - Resolved staleness manifest path.
 
 @param writeIfChanged - Reconciliation function from `write.ts`.
 
 @mutates content through sourceCaptureStorage.run callback invocation
 
 @example
 ```ts
 await writeLazyIfChanged({ manifestPath, dest, content: build, writeIfChanged });
 ```
 */
export async function writeLazyIfChanged(
  {
    dest,
    content,
    sourcePath,
    manifestPath,
    writeIfChanged,
  }: {
    readonly manifestPath: string;
    readonly dest: string;
    readonly content: ContentBuilder;
    readonly sourcePath?: string;
    readonly writeIfChanged: WriteIfChanged;
  },
): Promise<void> {
  /**
   Manifest key for this destination.
   */
  const key = stalenessKeyForDest(dest,);
  if (await freshStalenessEntryExists({
    manifestPath,
    key,
    kind: 'single',
  },))
    return;

  /**
   Generated content plus captured source dependencies.
   */
  const captured = await captureTrackedSources({ fn: content, },);
  await writeIfChanged(sourcePath === undefined
    ? {
      dest,
      content: captured.value,
      manifestPath,
      recordStaleness: false,
    }
    : {
      dest,
      content: captured.value,
      manifestPath,
      sourcePath,
      recordStaleness: false,
    },);
  await rememberFreshStalenessEntry({
    manifestPath,
    key,
    kind: 'single',
    trackedReads: captured.reads,
    trackedGlobs: captured.globs,
    destinations: [{
      path: dest,
      content: captured.value,
    },],
  },);
}

/**
 Writes a lazy glob mirror, skipping the builder when the manifest is fresh.
 
 @param destGlob - Destination glob pattern.
 
 @param files - Lazy glob-results builder.
 
 @param manifestPath - Resolved staleness manifest path.
 
 @param writeIfChanged - Reconciliation function from `write.ts`.
 
 @mutates files through sourceCaptureStorage.run callback invocation
 
 @example
 ```ts
 await writeLazyEach({ manifestPath, destGlob, files: readFiles, writeIfChanged });
 ```
 */
export async function writeLazyEach(
  {
    destGlob,
    files,
    manifestPath,
    writeIfChanged,
  }: {
    readonly manifestPath: string;
    readonly destGlob: string;
    readonly files: GlobResultsBuilder;
    readonly writeIfChanged: WriteIfChanged;
  },
): Promise<void> {
  /**
   Manifest key for this destination glob.
   */
  const key = stalenessKeyForDestGlob(destGlob,);
  if (await freshStalenessEntryExists({
    manifestPath,
    key,
    kind: 'each',
  },))
    return;

  /**
   Source files plus captured dependencies.
   */
  const captured = await captureTrackedSources({ fn: files, },);
  /**
   Concrete destinations for every source file.
   */
  const destinations = destinationsForFiles({
    destGlob,
    files: captured.value,
  },);
  await writeGlobDestinations({
    destinations,
    writeIfChanged,
    manifestPath,
    recordStaleness: false,
  },);
  await rememberFreshStalenessEntry({
    manifestPath,
    key,
    kind: 'each',
    trackedReads: captured.reads,
    trackedGlobs: captured.globs,
    destinations,
  },);
}
