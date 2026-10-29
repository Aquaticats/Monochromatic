/**
 Demand-driven effect index over active callable dependency closure.
 
 @module
 */

import type { SourceFile, } from 'typescript/ast';

import { scanDirectSummaries, } from './direct-source-scan.ts';
import type {
  DemandDrivenEffectIndexOptions,
  PendingEffectSummaryStore,
} from './effect-demand-index-options.ts';
import { sourceIdentity, } from './effect-source-identity.ts';
import { createDependencyClosureResolver, } from './effect-dependency-closure.ts';
import { propagateEffects, } from './effect-fixed-point-propagation.ts';
import { effectPublicSummary, } from './effect-public-summary.ts';
import type { ParameterIndex, } from './effect-slot-identity.ts';
import { SemanticBridgeError, } from './semantic-bridge-error.ts';
import { configuredProjectConfigFileName, } from './semantic-file-name.ts';
import {
  assertReachedCallSummaries,
  reachedSourceFileNames,
} from './effect-reached-edge.ts';
import {
  LAYERED_SUMMARY_CACHE_MISS,
  pruneDirectSummaryCache,
  readCachedSummariesForSource,
  storeCreatedSummariesForSource,
} from './effect-summary-cache.ts';
import {
  reportDirectSummaryOmissions,
  restoreCachedSummaryOmissions,
  restoreExternalOmissions,
} from './effect-summary-omission.ts';
import {
  callableKey,
  type MutableEffectSummary,
} from './effect-summary-model.ts';

import {
  type CallableEffectSummary,
  type EffectSummaryIndex,
  NO_EFFECT_SUMMARY,
} from './effect-summary-index.ts';
import { completeForeignBorrowedGraph, } from './foreign-borrowed-complete-graph.ts';
import {
  provenRootEntry,
  scopeNamesOwnershipMarker,
} from './foreign-borrowed-demand.ts';
import { createOmissionLedger, } from './effect-omission-ledger.ts';
import type { ExternalOmissionRecord, } from './effect-external-omission.ts';
import { createSharedKeyResolver, } from './shared-analysis-key.ts';
import {
  recordSharedStore,
  sharedVerificationEnabled,
} from './shared-analysis-telemetry.ts';
import { verifySharedSource, } from './shared-analysis-verification.ts';

/**
 Creates mutable index that expands only from requested callable sources.
 
 Foreign ownership is declaration-global rather than path-local,
 so proving it walks the complete owned source graph rather than the reached part of it.
 `proveForeignBorrowed` therefore stays a separate demand:
 every other fact `get` answers is already paid for by the expansion that reached the callable.
 
 @param options - Exact project,
 cache,
 ownership,
 and recursive external-analysis inputs.
 
 @returns effect index expanding monotonically within one project snapshot.
 
 @example
 ```ts
 const index = createDemandDrivenEffectIndex(options);
 ```
 */
