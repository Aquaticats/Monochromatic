/**
 Disposable real-loader harness for Pi goal runtime verification.
 
 @module
 */

import {
  createEventBus,
  discoverAndLoadExtensions,
  type Extension,
  type ExtensionCommandContext,
  type ExtensionContext,
  type ExtensionRuntime,
  SessionManager,
} from '@earendil-works/pi-coding-agent';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  bindRuntimeActions,
  type CapturedRuntimeMessage,
} from './pi-runtime-verifier-actions.ts';

//region Types

/**
 Generic loaded lifecycle callback driven by verifier.
 */
type RuntimeHandler = (
  input: ForeignBorrowed<{
    readonly event: Readonly<Record<string, unknown>>;
    readonly context: ExtensionContext;
  }>,
) => unknown;

/**
 Generic loaded slash-command callback driven by verifier.
 */
type RuntimeCommand = (
  input: ForeignBorrowed<{
    readonly args: string;
    readonly context: ExtensionCommandContext;
  }>,
) => Promise<void>;

/**
 Disposable package discovery result and stateful Pi boundaries.
 */
type GoalRuntimeHarness = {
  readonly extension: Extension;
  readonly runtime: ExtensionRuntime;
  readonly sessionManager: SessionManager;
  readonly context: ExtensionCommandContext;
  readonly messages: readonly CapturedRuntimeMessage[];
  readonly statuses: readonly string[];
  readonly notifications: readonly string[];
};

//endregion Types

//region Loader

/**
 Discover package manifest through real Pi loader with disposable global state.
 
 @param packageDirectory - repository package directory containing manifest
 
 @param agentDirectory - empty disposable Pi agent directory
 
 @param sessionDirectory - disposable session-file directory
 
 @returns loaded extension and bound runtime harness
 
 @throws when Pi reports discovery errors or package count differs
 
 @example
 ```ts
 await createGoalRuntimeHarness({ packageDirectory: '.', agentDirectory, sessionDirectory });
 ```
 */
async function createGoalRuntimeHarness(
  {
    packageDirectory,
    agentDirectory,
    sessionDirectory,
  }: {
    readonly packageDirectory: string;
    readonly agentDirectory: string;
    readonly sessionDirectory: string;
  },
): Promise<GoalRuntimeHarness> {
  /**
   Pi package discovery result using exact package directory.
   */
  const result = await discoverAndLoadExtensions(
    [packageDirectory,],
    packageDirectory,
    agentDirectory,
    createEventBus(),
  );
  if (result.errors
    .length
    > 0) {
    throw new Error(`Pi goal discovery failed: ${result.errors
      .map(function discoveryError(
        error: Readonly<(typeof result.errors)[number]>,
      ) {
        return error.error;
      },)
      .join('; ')}`,);
  }
  if (result.extensions
    .length
    !== 1)
    throw new Error(`expected one discovered goal extension, received ${result.extensions
      .length}`,);
  /**
   Sole package extension discovered from manifest.
   */
  const [extension,] = result.extensions;
  if (extension === undefined)
    throw new Error('Pi discovery returned no goal extension',);
  /**
   Real persisted session confined to disposable directory.
   */
  const sessionManager = SessionManager.create(
    packageDirectory,
    sessionDirectory,
  );
  /**
   Runtime-visible custom messages.
   */
  const messages: CapturedRuntimeMessage[] = [];
  /**
   Footer values in update order.
   */
  const statuses: string[] = [];
  /**
   UI notifications in update order.
   */
  const notifications: string[] = [];
  bindRuntimeActions({
    runtime: result.runtime,
    sessionManager,
    messages,
  },);
  /* oxlint-disable typescript/no-unsafe-type-assertion -- Disposable verifier implements only context members exercised by loaded goal callbacks. */
  /**
   Focused context for loaded command, lifecycle, and tool callbacks.
   */
  const context = {
    cwd: packageDirectory,
    mode: 'rpc',
    hasUI: false,
    ui: {
      setStatus(
        _key: string,
        // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- Mirrors external ExtensionUIContext.setStatus clear sentinel.
        text: string | undefined,
      ) {
        statuses.push(text ?? 'CLEARED',);
      },
      notify(message: string,) {
        notifications.push(message,);
      },
    },
    sessionManager,
    model: undefined,
    isIdle() {
      return true;
    },
    hasPendingMessages() {
      return false;
    },
  } as unknown as ExtensionCommandContext;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  return {
    extension,
    runtime: result.runtime,
    sessionManager,
    context,
    messages,
    statuses,
    notifications,
  };
}

//endregion Loader

export { createGoalRuntimeHarness, };
export type {
  GoalRuntimeHarness,
  RuntimeCommand,
  RuntimeHandler,
};
