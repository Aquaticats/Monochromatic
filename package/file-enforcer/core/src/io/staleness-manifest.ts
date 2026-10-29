import {
  join,
  resolve,
} from 'node:path';
import {
  readManifestFromDisk,
  writeMergedManifest,
} from './staleness-manifest-persist.ts';
import {
  findNodeModulesRoot,
  NODE_MODULES_DIRECTORY_NAME,
} from './staleness-root.ts';
import {
  CACHE_DIRECTORY_NAME,
  EACH_ENTRY_PREFIX,
  MANIFEST_FILE_NAME,
  SINGLE_ENTRY_PREFIX,
  type StalenessEntry,
  type StalenessManifest,
  type StalenessOptions,
} from './staleness-types.ts';

/**
 Read-only manifest shape accepted by the serializer.
 */
type ReadonlyStalenessManifest = Readonly<{
  /**
   Manifest schema version.
   */
  readonly version: number;

  /**
   Read-only entry map for serialization.
   */
  readonly entries: Readonly<Record<string, StalenessEntry>>;
}>;

/**
 Options for writing a manifest file.
 */
type WriteManifestOptions = Readonly<{
  /**
   Absolute manifest path.
   */
  readonly manifestPath: string;

  /**
   Manifest object to serialize.
   */
  readonly manifest: ReadonlyStalenessManifest;
}>;

/**
 In-memory manifest cache keyed by absolute manifest path.
 */
const manifestCache: Map<string, StalenessManifest> = new Map<string, StalenessManifest>();

/**
 Manifest paths staged for a current immediate flush attempt.
 */
const dirtyManifestPaths: Set<string> = new Set<string>();

/**
 Writes one cached manifest to disk via {@link writeMergedManifest}.
 
 @param manifestPath - Absolute manifest path to flush.
 
 @example
 ```ts
 await flushManifestPath('/repo/node_modules/.cache/file-enforcer/staleness-manifest.json');
 ```
 */
async function flushManifestPath(manifestPath: string,): Promise<void> {
  /**
   Cached manifest to flush.
   */
  const manifest = manifestCache.get(manifestPath,);
  if (manifest === undefined)
    return;

  /**
   Manifest merged with any entries other processes wrote while this process ran.
   */
  const mergedManifest = await writeMergedManifest({
    manifestPath,
    manifest,
  },);
  manifestCache.set(
    manifestPath,
    mergedManifest,
  );
}

/**
 Flushes all dirty manifests via {@link flushManifestPath} before a normal process exit.
 
 @example
 ```ts
 await flushDirtyManifests();
 ```
 */
async function flushDirtyManifests(): Promise<void> {
  await Promise.all(
    [...dirtyManifestPaths,].map(async function flushDirtyManifest(manifestPath,): Promise<void> {
      await flushManifestPath(manifestPath,);
    },),
  );
  dirtyManifestPaths.clear();
}

/**
 Flushes dirty manifests from the beforeExit hook and reports best-effort failures.
 
 @example
 ```ts
 await flushDirtyManifestsAndWarn();
 ```
 */
async function flushDirtyManifestsAndWarn(): Promise<void> {
  try {
    await flushDirtyManifests();
  }
  catch (flushError: unknown) {
    dirtyManifestPaths.clear();
    process.emitWarning(
      `Could not flush file-enforcer staleness manifests before exit: ${String(flushError,)}`,
    );
  }
}

/**
 Starts best-effort dirty manifest flushing when Node is otherwise ready to exit.
 
 @example
 ```ts
 flushDirtyManifestsBeforeExit();
 ```
 */
function flushDirtyManifestsBeforeExit(): void {
  if (dirtyManifestPaths.size === 0)
    return;

  void flushDirtyManifestsAndWarn();
}

process.on(
  'beforeExit',
  flushDirtyManifestsBeforeExit,
);

/**
 Resolves the manifest path for a write call.
 
 @param manifestPath - Optional caller-provided manifest path.
 
 @returns Absolute manifest file path.
 
 @example
 ```ts
 const path = await resolveManifestPath({ manifestPath: './cache.json' });
 ```
 */
export async function resolveManifestPath(
  {
    manifestPath,
  }: StalenessOptions,
): Promise<string> {
  if (manifestPath !== undefined)
    return resolve(manifestPath,);

  /**
   Workspace root discovered by {@link findNodeModulesRoot} walking up until `node_modules` exists.
   */
  const nodeModulesRoot = await findNodeModulesRoot({ startDirectory: process.cwd(), },);
  return join(
    nodeModulesRoot,
    NODE_MODULES_DIRECTORY_NAME,
    '.cache',
    CACHE_DIRECTORY_NAME,
    MANIFEST_FILE_NAME,
  );
}

/**
 Builds a manifest key for a single-destination overwrite.
 
 @param dest - Destination path passed to {@link overwrite}.
 
 @returns Stable manifest entry key.
 
 @example
 ```ts
 const key = stalenessKeyForDest('./CLAUDE.md');
 ```
 */
export function stalenessKeyForDest(dest: string,): string {
  return `${SINGLE_ENTRY_PREFIX}${resolve(dest,)}`;
}

/**
 Builds a manifest key for a glob-mirror overwrite.
 
 @param destGlob - Destination glob passed to {@link overwriteEach}.
 
 @returns Stable manifest entry key.
 
 @example
 ```ts
 const key = stalenessKeyForDestGlob('./out/*​/*.md');
 ```
 */
export function stalenessKeyForDestGlob(destGlob: string,): string {
  return `${EACH_ENTRY_PREFIX}${resolve(destGlob,)}`;
}

/**
 Loads a manifest via {@link readManifestFromDisk}, reusing the per-process
 cache after the first read.
 
 @param manifestPath - Absolute manifest path.
 
 @returns Mutable in-memory manifest object.
 
 @example
 ```ts
 const manifest = await loadManifest('/tmp/manifest.json');
 ```
 */
export async function loadManifest(manifestPath: string,): Promise<StalenessManifest> {
  /**
   Cached manifest for this path.
   */
  const cachedManifest = manifestCache.get(manifestPath,);
  if (cachedManifest !== undefined)
    return cachedManifest;

  /**
   Manifest loaded from disk or initialized empty.
   */
  const manifest = await readManifestFromDisk(manifestPath,);
  manifestCache.set(
    manifestPath,
    manifest,
  );
  return manifest;
}

/**
 Persists a manifest so async builders cannot race stale writes.
 
 @param manifestPath - Absolute manifest path.
 
 @param manifest - Manifest object to serialize.
 
 @example
 ```ts
 await writeManifest({ manifestPath: '/tmp/manifest.json', manifest });
 ```
 */
export async function writeManifest(
  {
    manifestPath,
    manifest,
  }: WriteManifestOptions,
): Promise<void> {
  manifestCache.set(
    manifestPath,
    {
      version: manifest.version,
      entries: { ...manifest.entries, },
    },
  );
  dirtyManifestPaths.add(manifestPath,);
  /**
   Cleanup that keeps failed immediate flushes fail-fast instead of retrying
   during process exit.
   */
  using _dirtyManifestCleanup = {
    [Symbol.dispose](): void {
      dirtyManifestPaths.delete(manifestPath,);
    },
  };
  await flushManifestPath(manifestPath,);
}
