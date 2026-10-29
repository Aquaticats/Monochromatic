import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  isCheckMode,
  recordDrift,
} from '../check-mode.ts';
import { l, } from '../logger.ts';
import {
  setWriteTimestamp,
  trackDest,
} from '../tracker.ts';
import {
  readCached,
  updateCache,
} from './cache.ts';
import type { GlobResults, } from './cat.ts';
import {
  caughtErrorHasCode,
} from './error.ts';
import { resolveManifestPath, } from './staleness.ts';
import { writeFileAtomically, } from './write-atomic.ts';
import {
  destinationsForFiles,
  writeGlobDestinations,
} from './write-each-destinations.ts';
import {
  FILE_ALREADY_EXISTS,
  writeFileIfAbsentAtomically,
} from './write-if-absent-atomic.ts';
import {
  isContentBuilder,
  isGlobResultsBuilder,
  type OverwriteContent,
  type OverwriteEachFiles,
  type WriteIfChanged,
  writeLazyEach,
  writeLazyIfChanged,
} from './write-lazy.ts';
import {
  rememberEagerEach,
  rememberEagerWrite,
} from './write-staleness.ts';

/**
 Sentinel for "file does not exist" returned by {@link readExisting}.
 A unique `Symbol` keeps the absent case out of a banned `string | undefined`
 union and stays distinguishable from real file content, including the empty
 string (a present but empty file is content, not absence).
 */
export const ABSENT_FILE_CONTENT: unique symbol = Symbol('file-enforcer/io/write: absent file content, returned by readExisting when a file does not exist',);

/**
 Reads the current content of a file via the read cache, returning
 {@link ABSENT_FILE_CONTENT} if the file does not exist. Used for content-based write skipping.
 
 @param filePath - Path to check
 
 @returns File content as string, or {@link ABSENT_FILE_CONTENT} if absent
 
 @throws Non-absence read failures so permissions, directories, and transient I/O
   errors are not mistaken for a missing destination.
 
 @example
 ```ts
 const existing = await readExisting('./dist/config.json');
 if (existing === ABSENT_FILE_CONTENT) {
   // file did not exist; safe to create
 }
 ```
 */
export async function readExisting(filePath: string,): Promise<string | typeof ABSENT_FILE_CONTENT> {
  try {
    return await readCached(filePath,);
  }
  catch (readError: unknown) {
    if (caughtErrorHasCode({
      error: readError,
      code: 'ENOENT',
    },))
      return ABSENT_FILE_CONTENT;

    throw readError;
  }
}

/**
 Core write logic shared by {@link overwrite} and {@link overwriteEach}.
 Compares content against existing file, skipping when identical.
 Handles directory creation, cache update, and write-time tracking.
 
 @param dest - Destination file path
 
 @param content - Content string to write
 
 @param sourcePath - Optional source path for log messages (used by overwriteEach)
 
 @param manifestPath - Resolved staleness manifest path
 
 @param recordStaleness - Whether reconciliation should record eager staleness metadata
 */
async function writeIfChanged(
  {
    dest,
    content,
    sourcePath,
    manifestPath,
    recordStaleness,
  }: {
    readonly dest: string;
    readonly content: string;
    readonly manifestPath: string;
    readonly sourcePath?: string;
    readonly recordStaleness?: boolean;
  },
): Promise<void> {
  /**
   Function-scoped logger tagged with the call site for traceable write logs.
   */
  const rl = tagged({
    tag: writeIfChanged.name,
    l,
  },);
  trackDest(dest,);
  /**
   Current file content, or ABSENT_FILE_CONTENT if file doesn't exist yet
   */
  const existing = await readExisting(dest,);
  if (existing === content) {
    rl.debug(
      `skip (unchanged): ${sourcePath !== undefined ? `${sourcePath} -> ` : ''}${dest}`,
    );
    if (recordStaleness !== false) {
      await rememberEagerWrite({
        dest,
        content,
        manifestPath,
      },);
    }
    return;
  }
  if (isCheckMode()) {
    // Check mode: record the drift and keep the would-be content in the read
    // cache, so later generators see what a real run would have written.
    recordDrift({ dest, },);
    updateCache({
      filePath: dest,
      content,
    },);
    rl.warn(`would change: ${sourcePath !== undefined ? `${sourcePath} -> ` : ''}${dest}`,);
    return;
  }
  /**
   Actual post-rename destination timestamp used for watch echo suppression.
   */
  const writeTimestamp = await writeFileAtomically({
    filePath: dest,
    content,
  },);
  updateCache({
    filePath: dest,
    content,
  },);
  setWriteTimestamp({
    filePath: dest,
    timestamp: writeTimestamp,
  },);
  if (recordStaleness !== false) {
    await rememberEagerWrite({
      dest,
      content,
      manifestPath,
    },);
  }
  rl.info(`${sourcePath !== undefined ? `${sourcePath} -> ` : '-> '}${dest}`,);
}

/**
 Public write function passed into lazy helpers without exposing `writeIfChanged`.
 */
const writeIfChangedForLazy: WriteIfChanged = writeIfChanged;

