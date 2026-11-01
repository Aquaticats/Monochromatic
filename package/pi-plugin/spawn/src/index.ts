/**
 Pi extension that coordinates spawn-pi result forwarding.
 
 @module
 */

import type {
  AgentEndEvent,
  ExtensionAPI,
  ExtensionContext,
  SessionStartEvent,
} from '@earendil-works/pi-coding-agent';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  SPAWN_EXTENSION_PATH_ENV,
  SPAWN_ID_ENV,
  SPAWN_PI_CUSTOM_TYPE,
} from './constants.ts';
import { extractLastAssistantText, } from './message-extract.ts';
import {
  autoSetupCli,
  NO_CLI_SETUP_WARNING,
} from './setup-cli.ts';
import {
  checkCompletedChildren,
  claimSpawn,
  completeSpawn,
  NOTHING_TO_REPORT,
  writePidMapping,
} from './state.ts';

//region Module constants

/**
 Current extension module path, passed to child Pi so result delivery loads during local development.
 
 @example
 ```typescript
 process.env.PI_SPAWN_EXTENSION_PATH = EXTENSION_PATH;
 ```
 */
const EXTENSION_PATH = import.meta.filename;

/**
 Poll interval for completed child spawn state files.
 
 @example
 ```typescript
 setInterval(check, SPAWN_MONITOR_INTERVAL_MS);
 ```
 */
const SPAWN_MONITOR_INTERVAL_MS = 1_000;

/**
 Active completed-child monitors keyed by Pi API instance.
 
 @example
 ```typescript
 monitorTimers.set(pi, setInterval(check, 1000));
 ```
 */
const monitorTimers = new WeakMap<ExtensionAPI, ReturnType<typeof setInterval>>();

//endregion Module constants

//region Extension entry point

/**
 Spawn-pi extension entry point.
 
 Registers session identity, child completion reporting, and parent result delivery.
 
 @param pi - {@link ExtensionAPI}.
 
 @example
 ```json
 { "packages": ["./packages/pi-plugin/spawn"] }
 ```
 */
export default function spawnPi(pi: ForeignBorrowed<ExtensionAPI>,): void {
  pi.on(
    'session_start',
    async function handleSessionStart(
      _event: ForeignBorrowed<SessionStartEvent>,
      ctx,
    ): Promise<void> {
      await registerSession({
        ctx,
        extensionPath: EXTENSION_PATH,
      },);
      await startCompletedChildMonitor({
        pi,
        ctx,
      },);
    },
  );

  pi.on(
    'session_shutdown',
    function handleSessionShutdown(): void {
      stopCompletedChildMonitor({ pi, },);
    },
  );

  pi.on(
    'agent_end',
    async function handleAgentEnd(
      event: ForeignBorrowed<AgentEndEvent>,
      ctx,
    ): Promise<void> {
      /**
       Spawn identifier inherited by child Pi process.
       */
      const spawnId = process.env[SPAWN_ID_ENV];
      if (spawnId === undefined)
        return;
      await reportChildCompletion({
        spawnId,
        sessionId: ctx
          .sessionManager
          .getSessionId(),
        lastMessage: extractLastAssistantText(event.messages,),
      },);
    },
  );
}

//endregion Extension entry point

//region Session registration

/**
 Registers current Pi process as spawn-pi parent candidate via {@link writePidMapping}, claims a
 child spawn via {@link claimSpawn} when applicable, and runs {@link autoSetupCli} for first-launch
 CLI setup.
 
 @param ctx - current {@link ExtensionContext}.
 
 @param extensionPath - extension module path to propagate to child Pi process.
 
 @example
 ```typescript
 registerSession({ ctx, extensionPath: '/pkg/dist/final/node/index.mjs' });
 ```
 */
async function registerSession(
  {
    ctx,
    extensionPath,
  }: {
    ctx: ExtensionContext;
    readonly extensionPath: string;
  },
): Promise<void> {
  process.env[SPAWN_EXTENSION_PATH_ENV] = extensionPath;

  /**
   Current Pi session identifier.
   */
  const sessionId = ctx
    .sessionManager
    .getSessionId();
  /**
   Current Pi session file path, empty for in-memory sessions.
   */
  const sessionFile = ctx
    .sessionManager
    .getSessionFile()
    ?? '';

  await writePidMapping({
    pid: process.pid,
    mapping: {
      sessionId,
      sessionFile,
      cwd: ctx.cwd,
      extensionPath,
    },
  },);

  /**
   Spawn identifier inherited by child Pi process.
   */
  const spawnId = process.env[SPAWN_ID_ENV];
  if (spawnId !== undefined) {
    await claimSpawn({
      spawnId,
      sessionId,
      sessionFile,
    },);
  }

  if (!ctx.hasUI)
    return;

  /**
   User-visible warning when CLI auto setup cannot fully complete.
   */
  const cliWarning = await autoSetupCli({ extensionPath, },);
  if (cliWarning !== NO_CLI_SETUP_WARNING) {
    ctx
      .ui
      .notify(
      cliWarning,
      'warning',
      );
  }
}

