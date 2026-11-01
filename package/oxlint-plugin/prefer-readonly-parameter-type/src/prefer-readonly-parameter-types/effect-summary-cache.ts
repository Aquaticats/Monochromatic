/**
 Process-bounded direct effect-summary cache.
 
 @module
 */

import type {
  EffectSummaryOmissionReason,
  PersistentEffectDependencyState,
} from './effect-cache-envelope.ts';
import type { EffectClosureEdges, } from './effect-dependency-closure.ts';
import type { ExternalOmissionRecord, } from './effect-external-omission.ts';
import type { EffectProjectSurfaces, } from './effect-project-fingerprint.ts';
import type { EffectSlot, } from './effect-slot-identity.ts';
import type { MutableEffectSummary, } from './effect-summary-model.ts';
import { recordSharedStore, } from './shared-analysis-telemetry.ts';
import {
  type EffectDependencyClosure,
  PERSISTENT_EFFECT_CACHE_MISS,
  type PersistentEffectCacheAddress,
  readPersistentEffectSummaries,
  writePersistentEffectSummaries,
} from './effect-summary-persistent-cache.ts';

/**
 Sentinel when neither process nor persistent layer proves an exact hit.
 */
export const LAYERED_SUMMARY_CACHE_MISS: unique symbol = Symbol(
  'layered direct summary cache miss',
);

/**
 Cached direct summaries for one exact source text.
 */
type CachedSourceSummaries = {
  readonly projectDigest: string;
  readonly sourceText: string;
  readonly summaries: ReadonlyMap<string, MutableEffectSummary>;
  readonly edges: EffectClosureEdges;
  readonly omittedCallableKeys: readonly string[];
  readonly omissionReasons: readonly EffectSummaryOmissionReason[];
  readonly externalOmissions: readonly ExternalOmissionRecord[];
};

/**
 Layered hit: cloned summaries,
 recorded closure edges,
 and deliberate direct-summary omissions.
 */
export type LayeredSummaryCacheHit = {
  readonly shared: boolean;
  readonly writerProject: string;
  readonly summaries: ReadonlyMap<string, MutableEffectSummary>;
  readonly edges: EffectClosureEdges;
  readonly omittedCallableKeys: readonly string[];
  readonly omissionReasons: readonly EffectSummaryOmissionReason[];
  readonly externalOmissions: readonly ExternalOmissionRecord[];
};

/**
 Shared identity for one source across both cache layers.
 */
type LayeredSourceIdentity = {
  readonly projectKey: string;
  readonly scopeKey: string;
  readonly projectDigest: string;
  readonly fileName: string;
  readonly sourceText: string;
  readonly sharedKeyDigest?: string;
  readonly cacheRootOverride?: string;
};

/**
 Direct summary cache keyed by configured project and source path.
 */
const summariesByProject = new Map<string, Map<string, CachedSourceSummaries>>();

/**
 Mutable counters used by deterministic cache regression tests.
 */
const counters = {
  directSummaryBuildCount: 0,
  sourceCacheHitCount: 0,
  persistentSourceCacheHitCount: 0,
  sharedSourceCacheHitCount: 0,
  persistentCacheWriteCount: 0,
};

/**
 Cache activity counts.
 
 @example
 ```ts
 const stats: EffectSummaryCacheStats = effectSummaryCacheStats();
 ```
 */
export type EffectSummaryCacheStats = {
  readonly directSummaryBuildCount: number;
  readonly sourceCacheHitCount: number;
  readonly persistentSourceCacheHitCount: number;
  readonly sharedSourceCacheHitCount: number;
  readonly persistentCacheWriteCount: number;
};

/**
 Builds the persistent address of one source.
 
 @param identity - Shared source identity.
 
 @param shared - Whether to address the cross-project entry,
 which exists only when the identity carries a key.
 
 @returns project-addressed or file-and-key-addressed persistent address.
 */
