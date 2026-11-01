/**
 Lifecycle of the TypeScript 7 synchronous semantic bridge for Oxlint JavaScript rules.

 Owns the native channel, its process-local bridge state, channel-death
 retirement, and idempotent shutdown; the file-open phases in
 `typescript-sync-adapter.ts` run against this state.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { version as typescriptVersion, } from 'typescript';
import {
  API,
  type Snapshot,
} from 'typescript/sync';

import { resetSemanticEffectCaches, } from './effect-cache-lifecycle.ts';
import { SemanticBridgeError, } from './semantic-bridge-error.ts';
import { overlayFileSystem, } from './semantic-overlay-filesystem.ts';
import {
  memoizeApiSnapshotRequests,
  type SnapshotRequestMemo,
} from './semantic-request-memo.ts';
import {
  assertTypeScriptSeven,
  configureNativeApiChildShutdown,
  nativeApiChild,
} from './typescript-sync-native-shutdown.ts';

/**
 Package logger for semantic bridge lifecycle.
 */
const l = tagged({ tag: 'prefer-readonly-parameter-types', },);

/**
 Sentinel before native API client starts.
 */
const NO_API: unique symbol = Symbol('TypeScript synchronous API not started',);

/**
 Sentinel before first semantic snapshot exists.
 */
export const NO_SNAPSHOT: unique symbol = Symbol('TypeScript semantic snapshot not created',);

/**
 Sentinel before bridge tracks current source for rename invalidation.
 */
export const NO_ACTIVE_FILE: unique symbol = Symbol('TypeScript semantic bridge has no active source',);

/**
 Sentinel before the request memo is installed on the native API client.
 */
const NO_REQUEST_MEMO: unique symbol = Symbol('TypeScript request memo not installed',);

/**
 Mutable process-local bridge state shared with the file-open phases of
 `typescript-sync-adapter.ts`.

 Never re-exported through `index.ts`: writes outside the lifecycle and file-open
 functions can strand a snapshot on a dead channel.
 */
export const bridgeState: {
  api: API | typeof NO_API;
  requestMemo: SnapshotRequestMemo | typeof NO_REQUEST_MEMO;
  snapshot: Snapshot | typeof NO_SNAPSHOT;
  readonly overlays: Map<string, string>;
  readonly projectByRoot: Map<string, string>;
  activeFileName: string | typeof NO_ACTIVE_FILE;
  projectDiscoveryCount: number;
  beforeExitHookRegistered: boolean;
  channelRetired: boolean;
} = {
  api: NO_API,
  requestMemo: NO_REQUEST_MEMO,
  snapshot: NO_SNAPSHOT,
  overlays: new Map(),
  projectByRoot: new Map(),
  activeFileName: NO_ACTIVE_FILE,
  projectDiscoveryCount: 0,
  beforeExitHookRegistered: false,
  channelRetired: false,
};

/**
 Starts native synchronous API once and registers process cleanup.

 Never re-exported through `index.ts`: package consumers start the bridge through
 {@link initializeSemanticBridge} instead of reaching the client directly.

 @returns reusable TypeScript API client.

 @throws {@link SemanticBridgeError} when API startup fails or the channel is retired.

 @example
 ```ts
 const api = getApi();
 ```
 */
export function getApi(): API {
  if (bridgeState.channelRetired)
    throw new SemanticBridgeError({
      reason: 'api-unavailable',
      message: `TypeScript ${typescriptVersion} synchronous API was retired after a terminal channel failure; this process does not start a replacement channel.`,
    },);
  if (bridgeState.api !== NO_API)
    return bridgeState.api;

  /**
   Function-tagged semantic lifecycle logger.
   */
  const rl = tagged({
    tag: getApi.name,
    l,
  },);
  assertTypeScriptSeven();
  try {
    /**
     Newly created API configured before process lifecycle hooks observe it.
     */
    const api = new API({
      cwd: process.cwd(),
      fs: overlayFileSystem({ overlays: bridgeState.overlays, },),
    },);
    /**
     Native child whose TypeScript-owned cleanup signal must remain quiet.
     */
    const child = nativeApiChild(api,);
    configureNativeApiChildShutdown(child,);
    bridgeState.requestMemo = memoizeApiSnapshotRequests({ api, },);
    bridgeState.api = api;
    rl.debug(`started TypeScript ${typescriptVersion} synchronous API`,);
  }
  catch (error) {
    rl.error(`failed to start TypeScript synchronous API: ${String(error,)}`,);
    throw new SemanticBridgeError({
      reason: 'api-unavailable',
      message: `TypeScript ${typescriptVersion} synchronous API failed to start: ${String(error,)}`,
    },);
  }

  if (!bridgeState.beforeExitHookRegistered) {
    process.once(
      'beforeExit',
      closeSemanticBridge,
    );
    bridgeState.beforeExitHookRegistered = true;
  }
  return bridgeState.api;
}

