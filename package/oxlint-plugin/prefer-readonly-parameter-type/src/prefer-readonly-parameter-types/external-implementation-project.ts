/**
 Disposable TypeScript project for shipped package implementation.
 
 @module
 */

import { dirname, } from 'node:path';

import type { SourceFile, } from 'typescript/ast';
import {
  API,
  type Project,
} from 'typescript/sync';

import { effectProjectFingerprint, } from './effect-project-fingerprint.ts';
import { externalProjectConfigPath, } from './external-project-config.ts';
import { externalRuntimeShadowFiles, } from './external-runtime-shadow-files.ts';
import { memoizeApiSnapshotRequests, } from './semantic-request-memo.ts';
import {
  configureNativeApiChildShutdown,
  nativeApiChild,
} from './typescript-sync-native-shutdown.ts';

/**
 Maximum declaration-shadow discovery passes before conservative rejection.
 */
const MAX_RUNTIME_SHADOW_PASSES = 32;

/**
 Sentinel when external implementation project cannot be opened.
 */
export const EXTERNAL_IMPLEMENTATION_PROJECT_UNAVAILABLE: unique symbol = Symbol(
  'external implementation configured project was unavailable',
);

/**
 Disposable TypeScript project over one shipped package implementation.
 */
export type ExternalImplementationSession = {
  readonly project: Project;
  readonly implementationSource: SourceFile;
  readonly snapshotIdentity: string;
  readonly close: () => void;
};

/**
 Reusable external implementation sessions by generated config path.
 */
const sessionByConfig = new Map<string, ExternalImplementationSession>();

/**
 Sentinel for the external API before it is started.
 */
const NO_EXTERNAL_API = undefined;

/**
 One native child shared by every external implementation project in this process.
 
 A child was created per generated config, mid-lint, and under oxlint's default worker count that
 spawn fails with `ENOMEM`: the analysis then reports every external call as unresolved, which is sound
 and silently discards the entire channel. Measured at every thread count on a 16-core host, the
 channel works through eight workers and dies at sixteen, so the deciding quantity is the reserved size
 at spawn time rather than merely spawning after the reservations exist.
 */
const externalApiState: { current: API | typeof NO_EXTERNAL_API; } = { current: NO_EXTERNAL_API, };

/**
 Starts the shared external child before oxlint reserves its per-worker buffers.
 
 Called beside `initializeSemanticBridge` for exactly the reason that call exists, and the cost is one
 extra native child per lint process whether or not any external call appears.
 
 @example
 ```ts
 initializeExternalImplementationApi();
 ```
 */
export function initializeExternalImplementationApi(): void {
  if (externalApiState.current !== NO_EXTERNAL_API)
    return;
  /**
   Shared client whose cwd is the process cwd, since every generated project names absolute files.
   */
  const api = new API({ cwd: process.cwd(), },);
  configureNativeApiChildShutdown(nativeApiChild(api,),);
  memoizeApiSnapshotRequests({ api, },);
  externalApiState.current = api;
}

/**
 Computes stable identity from project source snapshot signatures.
 
 @param project - External implementation project.
 
 @param implementationSource - Runtime entry source.
 
 @returns deterministic process snapshot identity.
 */
function externalSnapshotIdentity({
  project,
  implementationSource,
}: {
  readonly project: Project;
  readonly implementationSource: SourceFile;
}): string {
  return effectProjectFingerprint({
    project,
    activeSourceFile: implementationSource,
  },)
    .digest;
}

/**
 Opens one generated configured project without publishing it to process cache.
 
 @param configPath - Generated project configuration.
 
 @param implementationPath - Exact runtime source required from project.
 
 @returns disposable project session or unavailable sentinel.
 */