function persistentAddress({
  identity,
  shared,
}: {
  readonly identity: LayeredSourceIdentity;
  readonly shared: boolean;
},): PersistentEffectCacheAddress {
  return {
    projectKey: identity.scopeKey,
    fileName: identity.fileName,
    sourceText: identity.sourceText,
    ...((!shared) || (identity.sharedKeyDigest === undefined))
      ? {}
      : { sharedKeyDigest: identity.sharedKeyDigest, },
    ...(identity.cacheRootOverride === undefined)
      ? {}
      : { cacheRootOverride: identity.cacheRootOverride, },
  };
}

/**
 Clones direct summary before fixed-point propagation mutates it.
 
 @param summary - Cached or newly scanned direct summary.
 
 @returns independent mutable fixed-point seed.
 */
function cloneSummary(summary: MutableEffectSummary,): MutableEffectSummary {
  return {
    /* Shared rather than copied. Ownership is derived from the declaration and never
     * mutated, so every clone can point at the same table. */
    slots: summary.slots,
    /* Deep, matching `opaqueProvenanceBySlot` below. A shallow copy would share
     * each origin set with the cached summary, so any later write through one would
     * be visible through the other, which is exactly what cloning exists to prevent. */
    bindingOriginBySymbolId: new Map(
      [...summary.bindingOriginBySymbolId
        .entries(),]
        .map(function cloneOrigins([symbolId, origins,],): [
          number,
          Set<EffectSlot>,
        ] {
          return [
            symbolId,
            new Set(origins,),
          ];
        },),
    ),
    directMutated: new Set(summary.directMutated,),
    directInvoked: new Set(summary.directInvoked,),
    directOpaque: new Set(summary.directOpaque,),
    opaqueProvenanceBySlot: new Map(
      [...summary.opaqueProvenanceBySlot
        .entries(),]
        .map(function cloneProvenance([slot, facts,],): [
          EffectSlot,
          Set<string>,
        ] {
          return [
            slot,
            new Set(facts,),
          ];
        },),
    ),
    mutated: new Set(summary.mutated,),
    invoked: new Set(summary.invoked,),
    opaque: new Set(summary.opaque,),
    directForeignBorrowed: new Set(summary.directForeignBorrowed,),
    directReturned: new Set(summary.directReturned,),
    returned: new Set(summary.returned,),
    relations: [...summary.relations,],
    elementApplications: [...summary.elementApplications,],
    resultApplications: [...summary.resultApplications,],
    calls: [...summary.calls,],
  };
}

/**
 Clones every summary in one map.
 
 @param summaries - Cached or created summaries.
 
 @returns independent mutable clones keyed identically.
 */
function cloneSummaries(
  summaries: ReadonlyMap<string, MutableEffectSummary>,
): ReadonlyMap<string, MutableEffectSummary> {
  return new Map(
    [...summaries.entries(),]
      .map(function cloneEntry([key, summary,],): [
        string,
        MutableEffectSummary,
      ] {
        return [
          key,
          cloneSummary(summary,),
        ];
      },),
  );
}

/**
 Stores summaries and edges in the process memory layer.
 
 @param identity - Shared source identity.
 
 @param summaries - Summaries retained as independent clones.
 
 @param edges - Closure edges recorded beside summaries.
 
 @param omittedCallableKeys - Deliberately omitted direct summaries for this source.
 
 @param omissionReasons - Bounded reason categories for omission state.
 
 @param externalOmissions - Omissions met inside installed package implementations during the scan.
 */
