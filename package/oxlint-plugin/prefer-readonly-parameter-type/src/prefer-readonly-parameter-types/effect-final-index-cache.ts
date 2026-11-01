/**
 Process-local fixed-point effect index cache.
 
 @module
 */

import type { Project, } from 'typescript/sync';

import type { EffectSummaryIndex, } from './effect-summary-index.ts';

/**
 Sentinel when process-local project index cannot be reused.
 */
export const FINAL_EFFECT_INDEX_CACHE_MISS: unique symbol = Symbol(
  'final effect index cache miss',
);

/**
 Final index for one included source scope.
 */
type CachedFinalEffectIndex = {
  readonly fileListDigest: string;
  readonly index: EffectSummaryIndex;
};

/**
 Final indexes keyed by TypeScript's immutable semantic snapshot object.
 
 Held weakly. A snapshot is replaced whenever a source needs project discovery, so a process
 meets many `Project` objects for the same configured project, and the index of a replaced one
 is unreachable while still holding that project, its decoded sources, and the whole summary
 graph beneath them. Strong keys kept every generation alive for the life of the process.
 
 In a reassignable slot because clearing a `WeakMap` means replacing it, and
 `clearFinalEffectIndexCache` exists for lifecycle tests.
 */
const store = {
  indexes: new WeakMap<
    Project,
    Map<string, CachedFinalEffectIndex>
    >(),
};

/**
 Process-local fixed-point cache counters.
 */
const counters = {
  hitCount: 0,
  writeCount: 0,
};

/**
 Fixed-point cache activity counts.
 */
export type FinalEffectIndexCacheStats = {
  readonly hitCount: number;
  readonly writeCount: number;
};

/**
 Reads final index for exact semantic snapshot and included source scope.
 
 Project identity is TypeScript's immutable snapshot authority. A source,
 compiler-option, or graph change creates a different project object, so
 validation does not restat every configured source before each parameter.
 
 @param project - Immutable TypeScript semantic project snapshot.
 
 @param projectKey - Analysis-root and runtime-budget cache partition.
 
 @param fileListDigest - Included source-scope identity.
 
 @returns reusable final index or miss sentinel.
 
 @example
 ```ts
 cachedFinalEffectIndex({ project, projectKey, fileListDigest });
 ```
 */
export function cachedFinalEffectIndex({
  project,
  projectKey,
  fileListDigest,
}: {
  readonly project: Project;
  readonly projectKey: string;
  readonly fileListDigest: string;
}): EffectSummaryIndex | typeof FINAL_EFFECT_INDEX_CACHE_MISS {
  /**
   Prior fixed-point index for exact semantic snapshot and analysis scope.
   */
  const cached = store.indexes
    .get(project,)
    ?.get(projectKey,);
  if ((cached === undefined)
    || (cached.fileListDigest !== fileListDigest))
    return FINAL_EFFECT_INDEX_CACHE_MISS;
  counters.hitCount++;
  return cached.index;
}

/**
 Stores final fixed-point index for one semantic snapshot and source scope.
 
 @param project - Immutable TypeScript semantic project snapshot.
 
 @param projectKey - Analysis-root and runtime-budget cache partition.
 
 @param fileListDigest - Included source-scope identity.
 
 @param index - Immutable final summary lookup.
 
 @example
 ```ts
 cacheFinalEffectIndex({ project, projectKey, fileListDigest, index });
 ```
 */
export function cacheFinalEffectIndex({
  project,
  projectKey,
  fileListDigest,
  index,
}: {
  readonly project: Project;
  readonly projectKey: string;
  readonly fileListDigest: string;
  readonly index: EffectSummaryIndex;
}): void {
  /**
   Analysis partitions already cached for exact semantic snapshot.
   */
  const snapshotIndexes = store.indexes
    .get(project,)
    ?? new Map<string, CachedFinalEffectIndex>();
  snapshotIndexes.set(
    projectKey,
    {
      fileListDigest,
      index,
    },
  );
  store.indexes
    .set(
    project,
    snapshotIndexes,
    );
  counters.writeCount++;
}

/**
 Reads process-local final-index activity counts.
 
 @returns copied hit and write counters.
 
 @example
 ```ts
 finalEffectIndexCacheStats();
 ```
 */
export function finalEffectIndexCacheStats(): FinalEffectIndexCacheStats {
  return { ...counters, };
}

/**
 Clears process-local final indexes and counters for lifecycle tests.
 
 @example
 ```ts
 clearFinalEffectIndexCache();
 ```
 */
export function clearFinalEffectIndexCache(): void {
  store.indexes = new WeakMap<
    Project,
    Map<string, CachedFinalEffectIndex>
    >();
  counters.hitCount = 0;
  counters.writeCount = 0;
}
