/**
 File-open phases of the TypeScript 7 synchronous semantic bridge for Oxlint JavaScript rules.
 
 @module
 */

import { existsSync, } from 'node:fs';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { version as typescriptVersion, } from 'typescript';
import type {
  API,
  CreateSnapshotParams,
  Snapshot,
} from 'typescript/sync';

import {
  cachedProjectForFile,
  type SemanticBridgeCacheStats,
} from './semantic-bridge-cache.ts';
import { SemanticBridgeError, } from './semantic-bridge-error.ts';
import {
  bridgeState,
  getApi,
  NO_ACTIVE_FILE,
  NO_SNAPSHOT,
  retireSemanticBridgeAfterChannelFailure,
} from './semantic-bridge-lifecycle.ts';
import { isTerminalSemanticChannelFailure, } from './semantic-channel-failure.ts';
import {
  semanticFileSession,
  type SemanticFileSession,
} from './semantic-file-session.ts';
import {
  normalizeSemanticFileName,
  semanticProjectRootKey,
} from './semantic-file-name.ts';
import { snapshotHoldsSource, } from './semantic-snapshot-presence.ts';
import { sourceWithBOM, } from './semantic-source-text.ts';

/**
 Package logger for semantic bridge lifecycle.
 */
const l = tagged({ tag: 'prefer-readonly-parameter-types', },);

/**
 TypeScript project-service identity for source outside configured projects.
 */
const INFERRED_PROJECT_CONFIG = '/dev/null/inferred';

/**
 Builds the snapshot derivation chain for one source open.

 The chain head starts from the snapshot the service answers and advances with
 every derived snapshot. Keeping the head inside this helper contains its mutation
 at one boundary instead of leaking a function-scope cursor.

 @param api - Live native API client.

 @param first - Snapshot the service answers, or its absence sentinel.

 @returns deriveSnapshot applying opens, closes, and file notifications.
 */
function snapshotDerivationChain({
  api,
  first,
}: {
  readonly api: API;
  readonly first: Snapshot | typeof NO_SNAPSHOT;
},): (params: CreateSnapshotParams) => Snapshot {
  /**
   Newest derived snapshot every later update chains from, starting from the one the service answers.
   */
  let chainedSnapshot: Snapshot | typeof NO_SNAPSHOT = first;
  /**
   Derives the next immutable snapshot from the chained one and keeps it as the new chain head.

   TypeScript 7.1 scopes updates to a snapshot handle (`snapshot.update`) and creates the first one with
   `api.createSnapshot`, where 7.0 rebuilt the shared session on every `api.updateSnapshot` call. Chaining
   reproduces the 7.0 order: each call sees the previous one's opens, closes, and file notifications.

   `ensurePrograms: true` keeps 7.0's eager program rebuilds. 7.1 updates programs lazily by default, and this
   bridge reads `program.getSourceFile` text straight from the snapshot it keeps.

   @param params - Opens, closes, and file notifications to apply.

   @returns derived snapshot holding the applied changes.
   */
  function deriveSnapshot(params: CreateSnapshotParams,): Snapshot {
    /**
     Derived snapshot with program updates forced, chained from the current head.
     */
    const next = chainedSnapshot === NO_SNAPSHOT
      ? api
        .createSnapshot({
          ...params,
          ensurePrograms: true,
        },)
      : chainedSnapshot
        .update({
          ...params,
          ensurePrograms: true,
        },);
    chainedSnapshot = next;
    return next;
  }
  return deriveSnapshot;
}

/**
 Opens source through the live channel without classifying request failures.
 
 Terminal channel failures surface raw so {@link openSemanticFile} can retire the
 bridge around them instead of leaking transport errors to its callers.
 
 @param fileName - Source path reported by Oxlint.
 
 @param sourceText - Current in-memory source text.
 
 @param hasBOM - Whether Oxlint stripped leading byte-order mark.
 
 @returns semantic project, checker, source tree, and offset mapper.
 
 @throws {@link SemanticBridgeError} when project or source cannot be resolved.
 */
