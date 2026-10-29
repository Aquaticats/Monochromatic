/**
 Built-artifact tests for Pi goal lifecycle adapter.
 
 @module
 */

import type {
  ExtensionAPI,
  ExtensionCommandContext,
  ExtensionContext,
} from '@earendil-works/pi-coding-agent';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  registerGoalLifecycle,
  type GoalLifecycleHandle,
} from '../dist/final/node/index.mjs';

/**
 Broad lifecycle callback shape captured by fake Pi registration seam.
 */
type CapturedHandler = (
  event: unknown,
  context: ExtensionContext,
) => unknown;

/**
 Captured command callback shape.
 */
type CapturedCommandHandler = (
  args: string,
  context: ExtensionCommandContext,
) => Promise<void>;

/**
 Fake Pi lifecycle harness state.
 */
type LifecycleHarness = {
  readonly handlers: Readonly<Record<string, CapturedHandler>>;
  readonly command: CapturedCommandHandler;
  readonly lifecycle: GoalLifecycleHandle;
  readonly appended: unknown[];
  readonly messages: unknown[];
  readonly statuses: string[];
  readonly notifications: string[];
  readonly context: ExtensionCommandContext;
};

/**
 Build focused fake Pi API and context.
 
 @param branch - selected-branch entries returned by session manager
 
 @param idle - whether command can send kickoff immediately
 
 @returns captured registrations and observable effects
 
 @example
 ```ts
 const harness = lifecycleHarness({ branch: [] });
 ```
 */
function lifecycleHarness(
  {
    branch,
    idle = true,
  }: {
    readonly branch: readonly unknown[];
    readonly idle?: boolean;
  },
): LifecycleHarness {
  /** Captured lifecycle callbacks by event name. */
  const handlers: Record<string, CapturedHandler> = {};
  /** Captured command handlers by command name. */
  const commands: Record<string, CapturedCommandHandler> = {};
  /** Custom entries appended by extension. */
  const appended: unknown[] = [];
  /** Task-only custom messages sent by extension. */
  const messages: unknown[] = [];
  /** Footer status values observed in order. */
  const statuses: string[] = [];
  /** UI notifications observed in order. */
  const notifications: string[] = [];
  /** Focused fake API. */
  const api = {
    on(event: string, handler: CapturedHandler,) {
      handlers[event] = handler;
    },
    registerCommand(
      name: string,
      options: { readonly handler: CapturedCommandHandler; },
    ) {
      commands[name] = options.handler;
    },
    appendEntry(_customType: string, data: unknown,) {
      appended.push(data,);
    },
    sendMessage(message: unknown,) {
      messages.push(message,);
    },
  } as unknown as ExtensionAPI;
  /** Focused lifecycle context. */
  const context = {
    mode: 'tui',
    hasUI: true,
    ui: {
      // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- Mirrors external setStatus exactly.
      setStatus(_key: string, text: string | undefined,) {
        statuses.push(text ?? 'CLEARED',);
      },
      notify(message: string,) {
        notifications.push(message,);
      },
    },
    sessionManager: {
      getBranch() {
        return branch;
      },
      getLeafId() {
        return 'leaf-current';
      },
    },
    isIdle() {
      return idle;
    },
    hasPendingMessages() {
      return false;
    },
  } as unknown as ExtensionCommandContext;
  /** Registered lifecycle handle. */
  const lifecycle = registerGoalLifecycle({
    pi: api,
    services: {
      createId: function createFixtureId() {
        return 'fixture-id';
      },
      now: function fixtureNow() {
        return '2026-08-26T00:00:00.000Z';
      },
    },
  },);
  /** Registered strict goal command. */
  const command = commands.goal;
  if (command === undefined)
    throw new Error('goal command was not registered',);
  return {
    handlers,
    command,
    lifecycle,
    appended,
    messages,
    statuses,
    notifications,
    context,
  };
}

/**
 Retrieve required captured lifecycle handler.
 
 @param harness - fake lifecycle harness
 
 @param eventName - registered lifecycle event name
 
 @returns captured event callback
 
 @throws when lifecycle registration is absent
 
 @example
 ```ts
 requiredHandler(harness, 'session_start');
 ```
 */