function storeMemoryLayer({
  identity,
  summaries,
  edges,
  omittedCallableKeys,
  omissionReasons,
  externalOmissions,
}: {
  readonly identity: LayeredSourceIdentity;
  readonly summaries: ReadonlyMap<string, MutableEffectSummary>;
  readonly edges: EffectClosureEdges;
  readonly omittedCallableKeys: readonly string[];
  readonly omissionReasons: readonly EffectSummaryOmissionReason[];
  readonly externalOmissions: readonly ExternalOmissionRecord[];
},): void {
  /**
   Project-local cache bounded by configured source paths.
   */
  const projectCache: Map<string, CachedSourceSummaries> = summariesByProject.get(identity.projectKey,)
    ?? new Map<string, CachedSourceSummaries>();
  projectCache.set(
    identity.fileName,
    {
      projectDigest: identity.projectDigest,
      sourceText: identity.sourceText,
      summaries: cloneSummaries(summaries,),
      edges,
      omittedCallableKeys: [...omittedCallableKeys,],
      omissionReasons: [...omissionReasons,],
      externalOmissions: [...externalOmissions,],
    },
  );
  summariesByProject.set(
    identity.projectKey,
    projectCache,
  );
}

/**
 Reads cloned direct summaries from process or persistent layer.
 
 @param identity - Shared source identity.
 
 @param state - Current whole-scope surfaces and per-source digests.
 
 @returns cloned summaries with closure edges,
 or layered miss sentinel.
 
 @example
 ```ts
 readCachedSummariesForSource({ identity, state });
 ```
 */
export function readCachedSummariesForSource({
  identity,
  state,
}: {
  readonly identity: LayeredSourceIdentity;
  readonly state: PersistentEffectDependencyState;
},): LayeredSummaryCacheHit | typeof LAYERED_SUMMARY_CACHE_MISS {
  /**
   Prior direct summaries for exact path.
   */
  const cached = summariesByProject.get(identity.projectKey,)
    ?.get(identity.fileName,);
  if ((cached !== undefined)
    && (cached.projectDigest === identity.projectDigest)
    && (cached.sourceText === identity.sourceText)) {
    counters.sourceCacheHitCount++;
    return {
      shared: false,
      writerProject: identity.scopeKey,
      summaries: cloneSummaries(cached.summaries,),
      edges: cached.edges,
      omittedCallableKeys: [...cached.omittedCallableKeys,],
      omissionReasons: [...cached.omissionReasons,],
      externalOmissions: [...cached.externalOmissions,],
    };
  }
  /**
   Cross-project entry persisted by any project's Oxlint process,
   consulted first when the source may share.
   */
  const shared = identity.sharedKeyDigest === undefined
    ? PERSISTENT_EFFECT_CACHE_MISS
    : readPersistentEffectSummaries({
      address: persistentAddress({
        identity,
        shared: true,
      },),
      state,
    },);
  /**
   Direct summaries persisted by prior Oxlint process.
   */
  const persistent = shared === PERSISTENT_EFFECT_CACHE_MISS
    ? readPersistentEffectSummaries({
      address: persistentAddress({
        identity,
        shared: false,
      },),
      state,
    },)
    : shared;
  if (persistent === PERSISTENT_EFFECT_CACHE_MISS)
    return LAYERED_SUMMARY_CACHE_MISS;
  counters.persistentSourceCacheHitCount++;
  if (persistent.shared)
    counters.sharedSourceCacheHitCount++;
  /**
   Closure edges recorded at entry creation.
   */
  const edges: EffectClosureEdges = {
    resolved: persistent.dependenciesResolved,
    directDependencies: persistent.directDependencies,
  };
  storeMemoryLayer({
    identity,
    summaries: persistent.summaries,
    edges,
    omittedCallableKeys: persistent.omittedCallableKeys,
    omissionReasons: persistent.omissionReasons,
    externalOmissions: persistent.externalOmissions,
  },);
  return {
    shared: persistent.shared,
    writerProject: persistent.writerProject,
    summaries: cloneSummaries(persistent.summaries,),
    edges,
    omittedCallableKeys: [...persistent.omittedCallableKeys,],
    omissionReasons: [...persistent.omissionReasons,],
    externalOmissions: [...persistent.externalOmissions,],
  };
}