function openSemanticFileUnclassified({
  fileName,
  sourceText,
  hasBOM,
}: {
  readonly fileName: string;
  readonly sourceText: string;
  readonly hasBOM: boolean;
},): SemanticFileSession {
  /**
   Function-tagged snapshot lifecycle logger.
   */
  const rl = tagged({
    tag: openSemanticFile.name,
    l,
  },);
  /**
   Canonical source key shared by overlay and project service.
   */
  const normalizedFileName = normalizeSemanticFileName(fileName,);
  /* oxlint-disable no-restricted-syntax/no-sync -- Synchronous Oxlint visitor must classify prior-path deletion before synchronous snapshot update. */
  /**
   Previously active source removed from disk by rename or deletion.
   */
  const deletedFiles = (bridgeState.activeFileName !== NO_ACTIVE_FILE)
    && (bridgeState.activeFileName !== normalizedFileName)
    && (!existsSync(bridgeState.activeFileName,))
    ? [bridgeState.activeFileName,]
    : [];
  /* oxlint-enable no-restricted-syntax/no-sync */
  bridgeState.activeFileName = normalizedFileName;
  /* Every text handed to this bridge is kept, rather than cleared down to the active source.
   *
   * Clearing left the native server holding the text it was last given for a source the overlay
   * no longer claimed, because only the incoming source is reported through `fileNotifications` and
   * nothing ever reported the outgoing one. Reporting it instead would mean a snapshot update on
   * every source, including the reuse path that exists precisely to avoid one, and a snapshot
   * update replaces every `Project` object in the process.
   *
   * Retaining reaches the same invariant for nothing: the server's view of a source is always the
   * text this bridge handed it, never a text it replaced without saying so. Under Oxlint the two
   * agree anyway, since it reads from disk; under a caller supplying an unsaved buffer the buffer
   * is the authority for that source, which is what retaining preserves.
   *
   * Bounded by the sources one process lints, whose text the fingerprint already holds. */
  deletedFiles.forEach(function dropDeletedOverlay(deletedFileName,): void {
    bridgeState.overlays
      .delete(deletedFileName,);
  },);
  bridgeState
    .overlays
    .set(
    normalizedFileName,
    sourceWithBOM({
      sourceText,
      hasBOM,
    },),
    );

  /**
   Previously discovered configured project path for source.
   */
  const knownProject = cachedProjectForFile({
    fileName: normalizedFileName,
    projectByRoot: bridgeState.projectByRoot,
  },);
  /**
   Project already materialized in current immutable snapshot.
   */
  const snapshotProject = (knownProject === undefined)
    || (bridgeState.snapshot === NO_SNAPSHOT)
    ? undefined
    : bridgeState
      .snapshot
      .getConfiguredProject(knownProject,);
  /**
   Source already materialized in current immutable snapshot.
   */
  const snapshotSourceFile = snapshotProject
    ?.program
    .getSourceFile(normalizedFileName,);
  if ((deletedFiles.length === 0)
    && (snapshotProject !== undefined)
    && (snapshotSourceFile !== undefined)
    && (snapshotSourceFile.text === sourceWithBOM({
      sourceText,
      hasBOM,
    },))) {
    rl.debug(`reused unchanged snapshot for ${normalizedFileName} through ${snapshotProject.configFileName}`,);
    return semanticFileSession({
      fileName: normalizedFileName,
      project: snapshotProject,
      sourceFile: snapshotSourceFile,
      hasBOM,
    },);
  }
  /**
   Native API client reused across all linted files in process.
   */
  const api = getApi();
  /* No `clearSourceFileCache()` here. It is `sourceFileCache.clear()`, which drops every decoded
   * source for every project, and the snapshot update below already calls `retainForSnapshot` to
   * carry entries forward for exactly the paths the native server did not report as changed
   * (client-internal names as of 7.0.2).
   * Emptying the store first costs one full re-decode of every project a worker returns to,
   * measured at 154.6ms per project against 0.6ms for a pass that finds them present.
   *
   * What makes dropping it safe is not that the store is hashed. `getRetained` matches on the
   * retained reference and never rechecks the content hash. It is that the only text this bridge
   * ever changes is the active file's overlay, and it reports that file through `fileNotifications`
   * below, so retention excludes it and it alone is refetched.
   *
   * Every text this bridge hands over is kept, so the server's view of a source is always the text
   * it was given, and the file it is about to reread is named through `fileNotifications`. */
  /**
   Whether current source is reusable from project this bridge selected for it.
   */
  const sourcePreviouslyKnown = snapshotSourceFile !== undefined;
  /**
   Snapshot the native service is currently answering from, absent before first update.
   */
  const currentSnapshot = bridgeState.snapshot;
  /* Announcing a source is a different question from reusing one, and deriving the first from the
   * second is how an overlay came to be ignored. A source is created only where the service holds
   * no copy of it at all; where any materialized project holds one, it is changed, and saying
   * created instead leaves the service on the text it read first.
   *
   * The two answers part company whenever the project cache is silent about a source the service
   * already read: a nearer configured project the walk refuses to look past, a source pulled in by
   * an importer under another project, or, before the root key joined one identity, every single
   * lookup on Windows. A cache miss then cost correctness rather than one discovery. */
  /**
   Whether native service already holds this source under any materialized project.
   */
  const serviceHoldsSource = sourcePreviouslyKnown
    || ((currentSnapshot !== NO_SNAPSHOT)
      && snapshotHoldsSource({
        snapshot: currentSnapshot,
        fileName: normalizedFileName,
      },));
  /**
   Whether current source requires open-file project association.
   */
  const needsDiscovery = (knownProject === undefined) || (!sourcePreviouslyKnown);
  /**
   Derives the next immutable snapshot from the chained one and keeps it as the new chain head.
   */
  const deriveSnapshot = snapshotDerivationChain({
    api,
    first: currentSnapshot,
  },);
  /**
   Snapshot used for configured-project discovery on first encounter or created file.
   */
  const discoverySnapshot = needsDiscovery
    ? deriveSnapshot({
      openFiles: [normalizedFileName,],
      fileNotifications: {
        created: serviceHoldsSource ? [] : [normalizedFileName,],
        changed: serviceHoldsSource ? [normalizedFileName,] : [],
        deleted: deletedFiles,
      },
    },)
    : NO_SNAPSHOT;
  /**
   Config path discovered from temporary open-file association or prior cache.
   */
  const discoveredProject = {
    configFileName: sourcePreviouslyKnown ? knownProject : undefined,
  };
  if ((discoveredProject.configFileName === undefined)
    && (discoverySnapshot !== NO_SNAPSHOT)) {
    bridgeState.projectDiscoveryCount += 1;
    discoveredProject.configFileName = discoverySnapshot
      .getDefaultProjectForFile(normalizedFileName,)
      ?.configFileName;
  }
  /**
   Configured project identity after discovery narrowing.
   */
  const { configFileName, } = discoveredProject;
  if ((configFileName === undefined)
    || (configFileName === INFERRED_PROJECT_CONFIG)) {
    /* Drop the refused text before the update that rereads it, not after. Discovery already handed
     * this text to the service, so the service holds it whatever this map says next. Nothing else
     * would ever tell it otherwise: this bridge never reopens a path it refused, and the next
     * source to reach that path reaches it as an import, not as an open. Typing a configured source
     * against text this bridge refused, and that exists nowhere on disk, is the result. */
    bridgeState.overlays
      .delete(normalizedFileName,);
    /**
     Snapshot closing temporary open-file association after failed discovery.
     */
    const releaseSnapshot = deriveSnapshot({
      closeFiles: [normalizedFileName,],
      fileNotifications: {
        changed: [normalizedFileName,],
        created: [],
        deleted: [],
      },
    },);
    /* This failure advanced the service twice, and one of those updates may have reported the
     * previously active source as deleted. Keeping the snapshot from before them would leave the
     * bridge describing a service that still holds a source the service has dropped, and the next
     * open of that path would announce it as changed, which creates nothing: the project then omits
     * it and the open fails. The snapshot that closed the association is the current description,
     * so it is the one to keep. */
    if (bridgeState.snapshot !== NO_SNAPSHOT)
      bridgeState
        .snapshot
        .dispose();
    bridgeState.snapshot = releaseSnapshot;
    if (discoverySnapshot !== NO_SNAPSHOT)
      discoverySnapshot.dispose();
    bridgeState.activeFileName = NO_ACTIVE_FILE;
    throw new SemanticBridgeError({
      reason: 'project-not-found',
      message: `TypeScript found no configured project for ${normalizedFileName}.`,
    },);
  }
  bridgeState
    .projectByRoot
    .set(
    semanticProjectRootKey(configFileName,),
    configFileName,
    );

  /* This update follows the discovery one, so it describes a service that has already processed
   * whatever discovery reported. The source is held by now either way: discovery announced it, or
   * it was never absent. So it is changed here, never created, and the deletions discovery already
   * carried are not replayed. Sampling the announcement once and sending it twice would describe
   * the service as it was before the first update rather than as it is before this one. */
  /**
   New immutable project view reading current overlay outside LSP open-file cache.
   */
  const nextSnapshot = deriveSnapshot({
    ...needsDiscovery
      ? {
        openProjects: [configFileName,],
        closeFiles: [normalizedFileName,],
      }
      : {},
    fileNotifications: {
      changed: [normalizedFileName,],
      created: [],
      deleted: needsDiscovery ? [] : deletedFiles,
    },
  },);
  if (bridgeState.snapshot !== NO_SNAPSHOT)
    bridgeState
      .snapshot
      .dispose();
  if (discoverySnapshot !== NO_SNAPSHOT)
    discoverySnapshot.dispose();
  bridgeState.snapshot = nextSnapshot;

  /**
   Configured project selected through cached project identity.
   */
  const project = nextSnapshot.getConfiguredProject(configFileName,);
  if (project === undefined) {
    throw new SemanticBridgeError({
      reason: 'project-not-found',
      message: `TypeScript snapshot omitted configured project ${configFileName}.`,
    },);
  }
  /**
   Source tree loaded from current snapshot overlay.
   */
  const sourceFile = project
    .program
    .getSourceFile(normalizedFileName,);
  if (sourceFile === undefined) {
    throw new SemanticBridgeError({
      reason: 'source-file-not-found',
      message: `TypeScript project ${project.configFileName} omitted ${normalizedFileName}.`,
    },);
  }
  rl.debug(`opened ${normalizedFileName} through ${project.configFileName}`,);

  return semanticFileSession({
    fileName: normalizedFileName,
    project,
    sourceFile,
    hasBOM,
  },);
}

