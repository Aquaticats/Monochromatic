/**
 Verifies the built agent-settled notification extension at its package boundary.
 
 @module
 */

import type {
  ExtensionAPI,
  ExtensionFactory,
} from '@earendil-works/pi-coding-agent';

import {
  createAgentSettledEvent,
  createExtensionContext,
  createFakePiHarness,
  getOnlyHandler,
} from './test-fixtures.ts';

//region Constants

/**
 Built extension path consumed by Pi package discovery.
 */
const BUILT_EXTENSION_PATH = '../dist/final/node/index.mjs';

/**
 Sole Pi lifecycle event required by this extension.
 */
const AGENT_SETTLED_EVENT = 'agent_settled';

/**
 Executable expected at the host notification boundary.
 */
const EXPECTED_COMMAND = 'notify-send';

/**
 Static title sent to the local desktop-notification service.
 */
const EXPECTED_TITLE = 'Pi agent finished';

//endregion Constants

//region Types

/**
 Shape exported by the built extension package.
 */
type AgentSettledNotificationModule = {
  /**
   Default Pi extension factory.
   */
  readonly default: ExtensionFactory;

  /**
   Registration helper with injectable delivery boundary.
   */
  readonly registerAgentSettledNotification: (
    registration: {
      readonly pi: ExtensionAPI;
      readonly invoke: (invocation: {
        readonly command: string;
        readonly args: readonly string[];
      },) => Promise<void>;
    },
  ) => void;
};

//endregion Types

//region Verification

/**
 Narrows an imported module to the built extension's public shape.
 
 @param value - imported module namespace
 
 @returns whether module exposes the default factory and injectable registration helper
 
 @example
 ```ts
 isAgentSettledNotificationModule(await import('../dist/final/node/index.mjs'));
 ```
 */
function isAgentSettledNotificationModule(
  value: unknown,
): value is AgentSettledNotificationModule {
  if ((value === null) || ((typeof value) !== 'object'))
    return false;
  return ('default' in value)
    && ((typeof value.default) === 'function')
    && ('registerAgentSettledNotification' in value)
    && ((typeof value.registerAgentSettledNotification) === 'function');
}

/**
 Verifies built extension registration and delivery behavior without touching the real desktop.
 
 @returns verification result text
 
 @throws when built export, lifecycle registration, or delivery payload is incorrect
 
 @example
 ```ts
 console.log(await verifyBuiltExtension());
 ```
 */
async function verifyBuiltExtension(): Promise<string> {
  /**
   Module namespace imported from the same built entry Pi loads.
   */
  const mod: unknown = await import(BUILT_EXTENSION_PATH);
  if (!isAgentSettledNotificationModule(mod,))
    throw new Error('built agent-settled-notification package has an unexpected export shape',);

  /**
   Harness used to validate the default extension factory.
   */
  const defaultHarness = createFakePiHarness();
  await mod.default(defaultHarness.api,);
  /**
   Registered event names that must each be the settled-agent event.
   */
  const registeredEvents = [...defaultHarness.handlersByEvent
    .keys()];
  if (registeredEvents.some(function isNotAgentSettled(event: string,): boolean {
    return event !== AGENT_SETTLED_EVENT;
  },))
    throw new Error('built default extension registered a non-settled lifecycle event',);
  if (defaultHarness.handlersByEvent
    .get(AGENT_SETTLED_EVENT,)
    ?.length
    !== 1)
    throw new Error(`built default extension did not register exactly one ${AGENT_SETTLED_EVENT} handler`,);

  /**
   Harness used to drive the built handler through an injected desktop boundary.
   */
  const deliveryHarness = createFakePiHarness();
  /**
   Command invocations observed at the operating-system boundary.
   */
  const invocations: {
    readonly command: string;
    readonly args: readonly string[];
  }[] = [];
  mod.registerAgentSettledNotification({
    pi: deliveryHarness.api,
    invoke: function captureNotification(
      invocation: {
        readonly command: string;
        readonly args: readonly string[];
      },
    ): Promise<void> {
      invocations.push(invocation,);
      return Promise.resolve();
    },
  },);

  /**
   Built settled-event callback driven by the verifier.
   */
  const handler = getOnlyHandler({
    handlersByEvent: deliveryHarness.handlersByEvent,
    event: AGENT_SETTLED_EVENT,
  },);
  await handler(
    createAgentSettledEvent({ aborted: false, },),
    createExtensionContext(),
  );

  /**
   Sole notification boundary invocation produced by one settled event.
   */
  const [invocation,] = invocations;
  if (invocations.length !== 1)
    throw new Error(`expected one desktop notification, received ${invocations.length}`,);
  if (invocation?.command !== EXPECTED_COMMAND)
    throw new Error(`expected notification command ${EXPECTED_COMMAND}, received ${invocation?.command ?? '<none>'}`,);
  if (invocation.args
    .at(1,)
    !== EXPECTED_TITLE)
    throw new Error(`expected notification title ${EXPECTED_TITLE}, received ${invocation.args
      .at(1,)
      ?? '<none>'}`,);

  return 'agent-settled-notification extension verified: one notify-send request after agent_settled';
}

//endregion Verification

console.log(await verifyBuiltExtension(),);
