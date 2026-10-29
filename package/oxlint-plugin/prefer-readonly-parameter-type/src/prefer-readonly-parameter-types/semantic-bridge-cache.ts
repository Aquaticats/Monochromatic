/**
 Bounded cache lookup for TypeScript semantic bridge.
 
 @module
 */

import { existsSync, } from 'node:fs';
import {
  dirname,
  join,
} from 'node:path';

import { ancestorDirectories, } from './ancestor-directories.ts';

/**
 Semantic bridge cache evidence for lifecycle verification.
 
 `overlayCount` and `projectRootCount` are bounded sizes, one entry per source and per configured
 root a process reaches. `projectDiscoveryCount` is not a size but a running total, counting how
 often the bridge had to ask TypeScript which project owns a source, and it resets only with
 `closeSemanticBridge`. That total is what a root that never matches costs. Every other count
 stays put under such a miss, so it is the one number that tells a served cache from a bypassed
 one, and the evidence a host with its own path spelling can fail on.
 
 @example
 ```ts
 const stats: SemanticBridgeCacheStats = {
   overlayCount: 1,
   projectRootCount: 1,
   projectDiscoveryCount: 1,
 };
 ```
 */
export type SemanticBridgeCacheStats = {
  readonly overlayCount: number;
  readonly projectRootCount: number;
  readonly projectDiscoveryCount: number;
};

/**
 Whether a directory declares a configured project, remembered per process.
 
 The walk runs for every linted source and asks about the same few directories, so the answers
 are kept. A configuration file created during a sweep is not picked up, which matches the
 project snapshot that sweep is already reading.
 */
const configPresenceByDirectory = new Map<string, boolean>();

/**
 Tests whether a directory declares a configured project.
 
 @param directory - Ancestor directory of a source being opened.
 
 @returns whether directory holds `tsconfig.json`.
 
 @example
 ```ts
 declaresProject('/repo/package/logger/core');
 ```
 */
function declaresProject(directory: string,): boolean {
  /**
   Remembered answer for this directory, absent on first ask.
   */
  const remembered = configPresenceByDirectory.get(directory,);
  if (remembered !== undefined)
    return remembered;
  /**
   Whether a configuration file sits here.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-sync -- Synchronous visitor settles project identity before the synchronous snapshot it will read.
  const present = existsSync(join(
    directory,
    'tsconfig.json',
  ),);
  configPresenceByDirectory.set(
    directory,
    present,
  );
  return present;
}

/* oxlint-disable no-restricted-syntax/no-nullish-union -- Map lookup requires undefined fallback sentinel. */
/**
 Finds the discovered configured project nearest a source, refusing farther ones.
 
 Walks upward and stops at the first directory that either names a discovered project or
 declares an undiscovered one. Stopping at an undiscovered configuration is the point. Walking
 past it and answering with an ancestor is how a package source came to be analysed under the
 repository root, decided by nothing but which sources that worker reached earlier.
 
 `openSemanticFile` checks that the project it gets actually contains the source, which catches
 a wrong answer for most projects. It does not catch this one: the root `tsconfig.json` declares
 no `include`, so its program holds package sources too and the check passes.
 
 Measured on `package/logger/core/src/create-logger.ts`. Its own project analyses 117 sources
 and the root project 196, but 62 of the 117 are absent from the root. Those 62 are callers, and
 a caller that is not read is a mutation that is not charged, which is exactly how a wrong
 read-only offer is minted.
 
 Answering absent where a nearer project has not been discovered costs one discovery. Answering
 with an ancestor costs correctness.
 
 @param fileName - Canonical absolute source path.
 
 @param projectByRoot - Configured project paths keyed by root directory.
 
 @returns configured project path or discovery sentinel.
 
 @example
 ```ts
 cachedProjectForFile({ fileName, projectByRoot });
 ```
 */
export function cachedProjectForFile({
  fileName,
  projectByRoot,
}: {
  readonly fileName: string;
  readonly projectByRoot: ReadonlyMap<string, string>;
}): string | undefined {
  for (const directory of ancestorDirectories(dirname(fileName,),)) {
    /**
     Project already discovered at this ancestor, absent when none was.
     */
    const discovered = projectByRoot.get(directory,);
    if (discovered !== undefined)
      return discovered;
    if (declaresProject(directory,))
      return undefined;
  }
  return undefined;
}
/* oxlint-enable no-restricted-syntax/no-nullish-union */