/**
 Writes content to dest, skipping the write when existing content is identical.
 Always registers the path as a managed destination for watch-mode protection.
 Lazy content builders are skipped entirely when the staleness manifest proves
 that previous sources and destination metadata are unchanged.
 
 @param dest - Destination file path
 
 @param content - Content string to write, or lazy builder that returns it
 
 @param manifestPath - Optional staleness manifest path override
 
 @mutates content through sourceCaptureStorage.run callback invocation
 
 @example
 ```ts
 await overwrite({
   dest: './dist/config.json',
   content: async function buildConfig(): Promise<string> {
     return JSON.stringify(config, null, 2);
   },
 });
 ```
 */
export async function overwrite(
  {
    dest,
    content,
    manifestPath,
  }: {
    readonly dest: string;
    readonly content: OverwriteContent;
    readonly manifestPath?: string;
  },
): Promise<void> {
  /**
   Manifest path resolved once so internal staleness helpers receive a concrete path.
   */
  const resolvedManifestPath = await resolveManifestPath(manifestPath === undefined
    ? {}
    : { manifestPath, },);
  if (isContentBuilder(content,)) {
    await writeLazyIfChanged({
      manifestPath: resolvedManifestPath,
      dest,
      content,
      writeIfChanged: writeIfChangedForLazy,
    },);
    return;
  }

  await writeIfChanged({
    dest,
    content,
    manifestPath: resolvedManifestPath,
  },);
}

/**
 Writes content to dest only if the file does not already exist.
 Uses {@link readExisting} to check for the file rather than a separate
 `exists()` call, avoiding a TOCTOU race gap.
 Uses no-clobber final-path creation via {@link writeFileIfAbsentAtomically}
 (returning the {@link FILE_ALREADY_EXISTS} sentinel when another writer wins)
 after the absence check so a path that appears before create time is
 preserved instead of replaced.
 
 @param dest - Destination file path
 
 @param content - Content string to write
 
 @param manifestPath - Optional staleness manifest path override
 
 @example
 ```ts
 await overwriteIfNotExists({
   dest: './config/defaults.json',
   content: '{}',
 });
 ```
 */
export async function overwriteIfNotExists(
  {
    dest,
    content,
    manifestPath,
  }: {
    readonly dest: string;
    readonly content: string;
    readonly manifestPath?: string;
  },
): Promise<void> {
  /**
   Existing content, or ABSENT_FILE_CONTENT if file doesn't exist
   */
  const existing = await readExisting(dest,);
  if (existing !== ABSENT_FILE_CONTENT) {
    trackDest(dest,);
    l.debug(`skip (exists): ${dest}`,);
    return;
  }
  trackDest(dest,);
  if (isCheckMode()) {
    recordDrift({ dest, },);
    updateCache({
      filePath: dest,
      content,
    },);
    l.warn(`would create: ${dest}`,);
    return;
  }
  /**
   Actual post-create destination timestamp used for watch echo suppression,
   or a sentinel when another directory entry won the final path first.
   */
  const writeTimestamp = await writeFileIfAbsentAtomically({
    filePath: dest,
    content,
  },);
  if (writeTimestamp === FILE_ALREADY_EXISTS) {
    l.debug(`skip (exists): ${dest}`,);
    return;
  }
  updateCache({
    filePath: dest,
    content,
  },);
  setWriteTimestamp({
    filePath: dest,
    timestamp: writeTimestamp,
  },);
  await rememberEagerWrite({
    dest,
    content,
    manifestPath: await resolveManifestPath(manifestPath === undefined
      ? {}
      : { manifestPath, },),
  },);
  l.info(`-> ${dest}`,);
}

/**
 Writes each glob-matched file to its mirrored destination,
 skipping files whose content is already identical.
 Source glob is read from the {@link GlobResults} array produced by `cat()`.
 
 @param destGlob - Destination glob pattern with positional wildcards
 
 @param files - Glob results carrying the source pattern and file contents
 
 @mutates files through sourceCaptureStorage.run callback invocation
 
 @example
 ```ts
 await overwriteEach({
   destGlob: './dest/*\/*.md',
   files: await cat('./src/*\/*.md'),
 });
 ```
 */
export async function overwriteEach(
  {
    destGlob,
    files,
    manifestPath,
  }: {
    readonly destGlob: string;
    readonly files: OverwriteEachFiles;
    readonly manifestPath?: string;
  },
): Promise<void> {
  /**
   Manifest path resolved once so internal staleness helpers receive a concrete path.
   */
  const resolvedManifestPath = await resolveManifestPath(manifestPath === undefined
    ? {}
    : { manifestPath, },);
  if (isGlobResultsBuilder(files,)) {
    await writeLazyEach({
      manifestPath: resolvedManifestPath,
      destGlob,
      files,
      writeIfChanged: writeIfChangedForLazy,
    },);
    return;
  }

  l.info(`overwriteEach: ${String(files.length,)} files`,);
  /**
   Concrete destinations for the eager glob mirror rule.
   */
  const destinations = destinationsForFiles({
    destGlob,
    files,
  },);
  await writeGlobDestinations({
    destinations,
    writeIfChanged: writeIfChangedForLazy,
    manifestPath: resolvedManifestPath,
  },);
  await rememberEagerEach({
    destGlob,
    destinations,
    manifestPath: resolvedManifestPath,
  },);
}
