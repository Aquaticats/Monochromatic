import { resolve, } from 'node:path';
import {
  recordGlobInActiveCapture,
  recordReadInActiveCapture,
} from './tracker-capture.ts';

export {
  captureTrackedSources,
  type CapturedSources,
  type SourceCaptureCallback,
  type TrackedGlob,
} from './tracker-capture.ts';

//region Private mutable collections

/**
 Internal mutable set of files read during config execution.
 Exposed publicly as a {@link ReadonlySet} to prevent external mutation
 that could bypass path normalization or break invariants.
 */
const _reads: Set<string> = new Set<string>();

/**
 Internal mutable set of managed destination files.
 Exposed publicly as a {@link ReadonlySet}.
 */
const _writes: Set<string> = new Set<string>();

/**
 Internal mutable map of write timestamps for echo suppression.
 Exposed publicly as a {@link ReadonlyMap}.
 */
const _writeTimestamps: Map<string, number> = new Map<string, number>();

/**
 Internal mutable map of glob expansions observed during config execution.
 */
const _globs: Map<string, readonly string[]> = new Map<string, readonly string[]>();

//endregion Private mutable collections

//region Read-only public views

/**
 Absolute paths of files read during config execution.
 Populated by {@link cat} as a side effect so watch mode knows what to monitor.
 Read-only; use {@link trackRead} or {@link addWatchedPaths} to add entries.
 */
export const reads: ReadonlySet<string> = _reads;

/**
 Absolute paths of all managed destination files.
 Populated by {@link overwrite} / {@link overwriteEach} even when content
 is unchanged and the actual write is skipped.
 Read-only; use {@link trackDest} to add entries.
 */
export const writes: ReadonlySet<string> = _writes;

/**
 Timestamps (ms since epoch) of actual file writes, keyed by absolute path.
 Only populated when content was different and a real write occurred.
 Used by {@link classifyEvent} to distinguish our write echoes from external edits
 by comparing against the file's mtime.
 Read-only; use {@link trackWriteTime} or {@link setWriteTimestamp} to add entries.
 */
export const writeTimestamps: ReadonlyMap<string, number> = _writeTimestamps;

/**
 Glob expansions observed during config execution, keyed by pattern.
 Read-only; use {@link trackGlob} to add entries.
 */
export const globs: ReadonlyMap<string, readonly string[]> = _globs;

//endregion Read-only public views

//region Mutation functions

/**
 Clears tracking state between watch-mode re-runs so stale paths
 from a previous execution don't linger.
 Preserves {@link writeTimestamps} because echo detection needs to survive across re-runs.
 
 @example
 ```ts
 reset();
 ```
 */
export function reset(): void {
  _reads.clear();
  _writes.clear();
  _globs.clear();
}

/**
 Clears all write timestamps.
 Primarily for test cleanup; production watch mode preserves timestamps
 across re-runs via {@link reset} which intentionally skips this collection.
 
 @example
 ```ts
 resetWriteTimestamps();
 ```
 */
export function resetWriteTimestamps(): void {
  _writeTimestamps.clear();
}

/**
 Records a read path, resolving to absolute for reliable watch comparisons.
 
 @param filePath - path to register as a tracked read
 
 @example
 ```ts
 trackRead('./src/config.ts');
 ```
 */
export function trackRead(filePath: string,): void {
  /**
   Absolute read path used by the global tracker and any active capture.
   */
  const absolutePath = resolve(filePath,);
  _reads.add(absolutePath,);
  recordReadInActiveCapture(absolutePath,);
}

/**
 Registers a path as a managed destination without recording a write timestamp.
 Called unconditionally in write functions so watch mode knows to protect the
 file, even when content was unchanged and the actual write was skipped.
 
 @param filePath - path to register as a managed destination
 
 @example
 ```ts
 trackDest('./dist/output.js');
 ```
 */
export function trackDest(filePath: string,): void {
  _writes.add(resolve(filePath,),);
}

/**
 Records the timestamp of an actual file write for echo suppression.
 Must be called **after** the write completes so the file's mtime
 is guaranteed to be \<= the recorded timestamp.
 
 @param filePath - path that was just written
 
 @example
 ```ts
 trackWriteTime('./dist/output.js');
 ```
 */
export function trackWriteTime(filePath: string,): void {
  _writeTimestamps.set(
    resolve(filePath,),
    Date.now(),
  );
}

/**
 Directly sets a write timestamp for a path.
 Used when caller has authoritative filesystem metadata after durable writes,
 and by tests that need deterministic echo-suppression timestamps.
 
 @param filePath - path to set timestamp for (resolved to absolute)
 
 @param timestamp - ms since epoch
 
 @example
 ```ts
 setWriteTimestamp({ filePath: './dist/output.js', timestamp: Date.now() });
 ```
 */
export function setWriteTimestamp(
  {
    filePath,
    timestamp,
  }: {
    readonly filePath: string;
    readonly timestamp: number;
  },
): void {
  _writeTimestamps.set(
    resolve(filePath,),
    timestamp,
  );
}

/**
 Records a glob expansion in the active source capture.
 
 @param pattern - Glob pattern passed to {@link cat}.
 
 @param paths - Paths matched by the glob.
 
 @example
 ```ts
 trackGlob({ pattern: './src/*.ts', paths: ['./src/index.ts'] });
 ```
 */
export function trackGlob(
  {
    pattern,
    paths,
  }: {
    readonly pattern: string;
    readonly paths: readonly string[];
  },
): void {
  /**
   Absolute path set matched by this glob.
   */
  const absolutePaths = [...new Set(paths.map(function toAbsolutePath(path,): string {
    return resolve(path,);
  },),),].toSorted();
  _globs.set(
    pattern,
    absolutePaths,
  );
  recordGlobInActiveCapture({
    pattern,
    paths: absolutePaths,
  },);
}

/**
 Escape hatch for manually registering additional paths that watch mode
 should monitor. Useful for dependencies that {@link cat} cannot track
 automatically; for example, files consumed by {@link exec} or external
 tools whose inputs are opaque to the enforcer.
 
 @param paths - Array of file paths (resolved to absolute) to add to the read set
 
 @example
 ```ts
 addWatchedPaths(['./data/extra-input.json']);
 ```
 */
export function addWatchedPaths(paths: readonly string[],): void {
  paths.forEach(function addPath(filePath,): void {
    trackRead(filePath,);
  },);
}

//endregion Mutation functions