/**
 Stores created summaries in both cache layers.
 
 @param identity - Shared source identity.
 
 @param summaries - Freshly created direct summaries.
 
 @param surfaces - Whole-scope surface digests at creation time.
 
 @param closure - Dependency-closure snapshot for exact source.
 
 @param omittedCallableKeys - Direct summaries deliberately omitted for this source.
 
 @param omissionReasons - Bounded reasons encountered while scanning source.
 
 @param externalOmissions - Omissions met inside installed package implementations during the scan.
 
 @example
 ```ts
 storeCreatedSummariesForSource({
   identity,
   summaries,
   surfaces,
   closure,
   omittedCallableKeys,
   omissionReasons,
   externalOmissions,
 });
 ```
 */
export function storeCreatedSummariesForSource({
  identity,
  summaries,
  surfaces,
  closure,
  omittedCallableKeys,
  omissionReasons,
  externalOmissions,
}: {
  readonly identity: LayeredSourceIdentity;
  readonly summaries: ReadonlyMap<string, MutableEffectSummary>;
  readonly surfaces: EffectProjectSurfaces;
  readonly closure: EffectDependencyClosure;
  readonly omittedCallableKeys: readonly string[];
  readonly omissionReasons: readonly EffectSummaryOmissionReason[];
  readonly externalOmissions: readonly ExternalOmissionRecord[];
},): void {
  counters.directSummaryBuildCount += summaries.size;
  storeMemoryLayer({
    identity,
    summaries,
    edges: {
      resolved: closure.resolved,
      directDependencies: closure.directDependencies,
    },
    omittedCallableKeys,
    omissionReasons,
    externalOmissions,
  },);
  writePersistentEffectSummaries({
    /* A shared entry is written only when the dependency closure resolved. The cross-project key
     * lists the module references the parser lists, which leaves out an `import()` type whose
     * argument is not a string literal; `directModuleDependencies` fails the closure on that node
     * kind, so a source that reaches one never gets a shared entry. The key and this gate rest on
     * each other. An unresolved closure also snapshots its whole scope, which keeps it from
     * validating in a program with another file list, so only a program holding every file of the
     * first could read it back; `shared-analysis.unit.test.ts` holds that case. */
    address: persistentAddress({
      identity,
      shared: closure.resolved,
    },),
    summaries,
    surfaces,
    closure,
    omittedCallableKeys,
    omissionReasons,
    externalOmissions,
  },);
  counters.persistentCacheWriteCount++;
  if (closure.resolved && (identity.sharedKeyDigest !== undefined))
    recordSharedStore('write',);
}

/**
 Removes cached files no longer present in configured project.
 
 @param projectKey - Configured project identity.
 
 @param activeFiles - Current non-declaration project source paths.
 
 @example
 ```ts
 pruneDirectSummaryCache({ projectKey, activeFiles });
 ```
 */
export function pruneDirectSummaryCache({
  projectKey,
  activeFiles,
}: {
  readonly projectKey: string;
  readonly activeFiles: ReadonlySet<string>;
}): void {
  /**
   Existing project-local source cache, when initialized.
   */
  const projectCache = summariesByProject.get(projectKey,);
  if (projectCache === undefined)
    return;
  [...projectCache.keys(),].forEach(function prune(fileName,): void {
    if (!activeFiles.has(fileName,))
      projectCache.delete(fileName,);
  },);
}

/**
 Reads cache activity counters.
 
 @returns current direct-build and source-hit counts.
 
 @example
 ```ts
 effectSummaryCacheStats();
 ```
 */
export function effectSummaryCacheStats(): EffectSummaryCacheStats {
  return { ...counters, };
}

/**
 Clears direct cache and counters for lifecycle tests.
 
 @example
 ```ts
 clearEffectSummaryCache();
 ```
 */
export function clearEffectSummaryCache(): void {
  summariesByProject.clear();
  counters.directSummaryBuildCount = 0;
  counters.sourceCacheHitCount = 0;
  counters.persistentSourceCacheHitCount = 0;
  counters.sharedSourceCacheHitCount = 0;
  counters.persistentCacheWriteCount = 0;
}