export function createDemandDrivenEffectIndex(
  options: DemandDrivenEffectIndexOptions,
): EffectSummaryIndex {
  /**
   Exact project,
   cache,
   ownership,
   budget,
   and recursive external-analysis inputs.
   */
  const {
    project,
    indexedSourceFiles,
    projectFingerprint,
    scopeKey,
    projectDigest,
    cacheRootOverride,
    analysisRoot,
    buildIndex,
    analysisBudget,
  } = options;
  /**
   Exact owned source membership for closure validation.
   */
  const indexedFileNames = new Set(indexedSourceFiles.keys(),);
  /**
   Dependency resolver combining module and semantic call edges.
   */
  const closureResolver = createDependencyClosureResolver({
    project,
    indexedFileNames,
    sourceDigests: projectFingerprint.sourceDigests,
  },);
  /**
   Cross-project cache keys for the sources of this snapshot.
   */
  const sharedKeys = createSharedKeyResolver({
    project,
    fingerprint: projectFingerprint,
    ...(analysisRoot === undefined) ? {} : { analysisRoot, },
  },);
  /**
   Current program state validating persistent entries.
   */
  const dependencyState = projectFingerprint;
  /**
   Direct and propagated summaries reached in current snapshot.
   */
  const summaries = new Map<string, MutableEffectSummary>();
  /**
   Callables left out of the index because building their summary threw.
   
   Kept so the completeness assertion can tell a deliberate omission from a missing key it
   should refuse. The omission path is documented as fail-closed, callers taking opacity
   through the absent-callee branch, and the assertion did not know that, so one upstream
   panic cost every file in the program its analysis rather than one callable its summary.
   Measured in `doc/troubleshooting/typescript-go-tuple-type-panic.md`.
   */
  const omittedCallableKeys = new Set<string>();
  /**
   Which loaded sources lost callables and which sources each one reached.
   
   Answers `reachedOmissions`, which a consumer of this index records beside the summaries it
   builds from it. Kept apart from `omittedCallableKeys` because that set is one fact for the
   whole index, and this answer is per callable.
   */
  const omissionLedger = createOmissionLedger();
  /**
   Source paths already loaded into current graph.
   */
  const loadedFileNames = new Set<string>();
  /**
   Fresh summaries awaiting complete semantic-edge closure before persistence.
   */
  const pendingStores = new Map<string, PendingEffectSummaryStore>();
  /**
   Whether a proof could find any marker to anchor on in this scope.
   
   Computed once because it reads every indexed source, and consulted before every demanded
   proof. `foreign-borrowed-demand.ts` records what it is and is not equivalent to.
   */
  const markerReachable = scopeNamesOwnershipMarker({ indexedSourceFiles, },);
  /**
   Proven foreign parameters by the callable each closure was rooted at.
   
   Only the root's own entry is kept. A closure carries summaries for callers it reached, but
   those hold ownership seeds plus whichever outbound edges this root's walk discovered, not
   the summaries a closure rooted at the caller would have built: `getSignatureUsage`
   enumerates references rather than call edges, and a caller whose edge fails validation stays
   in the map with a synthetic unknown inbound in its place. Reusing those entries is what
   attempt two in `doc/planning/prefer-readonly-foreign-proof-cost.md` did, and it produced an
   offer for a parameter a write reaches. Membership doubles as the memo.
   */
  const foreignByProvenRoot = new Map<string, ReadonlySet<ParameterIndex>>();

  /**
   Loads one reached source from cache or exact semantic scan.
   
   @param sourceFile - Owned source newly entering effect graph.
   
   @returns direct summaries and semantic source dependencies.
   */
  function loadSource(sourceFile: SourceFile,): {
    readonly fileSummaries: ReadonlyMap<string, MutableEffectSummary>;
    readonly dependencies: readonly string[];
  } {
    /**
     Cross-project key of current source,
     ineligible when the source keeps its per-project entry.
     */
    const sharedKey = sharedKeys.keyFor(sourceFile,);
    /**
     Layered cache identity for current source.
     */
    const identity = sourceIdentity({
      project,
      scopeKey,
      projectDigest,
      sourceFile,
      ...(sharedKey.eligible) ? { sharedKeyDigest: sharedKey.digest, } : {},
      ...(cacheRootOverride === undefined) ? {} : { cacheRootOverride, },
    },);
    /**
     Validated cached direct summaries and dependency edges.
     */
    const hit = readCachedSummariesForSource({
      identity,
      state: dependencyState,
    },);
    if (hit !== LAYERED_SUMMARY_CACHE_MISS) {
      closureResolver.seedEdges({
        fileName: sourceFile.fileName,
        edges: hit.edges,
      },);
      restoreCachedSummaryOmissions({
        allOmittedKeys: omittedCallableKeys,
        restoredKeys: hit.omittedCallableKeys,
        sourceFileName: sourceFile.fileName,
        reasons: hit.omissionReasons,
      },);
      restoreExternalOmissions(hit.externalOmissions,);
      /**
       Owned source dependencies the restored summaries reach.
       */
      const restoredDependencies = reachedSourceFileNames({
        fileSummaries: hit.summaries,
        indexedFileNames,
      },);
      omissionLedger.record({
        fileName: sourceFile.fileName,
        source: hit,
        dependencies: restoredDependencies,
      },);
      if (hit.shared && sharedVerificationEnabled()) {
        verifySharedSource({
          project,
          sourceFile,
          scopeKey,
          hit,
          buildIndex,
          ...(analysisRoot === undefined) ? {} : { analysisRoot, },
        },);
      }
      return {
        fileSummaries: hit.summaries,
        dependencies: restoredDependencies,
      };
    }
    if (sharedKey.eligible)
      recordSharedStore('miss',);
    /**
     Fresh direct summaries and omissions for reached source.
     */
    const scan = scanDirectSummaries({
      project,
      sourceFile,
      allOmittedKeys: omittedCallableKeys,
      buildIndex,
      ...(analysisRoot === undefined) ? {} : { analysisRoot, },
    },);
    reportDirectSummaryOmissions({
      omittedKeys: scan.sourceOmittedCallableKeys,
      sourceFileName: sourceFile.fileName,
      reasons: scan.sourceOmissionReasons,
    },);
    /**
     Owned source dependencies discovered through semantic call edges.
     */
    const dependencies = reachedSourceFileNames({
      fileSummaries: scan.fileSummaries,
      indexedFileNames,
    },);
    /**
     Fresh summaries and omissions awaiting complete closure before persistence.
     */
    const pendingStore: PendingEffectSummaryStore = {
      sourceFile,
      summaries: scan.fileSummaries,
      omittedCallableKeys: [...scan.sourceOmittedCallableKeys,].toSorted(),
      omissionReasons: [...scan.sourceOmissionReasons,].toSorted(),
      externalOmissions: scan.sourceExternalOmissions,
    };
    omissionLedger.record({
      fileName: sourceFile.fileName,
      source: pendingStore,
      dependencies,
    },);
    closureResolver.includeDirectDependencies({
      fileName: sourceFile.fileName,
      dependencies,
    },);
    pendingStores.set(
      sourceFile.fileName,
      pendingStore,
    );
    return {
      fileSummaries: scan.fileSummaries,
      dependencies,
    };
  }

  /**
   Expands graph from one requested source and completes fixed points.
   
   @param sourceFile - Requested owned source root.
   */
  function ensureSource(sourceFile: SourceFile,): void {
    /**
     Work stack of reached source files awaiting direct summaries.
     */
    const pending: SourceFile[] = [sourceFile,];
    /**
     Whether current request expanded graph and requires propagation.
     */
    const expansion = { changed: false, };
    while (pending.length > 0) {
      /**
       Next reached source file.
       */
      const current = pending.pop();
      if ((current === undefined) || loadedFileNames.has(current.fileName,))
        continue;
      loadedFileNames.add(current.fileName,);
      expansion.changed = true;
      analysisBudget.assertAvailable(`source ${current.fileName}`,);
      /**
       Start time for one reached-source cache or semantic analysis.
       */
      const sourceStartedAt = analysisBudget.start();
      /**
       Current source summaries and their semantic dependencies.
       */
      const loaded = loadSource(current,);
      analysisBudget.record({
        startedAt: sourceStartedAt,
        phase: `source ${current.fileName}`,
      },);
      loaded.fileSummaries
        .forEach(function addSummary(
          summary,
          key,
        ): void {
          summaries.set(
            key,
            summary,
          );
        },);
      loaded.dependencies
        .forEach(function enqueueDependency(fileName,): void {
          /**
           Owned source corresponding to semantic edge.
           */
          const dependency = indexedSourceFiles.get(fileName,);
          if ((dependency !== undefined)
            && (!loadedFileNames.has(fileName,)))
            pending.push(dependency,);
        },);
    }
    if (!expansion.changed)
      return;
    /**
     Start time for cache publication and fixed-point completion.
     */
    const finalizationStartedAt = analysisBudget.start();
    assertReachedCallSummaries({
      summaries,
      omittedCallableKeys,
    },);
    pendingStores.forEach(function persistReachedSource(pendingStore,): void {
      /**
       Complete dependency closure after every reached edge is loaded.
       */
      const closure = closureResolver.closureFor(
        pendingStore.sourceFile
          .fileName,
      );
      /**
       Cross-project key of the stored source.
       */
      const storedKey = sharedKeys.keyFor(pendingStore.sourceFile,);
      storeCreatedSummariesForSource({
        identity: sourceIdentity({
          project,
          scopeKey,
          projectDigest,
          sourceFile: pendingStore.sourceFile,
          ...(storedKey.eligible) ? { sharedKeyDigest: storedKey.digest, } : {},
          ...(cacheRootOverride === undefined) ? {} : { cacheRootOverride, },
        },),
        summaries: pendingStore.summaries,
        surfaces: projectFingerprint.surfaces,
        closure,
        omittedCallableKeys: pendingStore.omittedCallableKeys,
        omissionReasons: pendingStore.omissionReasons,
        externalOmissions: pendingStore.externalOmissions,
      },);
    },);
    pendingStores.clear();
    pruneDirectSummaryCache({
      projectKey: configuredProjectConfigFileName({ project, },),
      activeFiles: indexedFileNames,
    },);
    /* The optimistic per-expansion foreign pass is gone with the gate it fed. It narrowed
     * candidates over whatever had been reached, and nothing reads that answer now: the
     * complete backwards closure decides every foreign question and walks the whole configured
     * scope rather than the reached part of it. Keeping the pass would have cost a full
     * inbound grouping after every expansion to produce a value no consumer looks at. */
    propagateEffects(summaries,);
    analysisBudget.record({
      startedAt: finalizationStartedAt,
      phase: 'fixed-point finalization',
    },);
  }

  return {
    get(declaration,): CallableEffectSummary | typeof NO_EFFECT_SUMMARY {
      /**
       Source from exact analyzer snapshot rather than foreign wrapper.
       */
      const sourceFile = indexedSourceFiles.get(declaration.getSourceFile()
        .fileName,);
      if (sourceFile === undefined)
        return NO_EFFECT_SUMMARY;
      ensureSource(sourceFile,);
      /**
       Stable declaration identity shared across source wrappers.
       */
      const key = callableKey(declaration,);
      /**
       Completed summary after demanded closure propagation.
       */
      const summary = summaries.get(key,);
      if (summary === undefined)
        return NO_EFFECT_SUMMARY;
      return effectPublicSummary({
        summary,
        declaration,
      },);
    },
    reachedOmissions(declaration,): readonly ExternalOmissionRecord[] {
      /**
       Source from exact analyzer snapshot rather than foreign wrapper.
       */
      const sourceFile = indexedSourceFiles.get(declaration.getSourceFile()
        .fileName,);
      if (sourceFile === undefined)
        return [];
      ensureSource(sourceFile,);
      return omissionLedger.reachedFrom(sourceFile.fileName,);
    },
    proveForeignBorrowed(declaration,): ReadonlySet<ParameterIndex> {
      /**
       Source from exact analyzer snapshot rather than foreign wrapper.
       */
      const sourceFile = indexedSourceFiles.get(declaration.getSourceFile()
        .fileName,);
      if (sourceFile === undefined) {
        /* An empty set here would read as "proven to own nothing foreign", which is the answer
         * that emits an offer, for a declaration this index never analyzed. `get` already
         * answers `NO_EFFECT_SUMMARY` for the same input, so a caller reaching this point asked
         * about a callable it had no summary for. */
        throw new SemanticBridgeError({
          reason: 'node-not-found',
          message: `Foreign ownership was demanded for ${callableKey(declaration,)}, which is outside indexed owned source.`,
        },);
      }
      ensureSource(sourceFile,);
      /**
       Stable declaration identity shared across source wrappers.
       */
      const key = callableKey(declaration,);
      /**
       Answer from an earlier demand for this same callable.
       */
      const proven = foreignByProvenRoot.get(key,);
      if (proven !== undefined)
        return proven;
      /* The proof is complete or absent, never partial. The reached graph cannot narrow it: the
       * hint that used to gate it was recomputed after each expansion over whatever had been
       * reached, which made the answer a fact about which files the lint run happened to visit
       * first. Measured in one process with no threads: `stateMatches` in
       * `package/electron-infra/core/src/wayland-state.ts` read `foreign=[]` before its
       * siblings were expanded and `foreign=[0]` after, because `wayland-test.ts` declares the
       * marker and reaches it. Skipping is not the safe direction either, since foreign
       * ownership suppresses the readonly offer, so the unproven answer emits an offer the
       * proven one withholds.
       *
       * `completeForeignBorrowedGraph` is order-independent: it walks `indexedSourceFiles`, the
       * whole configured scope, however little has been reached.
       * `doc/troubleshooting/prefer-readonly-parameter-type-thread-nondeterminism.md` records
       * the measurement. Deferring which callables are asked about therefore cannot move an
       * answer, only how many are computed. */
      /**
       Foreign parameters proven for this root, empty when no marker exists to prove from.
       
       A scope whose sources name no marker anywhere can yield no foreign parameter for any
       callable, since the closure walks exactly those sources, so the walk would return
       nothing every time.
       */
      const rootForeign = markerReachable
        ? provenRootEntry({
          completeForeign: completeForeignBorrowedGraph({
            project,
            indexedSourceFiles,
            rootDeclaration: declaration,
            analysisBudget,
            ...(analysisRoot === undefined) ? {} : { analysisRoot, },
          },),
          key,
        },)
        : new Set<ParameterIndex>();
      foreignByProvenRoot.set(
        key,
        rootForeign,
      );
      return rootForeign;
    },
  };
}