//endregion Session registration

//region Child reporting

/**
 Reports first completed child Pi agent loop into spawn state via {@link completeSpawn}, extracting
 the final assistant message with {@link extractLastAssistantText}.
 
 @param spawnId - Child spawn identifier from host environment.
 
 @param sessionId - Primitive session identity read at host boundary.
 
 @param lastMessage - Primitive assistant text extracted at host boundary.
 
 @example
 ```typescript
 reportChildCompletion({ spawnId, sessionId, lastMessage });
 ```
 */
async function reportChildCompletion(
  {
    spawnId,
    sessionId,
    lastMessage,
  }: Readonly<{
    spawnId: string;
    sessionId: string;
    lastMessage: string;
  }>,
): Promise<void> {
  await completeSpawn({
    spawnId,
    sessionId,
    lastMessage,
  },);
}

//endregion Child reporting

//region Parent delivery

/**
 Timer handle with optional Node-style `unref` method.
 
 @example
 ```typescript
 const timer: UnrefableTimer = setInterval(check, 1000);
 ```
 */
type UnrefableTimer = ReturnType<typeof setInterval> & {
  /**
   Allows interval not to keep process alive when runtime supports it.
   */
  readonly unref?: () => void;
};

/**
 Starts or replaces completed-child monitor for a Pi session, calling {@link deliverCompletedChildren}
 immediately and on each subsequent poll.
 
 @param pi - {@link ExtensionAPI} used for message injection.
 
 @param ctx - {@link ExtensionContext} for current session.
 
 @example
 ```typescript
 await startCompletedChildMonitor({ pi, ctx });
 ```
 */
async function startCompletedChildMonitor(
  {
    pi,
    ctx,
  }: {
    readonly pi: ExtensionAPI;
    readonly ctx: ExtensionContext;
  },
): Promise<void> {
  stopCompletedChildMonitor({ pi, },);

  await deliverCompletedChildren({
    pi,
    ctx,
  },);

  /**
   Timer that checks for child results while parent Pi remains open.
   */
  const timer: UnrefableTimer = setInterval(
    function pollCompletedChildren(): void {
      // Fire-and-forget poll; delivery resolves on its own microtask while the timer keeps running.
      void deliverCompletedChildren({
        pi,
        ctx,
      },);
    },
    SPAWN_MONITOR_INTERVAL_MS,
  );
  timer.unref?.();
  monitorTimers.set(
    pi,
    timer,
  );
}

/**
 Stops completed-child monitor for a Pi API instance.
 
 @param pi - {@link ExtensionAPI} whose monitor should stop.
 
 @example
 ```typescript
 stopCompletedChildMonitor({ pi });
 ```
 */
function stopCompletedChildMonitor(
  {
    pi,
  }: {
    readonly pi: ExtensionAPI;
  },
): void {
  /**
   Existing timer for this Pi API instance.
   */
  const timer = monitorTimers.get(pi,);
  if (timer === undefined)
    return;

  clearInterval(timer,);
  monitorTimers.delete(pi,);
}

/**
 Consumes completed child results via {@link checkCompletedChildren} and injects them through Pi's
 custom-message API.
 
 @param pi - {@link ExtensionAPI} used for `sendMessage`.
 
 @param ctx - current parent {@link ExtensionContext}.
 
 @returns whether a completed child result was delivered.
 
 @example
 ```typescript
 await deliverCompletedChildren({ pi, ctx });
 ```
 */
async function deliverCompletedChildren(
  {
    pi,
    ctx,
  }: {
    readonly pi: ExtensionAPI;
    readonly ctx: ExtensionContext;
  },
): Promise<boolean> {
  /**
   Parent session identifier whose children should be delivered.
   */
  const parentSessionId = ctx
    .sessionManager
    .getSessionId();

  /**
   Completed child result text consumed atomically from state directory.
   */
  const context = await checkCompletedChildren({
    parentSessionId,
    consume: true,
  },);

  if (context === NOTHING_TO_REPORT)
    return false;

  pi.sendMessage(
    {
      customType: SPAWN_PI_CUSTOM_TYPE,
      content: context,
      display: true,
    },
    {
      deliverAs: 'steer',
      triggerTurn: true,
    },
  );

  return true;
}

//endregion Parent delivery

export {
  deliverCompletedChildren,
  registerSession,
  reportChildCompletion,
  startCompletedChildMonitor,
  stopCompletedChildMonitor,
};