function openExternalProject({
  configPath,
  implementationPath,
}: {
  readonly configPath: string;
  readonly implementationPath: string;
}): ExternalImplementationSession | typeof EXTERNAL_IMPLEMENTATION_PROJECT_UNAVAILABLE {
  /* The shared child when one was started early, and a fresh one otherwise, so a consumer that never
   * called the initializer still works wherever the spawn can succeed. */
  if (externalApiState.current === NO_EXTERNAL_API)
    initializeExternalImplementationApi();
  /**
   Client hosting this generated project, shared across every external implementation.
   */
  const api = externalApiState.current ?? new API({ cwd: dirname(configPath,), },);
  /**
   Configured project snapshot for shipped implementation.
   
   TypeScript 7.1 creates independent snapshots (`api.createSnapshot`) where 7.0 added the project to one shared
   session with `api.updateSnapshot`; each external implementation holds and releases its own view. `ensurePrograms`
   forces the eager program rebuild 7.0 did by default, so the implementation source below reads fresh text.
   */
  const snapshot = api.createSnapshot({
    openProjects: [configPath,],
    ensurePrograms: true,
  },);
  /**
   Loaded generated project.
   */
  const project = snapshot.getConfiguredProject(configPath,);
  /**
   Shipped implementation source in generated project.
   */
  const implementationSource = project?.program
    .getSourceFile(implementationPath,);
  /* Only the snapshot is released here, never the client. It is shared by every generated project in
   * this process, so closing it on one project's failure would take the channel down for the rest. */
  if ((project === undefined) || (implementationSource === undefined)) {
    snapshot.dispose();
    return EXTERNAL_IMPLEMENTATION_PROJECT_UNAVAILABLE;
  }
  return {
    project,
    implementationSource,
    snapshotIdentity: externalSnapshotIdentity({
      project,
      implementationSource,
    },),
    close(): void {
      snapshot.dispose();
    },
  };
}

/**
 Opens disposable TypeScript project for shipped implementation.
 
 @param consumerProject - Caller project selecting cache root.
 
 @param packageRoot - Exact package root accepted by analyzer.
 
 @param implementationPath - Shipped implementation entry path.
 
 @param implementationDigest - Runtime and source-map content identity.
 
 @returns disposable semantic implementation session.
 
 @example
 ```ts
 const session = openExternalImplementation({
   consumerProject,
   packageRoot,
   implementationPath,
   implementationDigest,
 });
 ```
 */
export function openExternalImplementation({
  consumerProject,
  packageRoot,
  implementationPath,
  implementationDigest,
}: {
  readonly consumerProject: Project;
  readonly packageRoot: string;
  readonly implementationPath: string;
  readonly implementationDigest: string;
}): ExternalImplementationSession | typeof EXTERNAL_IMPLEMENTATION_PROJECT_UNAVAILABLE {
  /**
   Runtime roots discovered from exact entry and declaration-shadowed imports.
   */
  const implementationFiles = { current: [implementationPath,], };
  for (let pass = 0; pass < MAX_RUNTIME_SHADOW_PASSES; pass++) {
    /**
     Generated configured-project path for current reached roots.
     */
    const configPath = externalProjectConfigPath({
      consumerProject,
      packageRoot,
      implementationPath,
      implementationDigest,
      implementationFiles: implementationFiles.current,
    },);
    /**
     Prior process-local final session for exact reached roots.
     */
    const cached = sessionByConfig.get(configPath,);
    if (cached !== undefined)
      return cached;
    /**
     Current project used to discover declaration-shadowed runtime siblings.
     */
    const session = openExternalProject({
      configPath,
      implementationPath,
    },);
    if (session === EXTERNAL_IMPLEMENTATION_PROJECT_UNAVAILABLE)
      return session;
    /**
     Runtime siblings reached through declarations selected by module resolution.
     */
    const nextFiles = [...new Set([
      ...implementationFiles.current,
      ...externalRuntimeShadowFiles({
        project: session.project,
        packageRoot,
      },),
    ],),].toSorted();
    if (nextFiles.length
      === implementationFiles.current
        .length) {
      sessionByConfig.set(
        configPath,
        session,
      );
      return session;
    }
    session.close();
    implementationFiles.current = nextFiles;
  }
  return EXTERNAL_IMPLEMENTATION_PROJECT_UNAVAILABLE;
}

/**
 Closes every process-local external implementation project.
 
 @example
 ```ts
 closeExternalImplementationProjects();
 ```
 */
export function closeExternalImplementationProjects(): void {
  sessionByConfig.forEach(function closeSession(session,): void {
    session.close();
  },);
  sessionByConfig.clear();
  if (externalApiState.current === NO_EXTERNAL_API)
    return;
  externalApiState.current
    .close();
  externalApiState.current = NO_EXTERNAL_API;
}