/**
 Starts semantic child before Oxlint allocates one fixed AST buffer per worker.

 Oxlint reserves a multi-gigabyte virtual buffer for every Rust worker when a
 JavaScript plugin is active. Starting TypeScript after those reservations can
 make `child_process.spawn` fail with `ENOMEM` on high-core hosts.

 @example
 ```ts
 initializeSemanticBridge();
 ```
 */
export function initializeSemanticBridge(): void {
  getApi();
}

/**
 Retires the semantic bridge after its native channel stopped accepting writes.

 A dead channel can never answer again: a snapshot release or API close would each
 write into it and fail. Retirement marks the bridge dead first so re-entrant
 callers fail closed through the API guard, drops the snapshot without a release
 request, and closes the API while treating its own shutdown failures as noise.
 Idempotent, so several request failures inside one batch retire the bridge once.

 @param error - Caught terminal channel failure that motivated retirement.

 @example
 ```ts
 retireSemanticBridgeAfterChannelFailure({ error: caught, });
 ```
 */
export function retireSemanticBridgeAfterChannelFailure({
  error,
}: {
  readonly error: unknown;
},): void {
  if (bridgeState.channelRetired)
    return;
  bridgeState.channelRetired = true;
  /**
   Function-tagged retirement lifecycle logger.
   */
  const rl = tagged({
    tag: retireSemanticBridgeAfterChannelFailure.name,
    l,
  },);
  rl.warn(`retiring semantic bridge after terminal channel failure: ${String(error,)}`);
  resetSemanticEffectCaches();
  if (bridgeState.snapshot !== NO_SNAPSHOT)
    bridgeState.snapshot = NO_SNAPSHOT;
  if (bridgeState.requestMemo !== NO_REQUEST_MEMO) {
    rl.debug(`request memo totals ${JSON.stringify(bridgeState.requestMemo
      .stats(),)}`,);
    bridgeState.requestMemo = NO_REQUEST_MEMO;
  }
  if (bridgeState.api !== NO_API) {
    try {
      bridgeState
        .api
        .close();
    }
    catch (closeError) {
      rl.warn(`ignoring API close failure while retiring the bridge: ${String(closeError,)}`);
    }
    bridgeState.api = NO_API;
  }
  bridgeState
    .overlays
    .clear();
  bridgeState
    .projectByRoot
    .clear();
  bridgeState.activeFileName = NO_ACTIVE_FILE;
  bridgeState.projectDiscoveryCount = 0;
}

/**
 Disposes active snapshot and native TypeScript API process.

 Idempotent so tests and natural process shutdown may both invoke cleanup. Shutdown
 never raises: release and close failures against a channel whose child already died
 are logged and dropped so process exit stays clean.

 @example
 ```ts
 closeSemanticBridge();
 ```
 */
export function closeSemanticBridge(): void {
  resetSemanticEffectCaches();
  /**
   Function-tagged cleanup lifecycle logger.
   */
  const rl = tagged({
    tag: closeSemanticBridge.name,
    l,
  },);
  if (bridgeState.snapshot !== NO_SNAPSHOT) {
    try {
      bridgeState
        .snapshot
        .dispose();
    }
    catch (disposeError) {
      rl.warn(`ignoring snapshot release failure during shutdown: ${String(disposeError,)}`);
    }
    bridgeState.snapshot = NO_SNAPSHOT;
  }
  if (bridgeState.requestMemo !== NO_REQUEST_MEMO) {
    rl.debug(`request memo totals ${JSON.stringify(bridgeState.requestMemo
      .stats(),)}`,);
    bridgeState.requestMemo = NO_REQUEST_MEMO;
  }
  if (bridgeState.api !== NO_API) {
    try {
      bridgeState
        .api
        .close();
    }
    catch (closeError) {
      rl.warn(`ignoring API close failure during shutdown: ${String(closeError,)}`);
    }
    bridgeState.api = NO_API;
  }
  bridgeState
    .overlays
    .clear();
  bridgeState
    .projectByRoot
    .clear();
  bridgeState.activeFileName = NO_ACTIVE_FILE;
  bridgeState.projectDiscoveryCount = 0;
  rl.debug('closed TypeScript synchronous API',);
}
