import { configDependencyPaths, } from '../context.ts';
import {
  registerFreshPaths,
  stampsAreFresh,
} from './staleness-freshness.ts';
import {
  loadManifest,
  resolveManifestPath,
} from './staleness-manifest.ts';
import {
  selectDestinations,
  selectSources,
  uniqueGlobStamps,
  uniqueStampsByPath,
} from './staleness-aggregate.ts';
import type {
  StalenessEntry,
  StalenessOptions,
} from './staleness-types.ts';

/**
 Returns whether a manifest entry belongs to the active config file.
 
 @param entry - Manifest entry to inspect.
 
 @param configPaths - Active config paths that every matching entry must include.
 
 @returns Whether the entry was recorded for the active config.
 
 @example
 ```ts
 const belongs = entryBelongsToActiveConfig({ entry, configPaths: new Set(['/repo/file-enforcer.config.ts']) });
 ```
 */
function entryBelongsToActiveConfig(
  {
    entry,
    configPaths,
  }: {
    readonly entry: StalenessEntry;
    readonly configPaths: ReadonlySet<string>;
  },
): boolean {
  return entry
    .sourceFiles
    .some(function sourceIsActiveConfig(sourceFile,): boolean {
      return configPaths.has(sourceFile.path,);
    },);
}

/**
 Checks whether every persisted manifest entry for current config is fresh:
 loads the manifest via {@link resolveManifestPath} and {@link loadManifest},
 then verifies freshness with {@link stampsAreFresh} before re-registering
 watched paths through {@link registerFreshPaths}.
 This is a diagnostic/helper API; config import must still run so TypeScript
 config code can discover untracked and newly-added effects.
 
 @param manifestPath - Optional custom manifest path.
 
 @returns Whether all persisted entries are fresh.
 
 @example
 ```ts
 const fresh = await freshStalenessManifest({});
 ```
 */
export async function freshStalenessManifest(
  {
    manifestPath,
  }: StalenessOptions,
): Promise<boolean> {
  /**
   Absolute manifest path selected the same way writer calls select it.
   */
  const resolvedManifestPath = await resolveManifestPath(manifestPath === undefined
    ? {}
    : { manifestPath, },);
  /**
   Manifest loaded from the selected path.
   */
  const manifest = await loadManifest(resolvedManifestPath,);
  /**
   Active config dependency paths for this process invocation.
   */
  const configPaths = new Set(configDependencyPaths(),);
  /**
   Persisted entries for the active config path.
   */
  const entries = Object.values(manifest.entries,)
    .filter(function keepActiveConfigEntry(entry,): boolean {
      return entryBelongsToActiveConfig({
        entry,
        configPaths,
      },);
    },);
  if (entries.length === 0)
    return false;

  /**
   Unique source stamps across every active entry.
   */
  const sourceFiles = uniqueStampsByPath({
    entries,
    selectStamps: selectSources,
  },);
  /**
   Unique destination stamps across every active entry.
   */
  const destinationFiles = uniqueStampsByPath({
    entries,
    selectStamps: selectDestinations,
  },);
  /**
   Unique glob path-set stamps across every active entry.
   */
  const sourceGlobs = uniqueGlobStamps(entries,);
  if (!await stampsAreFresh({
    sourceFiles,
    sourceGlobs,
    destinationFiles,
  },))
    return false;

  registerFreshPaths({
    sourceFiles,
    destinationFiles,
  },);
  return true;
}
