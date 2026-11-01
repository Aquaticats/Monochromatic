/**
 Comparison of a shared cache hit with a fresh analysis of the same source.
 
 @module
 */

import type { SourceFile, } from 'typescript/ast';
import type { Project, } from 'typescript/sync';

import { scanDirectSummaries, } from './direct-source-scan.ts';
import type { EffectSummaryOmissionReason, } from './effect-cache-envelope.ts';
import type { ExternalOmissionRecord, } from './effect-external-omission.ts';
import type { LayeredSummaryCacheHit, } from './effect-summary-cache.ts';
import type { MutableEffectSummary, } from './effect-summary-model.ts';
import type { ExternalEffectIndexBuilder, } from './external-callable-effect.ts';
import { serializeEffectSummaries, } from './effect-summary-serialization.ts';
import { recordSharedHit, } from './shared-analysis-telemetry.ts';

/**
 What one side of the comparison knows about a source.
 */
export type SourceAnalysisSnapshot = {
  readonly summaries: ReadonlyMap<string, MutableEffectSummary>;
  readonly omittedCallableKeys: readonly string[];
  readonly omissionReasons: readonly EffectSummaryOmissionReason[];
  readonly externalOmissions: readonly ExternalOmissionRecord[];
};

/**
 Serializes each summary on its own so a difference can be named by callable.
 
 @param summaries - Direct summaries of one source.
 
 @returns serialized summary text keyed by callable identity.
 */
function serializedByCallable(
  summaries: ReadonlyMap<string, MutableEffectSummary>,
): ReadonlyMap<string, string> {
  return new Map(
    serializeEffectSummaries(summaries,)
      .map(function entryText([key, summary,],): readonly [
        string,
        string,
      ] {
        return [
          key,
          JSON.stringify(summary,),
        ];
      },),
  );
}

/**
 Lists the parts of two snapshots that differ.
 
 @param cached - Snapshot restored from the shared entry.
 
 @param fresh - Snapshot built by analysing the source again.
 
 @returns one description per differing part,
 empty when the snapshots agree.
 */
function differingParts({
  cached,
  fresh,
}: {
  readonly cached: SourceAnalysisSnapshot;
  readonly fresh: SourceAnalysisSnapshot;
},): readonly string[] {
  /**
   Cached summary text by callable.
   */
  const cachedText = serializedByCallable(cached.summaries,);
  /**
   Fresh summary text by callable.
   */
  const freshText = serializedByCallable(fresh.summaries,);
  /**
   Callables present on either side.
   */
  const keys = new Set([
    ...cachedText.keys(),
    ...freshText.keys(),
  ],);
  /**
   Description of every part that differs.
   */
  const parts: string[] = [...keys,]
    .toSorted()
    .flatMap(function describeKey(key,): readonly string[] {
      if (!freshText.has(key,))
        return [`payload: ${key} only in cached entry`,];
      if (!cachedText.has(key,))
        return [`payload: ${key} only in fresh analysis`,];
      return cachedText.get(key,) === freshText.get(key,)
        ? []
        : [`payload: ${key} differs`,];
    },);
  if (JSON.stringify([...cached.omittedCallableKeys,].toSorted(),)
    !== JSON.stringify([...fresh.omittedCallableKeys,].toSorted(),))
    parts.push('omittedCallableKeys differ',);
  if (JSON.stringify([...cached.omissionReasons,].toSorted(),)
    !== JSON.stringify([...fresh.omissionReasons,].toSorted(),))
    parts.push('omissionReasons differ',);
  if (JSON.stringify(cached.externalOmissions,) !== JSON.stringify(fresh.externalOmissions,))
    parts.push('externalOmissions differ',);
  return parts;
}

/**
 Compares a shared hit with a fresh analysis and records the outcome.
 
 @param fileName - Source path.
 
 @param project - Configuration path of the project that read the entry.
 
 @param writerProject - Configuration path of the project that wrote the entry.
 
 @param cached - Snapshot restored from the shared entry.
 
 @param fresh - Snapshot built by analysing the source again.
 
 @example
 ```ts
 verifySharedHit({ fileName, project, writerProject, cached, fresh });
 ```
 */
export function verifySharedHit({
  fileName,
  project,
  writerProject,
  cached,
  fresh,
}: {
  readonly fileName: string;
  readonly project: string;
  readonly writerProject: string;
  readonly cached: SourceAnalysisSnapshot;
  readonly fresh: SourceAnalysisSnapshot;
},): void {
  /**
   Parts that differ between the snapshots.
   */
  const parts = differingParts({
    cached,
    fresh,
  },);
  if (parts.length === 0) {
    recordSharedHit({ crossProject: project !== writerProject, },);
    return;
  }
  recordSharedHit({
    crossProject: project !== writerProject,
    mismatch: {
      fileName,
      project,
      writerProject,
      parts,
    },
  },);
}

/**
 Analyses a source afresh and compares the result with the shared hit that stood in for it.
 
 @param project - Project that read the entry.
 
 @param sourceFile - Source the entry answered for.
 
 @param scopeKey - Persistent scope of the project that read the entry.
 
 @param hit - Shared hit restored from the cache.
 
 @param buildIndex - Builder of the recursive external-analysis index.
 
 @param analysisRoot - External implementation root included despite library classification.
 
 @example
 ```ts
 verifySharedSource({ project, sourceFile, scopeKey, hit, buildIndex });
 ```
 */
export function verifySharedSource({
  project,
  sourceFile,
  scopeKey,
  hit,
  buildIndex,
  analysisRoot,
}: {
  readonly project: Project;
  readonly sourceFile: SourceFile;
  readonly scopeKey: string;
  readonly hit: LayeredSummaryCacheHit;
  readonly buildIndex: ExternalEffectIndexBuilder;
  readonly analysisRoot?: string;
},): void {
  /**
   Fresh scan whose omissions stay out of the build-wide set.
   */
  const scan = scanDirectSummaries({
    project,
    sourceFile,
    allOmittedKeys: new Set<string>(),
    buildIndex,
    ...(analysisRoot === undefined) ? {} : { analysisRoot, },
  },);
  verifySharedHit({
    fileName: sourceFile.fileName,
    project: scopeKey,
    writerProject: hit.writerProject,
    cached: hit,
    fresh: {
      summaries: scan.fileSummaries,
      omittedCallableKeys: [...scan.sourceOmittedCallableKeys,],
      omissionReasons: [...scan.sourceOmissionReasons,],
      externalOmissions: scan.sourceExternalOmissions,
    },
  },);
}
