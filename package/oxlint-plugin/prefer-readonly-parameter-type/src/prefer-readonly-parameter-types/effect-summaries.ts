/**
 Demand-driven parameter mutation summaries over TypeScript 7 semantic AST.
 
 @module
 */

import type { SourceFile, } from 'typescript/ast';
import type { Project, } from 'typescript/sync';

import { isDeclarationFileName, } from './declaration-file-name.ts';
import { createEffectAnalysisBudget, } from './effect-analysis-budget.ts';
import { createDemandDrivenEffectIndex, } from './effect-demand-index.ts';
import {
  cachedFinalEffectIndex,
  cacheFinalEffectIndex,
  FINAL_EFFECT_INDEX_CACHE_MISS,
} from './effect-final-index-cache.ts';
import { effectProjectFingerprint, } from './effect-project-fingerprint.ts';
import { contentDigest, } from './effect-summary-cache-identity.ts';
import {
  collectAstNodes,
  isEffectCallableDeclaration,
} from './effect-summary-model.ts';
import {
  type CallableEffectSummary,
  type EffectSummaryIndex,
  NO_EFFECT_SUMMARY,
} from './effect-summary-index.ts';
import { isWorkspaceSourceFileName, } from './workspace-source-path.ts';

export {
  type CallableEffectSummary,
  type EffectSummaryIndex,
  NO_EFFECT_SUMMARY,
} from './effect-summary-index.ts';

/**
 One derived inclusion scope: which files it covers and the digest naming them.
 */
type InclusionScope = {
  readonly names: ReadonlySet<string>;
  readonly digest: string;
};

/**
 Inclusion-scope identity per project, so the digest is derived once rather than per file.
 
 `buildEffectSummaryIndex` runs for every linted file and computes its cache key before it can
 consult `cachedFinalEffectIndex`, and that key costs more than the lookup saves: measured at
 30.7ms per call across 2080 calls, roughly 64 of the rule's 171 warm seconds, in
 `doc/planning/oxlint-warm-sweep-attribution.md`.
 
 Safe to share because `indexedSourceFileMap` treats every file except the active one
 identically whichever file is active: `activeSourceFile` decides only its own entry, once to
 substitute the Oxlint overlay and once to include it ahead of every other test. So the key set
 is a property of the project, unioned with the active file.
 
 Only the key set and its digest are kept, never the map. The map's values hold the overlay
 `SourceFile` of whichever run built it, which must not be handed to a later run, and the
 early-return path needs the digest alone.
 
 Keyed on the project object as `effect-final-index-cache.ts` keys its own store, so a new
 semantic snapshot reaches none of these entries.
 */
const inclusionScopeByProject = new WeakMap<
  Project,
  Map<string, InclusionScope>
  >();

/**
 Selects exact non-declaration source scope admitted by ownership policy.
 
 @param project - TypeScript project providing configured sources.
 
 @param activeSourceFile - Current Oxlint overlay source.
 
 @param fileNames - Stable configured project membership.
 
 @param analysisRoot - Optional external implementation root.
 
 @returns source wrappers keyed by exact path.
 */
function indexedSourceFileMap({
  project,
  activeSourceFile,
  fileNames,
  analysisRoot,
}: {
  readonly project: Project;
  readonly activeSourceFile: SourceFile;
  readonly fileNames: readonly string[];
  readonly analysisRoot?: string;
}): ReadonlyMap<string, SourceFile> {
  return new Map(fileNames.flatMap(function retainIndexedSource(fileName,): readonly (readonly [
    string,
    SourceFile,
  ])[] {
    /* Declaration files are refused a line below whichever way they are recognised, and asking
     * `getSourceFile` decodes an abstract syntax tree to answer it. Recognising them by name
     * first costs no decode and refuses exactly the same files. */
    if (isDeclarationFileName(fileName,))
      return [];
    /**
     Program source matching configured path or exact active wrapper.
     */
    const sourceFile = fileName === activeSourceFile.fileName
      ? activeSourceFile
      : project.program
        .getSourceFile(fileName,);
    if ((sourceFile === undefined) || sourceFile.isDeclarationFile)
      return [];
    if (sourceFile.fileName === activeSourceFile.fileName)
      return [[
        sourceFile.fileName,
        sourceFile,
      ],];
    if (!project
      .program
      .isSourceFileFromExternalLibrary(sourceFile,))
      return [[
        sourceFile.fileName,
        sourceFile,
      ],];
    /* Symlink-resolved workspace dependencies classify as external while
     * living at repository paths; their source stays inspectable. */
    if (isWorkspaceSourceFileName(fileName,))
      return [[
        sourceFile.fileName,
        sourceFile,
      ],];
    if ((analysisRoot !== undefined) && fileName.startsWith(analysisRoot,))
      return [[
        sourceFile.fileName,
        sourceFile,
      ],];
    return [];
  },),);
}

/**
 Seeds requested active source into a mutable demand index.
 
 @param index - Exact-snapshot index to expand.
 
 @param activeSourceFile - Current Oxlint source whose callables are linted.
 */
function includeActiveSource({
  index,
  activeSourceFile,
}: {
  readonly index: EffectSummaryIndex;
  readonly activeSourceFile: SourceFile;
}): void {
  collectAstNodes(activeSourceFile,)
    .forEach(function includeCallable(node,): void {
      if (isEffectCallableDeclaration(node,))
        index.get(node,);
    },);
}

