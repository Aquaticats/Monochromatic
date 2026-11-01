/**
 Cross-platform source-path identity for TypeScript semantic snapshots.
 
 @module
 */

import {
  dirname,
  resolve,
} from 'node:path';

import type { Project, } from 'typescript/sync';

import { SemanticBridgeError, } from './semantic-bridge-error.ts';

/**
 Normalizes source path for overlay and project lookup.
 
 TypeScript treats Windows source paths case-insensitively and may return a lowercased drive path with forward
 separators even when Oxlint supplied original casing and backslashes.
 Case folding keeps overlay callbacks and snapshot file changes on one identity while preserving case on other hosts.
 
 @param fileName - Host-provided absolute or relative source path.
 
 @returns absolute platform-normalized source path with Windows case folded for TypeScript identity.
 
 @example
 ```ts
 normalizeSemanticFileName('src/index.ts');
 ```
 */
export function normalizeSemanticFileName(fileName: string,): string {
  /**
   Absolute path before host case-sensitivity normalization.
   */
  const absoluteFileName = resolve(fileName,);
  return process.platform === 'win32'
    ? absoluteFileName.toLowerCase()
    : absoluteFileName;
}

/**
 Keys a discovered configured project by root directory in bridge source identity.
 
 TypeScript spells `configFileName` its own way, which on Windows carries forward separators and
 original casing, while every path looked up against this map has been through
 {@link normalizeSemanticFileName} and so carries backslashes and lower case. A key written in
 TypeScript's spelling is a key the ancestor walk in `cachedProjectForFile` cannot produce, and
 the lookup then misses for the life of the process. Both sides pass through here instead.
 
 @param configFileName - Configured project path as TypeScript spells it.
 
 @returns project root directory in same identity as normalized source paths.
 
 @example
 ```ts
 semanticProjectRootKey('/repo/package/logger/core/tsconfig.json');
 ```
 */
export function semanticProjectRootKey(configFileName: string,): string {
  return dirname(normalizeSemanticFileName(configFileName,),);
}

/**
 Config file path a configured project was loaded from.
 
 Configured-only analysis keys caches and paths by this value. The bridge refuses inferred projects before analysis
 starts (`typescript-sync-adapter` throws `project-not-found` first), so a project without one here means a snapshot
 slipped an inferred or synthetic project into configured-only work, where an undefined key would silently collide.
 
 @param project - Project handed to configured-only analysis.
 
 @returns config file path for the project, spelled the way TypeScript spells it.
 
 @throws {@link SemanticBridgeError} when the project has no config file.
 
 @example
 ```ts
 const configFileName = configuredProjectConfigFileName({ project, },);
 ```
 */
export function configuredProjectConfigFileName({
  project,
}: {
  readonly project: Project;
},): string {
  /**
   Config file name TypeScript resolved for the project.
   */
  const { configFileName, } = project;
  if (configFileName === undefined)
    throw new SemanticBridgeError({
      reason: 'project-not-found',
      message: 'TypeScript project reached configured-only analysis without a config file.',
    },);
  return configFileName;
}
