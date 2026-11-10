/**
 Registration accessors for real Pi goal loader verification.
 
 @module
 */

import type {
  ExtensionCommandContext,
  ExtensionContext,
} from '@earendil-works/pi-coding-agent';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type {
  GoalRuntimeHarness,
  RuntimeCommand,
  RuntimeHandler,
} from './pi-runtime-verifier-harness.ts';

//region Handler and command access

/**
 Retrieve loaded handlers for one lifecycle event as sequential composite.
 
 @param harness - loaded goal extension harness
 
 @param event - lifecycle event name
 
 @returns registered handlers invoked in extension registration order
 
 @throws when registration is absent
 
 @example
 ```ts
 getRuntimeHandler({ harness, event: 'agent_settled' });
 ```
 */
function getRuntimeHandler(
  {
    harness,
    event: eventName,
  }: {
    readonly harness: GoalRuntimeHarness;
    readonly event: string;
  },
): RuntimeHandler {
  /**
   Handlers captured by real extension loader.
   */
  const handlers = harness.extension
    .handlers
    .get(eventName,);
  if ((handlers === undefined) || (handlers.length === 0))
    throw new Error(`missing ${eventName} handler`,);
  /**
   Narrowed nonempty handler collection.
   */
  const loadedHandlers: NonNullable<typeof handlers> = handlers;
  /**
   Invoke every foreign callback using Pi registration order.
   
   @param input - externally owned lifecycle payload and Pi context
   
   @mutates input - handlers may mutate or retain event and context references
   
   @example
   ```ts
   await invokeLoadedHandlers({ event: { type: 'agent_settled' }, context });
   ```
   */
  async function invokeLoadedHandlers(
    input: ForeignBorrowed<{
      readonly event: Readonly<Record<string, unknown>>;
      readonly context: ExtensionContext;
    }>,
  ): Promise<void> {
    for (const handler of loadedHandlers) {
      // oxlint-disable-next-line no-await-in-loop -- Pi handlers chain in registration order.
      await handler(
        input.event,
        input.context,
      );
    }
  }
  return invokeLoadedHandlers;
}

/**
 Retrieve loaded `/goal` command callback.
 
 @param harness - loaded goal extension harness
 
 @returns registered command callback
 
 @throws when command is absent
 
 @example
 ```ts
 getGoalCommand(harness)({ args: 'objective', context: harness.context });
 ```
 */
function getGoalCommand(harness: GoalRuntimeHarness,): RuntimeCommand {
  /**
   Goal command discovered from package default factory.
   */
  const command = harness.extension
    .commands
    .get('goal',);
  if (command === undefined)
    throw new Error('discovered goal command is absent',);
  /**
   Narrowed command retained across nested declaration seam.
   */
  const loadedCommand: NonNullable<typeof command> = command;
  /**
   Invoke foreign goal command.
   
   @param input - exact arguments and Pi command context
   
   @mutates input - loaded command may invoke or retain context capabilities
   
   @example
   ```ts
   await invokeLoadedCommand({ args: 'objective', context });
   ```
   */
  async function invokeLoadedCommand(
    input: ForeignBorrowed<{
      readonly args: string;
      readonly context: ExtensionCommandContext;
    }>,
  ): Promise<void> {
    await loadedCommand.handler(
      input.args,
      input.context,
    );
  }
  return invokeLoadedCommand;
}

//endregion Handler and command access

//region Assertions

/**
 Require verifier condition with focused diagnostic.
 
 @param condition - runtime observation to require
 
 @param message - failure diagnostic
 
 @throws when condition is false
 
 @example
 ```ts
 requireCondition({ condition: true, message: 'must pass' });
 ```
 */
function requireCondition(
  {
    condition,
    message,
  }: {
    readonly condition: boolean;
    readonly message: string;
  },
): void {
  if (!condition)
    throw new Error(message,);
}

/**
 Require exact mutable capture count.
 
 @param actual - observed capture count
 
 @param expected - required capture count
 
 @param message - failure diagnostic
 
 @throws when counts differ
 
 @example
 ```ts
 requireCount({ actual: 1, expected: 1, message: 'one event' });
 ```
 */
function requireCount(
  {
    actual,
    expected,
    message,
  }: {
    readonly actual: number;
    readonly expected: number;
    readonly message: string;
  },
): void {
  if (actual !== expected)
    throw new Error(`${message}: expected ${expected}, received ${actual}`,);
}

//endregion Assertions

//region Event emission and settlement

/**
 Drive one real-loader event callback.
 
 @param harness - disposable loaded extension harness
 
 @param type - registered Pi lifecycle event
 
 @param event - focused event fixture
 
 @example
 ```ts
 await emitGoalEvent({ harness, type: 'agent_settled', event: { type: 'agent_settled' } });
 ```
 */
async function emitGoalEvent(
  {
    harness,
    type,
    event,
  }: {
    readonly harness: GoalRuntimeHarness;
    readonly type: string;
    readonly event: Readonly<Record<string, unknown>>;
  },
): Promise<void> {
  await getRuntimeHandler({
    harness,
    event: type,
  })({
    event,
    context: harness.context,
  },);
}

/**
 Emit final assistant stop reason followed by settlement.
 
 @param harness - disposable loaded extension harness
 
 @param stopReason - Pi assistant terminal reason
 
 @example
 ```ts
 await settleGoalRun({ harness, stopReason: 'stop' });
 ```
 */
async function settleGoalRun(
  {
    harness,
    stopReason,
  }: {
    readonly harness: GoalRuntimeHarness;
    readonly stopReason: 'aborted' | 'error' | 'length' | 'stop';
  },
): Promise<void> {
  await emitGoalEvent({
    harness,
    type: 'agent_end',
    event: {
      type: 'agent_end',
      messages: [{
        role: 'assistant',
        stopReason,
      },],
    },
  },);
  await emitGoalEvent({
    harness,
    type: 'agent_settled',
    event: { type: 'agent_settled', },
  },);
}

//endregion Event emission and settlement

export {
  emitGoalEvent,
  getGoalCommand,
  getRuntimeHandler,
  requireCondition,
  requireCount,
  settleGoalRun,
};
