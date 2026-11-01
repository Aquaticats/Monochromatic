/**
 Layered-cache identity for one reached source.
 
 Split out of `effect-demand-index.ts` for the code-line budget. It is the only piece of that
 module that captures nothing from the index it belongs to: everything else there is a closure
 over the graph being expanded, so this is what could move without threading state through a
 new parameter list.
 
 @module
 */

import type { SourceFile, } from 'typescript/ast';
import type { Project, } from 'typescript/sync';

import { configuredProjectConfigFileName, } from './semantic-file-name.ts';

/**
 Address a source's summaries are stored and looked up under.
 */
export type EffectSourceIdentity = {
  readonly projectKey: string;
  readonly scopeKey: string;
  readonly projectDigest: string;
  readonly fileName: string;
  readonly sourceText: string;
  readonly sharedKeyDigest?: string;
  readonly cacheRootOverride?: string;
};

/**
 Builds layered-cache identity for one reached source.
 
 @param project - Semantic project owning cache scope.
 
 @param scopeKey - Persistent scope identity.
 
 @param projectDigest - Exact semantic project identity.
 
 @param sourceFile - Reached source whose summaries are requested.
 
 @param sharedKeyDigest - Cross-project key digest when the source may share its entry.
 
 @param cacheRootOverride - Optional disposable cache root.
 
 @returns shared process and persistent cache identity.
 
 @example
 ```ts
 sourceIdentity({ project, scopeKey, projectDigest, sourceFile });
 ```
 */
export function sourceIdentity({
  project,
  scopeKey,
  projectDigest,
  sourceFile,
  sharedKeyDigest,
  cacheRootOverride,
}: {
  readonly project: Project;
  readonly scopeKey: string;
  readonly projectDigest: string;
  readonly sourceFile: SourceFile;
  readonly sharedKeyDigest?: string;
  readonly cacheRootOverride?: string;
}): EffectSourceIdentity {
  return {
    projectKey: configuredProjectConfigFileName({ project, },),
    scopeKey,
    projectDigest,
    fileName: sourceFile.fileName,
    sourceText: sourceFile.text,
    ...(sharedKeyDigest === undefined) ? {} : { sharedKeyDigest, },
    ...(cacheRootOverride === undefined) ? {} : { cacheRootOverride, },
  };
}