function requiredHandler(
  harness: LifecycleHarness,
  eventName: string,
): CapturedHandler {
  /** Captured handler for requested event. */
  const handler = harness.handlers[eventName];
  if (handler === undefined)
    throw new Error(`missing ${eventName} lifecycle registration`,);
  return handler;
}

await describe({
  name: registerGoalLifecycle.name,
  children: [
    it({
      name: 'restores active branch and injects only task objective',
      fn: async () => {
        /** Active branch fixture. */
        const harness = lifecycleHarness({
          branch: [{
            type: 'custom',
            customType: 'goal:state',
            data: {
              kind: 'run_started',
              runId: 'run-existing',
              generationId: 'generation-existing',
              objective: 'Restore exact objective',
              startedAt: '2026-08-25T00:00:00.000Z',
              startBoundary: 'leaf-start',
              continuationSequence: 0,
              transitionedAt: '2026-08-25T00:00:00.000Z',
            },
          },],
        },);
        await requiredHandler(harness, 'session_start',)(
          { type: 'session_start', reason: 'startup', },
          harness.context,
        );
        expect(harness.appended,).toHaveLength(1,);
        expect(harness.messages,).toHaveLength(0,);
        /** Per-turn prompt injection result. */
        const promptResult = await requiredHandler(harness, 'before_agent_start',)(
          {
            type: 'before_agent_start',
            prompt: 'continue',
            systemPrompt: 'base prompt',
            systemPromptOptions: { cwd: process.cwd(), },
          },
          harness.context,
        );
        const serialized = JSON.stringify(promptResult,);
        expect(serialized,).toContain('Restore exact objective',);
        expect(serialized,).not.toContain('goal_id',);
        expect(serialized,).not.toContain('review',);
        expect(Object.hasOwn(harness.handlers, 'agent_settled',),).toBe(false,);
        expect(Object.hasOwn(harness.handlers, 'tool_call',),).toBe(false,);
      },
    },),
    it({
      name: 'starts, replaces, rejects removed forms, and clears idempotently',
      fn: async () => {
        const harness = lifecycleHarness({ branch: [], },);
        await harness.command('First objective', harness.context,);
        await harness.command('Second objective', harness.context,);
        expect(harness.appended,).toHaveLength(2,);
        expect(harness.messages,).toHaveLength(2,);
        await harness.command('status', harness.context,);
        await harness.command('clear', harness.context,);
        await harness.command('clear', harness.context,);
        expect(harness.notifications,).toEqual([
          'Usage: /goal <objective> or /goal clear',
          'Goal cleared: Second objective',
          'No goal is active.',
        ],);
      },
    },),
    it({
      name: 'delivers deferred kickoff through private lifecycle handle',
      fn: async () => {
        const harness = lifecycleHarness({ branch: [], idle: false, },);
        await harness.command('Deferred objective', harness.context,);
        expect(harness.messages,).toHaveLength(0,);
        expect(harness.lifecycle.deliverPendingKickoff(harness.context,),).toBe(true,);
        expect(harness.messages,).toHaveLength(1,);
        expect(harness.messages[0],).toMatchObject({
          content: 'User objective (exact JSON string): "Deferred objective"',
        },);
        expect(harness.lifecycle.deliverPendingKickoff(harness.context,),).toBe(false,);
      },
    },),
    it({
      name: 'reconstructs tree and compaction then shuts down without model turn',
      fn: async () => {
        const harness = lifecycleHarness({ branch: [], },);
        await harness.command('Temporary objective', harness.context,);
        await requiredHandler(harness, 'session_tree',)(
          { type: 'session_tree', newLeafId: null, oldLeafId: 'leaf-current', },
          harness.context,
        );
        const [lastStatus,] = harness.statuses.toReversed();
        expect(lastStatus,).toBe('CLEARED',);
        await requiredHandler(harness, 'session_compact',)(
          {
            type: 'session_compact',
            compactionEntry: {},
            fromExtension: false,
            reason: 'manual',
            willRetry: false,
          },
          harness.context,
        );
        await requiredHandler(harness, 'session_shutdown',)(
          { type: 'session_shutdown', reason: 'quit', },
          harness.context,
        );
        expect(harness.messages,).toHaveLength(1,);
      },
    },),
  ],
},);
