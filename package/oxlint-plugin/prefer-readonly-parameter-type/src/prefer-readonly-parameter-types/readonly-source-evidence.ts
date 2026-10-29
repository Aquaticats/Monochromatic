import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { Context, } from '@oxlint/plugins';

import {
  callableKey,
  collectAstNodes,
  isEffectCallableDeclaration,
} from './effect-summary-model.ts';
import {
  buildEffectSummaryIndex,
  NO_EFFECT_SUMMARY,
  type EffectSummaryIndex,
} from './effect-summaries.ts';
import {
  readonlyCallableEvidence,
  type ReadonlyCallableEvidence,
} from './readonly-callable-evidence.ts';
import { openSemanticFile, } from './typescript-sync-adapter.ts';

/**
 Rule-independent evidence for one semantic source snapshot.
 
 @example
 ```ts
 evidence.callables.forEach(reportCallable);
 ```
 */
export type ReadonlySourceEvidence = {
  readonly project: ReturnType<typeof openSemanticFile>['project'];
  readonly sourceFile: ReturnType<typeof openSemanticFile>['sourceFile'];
  readonly effectIndex: EffectSummaryIndex;
  readonly callables: readonly ReadonlyCallableEvidence[];
};

/**
 Process-local evidence-cache measurements.
 
 @example
 ```ts
 const before = readonlySourceEvidenceCacheStats();
 ```
 */
export type ReadonlySourceEvidenceCacheStats = {
  readonly computations: number;
  readonly hits: number;
  readonly misses: number;
};

/**
 Rule lifecycle logger.
 */
const l = tagged({ tag: 'readonly-source-evidence', },);

/**
 Evidence keyed by exact immutable semantic source object.
 */
const evidenceBySourceFile = new WeakMap<
  ReturnType<typeof openSemanticFile>['sourceFile'],
  ReadonlySourceEvidence
  >();

/**
 Mutable cache counters exposed as immutable snapshots.
 */
const cacheCounters = {
  computations: 0,
  hits: 0,
  misses: 0,
};

/**
 Returns process-local evidence cache measurements.
 
 @returns immutable counter snapshot.
 
 @example
 ```ts
 readonlySourceEvidenceCacheStats();
 ```
 */
export function readonlySourceEvidenceCacheStats(): ReadonlySourceEvidenceCacheStats {
  return { ...cacheCounters, };
}

/**
 Opens current semantic snapshot and computes parameter evidence at most once.
 
 @param context - Current rule context supplying exact source overlay.
 
 @returns category-neutral evidence shared by every split reporter.
 
 @example
 ```ts
 const evidence = readonlySourceEvidence({ context });
 ```
 */
export function readonlySourceEvidence({
  context,
}: ForeignBorrowed<{
  readonly context: Context;
}>,): ReadonlySourceEvidence {
  /**
   Semantic file session for current Oxlint source overlay.
   */
  const session = openSemanticFile({
    fileName: context.filename,
    sourceText: context.sourceCode
      .text,
    hasBOM: context.sourceCode
      .hasBOM,
  },);
  /**
   Evidence previously completed for exact immutable source snapshot.
   */
  const cached = evidenceBySourceFile.get(session.sourceFile,);
  if (cached !== undefined) {
    cacheCounters.hits += 1;
    return cached;
  }
  cacheCounters.misses += 1;
  /**
   Whole-project callable effect summaries.
   */
  const effectIndex = buildEffectSummaryIndex({
    project: session.project,
    activeSourceFile: session.sourceFile,
  },);
  /**
   Implemented callables whose complete summaries and facts are available.
   */
  const callables = collectAstNodes(session.sourceFile,)
    .flatMap(function collectEvidence(semanticNode,): readonly ReadonlyCallableEvidence[] {
      if ((!isEffectCallableDeclaration(semanticNode,))
        || (!('body' in semanticNode))
        || (semanticNode.body === undefined))
        return [];
      /**
       Effect summary for current callable declaration.
       */
      const effectSummary = effectIndex.get(semanticNode,);
      if (effectSummary === NO_EFFECT_SUMMARY) {
        l.warn(`skipping ${callableKey(semanticNode,)}, which the effect index omitted`,);
        return [];
      }
      return [
        readonlyCallableEvidence({
          declaration: semanticNode,
          effectSummary,
          project: session.project,
          /**
           Demands complete foreign-ownership proof only when callable evidence reads it.
           
           @returns parameters held under foreign ownership.
           */
          proveForeignBorrowed(): ReturnType<typeof effectIndex.proveForeignBorrowed> {
            return effectIndex.proveForeignBorrowed(semanticNode,);
          },
        },),
      ];
    },);
  /**
   Completed immutable evidence safe for cross-rule reuse.
   */
  const evidence: ReadonlySourceEvidence = {
    project: session.project,
    sourceFile: session.sourceFile,
    effectIndex,
    callables,
  };
  evidenceBySourceFile.set(
    session.sourceFile,
    evidence,
  );
  cacheCounters.computations += 1;
  return evidence;
}