/**
 Opens current Oxlint source in reusable TypeScript project snapshot.
 
 @param fileName - Source path reported by Oxlint.
 
 @param sourceText - Current in-memory source text.
 
 @param hasBOM - Whether Oxlint stripped leading byte-order mark.
 
 @returns semantic project, checker, source tree, and offset mapper.
 
 @throws {@link SemanticBridgeError} when project or source cannot be resolved,
 or when the native channel died, after retiring the bridge once for the process.
 
 @example
 ```ts
 const session = openSemanticFile({ fileName, sourceText, hasBOM: false });
 ```
 */
export function openSemanticFile({
  fileName,
  sourceText,
  hasBOM,
}: {
  readonly fileName: string;
  readonly sourceText: string;
  readonly hasBOM: boolean;
},): SemanticFileSession {
  try {
    return openSemanticFileUnclassified({
      fileName,
      sourceText,
      hasBOM,
    },);
  }
  catch (error) {
    if (!isTerminalSemanticChannelFailure({ error, },))
      throw error;
    retireSemanticBridgeAfterChannelFailure({ error, },);
    throw new SemanticBridgeError({
      reason: 'api-unavailable',
      message: `TypeScript ${typescriptVersion} synchronous API stopped accepting requests: ${String(error,)
        .split('\n')[0]}`,
    },);
  }
}

export type { SemanticBridgeCacheStats, } from './semantic-bridge-cache.ts';
export type { SemanticFileSession, } from './semantic-file-session.ts';

/**
 Reads cache evidence without exposing mutable bridge storage.
 
 @returns current overlay and configured-project root counts, plus project discoveries so far.
 
 @example
 ```ts
 semanticBridgeCacheStats();
 ```
 */
export function semanticBridgeCacheStats(): SemanticBridgeCacheStats {
  return {
    overlayCount: bridgeState.overlays
      .size,
    projectRootCount: bridgeState.projectByRoot
      .size,
    projectDiscoveryCount: bridgeState.projectDiscoveryCount,
  };
}