/**
 Builds effect summaries from active source and reached owned callables.
 
 The mutable process index expands when later Oxlint visitors request another
 active source from the same exact project snapshot.
 
 @param project - TypeScript project snapshot to analyze.
 
 @param activeSourceFile - Current overlay source wrapper used by verifier.
 
 @param cacheRootOverride - Optional disposable persistent cache root used by tests.
 
 @param analysisRoot - Optional external implementation root included despite library classification.
 
 @param analysisBudgetMilliseconds - Optional disposable fail-closed budget used by tests.
 
 @returns exact declaration summary lookup.
 
 @example
 ```ts
 const effects = buildEffectSummaryIndex({ project, activeSourceFile });
 ```
 */
export function buildEffectSummaryIndex({
  project,
  activeSourceFile,
  cacheRootOverride,
  analysisRoot,
  analysisBudgetMilliseconds,
}: {
  readonly project: Project;
  readonly activeSourceFile: SourceFile;
  readonly cacheRootOverride?: string;
  readonly analysisRoot?: string;
  readonly analysisBudgetMilliseconds?: number;
}): EffectSummaryIndex {
  /**
   Cumulative fail-closed budget for project identity and reached analysis.
   */
  const analysisBudget = createEffectAnalysisBudget(analysisBudgetMilliseconds,);
  /**
   Process cache identity including optional external analysis scope.
   */
  const cacheProjectKey = `${project.configFileName}\0${analysisRoot ?? ''}\0${String(analysisBudgetMilliseconds,)}`;
  /**
   Inclusion scopes already derived for this project, by cache partition.
   */
  const scopesForProject = inclusionScopeByProject.get(project,)
    ?? new Map<string, InclusionScope>();
  inclusionScopeByProject.set(
    project,
    scopesForProject,
  );
  /**
   Inclusion scope from an earlier file of this project, when one applies to this file.
   
   Applies only when the active file is already in that scope. When it is not, the union adds
   a name and the digest differs, so the scope is rebuilt rather than adjusted.
   */
  const reusableScope = scopesForProject.get(cacheProjectKey,);
  /**
   Files that scope covers, absent when this project has derived none yet.
   */
  const reusableNames = reusableScope?.names;
  if ((reusableScope !== undefined)
    && (reusableNames !== undefined)
    && reusableNames.has(activeSourceFile.fileName,)) {
    /**
     Index reusable for this exact inclusion scope, without deriving the scope again.
     */
    const scopedIndex = cachedFinalEffectIndex({
      project,
      projectKey: cacheProjectKey,
      fileListDigest: reusableScope.digest,
    },);
    if (scopedIndex !== FINAL_EFFECT_INDEX_CACHE_MISS) {
      includeActiveSource({
        index: scopedIndex,
        activeSourceFile,
      },);
      return scopedIndex;
    }
  }
  /* Built here rather than above, because only the scope derivation reads it and the fast path
   * returns before reaching it. Above, every reused scope still paid for a program membership
   * call, a set, a spread and a sort of some hundreds of names. */
  /**
   Stable configured project membership including active external overlay.
   */
  const fileNames = [...new Set([
    ...project
      .program
      .getSourceFileNames(),
    activeSourceFile.fileName,
  ],),].toSorted();
  /**
   Exact source scope admitted by current ownership policy.
   */
  const indexedSourceFiles = indexedSourceFileMap({
    project,
    activeSourceFile,
    fileNames,
    ...(analysisRoot === undefined) ? {} : { analysisRoot, },
  },);
  /**
   Complete inclusion-scope identity for process-local index reuse.
   */
  const indexedFileListDigest = contentDigest(
    [...indexedSourceFiles.keys(),]
      .toSorted()
      .join('\0',),
  );
  /**
   Inclusion scope recorded for the next file of this project, so it derives none of this.
   */
  const derivedScope: InclusionScope = {
    names: new Set(indexedSourceFiles.keys(),),
    digest: indexedFileListDigest,
  };
  scopesForProject.set(
    cacheProjectKey,
    derivedScope,
  );
  /**
   Mutable demand index reusable for exact TypeScript semantic snapshot.
   */
  const cachedIndex = cachedFinalEffectIndex({
    project,
    projectKey: cacheProjectKey,
    fileListDigest: indexedFileListDigest,
  },);
  if (cachedIndex !== FINAL_EFFECT_INDEX_CACHE_MISS) {
    includeActiveSource({
      index: cachedIndex,
      activeSourceFile,
    },);
    return cachedIndex;
  }
  /**
   Exact project surfaces and per-source content identities.
   */
  const fingerprintStartedAt = analysisBudget.start();
  /**
   Exact project surfaces and per-source content identities.
   */
  const projectFingerprint = effectProjectFingerprint({
    project,
    activeSourceFile,
  },);
  analysisBudget.record({
    startedAt: fingerprintStartedAt,
    phase: 'project fingerprint',
  },);
  /**
   Persistent and process direct-summary identity including scope policy.
   */
  const projectDigest = contentDigest(
    `${projectFingerprint.digest}\0${analysisRoot ?? ''}`,
  );
  /**
   Persistent cache scope separating external analysis roots.
   */
  const scopeKey = `${project.configFileName}\0${analysisRoot ?? ''}`;
  /**
   Fresh mutable demand index for exact project snapshot.
   */
  const index = createDemandDrivenEffectIndex({
    project,
    indexedSourceFiles,
    projectFingerprint,
    scopeKey,
    projectDigest,
    buildIndex: buildEffectSummaryIndex,
    analysisBudget,
    ...(cacheRootOverride === undefined) ? {} : { cacheRootOverride, },
    ...(analysisRoot === undefined) ? {} : { analysisRoot, },
  },);
  includeActiveSource({
    index,
    activeSourceFile,
  },);
  cacheFinalEffectIndex({
    project,
    projectKey: cacheProjectKey,
    fileListDigest: indexedFileListDigest,
    index,
  },);
  return index;
}
