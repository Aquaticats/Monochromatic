/**
 Fresh direct-summary scan of one reached source.
 
 Split out of `effect-demand-index.ts` so the verification of a cached entry can run the same scan
 the cache miss path runs, without the miss path's reporting or cache writes.
 
 @module
 */

import type { SourceFile, } from 'typescript/ast';
import type { Project, } from 'typescript/sync';

import { directEffectSummary, } from './direct-effect-summary.ts';
import type { EffectSummaryOmissionReason, } from './effect-cache-envelope.ts';
import {
  type ExternalOmissionRecord,
  mergeExternalOmissions,
} from './effect-external-omission.ts';
import { recordDirectSummaryOmission, } from './effect-summary-omission.ts';
import {
  callableKey,
  collectAstNodes,
  type EffectCallableDeclaration,
  isEffectCallableDeclaration,
  type MutableEffectSummary,
} from './effect-summary-model.ts';
import {
  externalCallableEffect,
  type ExternalEffectIndexBuilder,
} from './external-callable-effect.ts';

/**
 Direct summaries of one source with the omissions met while building them.
 */
export type DirectSourceScan = {
  readonly fileSummaries: ReadonlyMap<string, MutableEffectSummary>;
  readonly sourceOmittedCallableKeys: ReadonlySet<string>;
  readonly sourceOmissionReasons: ReadonlySet<EffectSummaryOmissionReason>;
  readonly sourceExternalOmissions: readonly ExternalOmissionRecord[];
};

/**
 Builds direct summaries for every effect callable of one source.
 
 One callable whose summary cannot be built must not cost the rest of the file.
 Omitting it is what makes the rest analyzable, and omission is fail-closed on both sides:
 its callers hit the absent-callee branch in `effect-fixed-point-propagation.ts` and take
 opacity, while the rule skips verifying it rather than reporting an internal failure against
 code whose author cannot act on it. The live cause is an upstream panic recorded in
 `doc/troubleshooting/typescript-go-tuple-type-panic.md`, which no ordering of API calls avoids.
 
 @param project - Project providing the checker.
 
 @param sourceFile - Source whose callables are summarized.
 
 @param allOmittedKeys - Build-wide omissions the scan adds to.
 
 @param analysisRoot - External implementation root included despite library classification.
 
 @param buildIndex - Builder of the recursive external-analysis index.
 
 @returns summaries,
 source-local omissions,
 and the omissions met inside installed package implementations whose effects the summaries applied.
 
 @mutates allOmittedKeys - Adds every omitted callable identity.
 
 @example
 ```ts
 const scan = scanDirectSummaries({ project, sourceFile, allOmittedKeys, buildIndex });
 ```
 */
export function scanDirectSummaries({
  project,
  sourceFile,
  allOmittedKeys,
  analysisRoot,
  buildIndex,
}: {
  readonly project: Project;
  readonly sourceFile: SourceFile;
  readonly allOmittedKeys: Set<string>;
  readonly analysisRoot?: string;
  readonly buildIndex: ExternalEffectIndexBuilder;
},): DirectSourceScan {
  /**
   Callable declarations decoded only for reached source.
   */
  const declarations = collectAstNodes(sourceFile,)
    .filter(function retainEffectCallable(node,): node is EffectCallableDeclaration {
      return isEffectCallableDeclaration(node,);
    },);
  /**
   Callable identities omitted while scanning this exact source.
   */
  const sourceOmittedCallableKeys = new Set<string>();
  /**
   Bounded reasons for every omission encountered in current source.
   */
  const sourceOmissionReasons = new Set<EffectSummaryOmissionReason>();
  /**
   Omissions each installed package call met inside the package implementation.
   */
  const externalOmissionGroups: (readonly ExternalOmissionRecord[])[] = [];
  /**
   Fresh direct summaries for reached source.
   */
  const fileSummaries = new Map(declarations.flatMap(function gatherCallable(declaration,): readonly [
    string,
    MutableEffectSummary,
  ][] {
    try {
      return [[
        callableKey(declaration,),
        directEffectSummary({
          project,
          declaration,
          ...(analysisRoot === undefined) ? {} : { analysisRoot, },
          externalEffectResolver({
            consumerProject,
            call,
            declaration: externalDeclaration,
          },) {
            return externalCallableEffect({
              consumerProject,
              call,
              declaration: externalDeclaration,
              buildIndex,
              onOmissions(records,): void {
                externalOmissionGroups.push(records,);
              },
            },);
          },
        },),
      ],];
    }
    catch (error) {
      recordDirectSummaryOmission({
        allOmittedKeys,
        sourceOmittedKeys: sourceOmittedCallableKeys,
        sourceOmissionReasons,
        key: callableKey(declaration,),
        error,
      },);
      return [];
    }
  },),);
  return {
    fileSummaries,
    sourceOmittedCallableKeys,
    sourceOmissionReasons,
    sourceExternalOmissions: mergeExternalOmissions(externalOmissionGroups,),
  };
}
